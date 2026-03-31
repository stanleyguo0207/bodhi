//! 堆栈帧过滤器模块

use std::sync::{LazyLock, Mutex, OnceLock};

use super::{Error, Frame, Result};
use crate::errcode::BODHIERR_SYS;

/// 堆栈帧过滤器
///
/// 过滤器直接对可变帧引用列表进行原地处理。
pub type FramesFilter = dyn Fn(&mut Vec<&Frame>) + Send + Sync + 'static;

/// 过滤器构建阶段状态
enum BuildState {
  /// 启动期可注册
  Building(Vec<Box<FramesFilter>>),
  /// 已冻结，不再允许注册
  Frozen,
}

/// 启动期状态（仅用于注册/冻结控制）
static FRAME_FILTERS_BUILDING: LazyLock<Mutex<BuildState>> =
  LazyLock::new(|| Mutex::new(BuildState::Building(Vec::new())));

/// 构建完成后的静态过滤器数组（不可变）
static FRAME_FILTERS: OnceLock<Box<[Box<FramesFilter>]>> = OnceLock::new();

/// 注册一个堆栈帧过滤器
pub fn register_frames_filter<F>(filter: F) -> Result<()>
where
  F: Fn(&mut Vec<&Frame>) + Send + Sync + 'static,
{
  let mut state = FRAME_FILTERS_BUILDING.lock().map_err(|e| {
    Error::new(BODHIERR_SYS)
      .wrap_context_with(|| format!("Failed to acquire frame filter lock: {}", e))
  })?;
  match &mut *state {
    BuildState::Building(filters) => {
      filters.push(Box::new(filter));
      Ok(())
    }
    BuildState::Frozen => Err(
      Error::new(BODHIERR_SYS)
        .wrap_context("Frame filters have been frozen and are no longer registerable"),
    ),
  }
}

/// 完成过滤器构建并冻结为全局静态数组
///
/// 该函数应在服务启动阶段末尾调用。调用完成后，过滤器将不可再注册。
pub fn freeze_frames_filters() -> Result<()> {
  let mut state = FRAME_FILTERS_BUILDING.lock().map_err(|e| {
    Error::new(BODHIERR_SYS)
      .wrap_context_with(|| format!("Failed to acquire frame filter build lock: {}", e))
  })?;
  let frozen = match &mut *state {
    BuildState::Building(filters) => {
      let frozen = std::mem::take(filters).into_boxed_slice();
      *state = BuildState::Frozen;
      frozen
    }
    BuildState::Frozen => return Ok(()),
  };
  drop(state);

  FRAME_FILTERS.set(frozen).map_err(|_| {
    Error::new(BODHIERR_SYS).wrap_context("Failed to freeze frame filters: already frozen")
  })?;
  Ok(())
}

/// 应用堆栈帧过滤器
pub(crate) fn apply_frames_filters(frames: &mut Vec<&Frame>) {
  if let Some(filters) = FRAME_FILTERS.get() {
    if filters.is_empty() {
      return;
    }
    for filter in filters.iter() {
      filter(frames);
    }
  }
}

/// 默认堆栈帧过滤器
fn default_frames_filter(frames: &mut Vec<&Frame>) {
  let top_cutoff = frames
    .iter()
    .rposition(|x| x.is_post_panic_code())
    .map(|x| x + 2)
    .unwrap_or(0);

  let bottom_cutoff = frames
    .iter()
    .position(|x| x.is_runtime_init_code())
    .unwrap_or(frames.len());

  let rng = top_cutoff..=bottom_cutoff;
  frames.retain(|x| rng.contains(&x.n))
}

/// 注册默认堆栈帧过滤器
pub fn register_default_frames_filters() -> Result<()> {
  register_frames_filter(default_frames_filter)?;
  Ok(())
}

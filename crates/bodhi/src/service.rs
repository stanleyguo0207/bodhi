//! 服务模块占位符。
//!
//! 说明：这里放置与业务服务相关的共享功能，例如服务初始化、运行循环、公共配置解析等。
//! 当前仓库将应用的启动逻辑放在 `app/*/src/main.rs`，而共享服务组件可集中在此模块中以供复用。
//!
//! 示例（建议将具体实现放在子模块中）：
//! ```ignore
//! // pub fn init_service(cfg: &Config) -> Result<Service, Error> { ... }
//! ```

use crate::args;
use crate::error::Result;

/// 初始化 tracing（tracing-subscriber + tracing-error 的 ErrorLayer）。
fn init_tracing() -> Result<()> {
  use tracing_error::ErrorLayer;
  use tracing_subscriber::{EnvFilter, fmt, prelude::*};

  let fmt_layer = fmt::layer();
  let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
  let subscriber = tracing_subscriber::registry()
    .with(filter)
    .with(fmt_layer)
    .with(ErrorLayer::default());
  tracing::subscriber::set_global_default(subscriber)
    .map_err(|e| crate::error::Error::from(eyre::eyre!("set_global_default failed: {}", e)))?;
  Ok(())
}

/// 初始化全局错误处理器：在 debug 特性启用时安装 `color-eyre`，否则保持轻量。
fn init_error_handler() -> Result<()> {
  #[cfg(feature = "debug")]
  {
    color_eyre::install()?;
    tracing::info!("color-eyre installed (debug feature)");
  }

  #[cfg(not(feature = "debug"))]
  {
    // 保持轻量：release 模式不安装 color-eyre，以降低运行时开销。
    tracing::info!("color-eyre not installed (release mode)");
  }

  Ok(())
}

pub fn serve() -> Result<()> {
  init_tracing()?;
  init_error_handler()?;
  // 解析 CLI 参数（例如 --cfgd 指定的配置目录）
  let args = args::parse_args()?;
  tracing::info!("using config dir: {}", args.config_dir.display());

  Ok(())
}

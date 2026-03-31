use std::cell::Cell;
use std::sync::Arc;

use bodhi_error::define_error_codes;
use bodhi_error::errcode::bodhierr;
use bodhi_error::prelude::*;
use tokio::runtime::Builder;
use tokio::sync::Barrier;

define_error_codes! {
  demoerr [1, 100] {
    /// 参数不合法
    InvalidArgument = 1,
    /// 资源不存在
    NotFound = 2,
    /// 自定义
    Custom = 3,
  }
}

fn parse_number(input: &str) -> Result<i32> {
  input
    .parse::<i32>()
    .map_err(Error::from_std)
    .wrap_context("parse_number failed")
    .wrap_context_with(|| format!("input={input}"))
}

fn test_errcode_and_macro_api() {
  assert_eq!(BODHIERR_OK, 0);
  assert_eq!(BODHIERR_SYS, -1);
  assert_eq!(DEMOERR_INVALIDARGUMENT, 1);
  assert_eq!(DEMOERR_NOTFOUND, 2);

  bodhierr::register();
  demoerr::register();
}

fn test_error_debug_and_display_case() {
  let e = Error::new(DEMOERR_NOTFOUND)
    .wrap_context("user not found")
    .wrap_context_with(|| "lookup by id=42".to_string());
  assert_eq!(e.code(), DEMOERR_NOTFOUND);

  let display = format!("{e}");
  assert!(display.contains("code: 2"));
  assert!(display.contains("user not found"));
  assert!(display.contains("lookup by id=42"));

  let debug = format!("{e:?}");
  assert!(debug.contains("Meta:"));
  assert!(debug.contains("Contexts:"));
  assert!(debug.contains("[1] user not found"));
  assert!(debug.contains("[2] lookup by id=42"));

  let from_io: Error = std::io::Error::other("io convert").into();
  assert_eq!(from_io.code(), BODHIERR_SYS);

  let anyhow_err = anyhow::anyhow!("root cause")
    .context("service failed")
    .context("request failed");
  let from_anyhow: Error = anyhow_err.into();
  let anyhow_text = format!("{from_anyhow}");
  assert_eq!(from_anyhow.code(), BODHIERR_SYS);
  assert!(anyhow_text.contains("service failed"));
  assert!(anyhow_text.contains("root cause"));
}

fn test_result_wrap_context_and_wrap_context_with_api() {
  let ok_value = Ok::<i32, Error>(42)
    .wrap_context("should not apply")
    .wrap_context_with(|| "should also not apply".to_string())
    .expect("ok result should stay ok");
  assert_eq!(ok_value, 42);

  let called = Cell::new(false);
  let err_result = Err::<(), Error>(Error::new(BODHIERR_SYS))
    .wrap_context("outer context")
    .wrap_context_with(|| {
      called.set(true);
      "lazy context".to_string()
    });
  let err = err_result.expect_err("error expected");
  assert!(called.get());

  let text = format!("{err}");
  assert!(text.contains("outer context"));
  assert!(text.contains("lazy context"));

  let parse_err = parse_number("not-a-number").expect_err("parse should fail");
  assert_eq!(parse_err.code(), BODHIERR_SYS);
  assert!(format!("{parse_err}").contains("parse_number failed"));
}

fn test_option_related_api() {
  let present = Some("8080")
    .ok_or_err(DEMOERR_INVALIDARGUMENT)
    .expect("Some should convert to Ok");
  assert_eq!(present, "8080");

  let absent_err = Option::<&str>::None
    .ok_or_err(DEMOERR_INVALIDARGUMENT)
    .expect_err("None should convert to Err");
  assert_eq!(absent_err.code(), DEMOERR_INVALIDARGUMENT);
}

fn test_context_append_print_details() {
  let e = Error::new(BODHIERR_SYS)
    .wrap_context("ctx-1")
    .wrap_context("ctx-2")
    .wrap_context("ctx-3")
    .wrap_context("ctx-4")
    .wrap_context("ctx-5")
    .wrap_context("ctx-6");

  // Display 最多展示前 5 条并标记总数。
  let display = format!("{e}");
  assert!(display.contains("ctx-1 -> ctx-2 -> ctx-3 -> ctx-4 -> ctx-5 ...(6 total)"));

  // Debug 展示完整上下文序号明细。
  let debug = format!("{e:?}");
  assert!(debug.contains("Contexts:"));
  assert!(debug.contains("[1] ctx-1"));
  assert!(debug.contains("[6] ctx-6"));
}

fn test_frame_and_filter_api() {
  register_frames_filter(|frames| {
    // 使用真实回溯帧覆盖 Frame 方法 API。
    for frame in frames.iter() {
      let _ = frame.is_post_panic_code();
      let _ = frame.is_runtime_init_code();
    }

    frames.retain(|frame| frame.name.as_deref() != Some("my_app::internal"));
  })
  .expect("register custom filter");
  register_default_frames_filters().expect("register default filters");
  freeze_frames_filters().expect("freeze filters");

  // 冻结后再次注册应失败。
  let late_register = register_frames_filter(|_frames| {});
  assert!(late_register.is_err());
}

fn test_nested_function_error() {
  fn f0() -> Result<()> {
    Err(Error::new(DEMOERR_CUSTOM).wrap_context("f0 failed"))
  }

  fn f1() -> Result<()> {
    f0().wrap_context("f1 failed")
  }

  fn f2() -> Result<()> {
    f1().wrap_context("f2 failed")
  }

  let err = f2();

  if let Err(e) = err {
    let display = format!("{e}");
    let debug = format!("{e:?}");
    assert!(display.contains("f0 failed"));
    assert!(display.contains("f1 failed"));
    assert!(display.contains("f2 failed"));
    assert!(debug.contains("[1] f0 failed"));
    assert!(debug.contains("[3] f2 failed"));
  }
}

async fn async_worker(
  task_name: &'static str,
  input: Option<&'static str>,
  gate: Arc<Barrier>,
) -> Result<i32> {
  gate.wait().await;

  let input = input
    .ok_or_err(DEMOERR_INVALIDARGUMENT)
    .wrap_context_with(|| format!("task={task_name} missing input"))?;

  input
    .parse::<i32>()
    .map_err(Error::from_std)
    .wrap_context("async parse failed")
    .wrap_context_with(|| format!("task={task_name} thread={:?}", std::thread::current().id()))
}

fn test_async_multi_thread_error_with_tokio() {
  let runtime = Builder::new_multi_thread()
    .worker_threads(2)
    .enable_all()
    .build()
    .expect("build tokio runtime");

  runtime.block_on(async {
    let gate = Arc::new(Barrier::new(3));

    let ok_handle = tokio::spawn(async_worker("worker-ok", Some("7"), gate.clone()));
    let parse_handle = tokio::spawn(async_worker("worker-parse", Some("NaN"), gate.clone()));
    let option_handle = tokio::spawn(async_worker("worker-option", None, gate.clone()));

    let ok_value = ok_handle
      .await
      .expect("join ok")
      .expect("worker-ok should succeed");
    assert_eq!(ok_value, 7);

    let parse_err = parse_handle
      .await
      .expect("join parse")
      .expect_err("worker-parse should fail");
    assert_eq!(parse_err.code(), BODHIERR_SYS);
    let parse_display = format!("{parse_err}");
    let parse_debug = format!("{parse_err:?}");
    assert!(parse_display.contains("async parse failed"));
    assert!(parse_display.contains("task=worker-parse"));
    assert!(parse_display.contains("invalid digit"));
    assert!(parse_debug.contains("Source:"));
    assert!(parse_debug.contains("Contexts:"));

    let option_err = option_handle
      .await
      .expect("join option")
      .expect_err("worker-option should fail");
    assert_eq!(option_err.code(), DEMOERR_INVALIDARGUMENT);
    let option_display = format!("{option_err}");
    let option_debug = format!("{option_err:?}");
    assert!(option_display.contains("task=worker-option missing input"));
    assert!(option_debug.contains("[1] task=worker-option missing input"));
  });
}

fn main() {
  test_errcode_and_macro_api();
  test_error_debug_and_display_case();
  test_result_wrap_context_and_wrap_context_with_api();
  test_option_related_api();
  test_context_append_print_details();
  test_nested_function_error();
  test_async_multi_thread_error_with_tokio();
  test_frame_and_filter_api();
  println!("bodhi_error API example passed.");
}

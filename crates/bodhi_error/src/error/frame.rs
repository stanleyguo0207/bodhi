//! 堆栈追踪模块

use std::path::PathBuf;

/// 堆栈帧
#[derive(Debug)]
#[non_exhaustive]
pub struct Frame {
  pub n: usize,
  pub name: Option<String>,
  pub lineno: Option<u32>,
  pub filename: Option<PathBuf>,
}

impl Frame {
  /// 是否是 panic 后的代码
  pub fn is_post_panic_code(&self) -> bool {
    const SYM_PREFIXES: &[&str] = &[
      "_rust_begin_unwind",
      "rust_begin_unwind",
      "core::result::unwrap_failed",
      "core::option::expect_none_failed",
      "core::panicking::panic_fmt",
      "color_backtrace::create_panic_handler",
      "std::panicking::begin_panic",
      "begin_panic_fmt",
      "failure::backtrace::Backtrace::new",
      "backtrace::capture",
      "failure::error_message::err_msg",
      "<failure::error::Error as core::convert::From<F>>::from",
      "bodhi_error::error::core",
    ];

    match self.name.as_ref() {
      Some(name) => SYM_PREFIXES.iter().any(|x| name.starts_with(x)),
      None => false,
    }
  }

  /// 是否是运行时初始化代码
  pub fn is_runtime_init_code(&self) -> bool {
    const SYM_PREFIXES: &[&str] = &[
      "std::rt::lang_start::",
      "test::run_test::run_test_inner::",
      "std::sys_common::backtrace::__rust_begin_short_backtrace",
    ];

    let (name, file) = match (self.name.as_ref(), self.filename.as_ref()) {
      (Some(name), Some(filename)) => (name, filename.to_string_lossy()),
      _ => return false,
    };

    if SYM_PREFIXES.iter().any(|x| name.starts_with(x)) {
      return true;
    }

    if name == "{{closure}}" && file == "src/libtest/lib.rs" {
      return true;
    }

    false
  }
}

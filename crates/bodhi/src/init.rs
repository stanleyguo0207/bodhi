//! 初始化辅助：统一设置 tracing 与 eyre/color-eyre 错误处理器。
//!
//! 提供一个便捷函数 `init_tracing_and_errors()`，供二进制程序在启动时调用，
//! 以确保 tracing subscriber 与错误报告器的初始化一致。

use crate::error::Result;

/// 初始化 tracing（tracing-subscriber + tracing-error 的 ErrorLayer）。
pub fn init_tracing() -> Result<()> {
  use tracing_error::ErrorLayer;
  use tracing_subscriber::{EnvFilter, fmt, prelude::*};

  let fmt_layer = fmt::layer();
  let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
  let subscriber = tracing_subscriber::registry()
    .with(filter)
    .with(fmt_layer)
    .with(ErrorLayer::default());
  tracing::subscriber::set_global_default(subscriber)
    .map_err(|e| eyre::eyre!("set_global_default failed: {}", e))?;
  Ok(())
}

/// 初始化全局错误处理器：在 debug 特性启用时安装 `color-eyre`，否则保持轻量。
pub fn init_error_handler() -> Result<()> {
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

/// 便捷组合：同时初始化 tracing 与错误处理器。
pub fn init_tracing_and_errors() -> Result<()> {
  init_tracing()?;
  init_error_handler()?;
  Ok(())
}

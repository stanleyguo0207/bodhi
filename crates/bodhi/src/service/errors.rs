//! 全局错误处理器安装（color-eyre 在 debug 模式下）

use crate::error::Result;

pub(crate) fn init_error_handler() -> Result<()> {
  #[cfg(feature = "debug")]
  {
    color_eyre::install()?;
    tracing::info!("color-eyre installed (debug feature)");
  }

  #[cfg(not(feature = "debug"))]
  {
    tracing::info!("color-eyre not installed (release mode)");
  }

  Ok(())
}

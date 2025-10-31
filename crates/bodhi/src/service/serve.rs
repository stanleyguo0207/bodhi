use crate::args;
use crate::error::Result;

pub fn serve() -> Result<()> {
  // 初始化 tracing 与 error handler
  crate::service::tracing::init_tracing()?;
  crate::service::errors::init_error_handler()?;

  // 解析 CLI 参数（例如 --cfgd 指定的配置目录）
  let args = args::parse_args()?;
  tracing::info!("using config dir: {}", args.config_dir.display());

  Ok(())
}

use crate::error::Result;

use clap::Parser;
use eyre::eyre;

use super::cli::Cli;
use super::types::Args;

/// 解析命令行参数并做基本校验（当前仅校验配置目录存在）
pub fn parse_args() -> Result<Args> {
  let cli = Cli::parse();

  if !cli.config_dir.exists() {
    return Err(eyre!(
      "config dir '{}' does not exist",
      cli.config_dir.display()
    ));
  }

  Ok(Args {
    config_dir: cli.config_dir,
  })
}

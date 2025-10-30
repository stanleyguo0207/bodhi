use std::path::PathBuf;

use clap::Parser;

/// CLI parser for the `bodhi` crate/binary. (internal)
#[derive(Debug, Parser)]
#[command(name = "bodhi", about = "bodhi service")]
pub(crate) struct Cli {
  /// 配置目录
  #[arg(long = "cfgd", value_name = "CONFIG_DIR")]
  pub(crate) config_dir: PathBuf,
}

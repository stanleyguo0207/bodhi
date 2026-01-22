use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
  /// Path to the template directory
  #[arg(long, value_parser = clap::value_parser!(PathBuf))]
  pub template_dir: PathBuf,

  /// Path to a custom configuration file
  #[arg(long, value_parser = clap::value_parser!(PathBuf))]
  pub custom_config: PathBuf,

  /// Output directory
  #[arg(long, value_parser = clap::value_parser!(PathBuf))]
  pub out_dir: PathBuf,
}

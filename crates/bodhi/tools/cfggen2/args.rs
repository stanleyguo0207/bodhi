use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
  /// Configuration directory
  #[arg(long = "config_dir", value_parser = clap::value_parser!(PathBuf))]
  pub config_dir: PathBuf,

  /// Profile name
  #[arg(long)]
  pub profile: String,

  /// Output directory
  #[arg(long = "out_dir", value_parser = clap::value_parser!(PathBuf))]
  pub out_dir: PathBuf,

  #[arg(long, default_values = ["toml"])]
  pub format: Vec<String>,
}

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
  /// Configuration file path
  #[arg(
    long,
    value_name = "FILE",
    value_parser = clap::value_parser!(PathBuf),
    help = "Path to configuration file"
  )]
  pub config: PathBuf,

  /// Run as a daemon
  #[arg(long, help = "Run as a daemon")]
  pub daemon: bool,

  /// Subcommand to execute
  #[command(subcommand)]
  pub cmd: Command,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Command {
  /// Start the service
  Start,
  /// Stop the service
  Stop,
  /// Restart the service
  Restart,
  /// Reload the configuration
  Reload,
  /// Check the status of the service
  Check,
}

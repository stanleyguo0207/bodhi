use clap::Parser;
use std::path::PathBuf;

use bodhi_error::custom;
use bodhi_result::Result;

use crate::helper;

const ARG_PARSE_SOLUTION: &str =
  "Please check if the arguments are formatted correctly, use --help for assistance";

/// Command line argument parsing structure.
#[derive(Parser, Debug, PartialEq)]
#[command(version, about, long_about = None)]
pub struct Args {
  /// Configuration directory path.
  #[arg(
    long = "cfgd",
    value_name = "DIRECTORY",
    value_hint = clap::ValueHint::DirPath,
    value_parser = helper::validate_dir,
    help = "Path to the configuration files directory"
  )]
  pub config_dir: PathBuf,
}

/// Parse command line arguments.
pub fn parse_args() -> Result<Args> {
  Args::try_parse().map_err(|clap_err| {
    if clap_err.kind() == clap::error::ErrorKind::DisplayHelp
      || clap_err.kind() == clap::error::ErrorKind::DisplayVersion
    {
      clap_err.exit();
    }

    custom::error::Error::new(
      "Command line argument parsing failed",
      ARG_PARSE_SOLUTION,
      Box::new(clap_err),
    )
    .into()
  })
}

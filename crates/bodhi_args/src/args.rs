use clap::Parser;
use std::fs::metadata;
use std::path::PathBuf;

use bodhi_result::Result;

fn validate_dir(s: &str) -> Result<PathBuf> {
  let path = PathBuf::from(s);

  let meta = metadata(&path)?;
  if !meta.is_dir() {
    return Err(
      std::io::Error::new(
        std::io::ErrorKind::NotADirectory,
        format!("'{}' is not a directory", s),
      )
      .into(),
    );
  }

  Ok(path)
}

/// Command line parameter section parser.
#[derive(Parser, Debug, PartialEq)]
#[command(version, about, long_about = None)]
pub struct Args {
  /// Configuration directory path.
  #[arg(
    long = "cfgd",
    value_name = "DIRECTORY",
    value_hint = clap::ValueHint::DirPath,
    value_parser = validate_dir,
    help = "Path to the configuration directory"
  )]
  pub config_dir: PathBuf,
}

/// Parse command line parameter.
pub fn parse_args() -> Result<Args> {
  Args::try_parse().map_err(|e| {
    if e.kind() == clap::error::ErrorKind::DisplayHelp
      || e.kind() == clap::error::ErrorKind::DisplayVersion
    {
      e.exit();
    }
    e.into()
  })
}

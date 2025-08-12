use clap::Parser;

fn validate_dir(s: &str) -> Result<std::path::PathBuf, String> {
  let path = std::path::PathBuf::from(s);

  if !path.exists() {
    return Err(format!("path '{}' not exists", s));
  }

  if !path.is_dir() {
    return Err(format!("'{}' is not a directory", s));
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
    help = "path to the configuration directory"
  )]
  pub config_dir: std::path::PathBuf,
}

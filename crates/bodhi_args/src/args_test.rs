#[cfg(test)]
mod tests {
  use super::super::args::Args;
  use clap::Parser;
  use std::path::PathBuf;
  use tempfile::{NamedTempFile, tempdir};

  #[test]
  fn test_custom_config_dir() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    let cli = Args::try_parse_from(&["test_app", "--cfgd", temp_path]).unwrap();
    assert_eq!(cli.config_dir, PathBuf::from(temp_path));
  }

  #[test]
  fn test_nonexistent_directory() {
    let result = Args::try_parse_from(&["test_app", "--cfgd", "/nonexistent/path"]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("path '/nonexistent/path' not exists"));
  }

  #[test]
  fn test_file_instead_of_directory() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    let result = Args::try_parse_from(&["test_app", "--cfgd", file_path]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains(&format!("'{}' is not a directory", file_path)));
  }

  #[test]
  fn test_invalid_argument() {
    let result = Args::try_parse_from(&["test_app", "--invalid-arg"]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("unexpected argument"));
  }
}

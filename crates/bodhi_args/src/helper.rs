use std::fs::metadata;
use std::path::PathBuf;

use bodhi_error::custom;
use bodhi_result::Result;

const IO_ERROR_SOLUTION: &str =
  "Please check if the path is correct and the program has access permissions";
const INVALID_DIR_SOLUTION: &str = "Please provide an existing directory path";

/// Validate if the path is a valid directory.
pub fn validate_dir(s: &str) -> Result<PathBuf> {
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

  let meta = metadata(&path).map_err(|io_error| {
    custom::error::Error::new(
      &format!("Unable to access path '{}'", s),
      IO_ERROR_SOLUTION,
      Box::new(io_error),
    )
  })?;

  if !meta.is_dir() {
    let err = std::io::Error::new(
      std::io::ErrorKind::NotADirectory,
      format!("'{}' is not a directory", s),
    );
    return Err(
      custom::error::Error::new(
        &format!("Path '{}' is not a valid directory", s),
        INVALID_DIR_SOLUTION,
        Box::new(err),
      )
      .into(),
    );
  }

  Ok(path)
}

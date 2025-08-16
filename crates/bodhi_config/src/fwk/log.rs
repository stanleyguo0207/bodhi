use serde::Deserialize;

/// Log configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
  /// Log level.
  pub level: String,
  /// Log directory.
  pub dir: Option<String>,
  /// Log file.
  pub name: String,
}

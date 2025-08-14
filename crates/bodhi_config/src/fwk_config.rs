use serde::Deserialize;

/// Framework configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct FwkConfig {
  /// Log configuration.
  pub log: LogConfig,
}

/// Log configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
  /// Log level.
  pub level: String,
  /// Log directory.
  pub dir: Option<String>,
  /// Log file.
  pub name: String,
}

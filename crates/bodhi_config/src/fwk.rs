use serde::Deserialize;

pub mod log;

/// Framework configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
  /// Log configuration.
  pub log: log::Config,
}

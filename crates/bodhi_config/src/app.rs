use serde::Deserialize;

use crate::fwk;

/// Application configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct Config<T> {
  /// Framework configuration.
  pub framework: fwk::Config,
  /// Business configuration.
  pub business: T,
}

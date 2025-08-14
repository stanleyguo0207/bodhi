use serde::Deserialize;

use crate::fwk_config::FwkConfig;

/// Application configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig<BizConfig> {
  /// Framework configuration.
  pub framework: FwkConfig,
  /// Business configuration.
  pub business: BizConfig,
}

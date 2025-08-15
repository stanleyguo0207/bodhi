use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct GatewayConfig {
  pub listen_port: u16,
  pub upstream: String,
  pub timeout_ms: u32,
}

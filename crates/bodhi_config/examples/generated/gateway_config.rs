use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
  pub log: LogConfig,
  pub metrics: MetricsConfig,
  pub net: NetConfig,
  pub routes: RoutesConfig,
  pub server: ServerConfig,
  pub service: ServiceConfig,
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
  pub format: String,
  pub level: String,
  pub output: String,
}

#[derive(Debug, Deserialize)]
pub struct MetricsConfig {
  pub bind: String,
  pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct NetConfig {
  pub connect_timeout_ms: u64,
  pub listen_host: String,
  pub request_timeout_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct RoutesConfig {
  pub prefix: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
  pub grpc_port: u16,
  pub http_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
  pub name: String,
  pub shutdown_timeout_ms: u64,
}

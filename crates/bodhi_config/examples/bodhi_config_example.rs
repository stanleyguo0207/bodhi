#[allow(dead_code)]
#[path = "generated/gateway_config.rs"]
mod generated_gateway_config;

use bodhi_config::prelude::*;
use generated_gateway_config::Config;

fn main() -> Result<()> {
  let engine = ConfigEngine::new("config")?;
  let rust_output_path = engine.default_rust_output_path("dev", "gateway");
  engine.generate_rust_types("dev", "gateway", &rust_output_path)?;

  let config = engine.resolve("dev", "gateway")?;
  let typed: Config = config.extract(".")?;

  println!(
    "service={} log_level={} http_port={}",
    typed.service.name, typed.log.level, typed.server.http_port
  );
  Ok(())
}

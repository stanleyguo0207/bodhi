use bodhi_config::prelude::*;

const PROFILE: &str = "dev";

bodhi_config::service_config!();

fn main() -> Result<()> {
  let store: ServiceConfigStore = load_service_config_store(PROFILE)?;
  let snapshot = store.snapshot();
  bootstrap_gateway(snapshot.version(), snapshot.infra(), snapshot.service());

  Ok(())
}

fn bootstrap_gateway(version: u64, infra: &InfraConfig, service: &ServiceConfig) {
  println!("config_version = {}", version);
  init_gateway_infra(infra);
  init_gateway_server(infra, service);
}

fn init_gateway_infra(infra: &InfraConfig) {
  println!(
    "log = level:{} format:{} output:{}",
    infra.log.level, infra.log.format, infra.log.output
  );
  println!(
    "metrics = enabled:{} bind:{}",
    infra.metrics.enabled, infra.metrics.bind
  );
  println!(
    "service = name:{} shutdown_timeout_ms:{}",
    infra.service.name, infra.service.shutdown_timeout_ms
  );
}

fn init_gateway_server(infra: &InfraConfig, service: &ServiceConfig) {
  println!(
    "gateway_server = listen_host:{} http_port:{} grpc_port:{} connect_timeout_ms:{} request_timeout_ms:{} route_prefix:{}",
    infra.net.listen_host,
    service.server.http_port,
    service.server.grpc_port,
    infra.net.connect_timeout_ms,
    infra.net.request_timeout_ms,
    service.routes.prefix
  );
}

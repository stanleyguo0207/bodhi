use std::fs;
use std::sync::Arc;
use std::thread;

use bodhi_config::prelude::*;
use serde::Deserialize;
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
struct InfraConfig {
  log: LogConfig,
  service: ServiceMeta,
}

#[derive(Debug, Deserialize)]
struct LogConfig {
  level: String,
  output: String,
}

#[derive(Debug, Deserialize)]
struct ServiceMeta {
  name: String,
}

#[derive(Debug, Deserialize)]
struct GatewayServiceConfig {
  server: ServerConfig,
  routes: RouteConfig,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
  http_port: u16,
}

#[derive(Debug, Deserialize)]
struct RouteConfig {
  prefix: String,
}

#[test]
fn layered_resolve_should_keep_infra_and_service_separate() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");

  write_runtime_test_config(&config_dir, "stderr", 18080);

  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  let layers = engine
    .resolve_layers("dev", "gateway")
    .expect("resolve layered config");

  let infra: InfraConfig = layers.extract_infra(".").expect("extract infra");
  let service: GatewayServiceConfig = layers.extract_service(".").expect("extract service");
  let merged_name: String = layers
    .extract_merged("service.name")
    .expect("extract merged service name");

  assert_eq!(infra.log.level, "INFO");
  assert_eq!(infra.log.output, "stderr");
  assert_eq!(infra.service.name, "gateway");
  assert_eq!(service.server.http_port, 18080);
  assert_eq!(service.routes.prefix, "/api/v1");
  assert_eq!(merged_name, "gateway");
}

#[test]
fn config_store_should_share_snapshots_across_threads_and_reload() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");
  let profile_path = config_dir.join("profile/dev.toml");

  write_runtime_test_config(&config_dir, "stderr", 18080);

  let store = Arc::new(
    ConfigStore::<InfraConfig, GatewayServiceConfig>::load_from(&config_dir, "dev", "gateway")
      .expect("load config store"),
  );

  let handles: Vec<_> = (0..4)
    .map(|_| {
      let store = Arc::clone(&store);
      thread::spawn(move || {
        let snapshot = store.snapshot();
        (
          snapshot.version(),
          snapshot.infra().log.output.clone(),
          snapshot.service().server.http_port,
        )
      })
    })
    .collect();

  for handle in handles {
    let (version, output, port) = handle.join().expect("thread should finish");
    assert_eq!(version, 1);
    assert_eq!(output, "stderr");
    assert_eq!(port, 18080);
  }

  fs::write(
    &profile_path,
    concat!(
      "[infra.log]\n",
      "output = \"file\"\n",
      "\n",
      "[services.gateway.server]\n",
      "http_port = 28080\n"
    ),
  )
  .expect("rewrite profile");

  let snapshot = store.reload().expect("reload config");
  assert_eq!(snapshot.version(), 2);
  assert_eq!(snapshot.infra().log.output, "file");
  assert_eq!(snapshot.service().server.http_port, 28080);
  assert_eq!(store.current_version(), 2);
}

fn write_runtime_test_config(config_dir: &std::path::Path, log_output: &str, http_port: u16) {
  fs::create_dir_all(config_dir.join("template/infra")).expect("create template infra dir");
  fs::create_dir_all(config_dir.join("template/service")).expect("create template service dir");
  fs::create_dir_all(config_dir.join("profile")).expect("create profile dir");

  fs::write(
    config_dir.join("template/infra/log.toml"),
    "[log]\nlevel = \"INFO\"\noutput = \"stdout\"\n",
  )
  .expect("write infra log");
  fs::write(
    config_dir.join("template/infra/service.toml"),
    "[service]\nname = \"default\"\n",
  )
  .expect("write infra service");
  fs::write(
    config_dir.join("template/service/gateway.toml"),
    concat!(
      "[infra.service]\n",
      "name = \"gateway\"\n",
      "[server]\n",
      "http_port = 8080\n",
      "[routes]\n",
      "prefix = \"/api/v1\"\n"
    ),
  )
  .expect("write gateway template");
  fs::write(
    config_dir.join("profile/dev.toml"),
    format!(
      "[infra.log]\noutput = \"{log_output}\"\n\n[services.gateway.server]\nhttp_port = {http_port}\n"
    ),
  )
  .expect("write dev profile");
}

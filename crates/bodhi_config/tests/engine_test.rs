use std::fs;

use bodhi_config::prelude::*;
use serde::Deserialize;
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
struct LogConfig {
  level: String,
  output: String,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
  http_port: u16,
}

#[test]
fn engine_should_resolve_profile_and_service_overrides() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");

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
    "[infra.log]\nlevel = \"WARN\"\n[infra.service]\nname = \"gateway\"\n[server]\nhttp_port = 80\n",
  )
  .expect("write gateway template");
  fs::write(
    config_dir.join("profile/stanley.toml"),
    "[infra.log]\noutput = \"file\"\n[services.gateway.infra.log]\nlevel = \"DEBUG\"\n[services.gateway.server]\nhttp_port = 8080\n",
  )
  .expect("write profile");

  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  let resolved = engine
    .resolve("stanley", "gateway")
    .expect("resolve gateway config");

  let log: LogConfig = resolved.extract("log").expect("extract log config");
  let service_name: String = resolved
    .extract("service.name")
    .expect("extract service name");
  let server: ServerConfig = resolved.extract("server").expect("extract server config");

  assert_eq!(log.level, "DEBUG");
  assert_eq!(log.output, "file");
  assert_eq!(service_name, "gateway");
  assert_eq!(server.http_port, 8080);
}

#[test]
fn engine_should_generate_multiple_formats() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");

  fs::create_dir_all(config_dir.join("template/infra")).expect("create template infra dir");
  fs::create_dir_all(config_dir.join("template/service")).expect("create template service dir");
  fs::create_dir_all(config_dir.join("profile")).expect("create profile dir");

  fs::write(
    config_dir.join("template/infra/log.toml"),
    "[log]\nlevel = \"INFO\"\n",
  )
  .expect("write infra log");
  fs::write(
    config_dir.join("template/infra/service.toml"),
    "[service]\nname = \"default\"\n",
  )
  .expect("write infra service");
  fs::write(
    config_dir.join("template/service/gateway.toml"),
    "[infra.service]\nname = \"gateway\"\n",
  )
  .expect("write gateway template");
  fs::write(config_dir.join("profile/dev.toml"), "").expect("write dev profile");

  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  engine
    .generate(
      "dev",
      &[OutputFormat::Toml, OutputFormat::Json, OutputFormat::Yaml],
    )
    .expect("generate products");

  assert!(config_dir.join("product/dev/toml/gateway.toml").is_file());
  assert!(config_dir.join("product/dev/json/gateway.json").is_file());
  assert!(config_dir.join("product/dev/yaml/gateway.yaml").is_file());
}

#[test]
fn engine_should_reject_unknown_profile_field() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");

  fs::create_dir_all(config_dir.join("template/infra")).expect("create template infra dir");
  fs::create_dir_all(config_dir.join("template/service")).expect("create template service dir");
  fs::create_dir_all(config_dir.join("profile")).expect("create profile dir");

  fs::write(
    config_dir.join("template/infra/log.toml"),
    "[log]\nlevel = \"INFO\"\n",
  )
  .expect("write infra log");
  fs::write(
    config_dir.join("template/service/gateway.toml"),
    "[server]\nhttp_port = 80\n",
  )
  .expect("write gateway template");
  fs::write(
    config_dir.join("profile/dev.toml"),
    "[services.gateway.servr]\nhttp_port = 8080\n",
  )
  .expect("write invalid profile");

  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  let err = engine
    .resolve("dev", "gateway")
    .expect_err("resolve should fail");

  assert_eq!(err.code(), CONFIGERR_UNKNOWNFIELD);
  assert!(format!("{err}").contains("services.gateway.servr"));
}

#[test]
fn engine_should_reject_unknown_service_template_infra_field() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");

  fs::create_dir_all(config_dir.join("template/infra")).expect("create template infra dir");
  fs::create_dir_all(config_dir.join("template/service")).expect("create template service dir");
  fs::create_dir_all(config_dir.join("profile")).expect("create profile dir");

  fs::write(
    config_dir.join("template/infra/log.toml"),
    "[log]\nlevel = \"INFO\"\n",
  )
  .expect("write infra log");
  fs::write(
    config_dir.join("template/service/gateway.toml"),
    "[infra.loag]\nlevel = \"DEBUG\"\n",
  )
  .expect("write invalid gateway template");
  fs::write(config_dir.join("profile/dev.toml"), "").expect("write dev profile");

  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  let err = engine
    .resolve("dev", "gateway")
    .expect_err("resolve should fail");

  assert_eq!(err.code(), CONFIGERR_UNKNOWNFIELD);
  assert!(format!("{err}").contains("template.service.gateway.infra.loag"));
}

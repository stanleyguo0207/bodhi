use std::fs;

use bodhi_config::prelude::*;
use bodhi_error::errcode::BODHIERR_SYS;
use tempfile::tempdir;

#[test]
fn engine_should_render_rust_types_for_gateway_config() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");

  fs::create_dir_all(config_dir.join("template/infra")).expect("create template infra dir");
  fs::create_dir_all(config_dir.join("template/service")).expect("create template service dir");
  fs::create_dir_all(config_dir.join("profile")).expect("create profile dir");

  fs::write(
    config_dir.join("template/infra/log.toml"),
    "[log]\nformat = \"json\"\nlevel = \"INFO\"\noutput = \"stderr\"\n",
  )
  .expect("write infra log");
  fs::write(
    config_dir.join("template/infra/metrics.toml"),
    "[metrics]\nbind = \"127.0.0.1:9090\"\nenabled = true\n",
  )
  .expect("write infra metrics");
  fs::write(
    config_dir.join("template/infra/net.toml"),
    "[net]\nconnect_timeout_ms = 1000\nlisten_host = \"0.0.0.0\"\nrequest_timeout_ms = 3000\n",
  )
  .expect("write infra net");
  fs::write(
    config_dir.join("template/infra/service.toml"),
    "[service]\nname = \"default\"\nshutdown_timeout_ms = 5000\n",
  )
  .expect("write infra service");
  fs::write(
    config_dir.join("template/service/gateway.toml"),
    "[infra.service]\nname = \"gateway\"\n[server]\ngrpc_port = 50051\nhttp_port = 18080\n[routes]\nprefix = \"/api/v1\"\n",
  )
  .expect("write gateway template");
  fs::write(config_dir.join("profile/dev.toml"), "").expect("write dev profile");

  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  let code = engine
    .render_rust_types("dev", "gateway")
    .expect("render rust types");

  assert!(code.contains("pub mod merged"));
  assert!(code.contains("pub mod infra"));
  assert!(code.contains("pub mod service"));
  assert!(code.contains("pub struct Config"));
  assert!(code.contains("pub use merged::Config"));
  assert!(code.contains("pub log: LogConfig"));
  assert!(code.contains("pub struct ServerConfig"));
  assert!(code.contains("pub grpc_port: u16"));
  assert!(code.contains("pub http_port: u16"));
  assert!(code.contains("pub connect_timeout_ms: u64"));
  assert!(code.contains("pub shutdown_timeout_ms: u64"));
}

#[test]
fn engine_should_render_service_rust_types_without_profile() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");

  fs::create_dir_all(config_dir.join("template/infra")).expect("create template infra dir");
  fs::create_dir_all(config_dir.join("template/service")).expect("create template service dir");

  fs::write(
    config_dir.join("template/infra/log.toml"),
    "[log]\nformat = \"json\"\nlevel = \"INFO\"\noutput = \"stderr\"\n",
  )
  .expect("write infra log");
  fs::write(
    config_dir.join("template/infra/service.toml"),
    "[service]\nname = \"default\"\nshutdown_timeout_ms = 5000\n",
  )
  .expect("write infra service");
  fs::write(
    config_dir.join("template/service/lobby.toml"),
    "[infra.service]\nname = \"lobby\"\n[server]\ngrpc_port = 50052\nhttp_port = 18081\n[matchmaking]\nmax_rooms = 1024\ntick_ms = 200\n",
  )
  .expect("write lobby template");

  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  let code = engine
    .render_service_rust_types("lobby")
    .expect("render rust types");

  assert!(code.contains("pub mod infra"));
  assert!(code.contains("pub mod service"));
  assert!(code.contains("pub struct Config"));
  assert!(code.contains("pub matchmaking: MatchmakingConfig"));
  assert!(code.contains("pub max_rooms: u64"));
  assert!(code.contains("pub shutdown_timeout_ms: u64"));
}

#[test]
fn engine_should_write_rust_types_to_file() {
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

  let output_path = tempdir.path().join("generated/gateway_config.rs");
  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  engine
    .generate_rust_types("dev", "gateway", &output_path)
    .expect("generate rust types file");

  let content = fs::read_to_string(&output_path).expect("read generated rust types file");
  assert!(content.contains("pub mod infra"));
  assert!(content.contains("pub mod service"));
  assert!(content.contains("pub struct Config"));
  assert!(content.contains("pub use merged::Config"));
}

#[test]
fn engine_should_write_rust_types_for_all_services() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");

  fs::create_dir_all(config_dir.join("template/infra")).expect("create template infra dir");
  fs::create_dir_all(config_dir.join("template/service")).expect("create template service dir");
  fs::create_dir_all(config_dir.join("profile")).expect("create profile dir");

  fs::write(
    config_dir.join("template/infra/service.toml"),
    "[service]\nname = \"default\"\n",
  )
  .expect("write infra service");
  fs::write(
    config_dir.join("template/service/gateway.toml"),
    "[infra.service]\nname = \"gateway\"\n[server]\nhttp_port = 18080\n",
  )
  .expect("write gateway template");
  fs::write(
    config_dir.join("template/service/lobby.toml"),
    "[infra.service]\nname = \"lobby\"\n[server]\nhttp_port = 18081\n",
  )
  .expect("write lobby template");
  fs::write(config_dir.join("profile/dev.toml"), "").expect("write dev profile");

  let output_dir = tempdir.path().join("generated");
  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  engine
    .generate_all_rust_types("dev", &output_dir)
    .expect("generate rust types for all services");

  assert!(output_dir.join("gateway_config.rs").is_file());
  assert!(output_dir.join("lobby_config.rs").is_file());
}

#[test]
fn engine_should_write_rust_types_for_prefixed_services() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");

  fs::create_dir_all(config_dir.join("template/infra")).expect("create template infra dir");
  fs::create_dir_all(config_dir.join("template/service")).expect("create template service dir");
  fs::create_dir_all(config_dir.join("profile")).expect("create profile dir");

  fs::write(
    config_dir.join("template/infra/service.toml"),
    "[service]\nname = \"default\"\n",
  )
  .expect("write infra service");
  fs::write(
    config_dir.join("template/service/gateway.toml"),
    "[infra.service]\nname = \"gateway\"\n[server]\nhttp_port = 18080\n",
  )
  .expect("write gateway template");
  fs::write(
    config_dir.join("template/service/lobby.toml"),
    "[infra.service]\nname = \"lobby\"\n[server]\nhttp_port = 18081\n",
  )
  .expect("write lobby template");
  fs::write(config_dir.join("profile/dev.toml"), "").expect("write dev profile");

  let output_dir = tempdir.path().join("generated");
  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  engine
    .generate_prefixed_rust_types("dev", "gate", &output_dir)
    .expect("generate prefixed rust types");

  assert!(output_dir.join("gateway_config.rs").is_file());
  assert!(!output_dir.join("lobby_config.rs").exists());
}

#[test]
fn engine_should_reject_unknown_service_prefix() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");

  fs::create_dir_all(config_dir.join("template/service")).expect("create template service dir");
  fs::create_dir_all(config_dir.join("profile")).expect("create profile dir");

  fs::write(config_dir.join("template/service/gateway.toml"), "").expect("write gateway template");
  fs::write(config_dir.join("profile/dev.toml"), "").expect("write dev profile");

  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  let err = engine
    .services_with_prefix("missing")
    .expect_err("service prefix should not match any service");

  assert_eq!(err.code(), CONFIGERR_SERVICENOTFOUND);
  assert!(format!("{err}").contains("service_prefix=missing"));
}

#[test]
fn engine_should_apply_type_override_rules_from_file() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");

  fs::create_dir_all(config_dir.join("template/infra")).expect("create template infra dir");
  fs::create_dir_all(config_dir.join("template/service")).expect("create template service dir");
  fs::create_dir_all(config_dir.join("profile")).expect("create profile dir");

  fs::write(
    config_dir.join("template/infra/service.toml"),
    "[service]\nname = \"default\"\nshutdown_timeout_ms = 5000\nrequest_timeout_ms = 3000\n",
  )
  .expect("write infra service");
  fs::write(
    config_dir.join("template/service/gateway.toml"),
    "[infra.service]\nname = \"gateway\"\n[session]\nid = 1\n[server]\nhttp_port = 18080\n",
  )
  .expect("write gateway template");
  fs::write(config_dir.join("profile/dev.toml"), "").expect("write dev profile");

  let type_rules_path = tempdir.path().join("type_overrides.toml");
  fs::write(
    &type_rules_path,
    "[field_types]\nid = \"u64\"\n\n[suffix_types]\ntimeout_ms = \"u32\"\n\n[path_types]\n\"server.http_port\" = \"u32\"\n",
  )
  .expect("write type override rules");

  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  let code = engine
    .render_rust_types_with(
      "dev",
      "gateway",
      &RustCodegenOptions {
        root_struct_name: String::from("Config"),
        type_overrides: TypeOverrideRules::from_file(&type_rules_path)
          .expect("load type override rules"),
      },
    )
    .expect("render rust types with overrides");

  assert!(code.contains("pub id: u64"));
  assert!(code.contains("pub shutdown_timeout_ms: u32"));
  assert!(code.contains("pub request_timeout_ms: u32"));
  assert!(code.contains("pub http_port: u32"));
}

#[test]
fn engine_should_apply_wildcard_path_override_rules() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");

  fs::create_dir_all(config_dir.join("template/infra")).expect("create template infra dir");
  fs::create_dir_all(config_dir.join("template/service")).expect("create template service dir");
  fs::create_dir_all(config_dir.join("profile")).expect("create profile dir");

  fs::write(
    config_dir.join("template/infra/net.toml"),
    "[net]\nconnect_timeout_ms = 1000\nrequest_timeout_ms = 3000\n",
  )
  .expect("write infra net");
  fs::write(
    config_dir.join("template/infra/service.toml"),
    "[service]\nshutdown_timeout_ms = 5000\n",
  )
  .expect("write infra service");
  fs::write(
    config_dir.join("template/service/gateway.toml"),
    "[server]\nhttp_port = 18080\n",
  )
  .expect("write gateway template");
  fs::write(config_dir.join("profile/dev.toml"), "").expect("write dev profile");

  let type_rules_path = tempdir.path().join("type_overrides.toml");
  fs::write(
    &type_rules_path,
    "[path_types]\n\"**.*timeout_ms\" = \"u32\"\n",
  )
  .expect("write wildcard type override rules");

  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  let code = engine
    .render_rust_types_with(
      "dev",
      "gateway",
      &RustCodegenOptions {
        root_struct_name: String::from("Config"),
        type_overrides: TypeOverrideRules::from_file(&type_rules_path)
          .expect("load wildcard type override rules"),
      },
    )
    .expect("render rust types with wildcard overrides");

  assert!(code.contains("pub connect_timeout_ms: u32"));
  assert!(code.contains("pub request_timeout_ms: u32"));
  assert!(code.contains("pub shutdown_timeout_ms: u32"));
}

#[test]
fn type_override_rules_should_reject_invalid_rust_type_expr() {
  let tempdir = tempdir().expect("create tempdir");
  let type_rules_path = tempdir.path().join("type_overrides.toml");

  fs::write(&type_rules_path, "[field_types]\nid = \"Vec<u32\"\n")
    .expect("write invalid type rules");

  let err = TypeOverrideRules::from_file(&type_rules_path)
    .expect_err("invalid rust type expression should fail");

  assert_eq!(err.code(), BODHIERR_SYS);
  assert!(format!("{err}").contains("invalid Rust type expression"));
}

#[test]
fn engine_should_report_matched_type_override_rules() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");

  fs::create_dir_all(config_dir.join("template/infra")).expect("create template infra dir");
  fs::create_dir_all(config_dir.join("template/service")).expect("create template service dir");
  fs::create_dir_all(config_dir.join("profile")).expect("create profile dir");

  fs::write(
    config_dir.join("template/infra/service.toml"),
    "[service]\nid = 1\nshutdown_timeout_ms = 5000\n",
  )
  .expect("write infra service");
  fs::write(
    config_dir.join("template/service/gateway.toml"),
    "[server]\nhttp_port = 18080\nrequest_timeout_ms = 3000\n",
  )
  .expect("write gateway template");
  fs::write(config_dir.join("profile/dev.toml"), "").expect("write dev profile");

  let type_rules_path = tempdir.path().join("type_overrides.toml");
  fs::write(
    &type_rules_path,
    "[field_types]\nid = \"u64\"\n\n[suffix_types]\ntimeout_ms = \"u32\"\n\n[path_types]\n\"server.http_port\" = \"u32\"\n",
  )
  .expect("write type override rules");

  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  let report = engine
    .render_rust_types_report_with(
      "dev",
      "gateway",
      &RustCodegenOptions {
        root_struct_name: String::from("Config"),
        type_overrides: TypeOverrideRules::from_file(&type_rules_path)
          .expect("load type override rules"),
      },
    )
    .expect("render rust types report");

  assert!(report.matched_rules.iter().any(|hit| {
    hit.field_path == "service.id"
      && hit.rust_type == "u64"
      && hit.rule_key == "id"
      && hit.rule_source == TypeOverrideSource::Field
  }));
  assert!(report.matched_rules.iter().any(|hit| {
    hit.field_path == "service.shutdown_timeout_ms"
      && hit.rust_type == "u32"
      && hit.rule_key == "timeout_ms"
      && hit.rule_source == TypeOverrideSource::Suffix
  }));
  assert!(report.matched_rules.iter().any(|hit| {
    hit.field_path == "server.http_port"
      && hit.rust_type == "u32"
      && hit.rule_key == "server.http_port"
      && hit.rule_source == TypeOverrideSource::ExactPath
  }));
}

#[test]
fn engine_should_report_unused_type_override_rules() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");

  fs::create_dir_all(config_dir.join("template/infra")).expect("create template infra dir");
  fs::create_dir_all(config_dir.join("template/service")).expect("create template service dir");
  fs::create_dir_all(config_dir.join("profile")).expect("create profile dir");

  fs::write(
    config_dir.join("template/infra/service.toml"),
    "[service]\nid = 1\nshutdown_timeout_ms = 5000\n",
  )
  .expect("write infra service");
  fs::write(
    config_dir.join("template/service/gateway.toml"),
    "[server]\nhttp_port = 18080\nrequest_timeout_ms = 3000\n",
  )
  .expect("write gateway template");
  fs::write(config_dir.join("profile/dev.toml"), "").expect("write dev profile");

  let type_rules_path = tempdir.path().join("type_overrides.toml");
  fs::write(
    &type_rules_path,
    "[field_types]\nid = \"u64\"\ntrace_id = \"u64\"\n\n[suffix_types]\ntimeout_ms = \"u32\"\n\n[path_types]\n\"server.http_port\" = \"u32\"\n\"server.grpc_port\" = \"u16\"\n",
  )
  .expect("write type override rules");

  let engine = ConfigEngine::new(&config_dir).expect("create config engine");
  let report = engine
    .render_rust_types_report_with(
      "dev",
      "gateway",
      &RustCodegenOptions {
        root_struct_name: String::from("Config"),
        type_overrides: TypeOverrideRules::from_file(&type_rules_path)
          .expect("load type override rules"),
      },
    )
    .expect("render rust types report");

  assert!(report.unused_rules.iter().any(|rule| {
    rule.rule_key == "trace_id"
      && rule.rust_type == "u64"
      && rule.rule_source == TypeOverrideSource::Field
  }));
  assert!(report.unused_rules.iter().any(|rule| {
    rule.rule_key == "server.grpc_port"
      && rule.rust_type == "u16"
      && rule.rule_source == TypeOverrideSource::ExactPath
  }));
  assert!(
    !report
      .unused_rules
      .iter()
      .any(|rule| rule.rule_key == "id" && rule.rule_source == TypeOverrideSource::Field)
  );
}

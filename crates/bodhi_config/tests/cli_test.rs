use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

#[test]
fn gen_rust_should_write_text_report_to_file() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");
  let output_dir = tempdir.path().join("generated");
  let report_path = tempdir.path().join("reports/rules.txt");
  let type_rules_path = tempdir.path().join("type_overrides.toml");

  write_cli_test_config(&config_dir);
  fs::write(
    &type_rules_path,
    "[field_types]\nid = \"u64\"\n\n[path_types]\n\"server.http_port\" = \"u32\"\n",
  )
  .expect("write type rules");

  let output = Command::new(env!("CARGO_BIN_EXE_bodhi_config"))
    .arg("--config-dir")
    .arg(&config_dir)
    .arg("gen-rust")
    .arg("--profile")
    .arg("dev")
    .arg("--output")
    .arg(&output_dir)
    .arg("--type-rules")
    .arg(&type_rules_path)
    .arg("--report-output")
    .arg(&report_path)
    .output()
    .expect("run bodhi_config gen-rust");

  assert!(
    output.status.success(),
    "stderr={}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert_eq!(String::from_utf8_lossy(&output.stdout), "");

  let report = fs::read_to_string(&report_path).expect("read text report");
  assert!(report.contains("matched rules for gateway:"));
  assert!(report.contains("matched rules for lobby:"));
  assert!(report.contains("unused rules:"));
}

#[test]
fn gen_rust_json_report_should_include_per_service_and_global_unused_views() {
  let tempdir = tempdir().expect("create tempdir");
  let config_dir = tempdir.path().join("config");
  let output_dir = tempdir.path().join("generated");
  let report_path = tempdir.path().join("reports/rules.json");
  let type_rules_path = tempdir.path().join("type_overrides.toml");

  write_cli_test_config(&config_dir);
  fs::write(
    &type_rules_path,
    concat!(
      "[field_types]\n",
      "id = \"u64\"\n",
      "trace_id = \"u64\"\n\n",
      "[path_types]\n",
      "\"server.http_port\" = \"u32\"\n",
      "\"server.grpc_port\" = \"u16\"\n"
    ),
  )
  .expect("write type rules");

  let output = Command::new(env!("CARGO_BIN_EXE_bodhi_config"))
    .arg("--config-dir")
    .arg(&config_dir)
    .arg("gen-rust")
    .arg("--profile")
    .arg("dev")
    .arg("--output")
    .arg(&output_dir)
    .arg("--type-rules")
    .arg(&type_rules_path)
    .arg("--report-format")
    .arg("json")
    .arg("--report-output")
    .arg(&report_path)
    .output()
    .expect("run bodhi_config gen-rust");

  assert!(
    output.status.success(),
    "stderr={}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert_eq!(String::from_utf8_lossy(&output.stdout), "");

  let report: Value =
    serde_json::from_str(&fs::read_to_string(&report_path).expect("read json report"))
      .expect("parse json report");

  assert_eq!(report["profile"], "dev");

  let gateway = find_generated_service(&report, "gateway");
  let lobby = find_generated_service(&report, "lobby");

  assert!(unused_rule_keys(gateway).contains(&"trace_id".to_string()));
  assert!(!unused_rule_keys(gateway).contains(&"server.grpc_port".to_string()));

  assert!(unused_rule_keys(lobby).contains(&"trace_id".to_string()));
  assert!(unused_rule_keys(lobby).contains(&"server.grpc_port".to_string()));

  let global_unused = unused_rule_keys(&report);
  assert!(global_unused.contains(&"trace_id".to_string()));
  assert!(!global_unused.contains(&"server.grpc_port".to_string()));
}

fn write_cli_test_config(config_dir: &Path) {
  fs::create_dir_all(config_dir.join("template/infra")).expect("create template infra dir");
  fs::create_dir_all(config_dir.join("template/service")).expect("create template service dir");
  fs::create_dir_all(config_dir.join("profile")).expect("create profile dir");

  fs::write(
    config_dir.join("template/infra/service.toml"),
    "[service]\nid = 1\nname = \"default\"\n",
  )
  .expect("write infra service");
  fs::write(
    config_dir.join("template/service/gateway.toml"),
    "[server]\nhttp_port = 18080\ngrpc_port = 50051\n",
  )
  .expect("write gateway template");
  fs::write(
    config_dir.join("template/service/lobby.toml"),
    "[server]\nhttp_port = 18081\n",
  )
  .expect("write lobby template");
  fs::write(config_dir.join("profile/dev.toml"), "").expect("write dev profile");
}

fn find_generated_service<'a>(report: &'a Value, service: &str) -> &'a Value {
  report["generated"]
    .as_array()
    .expect("generated should be an array")
    .iter()
    .find(|item| item["service"] == service)
    .expect("service report should exist")
}

fn unused_rule_keys(report: &Value) -> Vec<String> {
  report["unused_rules"]
    .as_array()
    .expect("unused_rules should be an array")
    .iter()
    .map(|rule| {
      rule["rule_key"]
        .as_str()
        .expect("rule_key should be a string")
        .to_string()
    })
    .collect()
}

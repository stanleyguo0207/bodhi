//! 配置解析模块

use std::collections::BTreeMap;
use std::path::Path;

use bodhi_error::prelude::*;
use toml::Value;

use crate::engine::ResolvedConfig;
use crate::errcode::configerr::*;
use crate::loader::{load_infra_configs, load_profile, load_service_templates};
use crate::merge::deep_merge;
use crate::validate::{service_schema, validate_profile, validate_service_template};

pub fn resolve(config_dir: &Path, profile: &str, service: &str) -> Result<ResolvedConfig> {
  let base_infra = load_infra_configs(config_dir)?;
  let profile_cfg = load_profile(config_dir, profile)?;
  let service_templates = load_service_templates(config_dir)?;
  let service_cfg = service_templates.get(service).ok_or_else(|| {
    Error::new(CONFIGERR_SERVICENOTFOUND)
      .wrap_context("resolve target service not found")
      .wrap_context_with(|| format!("service={service}"))
  })?;

  validate_templates(&base_infra, &service_templates)?;
  validate_profile(profile, &profile_cfg, &base_infra, &service_templates)?;

  let profile_infra = clone_path(&profile_cfg, &["infra"]);
  let service_infra = clone_path(service_cfg, &["infra"]);
  let profile_service_infra = clone_path(&profile_cfg, &["services", service, "infra"]);

  let mut merged_infra = base_infra;
  if let Some(value) = profile_infra.as_ref() {
    deep_merge(&mut merged_infra, value);
  }
  if let Some(value) = service_infra.as_ref() {
    deep_merge(&mut merged_infra, value);
  }
  if let Some(value) = profile_service_infra.as_ref() {
    deep_merge(&mut merged_infra, value);
  }

  let mut merged_service = service_schema(service_cfg);
  let profile_service =
    clone_path(&profile_cfg, &["services", service]).unwrap_or_else(empty_table);
  let profile_service_without_infra = strip_top_level_key(&profile_service, "infra");
  deep_merge(&mut merged_service, &profile_service_without_infra);

  let mut final_value = merged_infra;
  deep_merge(&mut final_value, &merged_service);

  if !matches!(final_value, Value::Table(_)) {
    return Err(Error::new(CONFIGERR_MERGEFAILED).wrap_context("resolved config is not a table"));
  }

  Ok(ResolvedConfig::new(final_value))
}

fn validate_templates(
  base_infra: &Value,
  service_templates: &BTreeMap<String, Value>,
) -> Result<()> {
  for (service, service_cfg) in service_templates {
    validate_service_template(service, base_infra, service_cfg)?;
  }

  Ok(())
}

fn clone_path(root: &Value, path: &[&str]) -> Option<Value> {
  let mut current = root;
  for segment in path {
    let table = current.as_table()?;
    current = table.get(*segment)?;
  }
  Some(current.clone())
}

fn strip_top_level_key(root: &Value, key: &str) -> Value {
  match root {
    Value::Table(table) => {
      let mut cloned = table.clone();
      cloned.remove(key);
      Value::Table(cloned)
    }
    _ => empty_table(),
  }
}

fn empty_table() -> Value {
  Value::Table(Default::default())
}

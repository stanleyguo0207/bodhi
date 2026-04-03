//! 配置校验模块

use std::collections::BTreeMap;

use bodhi_error::prelude::*;
use toml::Value;

use crate::errcode::configerr::*;

pub fn validate_service_template(
  service: &str,
  base_infra: &Value,
  service_cfg: &Value,
) -> Result<()> {
  let service_table = expect_table(
    service_cfg,
    &format!("template.service.{service}"),
    "service template root must be a table",
  )?;

  if let Some(infra) = service_table.get("infra") {
    validate_overlay(
      infra,
      base_infra,
      &format!("template.service.{service}.infra"),
    )?;
  }

  Ok(())
}

pub fn validate_profile(
  profile: &str,
  profile_cfg: &Value,
  base_infra: &Value,
  service_templates: &BTreeMap<String, Value>,
) -> Result<()> {
  let profile_table = expect_table(
    profile_cfg,
    &format!("profile.{profile}"),
    "profile root must be a table",
  )?;

  for (key, value) in profile_table {
    match key.as_str() {
      "infra" => validate_overlay(value, base_infra, &format!("profile.{profile}.infra"))?,
      "services" => validate_profile_services(profile, value, base_infra, service_templates)?,
      _ => {
        return unknown_field(
          &format!("profile.{profile}.{key}"),
          "profile root only allows infra and services",
        );
      }
    }
  }

  Ok(())
}

pub fn service_schema(service_cfg: &Value) -> Value {
  match service_cfg {
    Value::Table(table) => {
      let mut cloned = table.clone();
      cloned.remove("infra");
      Value::Table(cloned)
    }
    _ => Value::Table(Default::default()),
  }
}

fn validate_profile_services(
  profile: &str,
  services_value: &Value,
  base_infra: &Value,
  service_templates: &BTreeMap<String, Value>,
) -> Result<()> {
  let services_table = expect_table(
    services_value,
    &format!("profile.{profile}.services"),
    "profile services must be a table",
  )?;

  for (service, service_override) in services_table {
    let Some(service_template) = service_templates.get(service) else {
      return Err(
        Error::new(CONFIGERR_SERVICENOTFOUND)
          .wrap_context("profile references unknown service")
          .wrap_context_with(|| format!("profile={profile} service={service}")),
      );
    };

    let service_override_table = expect_table(
      service_override,
      &format!("profile.{profile}.services.{service}"),
      "profile service override must be a table",
    )?;
    let service_schema = service_schema(service_template);
    let service_schema_table = expect_table(
      &service_schema,
      &format!("template.service.{service}"),
      "service schema must be a table",
    )?;

    for (key, value) in service_override_table {
      if key == "infra" {
        validate_overlay(
          value,
          base_infra,
          &format!("profile.{profile}.services.{service}.infra"),
        )?;
        continue;
      }

      let Some(schema_value) = service_schema_table.get(key) else {
        return unknown_field(
          &format!("profile.{profile}.services.{service}.{key}"),
          "field not found in service template",
        );
      };

      validate_overlay(
        value,
        schema_value,
        &format!("profile.{profile}.services.{service}.{key}"),
      )?;
    }
  }

  Ok(())
}

fn validate_overlay(overlay: &Value, schema: &Value, path: &str) -> Result<()> {
  match (overlay, schema) {
    (Value::Table(overlay_table), Value::Table(schema_table)) => {
      for (key, overlay_value) in overlay_table {
        let Some(schema_value) = schema_table.get(key) else {
          return unknown_field(&format!("{path}.{key}"), "field not found in schema");
        };

        validate_overlay(overlay_value, schema_value, &format!("{path}.{key}"))?;
      }

      Ok(())
    }
    _ if same_kind(overlay, schema) => Ok(()),
    _ => Err(
      Error::new(CONFIGERR_TYPEMISMATCH)
        .wrap_context("config value type mismatched")
        .wrap_context_with(|| {
          format!(
            "path={path} overlay_type={} schema_type={}",
            value_kind(overlay),
            value_kind(schema)
          )
        }),
    ),
  }
}

fn expect_table<'a>(
  value: &'a Value,
  path: &str,
  reason: &str,
) -> Result<&'a toml::map::Map<String, Value>> {
  value
    .as_table()
    .ok_or_else(|| Error::new(CONFIGERR_INVALIDSTRUCTURE))
    .wrap_context(reason)
    .wrap_context_with(|| format!("path={path}"))
}

fn unknown_field<T>(path: &str, reason: &str) -> Result<T> {
  Err(
    Error::new(CONFIGERR_UNKNOWNFIELD)
      .wrap_context(reason)
      .wrap_context_with(|| format!("path={path}")),
  )
}

fn same_kind(left: &Value, right: &Value) -> bool {
  matches!(
    (left, right),
    (Value::String(_), Value::String(_))
      | (Value::Integer(_), Value::Integer(_))
      | (Value::Float(_), Value::Float(_))
      | (Value::Boolean(_), Value::Boolean(_))
      | (Value::Datetime(_), Value::Datetime(_))
      | (Value::Array(_), Value::Array(_))
  )
}

fn value_kind(value: &Value) -> &'static str {
  match value {
    Value::String(_) => "string",
    Value::Integer(_) => "integer",
    Value::Float(_) => "float",
    Value::Boolean(_) => "boolean",
    Value::Datetime(_) => "datetime",
    Value::Array(_) => "array",
    Value::Table(_) => "table",
  }
}

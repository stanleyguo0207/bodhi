//! 配置输出模块

use std::fmt;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use bodhi_error::prelude::*;
use serde_json::Value as JsonValue;
use toml::Value;

use crate::errcode::configerr::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputFormat {
  Toml,
  Yaml,
  Json,
}

impl OutputFormat {
  pub fn all() -> &'static [Self] {
    &[Self::Toml, Self::Yaml, Self::Json]
  }

  pub fn as_str(self) -> &'static str {
    match self {
      Self::Toml => "toml",
      Self::Yaml => "yaml",
      Self::Json => "json",
    }
  }

  pub fn extension(self) -> &'static str {
    match self {
      Self::Toml => "toml",
      Self::Yaml => "yaml",
      Self::Json => "json",
    }
  }
}

impl fmt::Display for OutputFormat {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl FromStr for OutputFormat {
  type Err = Error;

  fn from_str(value: &str) -> Result<Self> {
    match value {
      "toml" => Ok(Self::Toml),
      "yaml" | "yml" => Ok(Self::Yaml),
      "json" => Ok(Self::Json),
      _ => Err(
        Error::new(CONFIGERR_UNSUPPORTEDFORMAT)
          .wrap_context_with(|| format!("format={value} unsupported")),
      ),
    }
  }
}

pub fn serialize_value(value: &Value, format: OutputFormat) -> Result<String> {
  match format {
    OutputFormat::Toml => toml::to_string_pretty(value)
      .map_err(Error::from_std)
      .wrap_context("serialize config to toml failed"),
    OutputFormat::Json => serde_json::to_string_pretty(value)
      .map_err(Error::from_std)
      .wrap_context("serialize config to json failed"),
    OutputFormat::Yaml => {
      let json_value: JsonValue = serde_json::to_value(value)
        .map_err(Error::from_std)
        .wrap_context("convert config to json value failed")?;
      serde_yml::to_string(&json_value)
        .map_err(Error::from_std)
        .wrap_context("serialize config to yaml failed")
    }
  }
}

pub fn write_product(
  config_dir: &Path,
  profile: &str,
  service: &str,
  value: &Value,
  format: OutputFormat,
) -> Result<()> {
  let product_dir = config_dir
    .join("product")
    .join(profile)
    .join(format.as_str());
  fs::create_dir_all(&product_dir)
    .map_err(Error::from_std)
    .wrap_context("create product directory failed")
    .wrap_context_with(|| format!("dir={}", product_dir.display()))?;

  let path = product_dir.join(format!("{service}.{}", format.extension()));
  let content = serialize_value(value, format)?;

  fs::write(&path, content)
    .map_err(Error::from_std)
    .wrap_context("write product file failed")
    .wrap_context_with(|| format!("path={}", path.display()))
}

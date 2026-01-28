use std::{fs, path::Path};
use toml::Value;

use bodhi::{Result, WrapContext};

pub fn render(value: &Value, format: &str, path: &Path) -> Result<()> {
  let content = match format {
    "toml" => toml::to_string_pretty(value).unwrap(),
    "json" => serde_json::to_string_pretty(value).unwrap(),
    "yaml" => serde_yaml::to_string(value).unwrap(),
    _ => unreachable!(),
  };

  fs::write(path, content).wrap_context_with(|| format!("Failed to write to {:?}", path))?;

  Ok(())
}

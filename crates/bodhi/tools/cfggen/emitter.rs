use std::{fs, path::Path};

use bodhi::Result;
use toml::Value;

pub fn emit(svc: &str, value: &Value, out_path: &Path, formats: &[String]) -> Result<()> {
  for fmt in formats {
    let file_path = out_path.join(format!("{svc}.{fmt}"));

    let content = match fmt.as_str() {
      "json" => serde_json::to_string_pretty(value)?,
      "yaml" => serde_yaml::to_string(value)?,
      _ => toml::to_string_pretty(value)?,
    };

    fs::write(file_path, content)?;
  }

  Ok(())
}

use std::{collections::HashMap, fs, path::Path};

use bodhi::Result;
use toml::Value;

pub fn emit(
  out: &Path,
  env: &str,
  profile: &str,
  svc: &str,
  cfg: &HashMap<String, Value>,
  fmts: &[String],
) -> Result<()> {
  let dir = out.join(env).join(profile);
  fs::create_dir_all(&dir)?;

  for fmt in fmts {
    let path = dir.join(format!("{svc}.{fmt}"));
    let content = match fmt.as_str() {
      "json" => serde_json::to_string_pretty(cfg)?,
      "yaml" => serde_yaml::to_string(cfg)?,
      _ => toml::to_string_pretty(cfg)?,
    };

    fs::write(path, content)?;
  }

  Ok(())
}

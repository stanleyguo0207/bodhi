use std::{collections::BTreeMap, fs, path::Path};

use bodhi::Result;
use toml::Value;

fn flatten(prefix: &str, value: &Value, out: &mut BTreeMap<String, Value>) {
  match value {
    Value::Table(table) => {
      for (k, v) in table {
        let new_prefix = if prefix.is_empty() {
          k.clone()
        } else {
          format!("{prefix}.{k}")
        };
        flatten(&new_prefix, v, out);
      }
    }
    _ => {
      out.insert(prefix.to_string(), value.clone());
    }
  }
}

pub fn load_flat(path: &Path) -> Result<BTreeMap<String, Value>> {
  let content = fs::read_to_string(path)?;
  let value: Value = toml::from_str(&content)?;
  let mut out = BTreeMap::new();
  flatten("", &value, &mut out);
  Ok(out)
}

pub fn load_flat_dir(dir: &Path) -> Result<BTreeMap<String, Value>> {
  let mut merged = BTreeMap::new();

  let mut entries: Vec<_> = fs::read_dir(dir)?
    .filter_map(core::result::Result::ok)
    .map(|e| e.path())
    .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("toml"))
    .collect();

  entries.sort();

  for path in entries {
    let content = fs::read_to_string(&path)?;
    let value: Value = toml::from_str(&content)?;
    flatten("", &value, &mut merged);
  }

  Ok(merged)
}

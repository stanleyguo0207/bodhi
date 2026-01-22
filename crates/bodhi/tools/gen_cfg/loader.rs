use std::{collections::HashMap, fs, path::Path};

use bodhi::Result;
use toml::Value;

pub type TomlMap = HashMap<String, Value>;

fn load_dir_recursive(path: &Path) -> Result<TomlMap> {
  let mut map = TomlMap::new();

  for entry in walkdir::WalkDir::new(path) {
    let entry = entry?;
    if entry.path().extension() == Some("toml".as_ref()) {
      let content = fs::read_to_string(entry.path())?;
      let value = toml::from_str(&content)?;
      let key = entry
        .path()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();
      map.insert(key, value);
    }
  }

  Ok(map)
}

pub fn load_templates(dir: &Path) -> Result<TomlMap> {
  load_dir_recursive(dir)
}

pub fn load_custom_config(path: &Path) -> Result<TomlMap> {
  load_dir_recursive(path)
}

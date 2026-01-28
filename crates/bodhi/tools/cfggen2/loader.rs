use crate::util::deep_merge;
use std::{fs, path::Path};
use toml::Value;

/// 加载 template/infra（只信 TOML 内容，不用文件名）
pub fn load_infra(dir: &Path) -> Value {
  let mut infra = Value::Table(Default::default());

  for entry in fs::read_dir(dir).unwrap() {
    let path = entry.unwrap().path();
    if path.extension().and_then(|s| s.to_str()) != Some("toml") {
      continue;
    }

    let content = fs::read_to_string(&path).unwrap();
    let value: Value = toml::from_str(&content).unwrap();

    deep_merge(&mut infra, &value);
  }

  infra
}

/// 加载 template/service
pub fn load_services(dir: &Path) -> Vec<(String, Value)> {
  let mut out = vec![];

  for entry in fs::read_dir(dir).unwrap() {
    let path = entry.unwrap().path();
    if path.extension().and_then(|s| s.to_str()) != Some("toml") {
      continue;
    }

    let name = path.file_stem().unwrap().to_str().unwrap().to_string();
    let content = fs::read_to_string(&path).unwrap();
    let value: Value = toml::from_str(&content).unwrap();

    out.push((name, value));
  }

  out
}

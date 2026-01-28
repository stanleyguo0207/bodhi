use std::collections::BTreeMap;

use toml::Value;

fn insert_path(root: &mut Value, path: &[&str], value: Value) {
  let mut cur = root;

  for key in &path[..path.len() - 1] {
    cur = cur
      .as_table_mut()
      .expect("root must be table")
      .entry(key.to_string())
      .or_insert_with(|| Value::Table(Default::default()));
  }

  cur
    .as_table_mut()
    .unwrap()
    .insert(path.last().unwrap().to_string(), value);
}

pub fn merge_kv(root: &mut Value, kv: &BTreeMap<String, Value>) {
  for (key, value) in kv {
    let parts: Vec<&str> = key.split('.').collect();
    insert_path(root, &parts, value.clone());
  }
}

pub fn normalize_service_override(
  service: &str,
  kv: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
  let mut out = BTreeMap::new();
  let prefix = format!("{service}.infra.");

  for (key, value) in kv {
    if let Some(stripped) = key.strip_prefix(&prefix) {
      out.insert(format!("infra.{stripped}"), value.clone());
    }
  }

  out
}

pub fn normalize_service_keys(
  service: &str,
  kv: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
  let mut out = BTreeMap::new();

  for (key, value) in kv {
    if key.starts_with(&format!("{service}.")) {
      out.insert(key.clone(), value.clone());
    } else {
      out.insert(format!("{service}.{key}"), value.clone());
    }
  }

  out
}

pub fn assemble_root(svc: Value, infra: Value) -> Value {
  let mut root = toml::map::Map::new();

  if let Value::Table(tbl) = svc {
    for (k, v) in tbl {
      root.insert(k, v);
    }
  }

  if let Value::Table(tbl) = infra {
    for (k, v) in tbl {
      root.insert(k, v);
    }
  }

  Value::Table(root)
}

use toml::Value;

pub fn deep_merge(base: &mut Value, overlay: &Value) {
  match (base, overlay) {
    (Value::Table(a), Value::Table(b)) => {
      for (k, v) in b {
        match a.get_mut(k) {
          Some(existing) => deep_merge(existing, v),
          None => {
            if !is_empty(v) {
              a.insert(k.clone(), v.clone());
            }
          }
        }
      }
    }
    (base, overlay) => {
      if !is_empty(overlay) {
        *base = overlay.clone();
      }
    }
  }
}

fn is_empty(v: &Value) -> bool {
  match v {
    Value::String(s) => s.is_empty(),
    Value::Table(t) => t.is_empty(),
    _ => false,
  }
}

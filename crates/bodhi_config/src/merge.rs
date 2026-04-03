//! 配置合并模块

use toml::Value;

/// 深度合并配置值
///
/// 合并策略：
/// - Table: 递归合并
/// - Array: 整体替换
/// - Scalar: 后值覆盖前值
pub fn deep_merge(base: &mut Value, overlay: &Value) {
  match (base, overlay) {
    (Value::Table(base_table), Value::Table(overlay_table)) => {
      for (key, overlay_value) in overlay_table {
        if let Some(base_value) = base_table.get_mut(key) {
          deep_merge(base_value, overlay_value);
        } else {
          base_table.insert(key.clone(), overlay_value.clone());
        }
      }
    }
    (base_value, overlay_value) => {
      *base_value = overlay_value.clone();
    }
  }
}

/// 按顺序合并多个配置值
pub fn merge_all<I>(values: I) -> Value
where
  I: IntoIterator<Item = Value>,
{
  let mut merged = Value::Table(Default::default());
  for value in values {
    deep_merge(&mut merged, &value);
  }
  merged
}

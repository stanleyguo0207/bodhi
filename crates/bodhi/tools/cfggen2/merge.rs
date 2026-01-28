use crate::util::deep_merge;
use toml::Value;

/// 构建单个 service 的最终配置
pub fn build_service_config(
  svc: &str,
  infra_tpl: &Value,
  svc_tpl: &Value,
  profile: &Value,
) -> Value {
  // ---------- infra ----------
  let mut infra = infra_tpl.clone();

  if let Some(p) = profile.get("infra") {
    deep_merge(&mut infra, p);
  }

  if let Some(p_svc) = profile.get(svc).and_then(|v| v.get("infra")) {
    deep_merge(&mut infra, p_svc);
  }

  // ---------- service ----------
  let mut service = svc_tpl.clone();

  if let Some(p_svc) = profile.get(svc) {
    if let Some(tbl) = p_svc.as_table() {
      for (k, v) in tbl {
        if k == "infra" {
          continue;
        }
        if let Some(dst) = service.get_mut(k) {
          deep_merge(dst, v);
        }
      }
    }
  }

  // ---------- final assembly ----------
  let mut out = toml::value::Table::new();

  // 1️⃣ infra 一定在前
  out.insert("infra".into(), infra);

  // 2️⃣ service 一定在后（整体）
  out.insert(
    svc.to_string(),
    service
      .get(svc)
      .expect("service template must contain root key")
      .clone(),
  );

  Value::Table(out)
}

use std::collections::HashMap;

use bodhi::Result;
use toml::Value;

use crate::loader::Context;

pub fn merge_services(ctx: Context) -> Result<HashMap<String, HashMap<String, Value>>> {
  let mut out = HashMap::new();

  for (svc, svc_cfg) in ctx.services {
    let mut cfg = ctx.infra.clone();

    merge(&mut cfg, &svc_cfg);
    merge_prefixed(&mut cfg, &ctx.profile, "infra");
    merge_prefixed(&mut cfg, &ctx.profile, &svc);

    out.insert(svc, cfg);
  }

  Ok(out)
}

fn merge(dst: &mut HashMap<String, Value>, src: &HashMap<String, Value>) {
  for (k, v) in src {
    dst.insert(k.clone(), v.clone());
  }
}

fn merge_prefixed(dst: &mut HashMap<String, Value>, src: &HashMap<String, Value>, prefix: &str) {
  for (k, v) in src {
    if k.starts_with(prefix) {
      dst.insert(k.clone(), v.clone());
    }
  }
}

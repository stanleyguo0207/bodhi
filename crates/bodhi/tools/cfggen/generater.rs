use std::path::PathBuf;

use bodhi::Result;

use crate::{
  emitter::emit,
  loader::{load_flat, load_flat_dir},
  merge::{assemble_root, merge_kv, normalize_service_keys, normalize_service_override},
};

pub fn generate(
  services: Vec<String>,
  infra_dir: &PathBuf,
  service_dir: &PathBuf,
  profile_path: &PathBuf,
  out_dir: &PathBuf,
  formats: &[String],
) -> Result<()> {
  let infra_cfg = load_flat_dir(infra_dir)?;
  let profile_cfg = load_flat(profile_path)?;

  for svc in services {
    let svc_cfg = load_flat(&service_dir.join(format!("{svc}.toml")))?;

    let mut infra_root = toml::Value::Table(Default::default());
    let mut svc_root = toml::Value::Table(Default::default());

    // infra base
    merge_kv(&mut infra_root, &infra_cfg);

    // service infra override
    merge_kv(
      &mut infra_root,
      &normalize_service_override(&svc, &profile_cfg),
    );

    // service base
    merge_kv(&mut svc_root, &normalize_service_keys(&svc, &svc_cfg));

    // service profile override
    merge_kv(&mut svc_root, &normalize_service_keys(&svc, &profile_cfg));

    let final_cfg = assemble_root(svc_root, infra_root);

    emit(&svc, &final_cfg, out_dir, formats)?;
  }

  Ok(())
}

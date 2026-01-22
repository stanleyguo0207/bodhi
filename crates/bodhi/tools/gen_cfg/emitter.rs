use std::{fs, path::Path};

use bodhi::Result;
use toml::Value;

pub fn emit(configs: &[(String, Value)], out_dir: &Path, user: &str) -> Result<()> {
  let base = out_dir.join(user);
  fs::create_dir_all(&base)?;

  for (name, value) in configs {
    let path = base.join(format!("{}.toml", name));
    fs::write(path, toml::to_string_pretty(value)?)?;
  }

  Ok(())
}

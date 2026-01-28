use std::path::PathBuf;

use bodhi::Result;

pub fn collect_services(dir: &PathBuf) -> Result<Vec<String>> {
  let mut services = Vec::new();

  for entry in std::fs::read_dir(dir)? {
    let path = entry?.path();
    if path.extension().and_then(|s| s.to_str()) == Some("toml") {
      if let Some(fname) = path.file_stem().and_then(|s| s.to_str()) {
        services.push(fname.to_string());
      }
    }
  }

  Ok(services)
}

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use bodhi_config::{ConfigEngine, RustCodegenOptions};

fn main() {
  let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
  let service = env::var("CARGO_PKG_NAME").unwrap();
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  let workspace_root = find_workspace_root(&manifest_dir)
    .expect("cannot find workspace root (Cargo.toml with [workspace])");
  let config_dir = workspace_root.join("config");
  let output_path = out_dir.join("config.rs");

  let engine = ConfigEngine::new(&config_dir).unwrap_or_else(|err| {
    panic!(
      "failed to open config directory {}: {err}",
      config_dir.display()
    )
  });

  engine
    .generate_service_rust_types_with(&service, &output_path, &RustCodegenOptions::default())
    .unwrap_or_else(|err| {
      panic!(
        "failed to generate {} for service {service}: {err}",
        output_path.display()
      )
    });

  println!("cargo:rerun-if-changed={}", config_dir.display());
}

fn find_workspace_root(start: &Path) -> Option<PathBuf> {
  let mut dir = start;
  loop {
    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.is_file()
      && let Ok(content) = fs::read_to_string(&cargo_toml)
      && content.contains("[workspace]")
    {
      return Some(dir.to_path_buf());
    }
    dir = dir.parent()?;
  }
}

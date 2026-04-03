use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
  let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
  let service = env::var("CARGO_PKG_NAME").unwrap();
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  let workspace_root = find_workspace_root(&manifest_dir)
    .expect("cannot find workspace root (Cargo.toml with [workspace])");

  let src = workspace_root
    .join(".bodhi")
    .join("bodhi_config")
    .join(&service)
    .join("config.rs");

  let dst = out_dir.join("config.rs");

  if src.exists() {
    fs::copy(&src, &dst)
      .unwrap_or_else(|e| panic!("failed to copy {} to {}: {e}", src.display(), dst.display()));
  } else {
    panic!(
      "generated config not found: {}. Run `cargo gen-config` first.",
      src.display()
    );
  }

  println!("cargo:rerun-if-changed={}", src.display());
}

fn find_workspace_root(start: &Path) -> Option<PathBuf> {
  let mut dir = start;
  loop {
    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.is_file() {
      if let Ok(content) = fs::read_to_string(&cargo_toml) {
        if content.contains("[workspace]") {
          return Some(dir.to_path_buf());
        }
      }
    }
    dir = dir.parent()?;
  }
}

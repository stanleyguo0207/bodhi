mod args;
mod emitter;
mod generater;
mod loader;
mod merge;
mod service;

use std::path::PathBuf;

use bodhi::{OptionExt, Result, WrapContext};
use clap::Parser;

use crate::args::Args;

fn main() -> Result<()> {
  let args = Args::parse();

  let workspace_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .ancestors()
    .nth(2)
    .unwrap()
    .to_path_buf();

  let base_dir = workspace_dir.join(&args.config_dir);

  let profile_path = base_dir
    .join("profile")
    .join(format!("{}.toml", args.profile));

  let profile_value: toml::Value = toml::from_str(
    &std::fs::read_to_string(&profile_path)
      .wrap_context_with(|| format!("Profile file not found {} ", profile_path.display()))?,
  )?;

  let env = profile_value
    .get("env")
    .and_then(|v| v.get("name"))
    .and_then(|v| v.as_str())
    .ok_or_error(&bodhi::BODHIERR_CONFIGKEYNOTFOUND)
    .wrap_context_with(|| format!("Missing env.name in profile at {}", profile_path.display()))?;

  let out_dir = base_dir.join(&args.out_dir).join(&env).join(&args.profile);
  std::fs::create_dir_all(&out_dir)?;

  let services = service::collect_services(&base_dir.join("template/service"))?;

  generater::generate(
    services,
    &base_dir.join("template/infra"),
    &base_dir.join("template/service"),
    &profile_path,
    &out_dir,
    &args.format,
  )?;

  Ok(())
}

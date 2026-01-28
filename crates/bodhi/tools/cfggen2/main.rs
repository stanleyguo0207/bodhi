mod args;
mod loader;
mod merge;
mod util;

use args::Args;
use clap::Parser;
use std::{fs, path::PathBuf};

use bodhi::{OptionExt, Result, WrapContext};

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

  let infra_tpl = loader::load_infra(&base_dir.join("template/infra"));

  let services = loader::load_services(&base_dir.join("template/service"));

  let profile: toml::Value = toml::from_str(&fs::read_to_string(profile_path)?)?;

  let out_dir = base_dir.join(&args.out_dir).join(&env).join(&args.profile);
  std::fs::create_dir_all(&out_dir)?;

  for (svc, svc_tpl) in services {
    let cfg = merge::build_service_config(&svc, &infra_tpl, &svc_tpl, &profile);

    for fmt in &args.format {
      let out = out_dir.join(&svc).with_extension(fmt);

      let content = match fmt.as_str() {
        "toml" => toml::to_string_pretty(&cfg)?,
        "json" => serde_json::to_string_pretty(&cfg)?,
        "yaml" => serde_yaml::to_string(&cfg)?,
        _ => continue,
      };

      fs::create_dir_all(out.parent().unwrap())?;
      fs::write(out, content)?;
    }
  }

  Ok(())
}

use std::{collections::HashMap, fs, path::PathBuf};

use bodhi::Result;
use toml::Value;

use crate::args::Args;
use crate::profile::Profile;

#[derive(Debug)]
pub struct Context {
  pub env: String,
  pub infra: HashMap<String, Value>,
  pub services: HashMap<String, HashMap<String, Value>>,
  pub profile: HashMap<String, Value>,
}

pub fn load_context(args: &Args) -> Result<Context> {
  let base: &PathBuf = &args.config_dir;

  let infra = load_infra(&base.join("template/infra"), "infra")?;
  let services = load_services(&base.join("template/service"))?;
  let profile = load_profile(&base.join("profile").join(format!("{}.toml", args.profile)))?;
  let env = profile.env.name.clone();
  let mut profile_overrides = HashMap::new();
  flatten("", &profile.overrides, &mut profile_overrides);

  Ok(Context {
    env,
    infra,
    services,
    profile: profile_overrides,
  })
}

fn flatten(prefix: &str, v: &Value, out: &mut HashMap<String, Value>) {
  match v {
    Value::Table(table) => {
      for (k, v) in table {
        let new_prefix = if prefix.is_empty() {
          k.clone()
        } else {
          format!("{}.{}", prefix, k)
        };
        flatten(&new_prefix, v, out);
      }
    }
    _ => {
      out.insert(prefix.to_string(), v.clone());
    }
  }
}

fn load_infra(dir: &PathBuf, prefix: &str) -> Result<HashMap<String, Value>> {
  let mut out = HashMap::new();

  for f in fs::read_dir(dir)? {
    let path = f?.path();
    let v: Value = toml::from_str(&fs::read_to_string(&path)?)?;
    flatten(prefix, &v, &mut out);
  }

  Ok(out)
}

fn load_services(dir: &PathBuf) -> Result<HashMap<String, HashMap<String, Value>>> {
  let mut out = HashMap::new();

  for f in fs::read_dir(dir)? {
    let path = f?.path();
    let name = path.file_stem().unwrap().to_str().unwrap().to_string();
    let v: Value = toml::from_str(&fs::read_to_string(&path)?)?;
    let mut map = HashMap::new();
    flatten(&name, &v, &mut map);
    out.insert(name, map);
  }

  Ok(out)
}

fn load_profile(path: &PathBuf) -> Result<Profile> {
  let content = fs::read_to_string(path)?;
  let profile: Profile = toml::from_str(&content)?;
  Ok(profile)
}

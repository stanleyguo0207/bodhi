use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Profile {
  pub env: EnvSection,

  #[serde(flatten)]
  pub overrides: toml::Value,
}

#[derive(Debug, Deserialize)]
pub struct EnvSection {
  pub name: String,
}

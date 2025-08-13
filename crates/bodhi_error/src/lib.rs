#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("args parse error: {0}")]
  ArgsParse(#[from] clap::error::Error),

  #[error("config parse error: {0}")]
  ConfigParse(#[from] toml::de::Error),
}

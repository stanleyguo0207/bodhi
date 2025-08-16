use crate::custom;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("Args parse error: {0}")]
  ArgsParse(#[from] clap::error::Error),

  #[error("Config parse error: {0}")]
  ConfigParse(#[from] toml::de::Error),

  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),

  #[error("{0}")]
  Custom(#[from] custom::error::Error),
}

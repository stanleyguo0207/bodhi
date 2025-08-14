use serde::Deserialize;
use std::fs;
use std::path::Path;

use bodhi_error::Error;
use bodhi_result::Result;

use crate::AppConfig;

pub fn load_config<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<AppConfig<T>> {
  let content = fs::read_to_string(path)?;
  toml::from_str(&content).map_err(Error::ConfigParse)
}

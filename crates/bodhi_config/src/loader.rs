use serde::Deserialize;
use std::fs;
use std::path::Path;

use bodhi_error::Error;
use bodhi_result::Result;

use crate::AppConfig;

pub fn load_config<BizConfig>(path: &Path) -> Result<AppConfig<BizConfig>>
where
  BizConfig: for<'de> Deserialize<'de>,
{
  let content = fs::read_to_string(path)?;
  toml::from_str(&content).map_err(Error::ConfigParse)
}

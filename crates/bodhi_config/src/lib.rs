//! # Bodhi 配置模块

pub mod codegen;
pub mod engine;
pub mod errcode;
pub mod loader;
pub mod merge;
pub mod output;
pub mod resolve;
pub mod validate;

#[doc(hidden)]
pub use toml;

pub use bodhi_config_macros::service_config;

pub use crate::codegen::{
  RustCodegenOptions, RustCodegenResult, TypeOverrideHit, TypeOverrideRule, TypeOverrideRules,
  TypeOverrideSource,
};
pub use crate::engine::{ConfigEngine, ResolvedConfig};
pub use crate::output::OutputFormat;

use std::path::Path;

use serde::de::DeserializeOwned;

pub fn load_config<T>(profile: &str, service: &str) -> prelude::Result<T>
where
  T: DeserializeOwned,
{
  load_config_from("config", profile, service)
}

pub fn load_config_from<T>(config_dir: impl AsRef<Path>, profile: &str, service: &str) -> prelude::Result<T>
where
  T: DeserializeOwned,
{
  let engine = ConfigEngine::find(config_dir)?;
  engine.resolve(profile, service)?.extract(".")
}

/// 预导入模块
pub mod prelude {
  pub use crate::codegen::{
    RustCodegenOptions, RustCodegenResult, TypeOverrideHit, TypeOverrideRule, TypeOverrideRules,
    TypeOverrideSource,
  };
  pub use crate::engine::{ConfigEngine, ResolvedConfig};
  pub use crate::errcode::configerr::*;
  pub use crate::load_config;
  pub use crate::load_config_from;
  pub use crate::output::OutputFormat;
  pub use bodhi_error::prelude::{Error, OptionExt, Result, ResultExt};
}

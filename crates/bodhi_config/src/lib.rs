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

pub use crate::codegen::{
  RustCodegenOptions, RustCodegenResult, TypeOverrideHit, TypeOverrideRule, TypeOverrideRules,
  TypeOverrideSource,
};
pub use crate::engine::{ConfigEngine, ResolvedConfig};
pub use crate::output::OutputFormat;

/// 预导入模块
pub mod prelude {
  pub use crate::codegen::{
    RustCodegenOptions, RustCodegenResult, TypeOverrideHit, TypeOverrideRule, TypeOverrideRules,
    TypeOverrideSource,
  };
  pub use crate::engine::{ConfigEngine, ResolvedConfig};
  pub use crate::errcode::configerr::*;
  pub use crate::output::OutputFormat;
  pub use bodhi_error::prelude::{Error, OptionExt, Result, ResultExt};
}

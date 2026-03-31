//! # Bodhi 错误处理库

pub mod errcode;
pub mod error;

#[doc(hidden)]
pub use paste;

/// 预导入模块
pub mod prelude {
  pub use crate::errcode::{bodhierr::*, register_error_code};
  pub use crate::error::{
    Error, OptionExt, Result, ResultExt, freeze_frames_filters, register_default_frames_filters,
    register_frames_filter,
  };
}

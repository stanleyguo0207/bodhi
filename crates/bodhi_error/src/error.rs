//! 错误处理核心模块

mod core;
mod ext;
mod filter;
mod frame;

use self::filter::apply_frames_filters;
use self::frame::Frame;

pub use self::core::Error;
pub use self::ext::{OptionExt, ResultExt};
pub use self::filter::{
  freeze_frames_filters, register_default_frames_filters, register_frames_filter,
};

pub type Result<T> = std::result::Result<T, Error>;

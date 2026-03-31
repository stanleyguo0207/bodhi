//! 错误码定义模块

mod macros;
mod registry;

pub use registry::register_error_code;

crate::define_error_codes! {
  bodhierr [-100, 0] {
    /// 成功
    Ok = 0,
    /// 系统错误
    Sys = -1,
  }
}

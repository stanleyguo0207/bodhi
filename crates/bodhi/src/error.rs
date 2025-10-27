//! 错误模块根：拆分实现并导出常用类型
//!
//! 结构：
//! - `types.rs`：定义 `Error`、`BoxErr` 和内部 MessageError 类型。
//! - `helpers.rs`：实现 `Error` 的 helper 方法（from_any、from_serialized_json、external_kind）。
//! - `impls.rs`：为常见类型实现 `From<...> for Error`.

pub mod helpers;
pub mod impls;
pub mod types;

// 将核心类型导出到 `bodhi::error::Error` 和 `bodhi::error::BoxErr`。
pub use types::{BoxErr, Error};

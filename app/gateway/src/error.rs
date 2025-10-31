//! Gateway 错误模块根：拆分实现并导出常用类型（参考 `crates/bodhi` 的设计）
//!
//! 结构：
//! - `types.rs`：定义 `GatewayError` 与本 crate 的 `Result` 简写。
//! - `helpers.rs`：实现 `GatewayError` 的便捷方法（例如序列化为远端 payload）。
//! - `impls.rs`：为 `GatewayError` 实现与外部类型的转换（例如 `From<GatewayError> for bodhi::Error`）。

mod helpers;
mod impls;
mod types;

// 导出核心类型以便在 `main.rs` 中使用 `use error::GatewayError`。
pub use types::GatewayError;

//! bodhi 核心库（crate）入口文档。
//!
//! 该 crate 提供基础类型与工具，供 `app/*` 下的二进制程序复用。
//!
//! 目录结构与职责：
//! - `error`：统一的错误类型与辅助函数（`Error`），用于跨 crate/网络传递错误信息。
//! - `service`：与服务相关的公用逻辑（本文件夹下具体实现）。
//!
//! 使用示例：
//! ```rust
//! // 将上层错误转换为 Error
//! let be: bodhi::error::Error = "something".into();
//! // 从远端 JSON 负载解析错误
//! let parsed = bodhi::error::Error::from_serialized_json("{\"type\":\"X\",\"message\":\"m\"}");
//! ```

pub mod error;
pub mod service;

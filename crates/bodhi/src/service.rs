//! 服务模块：初始化和运行时相关共享功能。
//!
//! 该模块已按职责拆分为子模块：`tracing`、`errors`（错误处理安装）和 `serve`（启动入口）。

pub mod errors;
mod serve;
pub mod tracing;

pub use serve::serve;

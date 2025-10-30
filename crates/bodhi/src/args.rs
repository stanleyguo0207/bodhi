//! 参数模块（拆分实现）
//!
//! 结构：
//! - `cli.rs`：clap 派生的命令行解析类型（crate 内部使用）。
//! - `types.rs`：对外暴露的 `Args` 类型。
//! - `parser.rs`：将 `Cli` 转换为 `Args` 并做基本校验（例如路径存在性）。

mod cli;
mod parser;
mod types;

// 将常用项导出为 `crate::args::Args` 和 `crate::args::parse_args()`。
pub use parser::parse_args;
pub use types::Args;

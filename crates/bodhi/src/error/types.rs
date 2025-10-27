//! 错误类型定义：包含核心错误枚举和用于包装任意上层错误的类型。
//!
//! 这里定义的类型会被 `error` 模块重新导出为 `bodhi::error::Error` 和 `bodhi::error::BoxErr`，
//! 供其它子 crate 和二进制使用。
use std::fmt;

/// 用于存放任意上层错误的 boxed 类型（线程安全、'static）。
pub type BoxErr = Box<dyn std::error::Error + Send + Sync + 'static>;

/// 一个用于将纯文本字符串包装为实现了 `Error` 的小包装类型，
/// 以便把远端/文本错误信息存入 `BoxErr`。
#[derive(Debug)]
pub(crate) struct MessageError(String);

impl MessageError {
  /// 创建一个新的 `MessageError`，用于把纯文本消息包装为实现 `Error` 的类型。
  ///
  /// 仅在 crate 内部可见（`pub(crate)`），外部依赖不应直接构造或依赖其内部结构。
  pub(crate) fn new(s: String) -> Self {
    Self(s)
  }
}

impl fmt::Display for MessageError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl std::error::Error for MessageError {}

#[derive(thiserror::Error, Debug)]
pub enum Error {
  /// 服务错误，通常用于内部业务错误的简短描述。
  #[error("Service error: {0}")]
  ServiceError(String),

  /// 未知错误。
  #[error("Unknown error")]
  Unknown,

  /// 外层/上层错误包装。
  ///
  /// 用途：当上层（例如 gateway/instance/world 等子crate 或外部服务）定义了自有错误类型，
  /// 可以将其包装进此变体以保留原始错误信息与类型名，方便上层解析与链路追踪。
  // 注意：`kind` 为可选项，Display 字符串中使用 Debug 格式显示以便可读性。
  #[error("External error ({kind:?}): {source}")]
  External {
    #[source]
    source: BoxErr,
    /// 可选的上层错误类型名（例如 Rust 类型名或远端 error_type 字段），用于快速识别来源。
    kind: Option<String>,
  },
}

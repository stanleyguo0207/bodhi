//! 错误类型定义：使用 `eyre` 作为错误报告载体，并用 `thiserror` 定义业务错误枚举。
//!
//! 设计目标：高性能、支持调用链与上下文（通过 `eyre::Report` 与 `tracing`），并在
//! debug 模式下启用彩色错误报告（`color-eyre`）。

/// 为当前 crate 定义的 Result 简写，使用 `eyre::Report` 作为错误类型。
pub type Result<T> = std::result::Result<T, eyre::Report>;

/// 业务错误枚举：使用 `thiserror` 派生 Display/Error，业务侧可直接匹配这些变体。
#[derive(thiserror::Error, Debug)]
pub enum Error {
  /// 网络相关错误
  #[error("Network error: {0}")]
  Network(String),

  /// 数据库相关错误
  #[error("Database error: {0}")]
  Database(String),

  /// 逻辑/业务错误（原来的 ServiceError）
  #[error("Service error: {0}")]
  ServiceError(String),

  /// 外部/第三方错误，直接从 `eyre::Report` 转换进来以保留链与上下文。
  ///
  /// 现在使用结构化变体以便保留额外元数据 —— `kind` 表示上层错误类型名（如果有），
  /// `remote_backtrace` 可选地包含远端回传的 backtrace 字符串。
  #[error("{report}")]
  External {
    /// 来自 eyre 的报告（错误链与上下文）
    report: eyre::Report,

    /// 可选的远端/上层错误类型标识（例如 `GatewayError`）
    kind: Option<String>,

    /// 可选的远端 backtrace 字符串（如果上游传递了的话）
    remote_backtrace: Option<String>,
  },
}

// 手动实现从 eyre::Report 到 Error 的转换，确保在直接把 Report 转入时
// 会把其它字段置为 None（兼容之前的简洁用法）。
impl From<eyre::Report> for Error {
  fn from(report: eyre::Report) -> Self {
    Error::External {
      report,
      kind: None,
      remote_backtrace: None,
    }
  }
}

impl Error {
  /// 便捷构造：保留与旧 `ServiceError` 的语义
  pub fn service<S: Into<String>>(s: S) -> Self {
    Error::ServiceError(s.into())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use eyre::{Report, WrapErr};

  #[test]
  fn wrap_err_includes_context() {
    let res: std::result::Result<(), Report> =
      Err(std::io::Error::new(std::io::ErrorKind::Other, "io fail")).wrap_err("加载玩家 42 失败");
    let report = res.unwrap_err();
    let s = format!("{}", report);
    assert!(s.contains("加载玩家 42 失败"));
  }

  #[test]
  fn service_error_works() {
    let e = Error::service("测试错误");
    let s = format!("{}", e);
    assert!(s.contains("测试错误"));
  }

  #[test]
  fn wrap_err_chain_prints_context() {
    let rep: Report = eyre::eyre!("base").wrap_err("上层上下文");
    let out = format!("{}", rep);
    assert!(out.contains("上层上下文"));
  }
}

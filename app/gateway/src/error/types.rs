//! Gateway 错误类型定义。
//!
//! 定义 `GatewayError`（业务错误）以及本 crate 的 `Result` 简写。

/// Gateway 侧的错误类型：简单封装 code + message。
#[derive(Debug, Clone)]
pub struct GatewayError {
  pub code: u16,
  pub message: String,
}

impl std::fmt::Display for GatewayError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "GatewayError(code={}, message={})",
      self.code, self.message
    )
  }
}

impl std::error::Error for GatewayError {}

impl GatewayError {
  /// 快捷构造器
  pub fn new<S: Into<String>>(code: u16, message: S) -> Self {
    GatewayError {
      code,
      message: message.into(),
    }
  }

  /// 预定义全局错误：403 Forbidden
  pub fn forbidden() -> Self {
    GatewayError::new(403, "forbidden")
  }

  /// 预定义：404 Not Found，允许自定义消息
  pub fn not_found<S: Into<String>>(msg: S) -> Self {
    GatewayError::new(404, msg)
  }

  /// 预定义：400 Bad Request，允许自定义消息
  pub fn bad_request<S: Into<String>>(msg: S) -> Self {
    GatewayError::new(400, msg)
  }

  /// 预定义：500 Internal Server Error，允许自定义消息
  pub fn internal<S: Into<String>>(msg: S) -> Self {
    GatewayError::new(500, msg)
  }
}

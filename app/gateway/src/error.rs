//! Gateway 专属错误模块。
//!
//! 目标：把 `GatewayError` 从 `main.rs` 中抽出，保持与 `crates/bodhi` 的错误模块分离风格，
//! 并提供一些便捷方法（序列化为远端负载、与 `bodhi::Error` 的转换）。

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
  /// 构造远端序列化负载（JSON string），供网络传输给接收方解析为 `bodhi::Error`。
  /// 结构保持示例代码中使用的格式：{"type":"GatewayError","message": "...", "code": N}
  pub fn to_remote_payload(&self) -> String {
    serde_json::json!({
      "type": "GatewayError",
      "message": self.to_string(),
      "code": self.code,
    })
    .to_string()
  }
}

/// 便捷：允许直接把 GatewayError 转换为 `bodhi::Error`（使用 `bodhi` 提供的 `from_any`）
impl From<GatewayError> for bodhi::Error {
  fn from(e: GatewayError) -> Self {
    bodhi::Error::from_any(e)
  }
}

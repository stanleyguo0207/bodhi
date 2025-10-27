//! 辅助函数与关联方法：为 `Error` 提供常用的构造/解析方法。
//!
//! 包含：
//! - `from_any`：把任意实现 `std::error::Error` 的错误包装为 `Error::External`。
//! - `from_serialized_json`：从远端/上层发送的 JSON 负载解析回 `Error`。
//! - `external_kind`：便捷访问器，用于读取 External 变体的 `kind` 字段。
use super::types::{Error, MessageError};

impl Error {
  /// 将任意实现 `std::error::Error + Send + Sync + 'static` 的错误转换为 `Error::External`。
  ///
  /// 会尝试记录原始类型名到 `kind` 字段，便于后续按类型判断或日志过滤。
  pub fn from_any<E>(e: E) -> Self
  where
    E: std::error::Error + Send + Sync + 'static,
  {
    Error::External {
      source: Box::new(e),
      kind: Some(std::any::type_name::<E>().to_string()),
    }
  }

  /// 尝试从 JSON 字符串解析上层定义的错误结构。
  /// 期望形如 {"type": "...", "message": "..."} 或 {"error_type": "...", "message": "..."}。
  /// 解析成功后会把 message 放到 `source`，type 放到 `kind`。
  pub fn from_serialized_json(s: &str) -> Self {
    // 局部定义用于解析常见字段名
    #[derive(serde::Deserialize)]
    struct RemoteErr {
      #[serde(rename = "type")]
      type_: Option<String>,
      #[serde(rename = "error_type")]
      error_type: Option<String>,
      message: Option<String>,
    }

    match serde_json::from_str::<RemoteErr>(s) {
      Ok(r) => {
        let kind = r.type_.or(r.error_type);
        let message = r.message.unwrap_or_else(|| s.to_string());
        Error::External {
          source: Box::new(MessageError::new(message)),
          kind,
        }
      }
      Err(_) => Error::ServiceError(s.to_string()),
    }
  }

  /// 可选：便捷方法，用于访问内部的 kind 名称。
  pub fn external_kind(&self) -> Option<&str> {
    match self {
      Error::External { kind, .. } => kind.as_deref(),
      _ => None,
    }
  }
}

//! 为 `Error` 提供常见的转换实现（`From<T>`）。
//!
//! 把一些常见错误类型或字符串直接转换为 `Error`，方便在 `?` 或 `map_err(Into::into)` 时使用。
use super::types::Error;

impl From<String> for Error {
  fn from(s: String) -> Self {
    Error::service(s)
  }
}

impl From<&str> for Error {
  fn from(s: &str) -> Self {
    Error::service(s.to_string())
  }
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self {
    Error::from_any(e)
  }
}

impl From<serde_json::Error> for Error {
  fn from(e: serde_json::Error) -> Self {
    Error::from_any(e)
  }
}

impl From<toml::de::Error> for Error {
  fn from(e: toml::de::Error) -> Self {
    Error::from_any(e)
  }
}

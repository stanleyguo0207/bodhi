//! 辅助函数与关联方法：为 `Error` 提供常用的构造/解析方法。
//!
//! 说明：现在 `Error::External` 使用 `eyre::Report` 作为载体，下面的 helper
//! 把任意实现 `std::error::Error` 的错误包装为 `Error::External`，并提供
//! 从序列化 JSON 解析远端错误的便捷函数。
use super::types::Error;
use eyre::Report;

impl Error {
  /// 将任意实现 `std::error::Error + Send + Sync + 'static` 的错误转换为 `Error::External`。
  pub fn from_any<E>(e: E) -> Self
  where
    E: std::error::Error + Send + Sync + 'static,
  {
    // Report 为 eyre 的错误载体，直接从 E 转换以保留 error chain（若可用）。
    Error::External {
      report: Report::from(e),
      kind: Some(std::any::type_name::<E>().to_string()),
      remote_backtrace: None,
    }
  }

  /// 尝试从 JSON 字符串解析上层定义的错误结构。
  /// 期望形如 {"type": "...", "message": "...", "backtrace": "..."}。
  /// 解析成功后会把 message 放到 Report 的消息中，并在消息中附加 type 信息以便识别。
  ///
  /// Example (illustrative — not executed as a doctest):
  /// ```rust,ignore
  /// use bodhi::error::Error;
  /// let json = r#"{"type":"GatewayError","message":"downstream failed","backtrace":"bt..."}"#;
  /// let e = Error::from_serialized_json(json);
  /// assert_eq!(e.external_kind(), Some("GatewayError"));
  /// if let Error::External { remote_backtrace: Some(bt), .. } = e {
  ///     assert!(bt.contains("bt"));
  /// }
  /// ```
  pub fn from_serialized_json(s: &str) -> Self {
    #[derive(serde::Deserialize)]
    struct RemoteErr {
      #[serde(rename = "type")]
      type_: Option<String>,
      #[serde(rename = "error_type")]
      error_type: Option<String>,
      message: Option<String>,
      backtrace: Option<String>,
    }

    match serde_json::from_str::<RemoteErr>(s) {
      Ok(r) => {
        let kind = r.type_.or(r.error_type);
        let message = r.message.unwrap_or_else(|| s.to_string());
        let remote_bt = r.backtrace;

        // 把 message 和 kind 放入 Report；把原始 backtrace 存到 remote_backtrace 字段
        let mut report = eyre::eyre!(message);
        if let Some(k) = kind.clone() {
          report = report.wrap_err(format!("remote_type={}", k));
        }

        Error::External {
          report,
          kind,
          remote_backtrace: remote_bt,
        }
      }
      Err(_) => Error::service(s.to_string()),
    }
  }

  /// 便捷：尝试从 External 变体中读取最顶层的 remote_type（如果曾被附加过的话）。
  /// 由于 eyre::Report 的内部链不暴露自定义字段，这里返回 None（占位），
  /// 如果需要更严格的元数据，建议将结构化元数据放入 JSON 的 message 或额外字段中。
  pub fn external_kind(&self) -> Option<&str> {
    match self {
      Error::External { kind, .. } => kind.as_deref(),
      _ => None,
    }
  }

  /// 便捷：读取 `External` 变体中可能存在的 `remote_backtrace` 字符串。
  /// 返回值为 `Option<&str>`，如果 `External::remote_backtrace` 为 `Some` 则返回对其借用引用。
  ///
  /// Example (illustrative):
  /// ```rust,ignore
  /// use bodhi::error::Error;
  /// let json = r#"{"type":"GatewayError","message":"m","backtrace":"bt"}"#;
  /// let e = Error::from_serialized_json(json);
  /// assert!(e.external_remote_backtrace().is_some());
  /// ```
  pub fn external_remote_backtrace(&self) -> Option<&str> {
    match self {
      Error::External {
        remote_backtrace, ..
      } => remote_backtrace.as_deref(),
      _ => None,
    }
  }
}

//! 辅助函数与关联方法：为 `Error` 提供常用的构造/解析方法。
//!
//! 说明：现在 `Error::External` 使用 `eyre::Report` 作为载体，下面的 helper
//! 把任意实现 `std::error::Error` 的错误包装为 `Error::External`，并提供
//! 从序列化 JSON 解析远端错误的便捷函数。
use super::types::Error;
use eyre::Report;

impl Error {
  /// 构造来自远端/上游的 External 错误。
  ///
  /// - `kind`：上游错误类型标识，建议使用带命名空间的字符串，例如 "gateway::GatewayError"，便于跨服务识别与聚合。
  /// - `msg`：上游错误的主要消息（会被放入 `eyre::Report`）。
  /// - `remote_backtrace`：可选的上游回溯字符串（如果上游传回）。
  ///
  /// 此方法会使用 `eyre::eyre!(msg)` 构造一个 `Report`，并在 Report 上附加一条 `remote_type=...` 的 wrap 上下文，
  /// 以便在 Display/日志中保留上游类型信息。字段 `kind` 与 `remote_backtrace` 会填入对应结构体字段，
  /// 便于接收端访问或选择性序列化。
  pub fn from_remote_payload(
    kind: Option<&str>,
    msg: &str,
    remote_backtrace: Option<String>,
  ) -> Self {
    let message = msg.to_string();
    // 为确保 report 的 Display 中包含上游的原始消息（便于序列化/日志），
    // 我们把原始消息放入一个可识别的前缀 `remote_msg=...` 中。
    // 这样即使 eyre 的默认格式层次有所不同，序列化的 `message` 字段仍包含可搜索的上游文本。
    let mut report = eyre::eyre!(format!("remote_msg={}", message));
    if let Some(k) = kind {
      report = report.wrap_err(format!("remote_type={}", k));
    }

    Error::External {
      report,
      kind: kind.map(|s| s.to_string()),
      remote_backtrace,
      remote_message: Some(message),
    }
  }

  /// 把 `Error`（主要针对 `External`）序列化为一个 JSON 对象，方便跨进程/跨语言传输。
  ///
  /// 默认行为：
  /// - `type` 字段保存 `kind`（如果存在）。
  /// - `message` 字段保存 `Display` 格式的 report（包含上下文与 wrap 添加的信息）。
  /// - `backtrace` 字段保存 `remote_backtrace`（如果存在）。
  ///
  /// 注意事项 / 脱敏策略：
  /// - `to_remote_payload` 会包含 `backtrace`（如果有），适合在受信任服务之间传输或调试环境下使用。
  /// - 在 production 路径上，推荐使用 `to_remote_payload_sanitized`，该函数不会包含 `backtrace`，以避免泄露敏感信息或产生过大的负载。
  pub fn to_remote_payload(&self) -> serde_json::Value {
    match self {
      Error::External {
        report,
        kind,
        remote_backtrace,
        remote_message,
      } => {
        // 优先使用 explicit 存储的 remote_message（如果存在），否则尝试从 report 的 Display 中提取。
        let msg = if let Some(m) = remote_message.clone() {
          m
        } else {
          let full = format!("{}", report);
          if let Some(idx) = full.find("remote_msg=") {
            let start = idx + "remote_msg=".len();
            let rest = &full[start..];
            rest
              .split(|c| c == '\n' || c == '\r')
              .next()
              .unwrap_or(rest)
              .trim()
              .to_string()
          } else {
            full
          }
        };

        serde_json::json!({
          "type": kind.clone(),
          "message": msg,
          "backtrace": remote_backtrace,
        })
      }
      // 非 External 的错误转换为一个简单的 message-only 对象
      other => serde_json::json!({
        "type": null,
        "message": format!("{}", other),
      }),
    }
  }

  /// 生成一个对外更安全的 JSON 表示：不会包括 `remote_backtrace` 字段，适合 production 场景。
  pub fn to_remote_payload_sanitized(&self) -> serde_json::Value {
    match self {
      Error::External {
        report,
        kind,
        remote_message,
        ..
      } => {
        let msg = if let Some(m) = remote_message.clone() {
          m
        } else {
          let full = format!("{}", report);
          if let Some(idx) = full.find("remote_msg=") {
            full[idx + "remote_msg=".len()..]
              .split(|c| c == '\n' || c == '\r')
              .next()
              .unwrap_or("")
              .trim()
              .to_string()
          } else {
            full
          }
        };

        serde_json::json!({
          "type": kind.clone(),
          "message": msg,
        })
      }
      other => serde_json::json!({
        "type": null,
        "message": format!("{}", other),
      }),
    }
  }
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
      remote_message: None,
    }
  }

  /// 将任意错误与一段上下文信息组合为 `Error::External`。
  ///
  /// 该方法在库内使用 `eyre::Report` 构造带上下文的报告，然后把它封装到 `Error::External` 中，
  /// 从而使上层调用方无需依赖 `eyre` 就能返回 `bodhi::Result`。
  pub fn from_any_with_context<E, S>(e: E, ctx: S) -> Self
  where
    E: std::error::Error + Send + Sync + 'static,
    S: Into<String>,
  {
    let mut report = Report::from(e);
    report = report.wrap_err(ctx.into());
    Error::External {
      report,
      kind: Some(std::any::type_name::<E>().to_string()),
      remote_backtrace: None,
      remote_message: None,
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
        let remote_msg = r.message.clone();
        let message = remote_msg.clone().unwrap_or_else(|| s.to_string());
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
          remote_message: remote_msg,
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

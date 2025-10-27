fn main() {
  println!("Hello, gateway example: convert and serialize errors");

  // 示例：gateway 定义自己的错误类型
  #[derive(Debug)]
  struct GatewayError {
    code: u16,
    message: String,
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

  // 模拟 gateway 产生错误并希望通过网络发送
  let gw_err = GatewayError {
    code: 403,
    message: "forbidden".into(),
  };

  // 方案 A：发送结构化 JSON 描述远端错误（常见模式）
  // 示例负载结构：{ "type": "GatewayError", "message": "forbidden" }
  let remote_payload = serde_json::json!({
    "type": "GatewayError",
    "message": gw_err.to_string(),
  })
  .to_string();

  println!("Serialized remote payload: {}", remote_payload);

  // 接收方（可能是另一个服务）将序列化的负载解析为 `Error`
  // 使用 `from_serialized_json`，它会把 `type` 保存到 `kind`，把 message 保存到 `source`。
  let be = bodhi::error::Error::from_serialized_json(&remote_payload);
  println!("Parsed Error: {be:#?}");

  // 方案 B：当 gateway 与接收方同属工作区的 Rust crate 时，
  // gateway 可直接将其错误转换为 `Error` 并传递 boxed 错误。
  // 这种方式适用于内部 IPC，双方共享 `bodhi` crate 的类型定义。
  // 使用 `bodhi` crate 提供的辅助函数直接转换。
  let be2 = bodhi::error::Error::from_any(gw_err);
  println!("Direct converted Error: {be2:#?}");
}

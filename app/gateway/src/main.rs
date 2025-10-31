mod error;

use error::GatewayError;

fn main() -> bodhi::Result<()> {
  // 使用 bodhi 提供的统一初始化函数，包含 tracing + 错误处理器（color-eyre）配置。
  bodhi::service::serve()?;

  println!("Hello, gateway example: convert and serialize errors");

  // Gateway 的错误类型已移入 `error` 模块。

  // 使用预定义的全局错误构造器
  let gw_err = GatewayError::forbidden();

  // 另外演示带自定义消息的预定义错误
  let not_found = GatewayError::not_found("player 42 not found");
  let internal = GatewayError::internal("db connection lost");

  // 方案 A：发送结构化 JSON 描述远端错误（常见模式）
  // 示例负载结构：{ "type": "GatewayError", "message": "forbidden" }
  let remote_payload = gw_err.to_remote_payload();

  println!("Serialized remote payload: {}", remote_payload);

  // 接收方（可能是另一个服务）将序列化的负载解析为 `Error`
  // 使用 `from_serialized_json`，它会把 `type` 保存到 `kind`，把 message 保存到 `source`。
  let be = bodhi::Error::from_serialized_json(&remote_payload);
  println!("Parsed Error: {be:#?}");

  // 方案 B：当 gateway 与接收方同属工作区的 Rust crate 时，
  // gateway 可直接将其错误转换为 `Error` 并传递 boxed 错误。
  // 这种方式适用于内部 IPC，双方共享 `bodhi` crate 的类型定义。
  // 使用 `bodhi` crate 提供的辅助函数直接转换。
  let be2 = bodhi::Error::from_any(gw_err);
  println!("Direct converted Error: {be2:#?}");

  // 展示其它全局错误的序列化/转换
  println!("NotFound payload: {}", not_found.to_remote_payload());
  println!(
    "Internal as bodhi::Error: {:#?}",
    bodhi::Error::from_any(internal)
  );

  // 为了让 color-eyre 打印像你期望的那样的错误链和回溯，我们构造一个小的调用链：
  // simulate_error -> server_main -> main。server_main 会把底层 IO 错误用中文上下文 wrap_err。

  fn simulate_error() -> std::io::Result<String> {
    // 故意读取不存在的文件以触发 `No such file or directory`
    std::fs::read_to_string("config.toml")
  }

  fn server_main() -> bodhi::Result<()> {
    // 将底层 IO 错误转换为 bodhi::Error 并附加上下文信息，
    // 这样上层二进制无需直接引用 `eyre`。
    match simulate_error() {
      Ok(_) => Ok(()),
      Err(e) => Err(bodhi::Error::from_any_with_context(
        e,
        "无法读取配置文件 config.toml",
      )),
    }
  }

  // 直接把 server_main 的结果返回给 runtime；当发生 Err(Report) 时，color-eyre 会负责漂亮地打印
  // 错误消息、Caused by 列表和 Stack backtrace（在启用 debug 特性并设置 RUST_BACKTRACE=1 时）。
  server_main()
}

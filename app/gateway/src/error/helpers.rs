//! Gateway 错误的 helper 方法（与类型分离）
use super::types::GatewayError;

impl GatewayError {
  /// 构造远端序列化负载（JSON string），供网络传输给接收方解析为 `bodhi::Error`。
  /// 结构采用示例中使用的格式：{"type":"GatewayError","message":"...","code":N}
  pub fn to_remote_payload(&self) -> String {
    serde_json::json!({
      "type": "GatewayError",
      "message": self.to_string(),
      "code": self.code,
    })
    .to_string()
  }
}

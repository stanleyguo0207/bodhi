bodhi::define_static_errors!(
  gatewayerr (1001 .. 2000) {
    Timeout => (1001, "网关请求超时"),
    Unauthorized => (1004, "未授权访问"),
  }
);

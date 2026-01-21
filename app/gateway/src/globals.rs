bodhi::define_static_errors!(
  gatewayerr (1001 .. 2000) {
    Timeout => (1001, "Gateway request timed out"),
    Unauthorized => (1004, "Unauthorized access"),
  }
);

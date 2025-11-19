crate::define_static_errors!(
  bodhierr (0 .. 1000) {
    Ok => (0, "No error"),
    Sys => (1, "System error"),
    Build => (2, "Build error"),
  }
);

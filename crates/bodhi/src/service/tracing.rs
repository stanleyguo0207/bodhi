//! tracing 初始化：tracing-subscriber + tracing-error 的 ErrorLayer

use crate::error::Result;

pub(crate) fn init_tracing() -> Result<()> {
  use tracing_error::ErrorLayer;
  use tracing_subscriber::{EnvFilter, fmt, prelude::*};

  let fmt_layer = fmt::layer();
  let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
  let subscriber = tracing_subscriber::registry()
    .with(filter)
    .with(fmt_layer)
    .with(ErrorLayer::default());
  tracing::subscriber::set_global_default(subscriber)
    .map_err(|e| crate::error::Error::from(eyre::eyre!("set_global_default failed: {}", e)))?;
  Ok(())
}

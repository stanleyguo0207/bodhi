//! Conversion impls for `GatewayError`.
use super::types::GatewayError;

impl From<GatewayError> for bodhi::Error {
  fn from(e: GatewayError) -> Self {
    bodhi::Error::from_any(e)
  }
}

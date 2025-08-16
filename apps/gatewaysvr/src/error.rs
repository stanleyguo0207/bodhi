use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
  #[error("{0}")]
  Bodhi(#[from] bodhi_error::Error),
}

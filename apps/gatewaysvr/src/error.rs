use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
  #[error("framework error: {0}")]
  Bodhi(#[from] bodhi_error::Error),
}

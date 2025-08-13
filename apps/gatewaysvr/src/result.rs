use crate::error::GatewayError;

pub type GatewayResult<T> = Result<T, GatewayError>;

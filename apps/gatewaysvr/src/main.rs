mod error;
mod result;

use bodhi_service::service;

use result::GatewayResult;

fn main() -> GatewayResult<()> {
  service::serve().map_err(Into::into)
}

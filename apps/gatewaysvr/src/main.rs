mod config;
mod error;
mod result;

use bodhi_service::service;

use config::GatewayConfig;
use result::GatewayResult;

fn main() -> GatewayResult<()> {
  service::serve::<GatewayConfig>().map_err(Into::into)
}

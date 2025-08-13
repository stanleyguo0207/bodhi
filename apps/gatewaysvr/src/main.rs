use result::GatewayResult;

mod error;
mod result;

fn main() -> GatewayResult<()> {
  bodhi_service::serve().map_err(Into::into)
}

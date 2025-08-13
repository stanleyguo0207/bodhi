use result::GatewayResult;

mod error;
mod result;

fn main() -> GatewayResult<()> {
  bodhi_service::serve()?;
  Ok(())
}

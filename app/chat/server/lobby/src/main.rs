use bodhi_config::prelude::*;

const PROFILE: &str = "dev";

bodhi_config::service_config!();

fn main() -> Result<()> {
  let config = load_service_config(PROFILE)?;

  println!("config = {:#?}", config);

  Ok(())
}

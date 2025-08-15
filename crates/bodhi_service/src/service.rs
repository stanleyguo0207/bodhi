use std::fmt::Debug;

use serde::Deserialize;

use bodhi_args::args;
use bodhi_config::loader;
use bodhi_result::Result;

pub fn serve<BizConfig>() -> Result<()>
where
  BizConfig: for<'de> Deserialize<'de> + Debug + Clone,
{
  let args = args::parse_args()?;
  println!("Configuration directory: {}", args.config_dir.display());

  let config = loader::load_config::<BizConfig>(&args.config_dir.join("config.toml"))?;
  println!("Framework configuration: {:?}", config.framework);
  println!("Business configuration: {:?}", config.business);

  Ok(())
}

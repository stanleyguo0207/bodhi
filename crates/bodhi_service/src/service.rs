use bodhi_args::args;
use bodhi_result::Result;

pub fn serve() -> Result<()> {
  let args = args::parse_args()?;
  println!("Configuration directory: {}", args.config_dir.display());
  Ok(())
}

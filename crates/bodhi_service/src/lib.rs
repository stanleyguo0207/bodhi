use bodhi_args::parse_args;
use bodhi_result::Result;

pub fn serve() -> Result<()> {
  let args = parse_args()?;
  println!("Configuration directory: {}", args.config_dir.display());
  Ok(())
}

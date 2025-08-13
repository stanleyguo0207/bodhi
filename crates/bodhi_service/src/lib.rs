use clap::Parser;

pub fn serve() -> bodhi_result::Result<()> {
  let args = bodhi_args::Args::try_parse()?;
  println!("Configuration directory: {}", args.config_dir.display());
  Ok(())
}

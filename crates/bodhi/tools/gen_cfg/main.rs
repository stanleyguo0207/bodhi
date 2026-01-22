mod cli;
mod emitter;
mod loader;
mod merge;
mod validator;

use bodhi::Result;
use clap::Parser;

use cli::Cli;

fn main() -> Result<()> {
  let cli = Cli::parse();

  let templates = loader::load_templates(&cli.template_dir)?;
  let overrides = loader::load_custom_config(&cli.custom_config)?;

  let merged = merge::merge_configs(templates, overrides)?;

  validator::validate(&merged)?;

  let custom_name = cli.custom_config.file_stem().unwrap().to_str().unwrap();

  emitter::emit(&merged, &cli.out_dir, custom_name)?;

  Ok(())
}

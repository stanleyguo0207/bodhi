mod args;
mod emitter;
mod loader;
mod merge;
mod profile;

use bodhi::Result;
use clap::Parser;

use args::Args;

use crate::loader::load_context;

fn main() -> Result<()> {
  let args = Args::parse();

  let ctx = load_context(&args)?;

  let env = ctx.env.clone();
  let svcs = merge::merge_services(ctx)?;

  for (svc, cfg) in svcs {
    emitter::emit(
      &args.out_dir,
      env.as_str(),
      &args.profile,
      &svc,
      &cfg,
      &args.format,
    )?;
  }

  Ok(())
}

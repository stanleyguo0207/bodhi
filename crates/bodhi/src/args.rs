mod args;

use clap::Parser;

use crate::Result;

use args::Args;

pub fn init() -> Result<Args> {
  let args = Args::try_parse().unwrap_or_else(|e| {
    e.print().ok();
    std::process::exit(e.exit_code());
  });
  Ok(args)
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_args_parse() {
    let args = Args::try_parse_from(&["bodhi", "--config", "config.toml", "start"]).unwrap();
    assert_eq!(args.config, std::path::PathBuf::from("config.toml"));
    assert_eq!(args.cmd, args::Command::Start);
    assert_eq!(args.daemon, false);
  }
}

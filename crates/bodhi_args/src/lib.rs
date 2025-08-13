pub use args::Args;
use bodhi_result::Result;
use clap::Parser;

mod args;
#[cfg(test)]
mod args_test;

pub fn parse_args() -> Result<Args> {
  Args::try_parse().map_err(Into::into)
}

mod config;
mod globals;

use bodhi::{Error, Result, WrapContext};

use globals::*;

fn main() -> Result<()> {
  println!("Hello, Gateway!");

  Ok(())
}

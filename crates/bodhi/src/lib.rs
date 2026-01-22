mod args;
mod config;
mod error;
mod globals;

#[doc(hidden)]
pub use paste;

pub use config::*;
pub use error::*;
pub use globals::*;

pub fn init() -> Result<()> {
  error::init()?;
  let args = args::init()?;

  println!("Using config file: {:?}", args);

  Ok(())
}

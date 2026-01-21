mod error;
mod globals;

#[doc(hidden)]
pub use paste;

pub use error::*;
pub use globals::*;

pub fn init() -> Result<()> {
  error::init()?;
  Ok(())
}

mod error;
mod filters;
mod frame;
mod macros;
mod result;
mod wrapper;

pub use error::{Error, ErrorMeta};
pub use filters::FramesFilter;
pub use frame::Frame;
pub use result::Result;
pub use wrapper::WrapContext;

pub fn init() -> Result<()> {
  filters::FiltersBuilder::new().build()?;
  Ok(())
}

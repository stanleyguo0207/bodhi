mod error_impl;
mod filters_builder_impl;
mod filters_impl;
mod frame_impl;
mod macros;
pub mod result;
pub mod types;
pub mod wrapper;

pub use result::Result;
pub use types::{Error, ErrorMeta, Frame, FramesFilter};
pub use wrapper::WrapContext;

pub fn init() -> Result<()> {
  types::FiltersBuilder::new().build()?;
  Ok(())
}

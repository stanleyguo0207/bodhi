mod error_impl;
pub mod ext;
mod filters_builder_impl;
mod filters_impl;
mod frame_impl;
mod macros;
pub mod result;
pub mod types;

pub use ext::WrapContext;
pub use result::Result;
pub use types::{Error, ErrorMeta, Frame, FramesFilter};

pub fn init() -> Result<()> {
  types::FiltersBuilder::new().build()?;
  Ok(())
}

mod error_impl;
pub mod ext;
mod filters_builder_impl;
mod filters_impl;
mod macros;
pub mod result;
pub mod types;

use std::sync::{Arc, OnceLock};
use types::Filters;

static ERROR_FILTERS: OnceLock<Arc<Filters>> = OnceLock::new();

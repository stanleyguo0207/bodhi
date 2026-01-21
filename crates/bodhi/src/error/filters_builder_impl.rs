use std::sync::Arc;

use crate::error::types::{ERROR_FILTERS, Filters, FiltersBuilder};
use crate::{BODHIERR_BUILD, Error, Frame, Result};

impl FiltersBuilder {
  pub fn new() -> Self {
    Self {
      frames_filters: Vec::new(),
    }
  }

  pub fn register_frames_filter<F>(mut self, filter: F) -> Self
  where
    F: Fn(&mut Vec<&Frame>) + Send + Sync + 'static,
  {
    self.frames_filters.push(Box::new(filter));
    self
  }

  pub fn build(self) -> Result<()> {
    let filters = Filters {
      frames_filters: Box::leak(self.frames_filters.into_boxed_slice()),
    };

    ERROR_FILTERS.set(Arc::new(filters)).map_err(|_| {
      Error::new(&BODHIERR_BUILD)
        .wrap_context("error filters setup failed")
        .capture_backtrace()
    })
  }
}

use std::sync::Arc;

use crate::error::{ERROR_FILTERS, Filters};
use crate::{BODHIERR_BUILD, Error, FiltersBuilder, Frame, Result};

impl FiltersBuilder {
  pub fn new() -> Self {
    Self {
      frame_filters: Vec::new(),
      frames_filters: Vec::new(),
    }
  }

  pub fn register_frame_filter<F>(mut self, filter: F) -> Self
  where
    F: Fn(&Frame) -> bool + Send + Sync + 'static,
  {
    self.frame_filters.push(Box::new(filter));
    self
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
      frame_filters: self.frame_filters.into(),
      frames_filters: self.frames_filters.into(),
    };

    ERROR_FILTERS.set(Arc::new(filters)).map_err(|_| {
      Error::new(&BODHIERR_BUILD)
        .wrap_context("error filters setup failed")
        .capture_backtrace()
    })
  }
}

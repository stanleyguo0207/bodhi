use crate::error::types::{Filters, Frame};

impl Filters {
  pub fn apply(&self, frames: &mut Vec<&Frame>) {
    for fliter in self.frames_filters {
      fliter(frames);
    }
  }
}

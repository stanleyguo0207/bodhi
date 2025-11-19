use crate::{Frame, error::Filters};

impl Filters {
  pub fn apply(&self, frames: &mut Vec<&Frame>) {
    frames.retain(|frame| self.frame_filters.iter().all(|fliter| fliter(frame)));
    for fliter in &*self.frames_filters {
      fliter(frames);
    }
  }
}

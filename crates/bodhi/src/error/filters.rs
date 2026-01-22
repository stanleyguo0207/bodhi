use std::sync::{Arc, OnceLock};

use crate::BODHIERR_BUILD;

use super::{Error, Result, frame::Frame};

pub type FramesFilter = dyn Fn(&mut Vec<&Frame>) + Send + Sync + 'static;

pub(in crate::error) struct Filters {
  pub frames_filters: &'static [Box<FramesFilter>],
}

pub(in crate::error) static ERROR_FILTERS: OnceLock<Arc<Filters>> = OnceLock::new();

impl Filters {
  pub fn apply(&self, frames: &mut Vec<&Frame>) {
    for fliter in self.frames_filters {
      fliter(frames);
    }
  }
}

pub(in crate::error) struct FiltersBuilder {
  pub frames_filters: Vec<Box<FramesFilter>>,
}

impl FiltersBuilder {
  pub fn new() -> Self {
    Self {
      frames_filters: Vec::new(),
    }
    .register_frames_filter(default_frames_filter)
    .register_frames_filter(bodhi_frames_filter)
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

    ERROR_FILTERS
      .set(Arc::new(filters))
      .map_err(|_| Error::new(&BODHIERR_BUILD).wrap_context("error filters setup failed"))
  }
}

fn default_frames_filter(frames: &mut Vec<&Frame>) {
  let top_cutoff = frames
    .iter()
    .rposition(|x| x.is_post_panic_code())
    .map(|x| x + 2)
    .unwrap_or(0);

  let bottom_cutoff = frames
    .iter()
    .position(|x| x.is_runtime_init_code())
    .unwrap_or(frames.len());

  let rng = top_cutoff..=bottom_cutoff;
  frames.retain(|x| rng.contains(&x.n))
}

fn bodhi_frames_filter(frames: &mut Vec<&Frame>) {
  const BODHI_PREFIXES: &[&str] = &[
    "bodhi::error::error_impl::impl",
    "bodhi::error::types::Error::capture_backtrace",
  ];

  frames.retain(|frame| {
    !BODHI_PREFIXES.iter().any(|f| {
      let name = if let Some(name) = frame.name.as_ref() {
        name.as_str()
      } else {
        return true;
      };

      name.starts_with(f)
    })
  });

  const BODHI_CONTAINS: &[&str] = &["bodhi::error::error_impl::impl"];

  frames.retain(|frame| {
    !BODHI_CONTAINS.iter().any(|f| {
      let name: &str = if let Some(name) = frame.name.as_ref() {
        name.as_str()
      } else {
        return true;
      };

      name.contains(f)
    })
  });
}

use backtrace::Backtrace;
use std::{error::Error as StdError, panic::Location, path::PathBuf, sync::Arc};

use tracing_error::SpanTrace;

pub struct ErrorMeta(pub u32, pub &'static str);

#[derive(Debug)]
#[non_exhaustive]
pub struct Frame {
  pub n: usize,
  pub name: Option<String>,
  pub lineno: Option<u32>,
  pub filename: Option<PathBuf>,
}

pub type FrameFilter = dyn Fn(&Frame) -> bool + Send + Sync + 'static;
pub type FramesFilter = dyn Fn(&mut Vec<&Frame>) + Send + Sync + 'static;

pub(in crate::error) struct Filters {
  pub frame_filters: Arc<[Box<FrameFilter>]>,
  pub frames_filters: Arc<[Box<FramesFilter>]>,
}

pub struct FiltersBuilder {
  pub(in crate::error) frame_filters: Vec<Box<FrameFilter>>,
  pub(in crate::error) frames_filters: Vec<Box<FramesFilter>>,
}

pub(in crate::error) struct ErrorInner {
  pub meta: &'static ErrorMeta,
  pub source: Option<Box<dyn StdError + Send + Sync>>,
  pub location: Option<&'static Location<'static>>,
  pub backtrace: Option<Backtrace>,
  pub span_trace: Option<SpanTrace>,
  pub contexts: Option<Vec<String>>,
}

pub struct Error {
  pub(in crate::error) inner: Arc<ErrorInner>,
}

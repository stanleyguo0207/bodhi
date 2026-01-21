use std::{
  error::Error as StdError,
  path::PathBuf,
  sync::{Arc, OnceLock},
};

use backtrace::Backtrace;

pub struct ErrorMeta(pub u32, pub &'static str);

#[derive(Debug)]
#[non_exhaustive]
pub struct Frame {
  pub n: usize,
  pub name: Option<String>,
  pub lineno: Option<u32>,
  pub filename: Option<PathBuf>,
}

pub type FramesFilter = dyn Fn(&mut Vec<&Frame>) + Send + Sync + 'static;

pub(in crate::error) struct Filters {
  pub frames_filters: &'static [Box<FramesFilter>],
}

pub(in crate::error) static ERROR_FILTERS: OnceLock<Arc<Filters>> = OnceLock::new();

pub(in crate::error) struct FiltersBuilder {
  pub frames_filters: Vec<Box<FramesFilter>>,
}

pub(in crate::error) struct ErrorInner {
  pub meta: &'static ErrorMeta,
  pub source: Option<Box<dyn StdError + Send + Sync + 'static>>,
  pub backtrace: Option<Backtrace>,
  pub contexts: Option<Vec<String>>,
  pub children: Option<Vec<Error>>,
}

pub struct Error {
  pub(in crate::error) inner: Arc<ErrorInner>,
}

#[cfg(test)]
mod _asserts {
  use super::*;

  trait _AssertSendSync {}
  impl<T> _AssertSendSync for T where T: Send + Sync {}

  fn _assert_error_is_send_sync<T: _AssertSendSync>() {}
  #[test]
  fn error_is_send_sync() {
    _assert_error_is_send_sync::<Error>();
  }
}

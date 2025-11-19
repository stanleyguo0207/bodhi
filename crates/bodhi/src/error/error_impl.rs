use backtrace::Backtrace;
use std::{
  error::Error as StdError,
  fmt::{self, Display},
  sync::Arc,
};

use tracing_error::SpanTrace;

use crate::error::{ERROR_FILTERS, types::ErrorInner};
use crate::{BODHIERR_SYS, Error, ErrorMeta, Frame};

impl Error {
  #[track_caller]
  pub fn new(meta: &'static ErrorMeta) -> Self {
    let location = std::panic::Location::caller();

    Self {
      inner: Arc::new(ErrorInner {
        meta,
        source: None,
        location: Some(location),
        backtrace: None,
        span_trace: SpanTrace::capture().into(),
        contexts: None,
      }),
    }
  }

  pub fn code(&self) -> u32 {
    self.inner.meta.0
  }

  pub fn desc(&self) -> &'static str {
    self.inner.meta.1
  }

  pub fn capture_backtrace(mut self) -> Self {
    Arc::get_mut(&mut self.inner).map(|inner| {
      inner.backtrace = Some(Backtrace::new());
    });

    self
  }

  pub fn capture_span_trace(mut self) -> Self {
    Arc::get_mut(&mut self.inner).map(|inner| {
      inner.span_trace = Some(SpanTrace::capture());
    });

    self
  }

  pub fn wrap_context<D>(mut self, context: D) -> Self
  where
    D: Display + Send + Sync + 'static,
  {
    let ctx_str = context.to_string();

    Arc::get_mut(&mut self.inner).map(|inner| {
      inner.contexts.get_or_insert_with(Vec::new).push(ctx_str);
    });

    self
  }

  pub fn wrap_context_with<F, D>(self, f: F) -> Self
  where
    F: FnOnce() -> D,
    D: Display + Send + Sync + 'static,
  {
    self.wrap_context(f())
  }

  fn debug<E>(&self, error: &(dyn StdError + 'static), f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "code: {}, desc: {}", self.code(), self.desc())?;

    if let Some(contexts) = &self.inner.contexts {
      write!(f, "Contexts:\n")?;
      for (n, ctx) in contexts.iter().enumerate() {
        write!(f, "  {}: {}\n", n, ctx)?;
      }
    }

    write!(f, "Location:\n")?;
    if let Some(location) = &self.inner.location {
      write!(f, "  {}:{}\n", location.file(), location.line())?;
    } else {
      write!(f, "  <unknown>\n")?;
    }

    if let Some(backtrace) = &self.inner.backtrace {
      let frames: Vec<_> = backtrace
        .frames()
        .iter()
        .flat_map(|frame| frame.symbols())
        .zip(1usize..)
        .map(|(sym, n)| Frame {
          n,
          name: sym.name().map(|n| n.to_string()),
          lineno: sym.lineno(),
          filename: sym.filename().map(|x| x.into()),
        })
        .collect();

      let mut filtered_frames: Vec<_> = frames.iter().collect();
      ERROR_FILTERS
        .get()
        .map(|filters| filters.apply(&mut filtered_frames));

      if !filtered_frames.is_empty() {
        for frame in &filtered_frames {
          let file = frame.filename.as_ref().map(|path| path.display());
          let file: &dyn fmt::Display = if let Some(ref filename) = file {
            filename
          } else {
            &"<unknown source file>"
          };

          write!(
            f,
            "  Frame {}: {} ({}:{})\n",
            frame.n,
            frame
              .name
              .as_ref()
              .map(|n| n.as_str())
              .unwrap_or("<unknown>"),
            file,
            frame.lineno.unwrap_or(0)
          )?;
        }
      }
    }

    Ok(())
  }

  fn display(&self, error: &(dyn StdError + 'static), f: &mut fmt::Formatter<'_>) -> fmt::Result {
    Ok(())
  }

  #[track_caller]
  pub fn from_std<E>(error: E) -> Self
  where
    E: StdError + Send + Sync + 'static,
  {
    let location = std::panic::Location::caller();

    Self {
      inner: Arc::new(ErrorInner {
        meta: &BODHIERR_SYS,
        source: Some(Box::new(error)),
        location: Some(location),
        backtrace: None,
        span_trace: SpanTrace::capture().into(),
        contexts: None,
      }),
    }
  }

  #[track_caller]
  pub fn from_msg<D, E>(msg: D, error: E) -> Self
  where
    D: Display + Send + Sync + 'static,
    E: StdError + Send + Sync + 'static,
  {
    let location = std::panic::Location::caller();

    Self {
      inner: Arc::new(ErrorInner {
        meta: &BODHIERR_SYS,
        source: Some(Box::new(error)),
        location: Some(location),
        backtrace: None,
        span_trace: SpanTrace::capture().into(),
        contexts: Some(vec![msg.to_string()]),
      }),
    }
  }
}

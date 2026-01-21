use std::{
  error::Error as StdError,
  fmt::{self, Debug, Display},
  sync::Arc,
};

use backtrace::Backtrace;

use crate::BODHIERR_SYS;
use crate::error::types::{ERROR_FILTERS, Error, ErrorInner, ErrorMeta, Frame};

impl Error {
  pub fn new(meta: &'static ErrorMeta) -> Self {
    Self {
      inner: Arc::new(ErrorInner {
        meta,
        source: None,
        backtrace: None,
        contexts: None,
      }),
    }
    .capture_backtrace()
  }

  pub fn code(&self) -> u32 {
    self.inner.meta.0
  }

  pub fn desc(&self) -> &'static str {
    self.inner.meta.1
  }

  fn capture_backtrace(mut self) -> Self {
    Arc::get_mut(&mut self.inner).map(|inner| {
      inner.backtrace = Some(Backtrace::new());
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

  fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Meta:\n")?;
    write!(f, "  code: {}, desc: {}\n", self.code(), self.desc())?;

    if let Some(source) = &self.inner.source {
      write!(f, "Error:\n")?;
      write!(f, "  {:?}\n", source)?;
    }

    if let Some(contexts) = &self.inner.contexts {
      write!(f, "Contexts:\n")?;
      for (n, ctx) in contexts.iter().enumerate() {
        write!(f, "  {}: {}\n", n, ctx)?;
      }
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
        write!(f, "BACKTRACE:\n")?;
        for frame in &filtered_frames {
          let file = frame.filename.as_ref().map(|path| path.display());
          let file: &dyn fmt::Display = if let Some(ref filename) = file {
            filename
          } else {
            &"<unknown source file>"
          };

          write!(
            f,
            "  {}: {} ({}:{})\n",
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

  fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "code: {}, desc: {}", self.code(), self.desc())?;
    if let Some(source) = &self.inner.source {
      write!(f, ", error: {}", source)?;
    }
    Ok(())
  }

  pub fn from_std<E>(error: E) -> Self
  where
    E: StdError + Send + Sync + 'static,
  {
    Self {
      inner: Arc::new(ErrorInner {
        meta: &BODHIERR_SYS,
        source: Some(Box::new(error)),
        backtrace: None,
        contexts: None,
      }),
    }
    .capture_backtrace()
  }
}

impl Debug for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.debug(f)
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.display(f)
  }
}

impl<E> From<E> for Error
where
  E: StdError + Send + Sync + 'static,
{
  fn from(error: E) -> Self {
    Error::from_std(error)
  }
}

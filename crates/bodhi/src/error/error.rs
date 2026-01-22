use std::{
  error::Error as StdError,
  fmt::{self, Debug, Display},
  sync::Arc,
};

use backtrace::Backtrace;

use crate::BODHIERR_SYS;

use super::filters::ERROR_FILTERS;
use super::frame::Frame;

pub struct ErrorMeta(pub u32, pub &'static str);

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

impl Error {
  pub fn new(meta: &'static ErrorMeta) -> Self {
    Self {
      inner: Arc::new(ErrorInner {
        meta,
        source: None,
        backtrace: None,
        contexts: None,
        children: None,
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

    if let Some(children) = &self.inner.children {
      if !children.is_empty() {
        write!(f, "CHILDREN:\n")?;
        for (n, child) in children.iter().enumerate() {
          write!(f, "CHILD=={}==:\n", n)?;
          child.debug(f)?;
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

  pub fn push_child(&mut self, child: Error) {
    Arc::get_mut(&mut self.inner).map(|inner| {
      inner.children.get_or_insert_with(Vec::new).push(child);
    });
  }

  pub fn is_same_meta(&self, meta: &'static ErrorMeta) -> bool {
    return self.code() == meta.0;
  }

  pub fn has_meta(&self, meta: &'static ErrorMeta) -> bool {
    if self.is_same_meta(meta) {
      return true;
    }

    if let Some(children) = &self.inner.children {
      for child in children {
        if child.is_same_meta(meta) {
          return true;
        }
      }
    }

    false
  }

  fn from_std<E>(error: E) -> Self
  where
    E: StdError + Send + Sync + 'static,
  {
    Self {
      inner: Arc::new(ErrorInner {
        meta: &BODHIERR_SYS,
        source: Some(Box::new(error)),
        backtrace: None,
        contexts: None,
        children: None,
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

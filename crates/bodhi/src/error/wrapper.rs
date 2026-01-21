use std::{error::Error as StdError, fmt::Display, result::Result as StdResult};

use crate::error::{Error, Result};

pub trait WrapContext<T> {
  fn wrap_context<D>(self, context: D) -> Result<T>
  where
    D: Display + Send + Sync + 'static;

  fn wrap_context_with<D, F>(self, f: F) -> Result<T>
  where
    D: Display + Send + Sync + 'static,
    F: FnOnce() -> D;
}

impl<T> WrapContext<T> for Result<T> {
  fn wrap_context<D>(self, context: D) -> Result<T>
  where
    D: Display + Send + Sync + 'static,
  {
    match self {
      Ok(value) => Ok(value),
      Err(e) => Err(e.wrap_context(context)),
    }
  }

  fn wrap_context_with<D, F>(self, f: F) -> Result<T>
  where
    D: Display + Send + Sync + 'static,
    F: FnOnce() -> D,
  {
    match self {
      Ok(value) => Ok(value),
      Err(e) => Err(e.wrap_context_with(f)),
    }
  }
}

impl<T, E> WrapContext<T> for StdResult<T, E>
where
  E: StdError + Send + Sync + 'static,
{
  fn wrap_context<D>(self, context: D) -> Result<T>
  where
    D: Display + Send + Sync + 'static,
  {
    match self {
      Ok(value) => Ok(value),
      Err(e) => Err(Into::<Error>::into(e).wrap_context(context)),
    }
  }

  fn wrap_context_with<D, F>(self, f: F) -> Result<T>
  where
    D: Display + Send + Sync + 'static,
    F: FnOnce() -> D,
  {
    match self {
      Ok(value) => Ok(value),
      Err(e) => Err(Into::<Error>::into(e).wrap_context_with(f)),
    }
  }
}

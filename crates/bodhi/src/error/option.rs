use super::{
  error::{Error, ErrorMeta},
  result::Result,
};

pub trait OptionExt<T> {
  fn ok_or_error(self, meta: &'static ErrorMeta) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
  fn ok_or_error(self, meta: &'static ErrorMeta) -> Result<T> {
    self.ok_or_else(|| Error::new(meta))
  }
}

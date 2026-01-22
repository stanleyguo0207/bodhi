mod error;
mod filters;
mod frame;
mod macros;
mod option;
mod result;
mod wrapper;

pub use error::{Error, ErrorMeta};
pub use filters::FramesFilter;
pub use frame::Frame;
pub use option::OptionExt;
pub use result::Result;
pub use wrapper::WrapContext;

pub fn init() -> Result<()> {
  filters::FiltersBuilder::new().build()?;
  Ok(())
}

#[cfg(test)]
mod test {
  use std::sync::Once;

  use super::*;

  static TEST_INIT: Once = Once::new();

  fn init_test_error_system() {
    TEST_INIT.call_once(|| {
      init().unwrap();
    });
  }

  pub static TESTERR_INVALID: ErrorMeta = ErrorMeta(10000, "Invalid");
  pub static TESTERR_TEST1: ErrorMeta = ErrorMeta(10001, "test1");
  pub static TESTERR_TEST2: ErrorMeta = ErrorMeta(10002, "test2");

  fn g1() -> Result<()> {
    Err(Error::new(&TESTERR_INVALID))
  }

  fn g2() -> Result<()> {
    g1().wrap_context_with(|| format!("called g2"))?;
    Ok(())
  }

  fn g3() -> Result<()> {
    g2().wrap_context("g3")?;
    Ok(())
  }

  #[test]
  fn backtrace() {
    if let Err(e) = g3() {
      // debug
      println!("Error from g3:\n{:?}", e);

      // display
      println!("Error from g3:\n{}", e);
    }
  }

  #[test]
  fn filter() {
    init_test_error_system();

    if let Err(e) = g3() {
      // debug
      println!("Error from g3:\n{:?}", e);

      // display
      println!("Error from g3:\n{}", e);
    }
  }

  #[test]
  fn stdresult() {
    init_test_error_system();

    fn g1() -> std::result::Result<(), std::io::Error> {
      Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "std io failed",
      ))
    }

    fn g2() -> Result<()> {
      g1().wrap_context("g2")?;
      Ok(())
    }

    fn g3() -> Result<()> {
      g2().wrap_context("g3")?;
      Ok(())
    }

    if let Err(e) = g3() {
      // debug
      println!("Error from g3:\n{:?}", e);

      // display
      println!("Error from g3:\n{}", e);
    }
  }

  #[test]
  fn from_stderror() {
    init_test_error_system();

    fn g1() -> Result<()> {
      Err(std::io::Error::new(std::io::ErrorKind::Other, "std io failed").into())
    }

    fn g2() -> Result<()> {
      g1().wrap_context("g2")?;
      Ok(())
    }

    fn g3() -> Result<()> {
      g2().wrap_context("g3")?;
      Ok(())
    }

    if let Err(e) = g3() {
      // debug
      println!("Error from g3:\n{:?}", e);

      // display
      println!("Error from g3:\n{}", e);
    }
  }

  #[test]
  fn test_child_errors() {
    init_test_error_system();

    fn g1() -> Result<()> {
      let mut parent_err = Error::new(&TESTERR_INVALID).wrap_context("parent error");

      let child_err1 = Error::new(&TESTERR_TEST1).wrap_context("child error 1");
      let child_err2 = Error::new(&TESTERR_TEST2).wrap_context("child error 2");

      parent_err.push_child(child_err1);
      parent_err.push_child(child_err2);

      Err(parent_err)
    }

    fn g2() -> Result<()> {
      g1().wrap_context("g2")?;
      Ok(())
    }

    fn g3() -> Result<()> {
      g2().wrap_context("g3")?;
      Ok(())
    }

    if let Err(e) = g3() {
      // debug
      println!("Error from g3:\n{:?}", e);

      // display
      println!("Error from g3:\n{}", e);
    }
  }

  #[test]
  fn test_is_same_meta() {
    init_test_error_system();

    fn g1() -> Result<()> {
      Err(Error::new(&TESTERR_TEST1))
    }

    fn g2() -> Result<()> {
      g1().wrap_context("g2")?;
      Ok(())
    }

    fn g3() -> Result<()> {
      g2().wrap_context("g3")?;
      Ok(())
    }

    if let Err(e) = g3() {
      assert!(e.is_same_meta(&TESTERR_TEST1));
      assert!(!e.is_same_meta(&TESTERR_TEST2));
    }
  }

  #[test]
  fn test_has_meta() {
    init_test_error_system();

    fn g1() -> Result<()> {
      let mut parent_err = Error::new(&TESTERR_INVALID).wrap_context("parent error");

      let child_err1 = Error::new(&TESTERR_TEST1).wrap_context("child error 1");
      let child_err2 = Error::new(&TESTERR_TEST2).wrap_context("child error 2");

      parent_err.push_child(child_err1);
      parent_err.push_child(child_err2);

      Err(parent_err)
    }

    fn g2() -> Result<()> {
      g1().wrap_context("g2")?;
      Ok(())
    }

    fn g3() -> Result<()> {
      g2().wrap_context("g3")?;
      Ok(())
    }

    if let Err(e) = g3() {
      assert!(e.has_meta(&TESTERR_INVALID));
      assert!(e.has_meta(&TESTERR_TEST1));
      assert!(e.has_meta(&TESTERR_TEST2));
    }
  }

  #[test]
  fn test_async_safe() {
    init_test_error_system();

    async fn g1() -> Result<()> {
      Err(Error::new(&TESTERR_INVALID))
    }

    async fn g2() -> Result<()> {
      g1().await.wrap_context("g2")?;
      Ok(())
    }

    async fn g3() -> Result<()> {
      g2().await.wrap_context("g3")?;
      Ok(())
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    if let Err(e) = rt.block_on(g3()) {
      // debug
      println!("Error from g3:\n{:?}", e);

      // display
      println!("Error from g3:\n{}", e);
    }
  }
}

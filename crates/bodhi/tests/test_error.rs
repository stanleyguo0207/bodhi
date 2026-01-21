use std::sync::Once;

use bodhi::{Error, Result, WrapContext};

static TEST_INIT: Once = Once::new();

fn init_test_error_system() {
  TEST_INIT.call_once(|| {
    bodhi::init().unwrap();
  });
}

pub static TESTERR_INVALID: bodhi::ErrorMeta = bodhi::ErrorMeta(10000, "Invalid");
pub static TESTERR_TEST1: bodhi::ErrorMeta = bodhi::ErrorMeta(10001, "test1");
pub static TESTERR_TEST2: bodhi::ErrorMeta = bodhi::ErrorMeta(10002, "test2");

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

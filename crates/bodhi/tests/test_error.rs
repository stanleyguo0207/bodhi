use std::sync::Once;

use bodhi::{Error, Result, WrapContext};

static TEST_INIT: Once = Once::new();

fn init_test_error_system() {
  TEST_INIT.call_once(|| {
    bodhi::init().unwrap();
  });
}

bodhi::define_static_errors!(
  testerr (999 .. 999) {
    Invalid => (999, "Invalid"),
  }
);

fn g1() -> Result<()> {
  Err(Error::new(&TESTERR_INVALID))
}

fn g2() -> Result<()> {
  g1().wrap_context("g2")?;
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
fn stderror() {
  init_test_error_system();

  fn g1() -> Result<()> {
    let e = std::io::Error::new(std::io::ErrorKind::Other, "io failed");
    Err(Error::from_std(e))
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

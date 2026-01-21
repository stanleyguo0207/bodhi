mod globals;

use bodhi::{Error, Result, WrapContext};

use globals::*;

fn f1() -> Result<()> {
  Err(Error::new(&GATEWAYERR_TIMEOUT).capture_backtrace())
}

fn f2() -> Result<()> {
  f1().wrap_context("f2")?;
  Ok(())
}

fn f3() -> Result<()> {
  f2().wrap_context("f3")?;
  Ok(())
}

fn f4() -> Result<()> {
  let e = std::io::Error::new(std::io::ErrorKind::Other, "io failed");
  Err(Error::from_std(e).capture_backtrace())
}

fn f5() -> Result<()> {
  f4().wrap_context("f5")?;
  Ok(())
}

fn f6() -> Result<()> {
  f5().wrap_context("f6")?;
  Ok(())
}

fn f7() -> std::result::Result<(), std::io::Error> {
  Err(std::io::Error::new(
    std::io::ErrorKind::Other,
    "std io failed",
  ))
}

fn f8() -> Result<()> {
  f7().wrap_context("f8")?;
  Ok(())
}

fn f9() -> Result<()> {
  f8().wrap_context("f9")?;
  Ok(())
}

fn main() -> Result<()> {
  println!("Hello, Gateway!");

  if let Err(e) = f3() {
    // debug
    println!("Error from f3:\n{:?}", e);

    // display
    println!("Error from f3:\n{}", e);
  }

  if let Err(e) = f6() {
    // debug
    println!("Error from f6:\n{:?}", e);

    // display
    println!("Error from f6:\n{}", e);
  }

  if let Err(e) = f9() {
    // debug
    println!("Error from f9:\n{:?}", e);

    // display
    println!("Error from f9:\n{}", e);
  }

  Ok(())
}

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

fn main() -> Result<()> {
  println!("Hello, Gateway!");

  // let err = Error::new(&bodhi::BODHIERR_EXTERNAL).capture_backtrace();
  // println!("Full Error:\n{}", err);

  // let err2 = Error::with_source(std::io::Error::new(
  //   std::io::ErrorKind::Other,
  //   "Underlying IO error",
  // ));
  // println!("External Error:\n{}", err2);

  // println!("f3 result: {}", f3().err().unwrap());

  if let Err(e) = f3() {
    println!("Error from f3:\n{}", e);
  }

  Ok(())
}

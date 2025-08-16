#[derive(Debug)]
pub struct Error {
  problem: String,
  cause: String,
  solution: String,
  source: Box<dyn std::error::Error + Send + Sync>,
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "problem: {}\ncause: {}\nsolution: {}\nsource: {}",
      self.problem, self.cause, self.solution, self.source,
    )
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(&*self.source)
  }
}

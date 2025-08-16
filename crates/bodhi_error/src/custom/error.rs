/// Custom error.
#[derive(Debug)]
pub struct Error {
  problem: String,
  solution: String,
  source: Box<dyn std::error::Error + Send + Sync>,
}

impl Error {
  pub fn new(
    problem: &str,
    solution: &str,
    source: Box<dyn std::error::Error + Send + Sync>,
  ) -> Self {
    Self {
      problem: problem.to_string(),
      solution: solution.to_string(),
      source,
    }
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "{}", self.problem)?;
    writeln!(f, "solution: {}", self.solution)?;
    writeln!(f, "source: {}", self.source)
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(&*self.source)
  }
}

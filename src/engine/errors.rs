use core::fmt;

#[derive(Debug)]
pub enum EngineError {
  ArgumentError {
    index: u8,
    name: String
  },
  ModelLoadError {
    err: anyhow::Error,
    filename: String
  },
  MaxComponentsError {
    insertion_loc: String
  },
  Custom(String)
}

impl fmt::Display for EngineError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::ArgumentError {index, name} => write!(f, "Invalid argument at index {}: {}", index, name),
      Self::ModelLoadError { err, filename, } => write!(f, "Failed to load file at path {}", filename),
      Self::MaxComponentsError { insertion_loc } => write!(f, "Maximum number of components added to scene. Insertion at function {} invalid", insertion_loc),
      Self::Custom(ref err) => write!(f, "Error: {}", err),
    }
  }
}

impl std::error::Error for EngineError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::ArgumentError { index, name } => None,
      Self::ModelLoadError { err, filename } => err.source(),
      Self::MaxComponentsError { insertion_loc } => None,
      Self::Custom(ref err) => None,
    }
  }

  fn description(&self) -> &str {
    match self {
      EngineError::ArgumentError { index, name } => "Invalid argument provided",
      EngineError::ModelLoadError { err, filename } => "Failed to load model for given filepath",
      Self::MaxComponentsError { insertion_loc } => "Component store full",
      EngineError::Custom(ref err) => "Unknown error type",
    }
  }
}
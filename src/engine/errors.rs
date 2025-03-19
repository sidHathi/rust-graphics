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
  StateAccessError {
    state_key: String
  },
  LockError {
    message: String
  },
  Custom(String)
}

impl fmt::Display for EngineError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::ArgumentError {index, name} => write!(f, "Invalid argument at index {}: {}", index, name),
      Self::ModelLoadError { err: _, filename, } => write!(f, "Failed to load file at path {}", filename),
      Self::StateAccessError { state_key } => write!(f, "Unable to access state variable with key {}", state_key),
      Self::MaxComponentsError { insertion_loc } => write!(f, "Maximum number of components added to scene. Insertion at function {} invalid", insertion_loc),
      Self::LockError { message } => write!(f, "{}", message),
      Self::Custom(ref err) => write!(f, "Error: {}", err),
    }
  }
}

impl std::error::Error for EngineError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::ArgumentError { index: _, name: _ } => None,
      Self::ModelLoadError { err, filename: _ } => err.source(),
      Self::MaxComponentsError { insertion_loc: _ } => None,
      Self::StateAccessError { state_key: _ } => None,
      Self::LockError { message: _ } => None,
      Self::Custom(ref _err) => None,
    }
  }

  fn description(&self) -> &str {
    match self {
      EngineError::ArgumentError { index: _, name: _ } => "Invalid argument provided",
      EngineError::ModelLoadError { err: _, filename: _ } => "Failed to load model for given filepath",
      Self::MaxComponentsError { insertion_loc: _ } => "Component store full",
      Self::StateAccessError { state_key: _ } => "State access attempt failed",
      Self::LockError { message: _ } => "Lock attempt failed",
      EngineError::Custom(ref _err) => "Unknown error type",
    }
  }
}
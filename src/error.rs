  // src/errors.rs - Unified Error Handling
  
  use thiserror::Error;
  
  /// BL1NK Error Types
  #[derive(Error, Debug)]
  pub enum BlinkError {
      #[error("Configuration error: {0}")]
      ConfigError(String),
  
      #[error("Execution error: {0}")]
      ExecutionError(String),
  
      #[error("Git error: {0}")]
      GitError(#[from] git2::Error),
  
      #[error("Provider error: {0}")]
      ProviderError(String),
  
      #[error("IO error: {0}")]
      IoError(#[from] std::io::Error),
  
      #[error("Serialization error: {0}")]
      SerializationError(#[from] serde_json::Error),
  
      #[error("Session not found: {0}")]
      SessionNotFound(String),
  
      #[error("Invalid state: {0}")]
      InvalidState(String),
  
      #[error("Unknown error")]
      Unknown,
  }
  
  /// Result type alias for BL1NK operations
  pub type Result<T> = std::result::Result<T, BlinkError>;
  // src/providers/base.rs - Provider Interface
  
  use async_trait::async_trait;
  use serde_json::Value;
  use futures::stream::BoxStream;
  use crate::Result;
  
  /// Provider response structure
  #[derive(Debug, Clone)]
  pub struct ProviderResponse {
      pub content: String,
      pub tokens_used: u32,
      pub model: String,
  }
  
  /// AI Provider trait
  #[async_trait]
  pub trait Provider: Send + Sync {
      /// Execute a prompt and get response
      async fn execute(
          &self,
          prompt: &str,
          context: Option<Value>,
      ) -> Result<ProviderResponse>;
  
      /// Stream execution for real-time output
      async fn stream_execute(
          &self,
          prompt: &str,
          context: Option<Value>,
      ) -> Result<BoxStream<'static, Result<String>>>;
  
      /// Get provider name
      fn name(&self) -> &str;
  }
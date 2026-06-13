//! Model information and management operations for `HuggingFace` API.

mod private
{
use crate::
{
  client::Client,
  components::
  {
  models::ModelInfo,
  // common::TaskType,
  },
  error::Result,
  validation::validate_model_identifier,
};

#[ cfg( feature = "env-config" ) ]
use crate::environment::{ HuggingFaceEnvironment, EnvironmentInterface };

use serde::{ Deserialize, Serialize };
use core::time::Duration;

/// HuggingFace Hub API base URL — separate from the inference API base URL
const HF_HUB_API_BASE : &str = "https://huggingface.co/api";

/// Configuration for model waiting behavior
#[ derive( Debug, Clone ) ]
pub struct ModelWaitConfig
{
  /// Polling interval between status checks
  pub poll_interval : Duration,
}

impl ModelWaitConfig
{
  /// Create explicit model wait configuration
  #[ inline ]
  #[ must_use ]
  pub fn with_explicit_config( poll_interval : Duration ) -> Self
  {
  Self { poll_interval }
  }

  /// Create model wait configuration with HuggingFace-recommended values
  ///
  /// # Governing Principle Compliance
  ///
  /// This provides HuggingFace-recommended polling configuration without making it implicit.
  /// Developers must explicitly choose to use these recommended values.
  #[ inline ]
  #[ must_use ]
  pub fn recommended() -> Self
  {
  Self
  {
      poll_interval : Duration::from_secs( 5 ), // Balanced polling for model loading
  }
  }

  /// Create conservative configuration for production environments
  #[ inline ]
  #[ must_use ]
  pub fn conservative() -> Self
  {
  Self
  {
      poll_interval : Duration::from_secs( 10 ), // Longer intervals to reduce API load
  }
  }

  /// Create aggressive configuration for development/testing
  #[ inline ]
  #[ must_use ]
  pub fn aggressive() -> Self
  {
  Self
  {
      poll_interval : Duration::from_secs( 2 ), // Faster polling for development
  }
  }
}

/// API group for `HuggingFace` model operations
#[ derive( Debug ) ]
pub struct Models< E >
where
  E : Clone,
{
  client : Client< E >,
}

#[ cfg( feature = "env-config" ) ]
impl< E > Models< E >
where
  E : HuggingFaceEnvironment + EnvironmentInterface + Send + Sync + 'static + Clone,
{
  /// Create a new Models API group
  #[ inline ]
  #[ must_use ]
  pub fn new( client : &Client< E > ) -> Self
  {
  Self
  {
      client : client.clone(),
  }
  }
  
  /// Get information about a specific model
  ///
  /// # Arguments
  /// - `model_id`: Model identifier (e.g., "gpt2", "meta-llama/Llama-2-7b-hf")
  ///
  /// # Errors
  /// Returns error if the model is not found or request fails
  #[ inline ]
  pub async fn get( &self, model_id : impl AsRef< str > ) -> Result< ModelInfo >
  {
  let model_ref = model_id.as_ref();
  
  // Validate model identifier
  validate_model_identifier( model_ref )?;
  
  let url = format!( "{HF_HUB_API_BASE}/models/{model_ref}" );

  self.client.get( &url ).await
  }
  
  /// Check if a model is available for inference
  ///
  /// # Arguments
  /// - `model_id`: Model identifier to check
  ///
  /// # Errors
  /// Returns error if the availability check fails
  #[ inline ]
  pub async fn is_available( &self, model_id : impl AsRef< str > ) -> Result< bool >
  {
  let model_ref = model_id.as_ref();
  
  // Validate model identifier
  validate_model_identifier( model_ref )?;
  
  let endpoint = format!( "/models/{model_ref}" );
  let url = self.client.environment.endpoint_url( &endpoint )?;
  
  // Send a minimal request to check availability
  let test_request = serde_json::json!
  ({
      "inputs": "test",
      "options": {
  "wait_for_model": false
      }
  });
  
  match self.client.post::< serde_json::Value, serde_json::Value >( url.as_str(), &test_request ).await
  {
      Ok( _ ) => Ok( true ),
      Err( _ ) => Ok( false ),
  }
  }
  
  /// Get model status information
  ///
  /// # Arguments
  /// - `model_id`: Model identifier
  ///
  /// # Errors
  /// Returns error if the status check fails
  #[ inline ]
  pub async fn status( &self, model_id : impl AsRef< str > ) -> Result< ModelStatus >
  {
  let model_ref = model_id.as_ref();
  
  // Validate model identifier
  validate_model_identifier( model_ref )?;
  
  let endpoint = format!( "/models/{model_ref}" );
  let url = self.client.environment.endpoint_url( &endpoint )?;
  
  // Send a minimal request with wait_for_model=false to get status
  let status_request = serde_json::json!
  ({
      "inputs": "status check",
      "options": {
  "wait_for_model": false
      }
  });
  
  match self.client.post::< serde_json::Value, serde_json::Value >( url.as_str(), &status_request ).await
  {
      Ok( _ ) => Ok( ModelStatus::Available ),
      Err( e ) =>
      {
  let error_msg = e.to_string().to_lowercase();
  if error_msg.contains( "loading" ) || error_msg.contains( "cold" )
  {
          Ok( ModelStatus::Loading )
  }
  else if error_msg.contains( "not found" ) || error_msg.contains( "does not exist" )
  {
          Ok( ModelStatus::NotFound )
  }
  else
  {
          Ok( ModelStatus::Error( e.to_string() ) )
  }
      }
  }
  }
  
  /// Wait for a model to become available with explicit configuration
  ///
  /// # Governing Principle Compliance
  ///
  /// This requires explicit configuration for polling behavior, providing full transparency
  /// and control over model waiting strategy.
  ///
  /// # Arguments
  /// - `model_id`: Model identifier to wait for
  /// - `timeout_secs`: Maximum time to wait in seconds
  /// - `wait_config`: Explicit configuration for polling behavior
  ///
  /// # Errors
  /// Returns error if the model doesn't become available within timeout
  #[ inline ]
  pub async fn wait_for_model_with_config(
  &self,
  model_id : impl AsRef< str >,
  timeout_secs : u64,
  wait_config : ModelWaitConfig,
  ) -> Result< () >
  {
  use tokio::time::sleep;

  let model_ref = model_id.as_ref();

  // Validate model identifier
  validate_model_identifier( model_ref )?;

  let mut elapsed = 0;
  let poll_interval_secs = wait_config.poll_interval.as_secs();

  while elapsed < timeout_secs
  {
      match self.status( model_ref ).await?
      {
  ModelStatus::Available => return Ok( () ),
  ModelStatus::Loading =>
  {
          sleep( wait_config.poll_interval ).await;
          elapsed += poll_interval_secs;
  },
  ModelStatus::NotFound =>
  {
          return Err( crate::error::HuggingFaceError::ModelUnavailable(
      format!( "Model '{model_ref}' not found" )
          ) );
  },
  ModelStatus::Error( msg ) =>
  {
          return Err( crate::error::HuggingFaceError::ModelUnavailable(
      format!( "Model '{model_ref}' error : {msg}" )
          ) );
  }
      }
  }

  Err( crate::error::HuggingFaceError::ModelUnavailable(
      format!( "Model '{model_ref}' did not become available within {timeout_secs} seconds" )
  ) )
  }

  /// Wait for a model to become available with recommended configuration
  ///
  /// # Governing Principle Compliance
  ///
  /// This provides HuggingFace-recommended waiting configuration without making it implicit.
  /// Developers must explicitly choose to use this recommended approach.
  ///
  /// # Arguments
  /// - `model_id`: Model identifier to wait for
  /// - `timeout_secs`: Maximum time to wait in seconds
  ///
  /// # Errors
  /// Returns error if the model doesn't become available within timeout
  #[ inline ]
  pub async fn wait_for_model(
  &self,
  model_id : impl AsRef< str >,
  timeout_secs : u64
  ) -> Result< () >
  {
  self.wait_for_model_with_config(
      model_id,
      timeout_secs,
      ModelWaitConfig::recommended()
  ).await
  }
}

// Basic implementation for when env-config is not available
#[ cfg( not( feature = "env-config" ) ) ]
impl< E > Models< E >
where
  E : Clone,
{
  /// Create a new Models API group
  #[ inline ]
  #[ must_use ]
  pub fn new( client : &Client< E > ) -> Self
  {
  Self
  {
      client : (*client).clone(),
  }
  }
}

/// Status of a `HuggingFace` model
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub enum ModelStatus
{
  /// Model is available and ready for inference
  Available,
  
  /// Model is currently loading
  Loading,
  
  /// Model was not found
  NotFound,
  
  /// Model encountered an error
  Error( String ),
}

} // end mod private

crate::mod_interface!
{
  exposed use 
  {
  private::Models,
  private::ModelStatus,
  };
}
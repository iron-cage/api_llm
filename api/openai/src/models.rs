// src/api/models.rs
//! This module defines the `Models` API client, which provides methods
//! for interacting with the `OpenAI` Models API.
//!
//! For more details, refer to the [`OpenAI` Models API documentation](https://platform.openai.com/docs/api-reference/models).

/// Define a private namespace for all its items.
mod private
{
  // Use crate root for base access
  use crate::
  {
    client ::Client,
    error ::Result,
    environment ::{ OpenaiEnvironment, EnvironmentInterface },
  };
  use crate::components::models::
  {
    Model,
    ListModelsResponse,
    EnhancedModel,
    EnhancedListModelsResponse,
    ModelPricing,
    ModelCapabilities,
    ModelLimitations,
    ModelLifecycle,
    ModelStatus,
    ResponseMetadata,
  };

  // External crates
  use serde_json;

  /// The client for the `OpenAI` Models API.
  #[ derive( Debug, Clone ) ]
  pub struct Models< 'client, E >
  where
    E : OpenaiEnvironment + EnvironmentInterface + Send + Sync + 'static,
  {
    client : &'client Client< E >,
  }

  impl< 'client, E > Models< 'client, E >
  where
    E : OpenaiEnvironment + EnvironmentInterface + Send + Sync + 'static,
  {
    /// Creates a new `Models` client.
    ///
    /// # Arguments
    /// - `client`: The core `OpenAI` `Client` to use for requests.
    #[ inline ]
    pub(crate) fn new( client : &'client Client< E > ) -> Self
    {
      Self { client }
    }

    /// Lists all available models.
    ///
    /// # Errors
    /// Returns `OpenAIError` if the request fails.
    #[ inline ]
    pub async fn list( &self ) -> Result< ListModelsResponse >
    {
      self.client.get( "models" ).await
    }

    /// Retrieves a model.
    ///
    /// # Arguments
    /// - `model_id`: The ID of the model to retrieve.
    ///
    /// # Errors
    /// Returns `OpenAIError` if the request fails.
    #[ inline ]
    pub async fn retrieve( &self, model_id : &str ) -> Result< Model >
    {
      let path = format!( "/models/{model_id}" );
      self.client.get( &path ).await
    }

    /// Deletes a fine-tuned model.
    ///
    /// # Arguments
    /// - `model_id`: The ID of the model to delete.
    ///
    /// # Errors
    /// Returns `OpenAIError` if the request fails.
    #[ inline ]
    pub async fn delete( &self, model_id : &str ) -> Result< serde_json::Value >
    {
      let path = format!( "/models/{model_id}" );
      self.client.delete( &path ).await
    }

    /// Lists all available models with enhanced metadata.
    ///
    /// This method provides comprehensive model information including pricing,
    /// capabilities, limitations, and lifecycle information.
    ///
    /// # Errors
    /// Returns `OpenAIError` if the request fails.
    #[ inline ]
    pub async fn list_enhanced( &self ) -> Result< EnhancedListModelsResponse >
    {
      // First get the basic model list
      let basic_response : ListModelsResponse = self.client.get( "/models" ).await?;

      // Convert to enhanced models with metadata
      let mut enhanced_models = Vec::new();
      let mut active_count = 0;
      let mut deprecated_count = 0;
      let mut beta_count = 0;

      for model in basic_response.data
      {
        let enhanced_model = Self::enhance_model( model );

        // Count by status
        match enhanced_model.lifecycle.status
        {
          ModelStatus::Active => active_count += 1,
          ModelStatus::Deprecated => deprecated_count += 1,
          ModelStatus::Beta => beta_count += 1,
          ModelStatus::Sunset => {},
        }

        enhanced_models.push( enhanced_model );
      }

      let metadata = ResponseMetadata
      {
        total_models : enhanced_models.len().try_into().unwrap_or( u32::MAX ),
        active_models : active_count,
        deprecated_models : deprecated_count,
        beta_models : beta_count,
      };

      Ok( EnhancedListModelsResponse
      {
        object : basic_response.object,
        data : enhanced_models,
        metadata,
      })
    }

    /// Retrieves an enhanced model with comprehensive metadata.
    ///
    /// # Arguments
    /// - `model_id`: The ID of the model to retrieve.
    ///
    /// # Errors
    /// Returns `OpenAIError` if the request fails.
    #[ inline ]
    pub async fn retrieve_enhanced( &self, model_id : &str ) -> Result< EnhancedModel >
    {
      // First get the basic model
      let basic_model : Model = self.retrieve( model_id ).await?;

      // Enhance with metadata
      Ok( Self::enhance_model( basic_model ) )
    }

    /// Internal method to enhance a basic model with additional metadata.
    ///
    /// # Arguments
    /// - `model`: The basic model to enhance.
    ///
    fn enhance_model( model : Model ) -> EnhancedModel
    {
      // Determine pricing based on model ID patterns
      let pricing = Self::get_model_pricing( &model.id );

      // Determine capabilities based on model ID patterns
      let capabilities = Self::get_model_capabilities( &model.id );

      // Determine limitations based on model ID patterns
      let limitations = Self::get_model_limitations( &model.id );

      // Determine lifecycle status based on model ID patterns
      let lifecycle = Self::get_model_lifecycle( &model.id );

      EnhancedModel
      {
        id : model.id,
        created : model.created,
        object : model.object,
        owned_by : model.owned_by,
        pricing,
        capabilities,
        limitations,
        lifecycle,
      }
    }

    /// Get pricing information for a model based on its ID.
    fn get_model_pricing( model_id : &str ) -> Option< ModelPricing >
    {
      // This would typically come from a pricing API or configuration
      // For now, we'll use hardcoded values based on known model patterns
      match model_id
      {
        id if id.starts_with( "gpt-5.1-chat-latest" ) => Some( ModelPricing
        {
          input_cost_per_1k_tokens : 0.005,
          output_cost_per_1k_tokens : 0.015,
          currency : "USD".to_string(),
          effective_date : "2024-01-01".to_string(),
        }),
        id if id.starts_with( "gpt-4" ) => Some( ModelPricing
        {
          input_cost_per_1k_tokens : 0.03,
          output_cost_per_1k_tokens : 0.06,
          currency : "USD".to_string(),
          effective_date : "2024-01-01".to_string(),
        }),
        id if id.starts_with( "gpt-3.5" ) => Some( ModelPricing
        {
          input_cost_per_1k_tokens : 0.001,
          output_cost_per_1k_tokens : 0.002,
          currency : "USD".to_string(),
          effective_date : "2024-01-01".to_string(),
        }),
        id if id.contains( "embedding" ) => Some( ModelPricing
        {
          input_cost_per_1k_tokens : 0.0001,
          output_cost_per_1k_tokens : 0.0,
          currency : "USD".to_string(),
          effective_date : "2024-01-01".to_string(),
        }),
        _ => None,
      }
    }

    /// Get capabilities for a model based on its ID.
    fn get_model_capabilities( model_id : &str ) -> ModelCapabilities
    {
      match model_id
      {
        id if id.starts_with( "gpt-5.1-chat-latest" ) => ModelCapabilities
        {
          supports_function_calling : true,
          supports_vision : true,
          supports_streaming : true,
          max_context_window : 128_000,
          max_output_tokens : 4096,
          supported_formats : vec![ "text".to_string(), "image".to_string() ],
        },
        id if id.starts_with( "gpt-4" ) && id.contains( "vision" ) => ModelCapabilities
        {
          supports_function_calling : true,
          supports_vision : true,
          supports_streaming : true,
          max_context_window : 8192,
          max_output_tokens : 4096,
          supported_formats : vec![ "text".to_string(), "image".to_string() ],
        },
        id if id.starts_with( "gpt-4" ) => ModelCapabilities
        {
          supports_function_calling : true,
          supports_vision : false,
          supports_streaming : true,
          max_context_window : 8192,
          max_output_tokens : 4096,
          supported_formats : vec![ "text".to_string() ],
        },
        id if id.starts_with( "gpt-3.5" ) => ModelCapabilities
        {
          supports_function_calling : true,
          supports_vision : false,
          supports_streaming : true,
          max_context_window : 4096,
          max_output_tokens : 4096,
          supported_formats : vec![ "text".to_string() ],
        },
        id if id.contains( "embedding" ) => ModelCapabilities
        {
          supports_function_calling : false,
          supports_vision : false,
          supports_streaming : false,
          max_context_window : 8192,
          max_output_tokens : 0,
          supported_formats : vec![ "text".to_string() ],
        },
        _ => ModelCapabilities
        {
          supports_function_calling : false,
          supports_vision : false,
          supports_streaming : false,
          max_context_window : 2048,
          max_output_tokens : 2048,
          supported_formats : vec![ "text".to_string() ],
        },
      }
    }

    /// Get limitations for a model based on its ID.
    fn get_model_limitations( model_id : &str ) -> ModelLimitations
    {
      match model_id
      {
        id if id.starts_with( "gpt-4" ) => ModelLimitations
        {
          rate_limit_rpm : Some( 500 ),
          rate_limit_tpm : Some( 30000 ),
          concurrent_requests : Some( 50 ),
        },
        id if id.starts_with( "gpt-3.5" ) => ModelLimitations
        {
          rate_limit_rpm : Some( 3500 ),
          rate_limit_tpm : Some( 90000 ),
          concurrent_requests : Some( 200 ),
        },
        id if id.contains( "embedding" ) => ModelLimitations
        {
          rate_limit_rpm : Some( 3000 ),
          rate_limit_tpm : Some( 1_000_000 ),
          concurrent_requests : Some( 100 ),
        },
        _ => ModelLimitations
        {
          rate_limit_rpm : Some( 1000 ),
          rate_limit_tpm : Some( 50000 ),
          concurrent_requests : Some( 20 ),
        },
      }
    }

    /// Get lifecycle information for a model based on its ID.
    fn get_model_lifecycle( model_id : &str ) -> ModelLifecycle
    {
      match model_id
      {
        // Deprecated models
        "text-davinci-003" | "text-davinci-002" | "code-davinci-002" => ModelLifecycle
        {
          status : ModelStatus::Deprecated,
          deprecation_date : Some( "2024-01-04".to_string() ),
          sunset_date : Some( "2024-09-13".to_string() ),
          replacement_model : Some( "gpt-3.5-turbo-instruct".to_string() ),
        },
        "text-embedding-ada-002" if model_id.contains( "001" ) => ModelLifecycle
        {
          status : ModelStatus::Deprecated,
          deprecation_date : Some( "2023-01-01".to_string() ),
          sunset_date : Some( "2024-01-01".to_string() ),
          replacement_model : Some( "text-embedding-ada-002".to_string() ),
        },
        // Beta models
        id if id.contains( "preview" ) || id.contains( "beta" ) => ModelLifecycle
        {
          status : ModelStatus::Beta,
          deprecation_date : None,
          sunset_date : None,
          replacement_model : None,
        },
        // Active models (default)
        _ => ModelLifecycle
        {
          status : ModelStatus::Active,
          deprecation_date : None,
          sunset_date : None,
          replacement_model : None,
        },
      }
    }

    /// Generate a cURL command for listing models.
    ///
    /// # Errors
    /// Returns `OpenAIError` if curl generation fails.
    #[ inline ]
    pub fn list_to_curl( &self ) -> crate::error::Result< String >
    {
      let base_url = self.client.environment.base_url();
      let url = format!( "{base_url}models" );

      let mut headers = vec![
        ( "User-Agent".to_string(), "api-openai/1.0.0".to_string() ),
      ];

      // Add headers from environment (includes authorization, organization, project)
      let env_headers = self.client.environment.headers().map_err( |e|
        crate ::error::OpenAIError::Internal( format!( "Failed to get headers from environment : {e}" ) )
      )?;

      for ( key, value ) in &env_headers
      {
        if let Ok( value_str ) = value.to_str()
        {
          headers.push( ( key.as_str().to_string(), value_str.to_string() ) );
        }
      }

      let curl_request = crate::curl_generation::build_curl_request( "GET", &url, &headers, None );
      Ok( curl_request.to_curl_command() )
    }

    /// Generate a cURL command for retrieving a specific model.
    ///
    /// # Arguments
    /// - `model_id`: The ID of the model to retrieve.
    ///
    /// # Errors
    /// Returns `OpenAIError` if curl generation fails.
    #[ inline ]
    pub fn retrieve_to_curl( &self, model_id : &str ) -> crate::error::Result< String >
    {
      let base_url = self.client.environment.base_url();
      let url = format!( "{base_url}models/{model_id}" );

      let mut headers = vec![
        ( "User-Agent".to_string(), "api-openai/1.0.0".to_string() ),
      ];

      // Add headers from environment (includes authorization, organization, project)
      let env_headers = self.client.environment.headers().map_err( |e|
        crate ::error::OpenAIError::Internal( format!( "Failed to get headers from environment : {e}" ) )
      )?;

      for ( key, value ) in &env_headers
      {
        if let Ok( value_str ) = value.to_str()
        {
          headers.push( ( key.as_str().to_string(), value_str.to_string() ) );
        }
      }

      let curl_request = crate::curl_generation::build_curl_request( "GET", &url, &headers, None );
      Ok( curl_request.to_curl_command() )
    }
  }
} // end mod private

crate ::mod_interface!
{
  // Expose all structs defined in this module
  exposed use
  {
    Models,
  };
}
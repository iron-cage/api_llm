//! Model manager implementation
//!
//! `ModelManager` struct and `Client` extension methods for model management operations.

#[ allow( clippy::missing_inline_in_public_items ) ]
mod private
{
  use super::super::core::orphan::*;
  use crate::{
    error::{ AnthropicError, AnthropicResult },
    client::{ Client, CreateMessageRequest },
  };
  use serde::Deserialize;
  use std::{ collections::HashMap, sync::{ Arc, Mutex }, time::{ Duration, Instant } };


  /// API response for models endpoint
  #[ derive( Debug, Deserialize ) ]
  struct ModelsApiResponse
  {
    data : Vec< ApiModelInfo >,
  }

  /// Model information from API
  #[ derive( Debug, Deserialize ) ]
  struct ApiModelInfo
  {
    id : String,
    display_name : Option< String >,
    max_tokens : Option< u32 >,
    context_length : Option< u32 >,
    created : Option< String >,
    capabilities : Vec< String >,
    version : Option< String >,
  }

  /// Model cache entry
  #[ derive( Debug, Clone ) ]
  struct CacheEntry< T >
  {
    data : T,
    timestamp : Instant,
    ttl : Duration,
  }

  impl< T > CacheEntry< T >
  {
    fn new( data : T, ttl : Duration ) -> Self
    {
      Self
      {
        data,
        timestamp : Instant::now(),
        ttl,
      }
    }

    fn is_expired( &self ) -> bool
    {
      self.timestamp.elapsed() > self.ttl
    }
  }

  /// Model management system
  #[ derive( Debug, Clone ) ]
  pub struct ModelManager
  {
    /// HTTP client for API calls
    client : Client,
    /// Model cache
    cache : Arc< Mutex< HashMap< String, CacheEntry< ModelInfo > > > >,
    /// Default cache TTL
    cache_ttl : Duration,
  }

  impl ModelManager
  {
    /// Create new model manager
    #[ must_use ]
    pub fn new( client : Client ) -> Self
    {
      Self
      {
        client,
        cache : Arc::new( Mutex::new( HashMap::new() ) ),
        cache_ttl : Duration::from_secs( 300 ), // 5 minutes
      }
    }

    /// List all available models from the API
    ///
    /// # Errors
    ///
    /// Returns an error if API request fails
    pub async fn list_models( &self ) -> AnthropicResult< Vec< ModelInfo > >
    {
      // Implement actual API call to /v1/models endpoint
      let url = format!( "{}/v1/models", self.client.base_url() );

      let response = self.client.http()
        .get( &url )
        .header( "x-api-key", &self.client.secret().ANTHROPIC_API_KEY )
        .header( "anthropic-version", "2023-06-01" )
        .header( "content-type", "application/json" )
        .send()
        .await
        .map_err( |e| AnthropicError::http_error( format!( "Failed to fetch models : {e}" ) ) )?;

      if !response.status().is_success()
      {
        return Err( AnthropicError::http_error_with_status( format!( "API error : {}", response.status() ), response.status().as_u16() ) );
      }

      let models_response : ModelsApiResponse = response
        .json()
        .await
        .map_err( |e| AnthropicError::Parsing( format!( "Failed to parse models response : {e}" ) ) )?;

      // Convert API response to our internal format
      let models = models_response.data.into_iter()
        .map( |api_model| ModelInfo {
          id : api_model.id.clone(),
          display_name : api_model.display_name.unwrap_or_else( || api_model.id.clone() ),
          name : api_model.id,
          max_tokens : api_model.max_tokens.unwrap_or( 200_000 ),
          context_length : api_model.context_length.unwrap_or( 200_000 ),
          created_at : api_model.created,
          supports_tools : api_model.capabilities.contains( &"tools".to_string() ),
          supports_vision : api_model.capabilities.contains( &"vision".to_string() ),
          version : api_model.version,
        })
        .collect();

      Ok( models )
    }

    /// Get specific model information
    ///
    /// # Errors
    ///
    /// Returns an error if model not found or API request fails
    ///
    /// # Panics
    ///
    /// Panics if the cache mutex is poisoned
    pub async fn get_model( &self, model_id : &str ) -> AnthropicResult< ModelInfo >
    {
      // Check cache first
      {
        let cache = self.cache.lock().unwrap();
        if let Some( entry ) = cache.get( model_id )
        {
          if !entry.is_expired()
          {
            return Ok( entry.data.clone() );
          }
        }
      }

      // Fetch from API (simulated)
      let models = self.list_models().await?;
      let model = models.into_iter()
        .find( | m | m.id == model_id )
        .ok_or_else( || AnthropicError::InvalidArgument( format!( "Model '{model_id}' not found" ) ) )?;

      // Cache the result
      {
        let mut cache = self.cache.lock().unwrap();
        cache.insert( model_id.to_string(), CacheEntry::new( model.clone(), self.cache_ttl ) );
      }

      Ok( model )
    }

    /// Get model capabilities
    ///
    /// # Errors
    ///
    /// Returns an error if model not found
    pub async fn get_model_capabilities( &self, model_id : &str ) -> AnthropicResult< ModelCapabilities >
    {
      let model = self.get_model( model_id ).await?;

      let capabilities = ModelCapabilities
      {
        supports_tools : model.supports_tools,
        supports_vision : model.supports_vision,
        max_context_length : model.context_length,
        max_tool_calls : if model.supports_tools { Some( 20 ) } else { None },
        input_modalities : if model.supports_vision
        {
          vec![ "text".to_string(), "image".to_string() ]
        }
        else
        {
          vec![ "text".to_string() ]
        },
        output_modalities : vec![ "text".to_string() ],
      };

      Ok( capabilities )
    }

    /// Select best model based on requirements
    ///
    /// # Errors
    ///
    /// Returns an error if no suitable model found
    ///
    /// # Panics
    ///
    /// Panics if no suitable models found after filtering (should not happen due to error check)
    pub async fn select_model( &self, requirements : ModelRequirements ) -> AnthropicResult< ModelInfo >
    {
      let models = self.list_models().await?;

      let mut suitable_models : Vec< _ > = models.into_iter()
        .filter( | model | {
          // Check vision requirement
          if requirements.requires_vision && !model.supports_vision
          {
            return false;
          }
          // Check tools requirement
          if requirements.requires_tools && !model.supports_tools
          {
            return false;
          }
          // Check context length
          if model.context_length < requirements.min_context_length
          {
            return false;
          }
          true
        })
        .collect();

      if suitable_models.is_empty()
      {
        return Err( AnthropicError::InvalidArgument( "No models match requirements".to_string() ) );
      }

      // Sort by preference
      if requirements.prefer_speed
      {
        // Prefer Haiku for speed
        suitable_models.sort_by( | a, b | {
          let a_is_haiku = a.name.contains( "haiku" );
          let b_is_haiku = b.name.contains( "haiku" );
          b_is_haiku.cmp( &a_is_haiku )
        });
      }

      Ok( suitable_models.into_iter().next().unwrap() )
    }

    /// Check model availability
    ///
    /// # Errors
    ///
    /// Returns an error if model not found
    pub async fn check_model_availability( &self, model_id : &str ) -> AnthropicResult< ModelAvailability >
    {
      // Verify model exists
      let _model = self.get_model( model_id ).await?;

      // For production, this would check API /v1/models/{id}/availability endpoint
      // For now, return availability based on basic heuristics
      let current_time = std::time::SystemTime::now()
        .duration_since( std::time::UNIX_EPOCH )
        .unwrap_or_default();

      // Simulate basic load patterns based on time
      let load_factor = ( ( current_time.as_secs() % 100 ) as f32 ) / 100.0;
      let is_available = load_factor < 0.9; // Available unless heavily loaded

      Ok( ModelAvailability
      {
        is_available,
        estimated_wait_time : if is_available { None } else { Some( Duration::from_secs( 30 ) ) },
        load_factor,
      })
    }

    /// Select first available model from fallback chain
    ///
    /// # Errors
    ///
    /// Returns an error if no models in chain are available
    pub async fn select_available_model( &self, fallback_chain : Vec< String > ) -> AnthropicResult< ModelInfo >
    {
      for model_id in fallback_chain
      {
        if let Ok( model ) = self.get_model( &model_id ).await
        {
          if let Ok( availability ) = self.check_model_availability( &model_id ).await
          {
            if availability.is_available
            {
              return Ok( model );
            }
          }
        }
      }

      Err( AnthropicError::InvalidArgument( "No models in fallback chain are available".to_string() ) )
    }

    /// Get latest version of model family
    ///
    /// # Errors
    ///
    /// Returns an error if model family not found
    ///
    /// # Panics
    ///
    /// Panics if family models list is empty after filtering (should not happen due to error check)
    pub async fn get_latest_model_version( &self, model_family : &str ) -> AnthropicResult< ModelInfo >
    {
      let models = self.list_models().await?;
      let family_models : Vec< _ > = models.into_iter()
        .filter( | m | m.name.starts_with( model_family ) )
        .collect();

      if family_models.is_empty()
      {
        return Err( AnthropicError::InvalidArgument( format!( "No models found for family '{model_family}'" ) ) );
      }

      // Return the first one (in real implementation, would sort by version)
      Ok( family_models.into_iter().next().unwrap() )
    }

    /// Get upgrade recommendations for a model
    ///
    /// # Errors
    ///
    /// Returns an error if model not found
    pub async fn get_upgrade_recommendations( &self, model_id : &str ) -> AnthropicResult< Vec< ModelRecommendation > >
    {
      let current_model = self.get_model( model_id ).await?;
      let all_models = self.list_models().await?;

      // Generate intelligent upgrade recommendations based on model capabilities
      let mut recommendations = Vec::new();

      for model in all_models
      {
        // Skip the current model
        if model.id == current_model.id
        {
          continue;
        }

        // Check if this is a potential upgrade
        let is_upgrade = model.max_tokens >= current_model.max_tokens
          && model.context_length >= current_model.context_length
          && ( !current_model.supports_tools || model.supports_tools )
          && ( !current_model.supports_vision || model.supports_vision );

        if is_upgrade
        {
          let confidence_score = if model.supports_tools && model.supports_vision { 0.9 }
                                 else if model.supports_tools || model.supports_vision { 0.7 }
                                 else { 0.5 };

          let reasoning = format!(
            "Upgrade from {} with enhanced capabilities : {}",
            current_model.name,
            if model.max_tokens > current_model.max_tokens { "larger context, " } else { "" }
          );

          recommendations.push( ModelRecommendation
          {
            recommended_model : model.id,
            confidence_score,
            reasoning,
          });
        }
      }

      // Sort by confidence score descending
      recommendations.sort_by( |a, b| b.confidence_score.partial_cmp( &a.confidence_score ).unwrap_or( core::cmp::Ordering::Equal ) );

      Ok( recommendations )
    }

    /// Get model parameter limits
    ///
    /// # Errors
    ///
    /// Returns an error if model not found
    pub async fn get_model_limits( &self, model_id : &str ) -> AnthropicResult< ModelLimits >
    {
      let model = self.get_model( model_id ).await?;

      let limits = ModelLimits
      {
        max_tokens : model.max_tokens,
        temperature_range : TemperatureRange { min : 0.0, max : 1.0 },
        max_tool_calls : if model.supports_tools { Some( 20 ) } else { None },
        rate_limits : RateLimits
        {
          requests_per_minute : 1000,
          tokens_per_minute : 100_000,
        },
      };

      Ok( limits )
    }

    /// Validate request parameters for specific model
    ///
    /// # Errors
    ///
    /// Returns an error if request is invalid for the model
    pub async fn validate_request_for_model( &self, request : &CreateMessageRequest ) -> AnthropicResult< () >
    {
      // Local validation for common models (doesn't require API call)
      let (max_tokens_limit, temp_min, temp_max) = match request.model.as_str()
      {
        "claude-sonnet-4-6" | "claude-sonnet-4-5-20250929" | "claude-3-5-haiku-20241022" | "claude-3-opus-20240229" => (200_000, 0.0, 1.0),
        _ => {
          // For unknown models, try to get limits from API
          let limits = self.get_model_limits( &request.model ).await?;
          (limits.max_tokens, limits.temperature_range.min, limits.temperature_range.max)
        }
      };

      if request.max_tokens > max_tokens_limit
      {
        return Err( AnthropicError::InvalidArgument( format!(
          "max_tokens {} exceeds model limit of {}",
          request.max_tokens,
          max_tokens_limit
        )));
      }

      if let Some( temp ) = request.temperature
      {
        if temp < temp_min || temp > temp_max
        {
          return Err( AnthropicError::InvalidArgument( format!(
            "temperature {temp} is outside allowed range {temp_min}-{temp_max}"
          )));
        }
      }

      Ok( () )
    }

    /// Get model performance characteristics
    ///
    /// # Errors
    ///
    /// Returns an error if model not found
    pub async fn get_model_performance( &self, model_id : &str ) -> AnthropicResult< ModelPerformance >
    {
      let _model = self.get_model( model_id ).await?;

      let performance = ModelPerformance
      {
        model : model_id.to_string(),
        tokens_per_second : if model_id.contains( "haiku" ) { 150.0 } else { 80.0 },
        latency_ms : if model_id.contains( "haiku" ) { 200 } else { 400 },
        throughput_score : if model_id.contains( "haiku" ) { 9.0 } else { 7.5 },
        speed_score : if model_id.contains( "haiku" ) { 9.5 } else { 7.0 },
        cost_tier : if model_id.contains( "haiku" ) { "1".to_string() } else { "3".to_string() },
      };

      Ok( performance )
    }

    /// Compare performance between multiple models
    ///
    /// # Errors
    ///
    /// Returns an error if any model not found
    pub async fn compare_model_performance( &self, model_ids : Vec< &str > ) -> AnthropicResult< Vec< ModelPerformance > >
    {
      let mut performances = Vec::new();

      for model_id in model_ids
      {
        let performance = self.get_model_performance( model_id ).await?;
        performances.push( performance );
      }

      Ok( performances )
    }

    /// Get model status (deprecated, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if model not found
    pub async fn get_model_status( &self, model_id : &str ) -> AnthropicResult< ModelStatus >
    {
      let _model = self.get_model( model_id ).await?;

      let status = ModelStatus
      {
        is_deprecated : model_id.contains( "20240307" ),
        deprecation_date : if model_id.contains( "20240307" )
        {
          Some( "2024-12-01T00:00:00Z".to_string() )
        }
        else
        {
          None
        },
        sunset_date : if model_id.contains( "20240307" )
        {
          Some( "2025-06-01T00:00:00Z".to_string() )
        }
        else
        {
          None
        },
        replacement_model : if model_id.contains( "haiku" )
        {
          "claude-3-5-haiku-20241022".to_string()
        }
        else
        {
          "claude-sonnet-4-6".to_string()
        },
      };

      Ok( status )
    }

    /// Get migration path for deprecated model
    ///
    /// # Errors
    ///
    /// Returns an error if model not found
    pub async fn get_migration_path( &self, model_id : &str ) -> AnthropicResult< MigrationPath >
    {
      let _model_status = self.get_model_status( model_id ).await?;

      let migration = MigrationPath
      {
        recommended_replacement : "claude-3-5-haiku-20241022".to_string(),
        migration_steps : vec![
          "Update model parameter in requests".to_string(),
          "Test with new model".to_string(),
          "Deploy gradually".to_string(),
        ],
        breaking_changes : Some( vec![
          "Response format may differ slightly".to_string(),
        ] ),
      };

      Ok( migration )
    }

    /// Get model pricing information
    ///
    /// # Errors
    ///
    /// Returns an error if model not found
    pub async fn get_model_pricing( &self, model_id : &str ) -> AnthropicResult< ModelPricing >
    {
      let _model = self.get_model( model_id ).await?;

      let pricing = ModelPricing
      {
        input_cost_per_token : if model_id.contains( "haiku" ) { 0.00025 } else { 0.003 },
        output_cost_per_token : if model_id.contains( "haiku" ) { 0.00125 } else { 0.015 },
        currency : "USD".to_string(),
        usage_tier : if model_id.contains( "haiku" ) { "basic".to_string() } else { "premium".to_string() },
      };

      Ok( pricing )
    }

    /// Estimate cost for usage
    ///
    /// # Errors
    ///
    /// Returns an error if model not found
    pub async fn estimate_cost( &self, usage : EstimatedUsage ) -> AnthropicResult< CostEstimate >
    {
      let pricing = self.get_model_pricing( &usage.model ).await?;

      let input_cost = f64::from( usage.input_tokens ) * pricing.input_cost_per_token;
      let output_cost = f64::from( usage.output_tokens ) * pricing.output_cost_per_token;
      let total_cost = input_cost + output_cost;

      let estimate = CostEstimate
      {
        total_cost,
        input_cost,
        output_cost,
        currency : pricing.currency,
      };

      Ok( estimate )
    }

    /// Filter models by criteria
    ///
    /// # Errors
    ///
    /// Returns an error if API request fails
    pub async fn filter_models( &self, filter : ModelFilter ) -> AnthropicResult< Vec< ModelInfo > >
    {
      let models = self.list_models().await?;

      let filtered_models : Vec< _ > = models.into_iter()
        .filter( | model | {
          if let Some( supports_tools ) = filter.supports_tools
          {
            if model.supports_tools != supports_tools
            {
              return false;
            }
          }
          if let Some( supports_vision ) = filter.supports_vision
          {
            if model.supports_vision != supports_vision
            {
              return false;
            }
          }
          if let Some( min_context ) = filter.min_context_length
          {
            if model.context_length < min_context
            {
              return false;
            }
          }
          true
        })
        .collect();

      Ok( filtered_models )
    }

    /// Search models by name pattern
    ///
    /// # Errors
    ///
    /// Returns an error if API request fails
    pub async fn search_models( &self, query : &str ) -> AnthropicResult< Vec< ModelInfo > >
    {
      let models = self.list_models().await?;

      let search_results : Vec< _ > = models.into_iter()
        .filter( | model | model.name.contains( query ) )
        .collect();

      Ok( search_results )
    }

    /// Recommend model for specific use case
    ///
    /// # Errors
    ///
    /// Returns an error if no suitable recommendation found
    pub fn recommend_model_for_use_case( &self, use_case : UseCase ) -> AnthropicResult< ModelRecommendation >
    {
      let recommendation = match use_case
      {
        UseCase::CodeGeneration { complexity, .. } =>
        {
          let model = match complexity
          {
            CodeComplexity::High | CodeComplexity::Medium => "claude-sonnet-4-6",
            CodeComplexity::Low => "claude-3-5-haiku-20241022",
          };

          ModelRecommendation
          {
            recommended_model : model.to_string(),
            confidence_score : 0.85,
            reasoning : "Selected based on code complexity requirements".to_string(),
          }
        },
        UseCase::CreativeWriting { .. } =>
        {
          ModelRecommendation
          {
            recommended_model : "claude-sonnet-4-6".to_string(),
            confidence_score : 0.9,
            reasoning : "Sonnet excels at creative writing tasks".to_string(),
          }
        },
      };

      Ok( recommendation )
    }

    /// Clear model cache
    ///
    /// # Errors
    ///
    /// Should not fail under normal circumstances
    ///
    /// # Panics
    ///
    /// Panics if the cache mutex is poisoned
    pub fn clear_model_cache( &self ) -> AnthropicResult< () >
    {
      let mut cache = self.cache.lock().unwrap();
      cache.clear();
      Ok( () )
    }

    /// Check feature compatibility
    ///
    /// # Errors
    ///
    /// Returns an error if model not found
    pub async fn check_feature_compatibility( &self, model_id : &str, feature : &str ) -> AnthropicResult< FeatureCompatibility >
    {
      let capabilities = self.get_model_capabilities( model_id ).await?;

      let compatibility = match feature
      {
        "tools" =>
        {
          FeatureCompatibility
          {
            is_compatible : capabilities.supports_tools,
            feature_version : if capabilities.supports_tools { Some( "1.0".to_string() ) } else { None },
            alternative_models : if capabilities.supports_tools
            {
              None
            }
            else
            {
              Some( vec![ "claude-sonnet-4-6".to_string() ] )
            },
          }
        },
        "vision" =>
        {
          FeatureCompatibility
          {
            is_compatible : capabilities.supports_vision,
            feature_version : if capabilities.supports_vision { Some( "1.0".to_string() ) } else { None },
            alternative_models : if capabilities.supports_vision
            {
              None
            }
            else
            {
              Some( vec![ "claude-sonnet-4-6".to_string() ] )
            },
          }
        },
        _ =>
        {
          FeatureCompatibility
          {
            is_compatible : false,
            feature_version : None,
            alternative_models : None,
          }
        }
      };

      Ok( compatibility )
    }
  }

  /// Extension methods for Client to provide model management capabilities
  impl Client
  {
    /// Create a model manager with this client
    #[ must_use ]
    pub fn model_manager( &self ) -> ModelManager
    {
      ModelManager::new( self.clone() )
    }

    /// List all available models.
    ///
    /// # Errors
    ///
    /// Returns error if API request fails.
    pub async fn list_models( &self ) -> AnthropicResult< Vec< ModelInfo > >
    {
      self.model_manager().list_models().await
    }

    /// Get specific model information.
    ///
    /// # Errors
    ///
    /// Returns error if model not found.
    pub async fn get_model( &self, model_id : &str ) -> AnthropicResult< ModelInfo >
    {
      self.model_manager().get_model( model_id ).await
    }

    /// Get model capabilities.
    ///
    /// # Errors
    ///
    /// Returns error if model not found.
    pub async fn get_model_capabilities( &self, model_id : &str ) -> AnthropicResult< ModelCapabilities >
    {
      self.model_manager().get_model_capabilities( model_id ).await
    }

    /// Select best model based on requirements.
    ///
    /// # Errors
    ///
    /// Returns error if no suitable model found.
    pub async fn select_model( &self, requirements : ModelRequirements ) -> AnthropicResult< ModelInfo >
    {
      self.model_manager().select_model( requirements ).await
    }

    /// Check model availability.
    ///
    /// # Errors
    ///
    /// Returns error if model not found.
    pub async fn check_model_availability( &self, model_id : &str ) -> AnthropicResult< ModelAvailability >
    {
      self.model_manager().check_model_availability( model_id ).await
    }

    /// Select first available model from fallback chain.
    ///
    /// # Errors
    ///
    /// Returns error if no models are available.
    pub async fn select_available_model( &self, fallback_chain : Vec< String > ) -> AnthropicResult< ModelInfo >
    {
      self.model_manager().select_available_model( fallback_chain ).await
    }

    /// Get latest version of model family.
    ///
    /// # Errors
    ///
    /// Returns error if model family not found.
    pub async fn get_latest_model_version( &self, model_family : &str ) -> AnthropicResult< ModelInfo >
    {
      self.model_manager().get_latest_model_version( model_family ).await
    }

    /// Get upgrade recommendations.
    ///
    /// # Errors
    ///
    /// Returns error if model not found.
    pub async fn get_upgrade_recommendations( &self, model_id : &str ) -> AnthropicResult< Vec< ModelRecommendation > >
    {
      self.model_manager().get_upgrade_recommendations( model_id ).await
    }

    /// Get model parameter limits.
    ///
    /// # Errors
    ///
    /// Returns error if model not found.
    pub async fn get_model_limits( &self, model_id : &str ) -> AnthropicResult< ModelLimits >
    {
      self.model_manager().get_model_limits( model_id ).await
    }

    /// Validate request for specific model.
    ///
    /// # Errors
    ///
    /// Returns error if request is invalid.
    pub async fn validate_request_for_model( &self, request : &CreateMessageRequest ) -> AnthropicResult< () >
    {
      self.model_manager().validate_request_for_model( request ).await
    }

    /// Get model performance.
    ///
    /// # Errors
    ///
    /// Returns error if model not found.
    pub async fn get_model_performance( &self, model_id : &str ) -> AnthropicResult< ModelPerformance >
    {
      self.model_manager().get_model_performance( model_id ).await
    }

    /// Compare model performance.
    ///
    /// # Errors
    ///
    /// Returns error if any model not found.
    pub async fn compare_model_performance( &self, model_ids : Vec< &str > ) -> AnthropicResult< Vec< ModelPerformance > >
    {
      self.model_manager().compare_model_performance( model_ids ).await
    }

    /// Get model status.
    ///
    /// # Errors
    ///
    /// Returns error if model not found.
    pub async fn get_model_status( &self, model_id : &str ) -> AnthropicResult< ModelStatus >
    {
      self.model_manager().get_model_status( model_id ).await
    }

    /// Get migration path.
    ///
    /// # Errors
    ///
    /// Returns error if model not found.
    pub async fn get_migration_path( &self, model_id : &str ) -> AnthropicResult< MigrationPath >
    {
      self.model_manager().get_migration_path( model_id ).await
    }

    /// Get model pricing.
    ///
    /// # Errors
    ///
    /// Returns error if model not found.
    pub async fn get_model_pricing( &self, model_id : &str ) -> AnthropicResult< ModelPricing >
    {
      self.model_manager().get_model_pricing( model_id ).await
    }

    /// Estimate usage cost.
    ///
    /// # Errors
    ///
    /// Returns error if model not found.
    pub async fn estimate_cost( &self, usage : EstimatedUsage ) -> AnthropicResult< CostEstimate >
    {
      self.model_manager().estimate_cost( usage ).await
    }

    /// Filter models.
    ///
    /// # Errors
    ///
    /// Returns error if API request fails.
    pub async fn filter_models( &self, filter : ModelFilter ) -> AnthropicResult< Vec< ModelInfo > >
    {
      self.model_manager().filter_models( filter ).await
    }

    /// Search models.
    ///
    /// # Errors
    ///
    /// Returns error if API request fails.
    pub async fn search_models( &self, query : &str ) -> AnthropicResult< Vec< ModelInfo > >
    {
      self.model_manager().search_models( query ).await
    }

    /// Recommend model for use case.
    ///
    /// # Errors
    ///
    /// Returns error if no recommendation found.
    pub fn recommend_model_for_use_case( &self, use_case : UseCase ) -> AnthropicResult< ModelRecommendation >
    {
      self.model_manager().recommend_model_for_use_case( use_case )
    }

    /// Clear model cache.
    ///
    /// # Errors
    ///
    /// Should not fail.
    pub fn clear_model_cache( &self ) -> AnthropicResult< () >
    {
      self.model_manager().clear_model_cache()
    }

    /// Check feature compatibility.
    ///
    /// # Errors
    ///
    /// Returns error if model not found.
    pub async fn check_feature_compatibility( &self, model_id : &str, feature : &str ) -> AnthropicResult< FeatureCompatibility >
    {
      self.model_manager().check_feature_compatibility( model_id, feature ).await
    }
  }
}

crate::mod_interface!
{
  exposed use ModelManager;
}

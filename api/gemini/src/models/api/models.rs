//! Model metadata and discovery API implementation.
//!
//! This module provides access to model information and metadata through the Gemini API.

use reqwest::Method;
use crate::client::ModelsApi;
use crate::error::Error;
use crate::models::{ Model, ListModelsResponse };
use crate::internal::http;

use super::ModelApi;

impl ModelsApi< '_ >
{
  /// Lists all available Gemini models.
  ///
  /// This method fetches the complete list of models available through the Gemini API,
  /// including both generative and embedding models. The response includes detailed
  /// information about each model such as supported features, input/output token limits,
  /// and version information.
  ///
  /// # Returns
  ///
  /// Returns a [`ListModelsResponse`] containing:
  /// - `models`: Vector of [`Model`] objects with detailed model information
  /// - `next_page_token`: Token for pagination (if applicable)
  ///
  /// # Errors
  ///
  /// This method returns an error in the following cases:
  /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
  /// - [`Error::AuthenticationError`] - Invalid or missing API key
  /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
  /// - [`Error::DeserializationError`] - Failed to parse the API response
  /// - [`Error::ApiError`] - Other API-related errors
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
  ///
  /// // List all available models
  /// let response = models_api.list().await?;
  /// println!( "Found {} models", response.models.len() );
  ///
  /// // Print model names and capabilities
  /// for model in &response.models {
  ///   println!( "Model : {} - {}", model.name, model.display_name.as_deref().unwrap_or("N/A") );
  ///   if let Some( ref supported_generation_methods ) = model.supported_generation_methods {
  ///     println!( "  Supported methods : {:?}", supported_generation_methods );
  ///   }
  /// }
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn list( &self ) -> Result< ListModelsResponse, Error >
  {
    let url = format!( "{}/v1beta/models", self.client.base_url );

    http ::execute_with_optional_retries::< (), ListModelsResponse >
    (
      self.client,
      Method::GET,
      &url,
      &self.client.api_key,
      None,
    )
    .await
    .map_err( Self::enhance_list_error )
  }

  /// Gets information about a specific model by ID.
  ///
  /// This method retrieves detailed information about a specific Gemini model,
  /// including its capabilities, token limits, supported features, and version details.
  /// Model IDs can be provided with or without the "models/" prefix.
  ///
  /// # Arguments
  ///
  /// * `model_id` - The model identifier. Can be:
  ///   - Simple name : `"gemini-2.5-flash"`
  ///   - Full name : `"models/gemini-2.5-flash"`
  ///
  /// # Returns
  ///
  /// Returns a [`Model`] object containing detailed information about the model:
  /// - Basic info : name, display name, description
  /// - Capabilities : supported generation methods, modalities
  /// - Limits : input/output token limits, rate limits
  /// - Version info : version, creation date
  ///
  /// # Errors
  ///
  /// This method returns an error in the following cases:
  /// - [`Error::InvalidArgument`] - Invalid model ID format or model not found (404)
  /// - [`Error::NetworkError`] - Network connectivity issues or timeout
  /// - [`Error::AuthenticationError`] - Invalid or missing API key
  /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
  /// - [`Error::DeserializationError`] - Failed to parse the API response
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
  ///
  /// // Get model by simple name
  /// let model = models_api.get( "gemini-2.5-flash" ).await?;
  /// println!( "Model : {} - {}",
  ///   model.display_name.as_deref().unwrap_or( "N/A" ),
  ///   model.name
  /// );
  ///
  /// // Check capabilities
  /// if let Some( ref methods ) = model.supported_generation_methods {
  ///   println!( "Supported generation methods:" );
  ///   for method in methods {
  ///     println!( "  - {}", method );
  ///   }
  /// }
  ///
  /// // Check token limits
  /// if let Some( input_limit ) = model.input_token_limit {
  ///   println!( "Max input tokens : {}", input_limit );
  /// }
  /// if let Some( output_limit ) = model.output_token_limit {
  ///   println!( "Max output tokens : {}", output_limit );
  /// }
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn get( &self, model_id : &str ) -> Result< Model, Error >
  {
    // Remove "models/" prefix if already present for clean URL construction
    let clean_model_id = model_id.strip_prefix( "models/" ).unwrap_or( model_id );

    // Validate model ID format
    if clean_model_id.is_empty()
    {
      return Err( Error::InvalidArgument(
        "Model ID cannot be empty. Please provide a valid model identifier.".to_string()
      ) );
    }

    let url = format!( "{}/v1beta/models/{clean_model_id}", self.client.base_url );

    http ::execute_with_optional_retries::< (), Model >
    (
      self.client,
      Method::GET,
      &url,
      &self.client.api_key,
      None,
    )
    .await
    .map_err( | e | Self::enhance_get_error( model_id, e ) )
  }

  /// Creates a handle for a specific model by name.
  ///
  /// This method returns a [`ModelApi`] handle that can be used to interact
  /// with the specified model. The model name can be provided with or without
  /// the "models/" prefix.
  ///
  /// # Arguments
  ///
  /// * `model_id` - The model identifier (e.g., "gemini-2.5-flash")
  ///
  /// # Returns
  ///
  /// Returns a [`ModelApi`] handle for the specified model.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # use api_gemini::models::{ GenerateContentRequest, Content, Part };
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
  /// let model = models_api.by_name( "gemini-2.5-flash" );
  ///
  /// // Use the model handle for operations
  /// let request = GenerateContentRequest {
  ///   contents : vec![ Content {
  ///     parts : vec![ Part {
  ///       text : Some( "Hello!".to_string() ),
  ///       ..Default::default()
  ///     } ],
  ///     role : "user".to_string(),
  ///   } ],
  ///   ..Default::default()
  /// };
  ///
  /// let response = model.generate_content( &request ).await?;
  /// # Ok( () )
  /// # }
  /// ```
  #[ must_use ]
  #[ inline ]
  pub fn by_name( &self, model_id : &str ) -> ModelApi< '_ >
  {
    ModelApi {
      client : self.client,
      model_id : model_id.to_string(),
    }
  }

  /// Enhance errors from list operation with additional context.
  fn enhance_list_error( error : Error ) -> Error
  {
    match error
    {
      Error::NetworkError( ref msg ) =>
        Error::NetworkError(
          format!( "Failed to retrieve model list : {msg}. Please check your internet connection and try again." )
        ),
      Error::AuthenticationError( ref msg ) =>
        Error::AuthenticationError(
          format!( "Authentication failed while listing models : {msg}. Please verify your API key." )
        ),
      Error::ServerError( ref msg ) =>
        Error::ServerError(
          format!( "Gemini API server error while listing models : {msg}" )
        ),
      other => other,
    }
  }

  /// Enhance errors from get operation with model-specific context.
  fn enhance_get_error( model_id : &str, error : Error ) -> Error
  {
    match error
    {
      Error::NetworkError( ref msg ) =>
        Error::NetworkError(
          format!( "Failed to retrieve model '{model_id}': {msg}. Please check your internet connection." )
        ),
      Error::AuthenticationError( ref msg ) =>
        Error::AuthenticationError(
          format!( "Authentication failed for model '{model_id}': {msg}. Please verify your API key." )
        ),
      Error::ServerError( ref msg ) if msg.contains( "404" ) || msg.contains( "not found" ) =>
        Error::InvalidArgument(
          format!( "Model '{model_id}' not found. Please check the model ID and try again. Use models().list() to see available models." )
        ),
      Error::ServerError( ref msg ) =>
        Error::ServerError(
          format!( "Gemini API server error for model '{model_id}': {msg}" )
        ),
      other => other,
    }
  }

  /// Counts tokens for the specified model.
  ///
  /// This convenience method provides direct access to token counting functionality
  /// without needing to create a model handle first. It's equivalent to calling
  /// `models_api.by_name(model_id).count_tokens(request)`.
  ///
  /// # Arguments
  ///
  /// * `model_id` - The model identifier (e.g., "gemini-1.5-flash", "gemini-1.5-pro-latest")
  /// * `request` - A [`crate::models::CountTokensRequest`] containing the content to count
  ///
  /// # Returns
  ///
  /// Returns a [`crate::models::CountTokensResponse`] with token count information.
  ///
  /// # Errors
  ///
  /// Returns the same errors as [`ModelApi::count_tokens`] method.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # use api_gemini::models::{ Content, Part, CountTokensRequest };
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
  ///
  /// let content = Content {
  ///   parts : vec![ Part {
  ///     text : Some( "Hello world".to_string() ),
  ///     ..Default::default()
  ///   } ],
  ///   role : "user".to_string(),
  /// };
  ///
  /// let request = CountTokensRequest {
  ///   contents : vec![ content ],
  ///   generate_content_request : None,
  /// };
  ///
  /// // Direct token counting
  /// let response = models_api.count_tokens( "gemini-1.5-flash", &request ).await?;
  /// println!( "Tokens : {}", response.total_tokens );
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn count_tokens
  (
    &self,
    model_id : &str,
    request : &crate::models::CountTokensRequest,
  )
  ->
  Result< crate::models::CountTokensResponse, Error >
  {
    self.by_name( model_id ).count_tokens( request ).await
  }
}

impl ModelApi< '_ >
{
  /// Gets information about this specific model.
  ///
  /// This method retrieves detailed information about the model bound to this
  /// handle, including its capabilities, token limits, supported features,
  /// and version details. This is useful for understanding what the model
  /// can do before making requests.
  ///
  /// # Returns
  ///
  /// Returns a [`Model`] object containing:
  /// - Basic information : name, display name, description
  /// - Capabilities : supported generation methods, input/output modalities
  /// - Limits : token limits, rate limits, safety settings
  /// - Version : model version, creation date, update date
  ///
  /// # Errors
  ///
  /// This method returns an error in the following cases:
  /// - [`Error::InvalidArgument`] - Model not found or invalid model ID
  /// - [`Error::NetworkError`] - Network connectivity issues or timeout
  /// - [`Error::AuthenticationError`] - Invalid or missing API key
  /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
  /// - [`Error::DeserializationError`] - Failed to parse the API response
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
  /// let model = models_api.by_name( "gemini-2.5-flash" );
  ///
  /// let info = model.get().await?;
  /// println!( "Model : {} ({})", info.display_name.as_deref().unwrap_or( "N/A" ), info.name );
  ///
  /// // Check what the model can do
  /// if let Some( ref methods ) = info.supported_generation_methods {
  ///   println!( "Supported operations:" );
  ///   for method in methods {
  ///     match method.as_str() {
  ///       "generateContent" => println!( "  ✓ Text generation" ),
  ///       "embedContent" => println!( "  ✓ Text embeddings" ),
  ///       other => println!( "  ✓ {}", other ),
  ///     }
  ///   }
  /// }
  ///
  /// // Check token limits
  /// if let Some( input_limit ) = info.input_token_limit {
  ///   println!( "Max input tokens : {}", input_limit );
  /// }
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn get( &self ) -> Result< Model, Error >
  {
    // Remove "models/" prefix if already present for clean URL construction
    let clean_model_id = self.model_id.strip_prefix( "models/" ).unwrap_or( &self.model_id );

    let url = format!( "{}/v1beta/models/{clean_model_id}", self.client.base_url );

    http ::execute_with_optional_retries::< (), Model >
    (
      self.client,
      Method::GET,
      &url,
      &self.client.api_key,
      None,
    )
    .await
    .map_err( | e | self.enhance_model_operation_error( "retrieve model information", e ) )
  }

  /// Returns the model ID for this model handle.
  ///
  /// This is a simple accessor method that returns the model identifier
  /// associated with this `ModelApi` instance.
  ///
  /// # Returns
  ///
  /// The model ID string.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use api_gemini::client::Client;
  /// # fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
  /// let model = models_api.by_name( "gemini-2.5-flash" );
  ///
  /// assert_eq!( model.model_id(), "gemini-2.5-flash" );
  /// # Ok( () )
  /// # }
  /// ```
  #[ must_use ]
  #[ inline ]
  pub fn model_id( &self ) -> &str
  {
    &self.model_id
  }

  /// Create a WebSocket stream for real-time bidirectional communication.
  ///
  /// This method provides WebSocket-like functionality with fallback to HTTP streaming.
  /// Since the Gemini API uses HTTP with Server-Sent Events, this implementation
  /// simulates WebSocket behavior over the existing streaming infrastructure.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// use std::time::Duration;
  ///
  /// let client = Client::new()?;
  ///
  /// let mut websocket_stream = client
  ///   .models()
  ///   .by_name( "gemini-pro" )
  ///   .websocket_stream()
  ///   .with_message( "Hello, let's have a conversation!" )
  ///   .with_keepalive( Duration::from_secs( 30 ) )
  ///   .with_reconnect( true )
  ///   .connect()
  ///   .await?;
  ///
  /// // Use websocket_stream for bidirectional communication
  /// # Ok( () )
  /// # }
  /// ```
  #[ must_use ]
  #[ inline ]
  pub fn websocket_stream( &self ) -> crate::models::websocket_streaming::WebSocketStreamBuilder< '_ >
  {
    crate ::models::websocket_streaming::WebSocketStreamBuilder::new( self )
  }

  /// Create a fine-tuning job for custom model training.
  ///
  /// This method provides comprehensive fine-tuning capabilities including supervised learning,
  /// parameter-efficient training methods (`LoRA`, adapters), and hyperparameter optimization.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// use api_gemini::models::{ HyperparameterConfig, LoRAConfig, TrainingObjective };
  ///
  /// let client = Client::new()?;
  ///
  /// let training_job = client
  ///   .models()
  ///   .by_name( "gemini-pro" )
  ///   .fine_tune()
  ///   .with_training_data( "path/to/training_data.jsonl" )
  ///   .with_validation_data( "path/to/validation_data.jsonl" )
  ///   .with_epochs( 3 )
  ///   .with_learning_rate( 0.0001 )
  ///   .with_lora_config( LoRAConfig::builder()
  ///     .rank( 16 )
  ///     .alpha( 32.0 )
  ///     .build()? )
  ///   .start_training()
  ///   .await?;
  ///
  /// // Monitor training progress
  /// let mut progress_receiver = training_job.subscribe_progress();
  /// # Ok( () )
  /// # }
  /// ```
  #[ must_use ]
  #[ inline ]
  pub fn fine_tune( &self ) -> crate::models::model_tuning::FineTuningBuilder< '_ >
  {
    crate ::models::model_tuning::FineTuningBuilder::new( self )
  }

  /// Deploy a model to production environments.
  ///
  /// This method provides comprehensive deployment capabilities including orchestration,
  /// scaling, monitoring, and various deployment strategies (blue-green, canary, rolling).
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  ///
  /// // Get deployment builder (API for configuration and deployment)
  /// let _deployment_builder = client
  ///   .models()
  ///   .by_name( "gemini-pro" )
  ///   .deploy();
  ///
  /// // Use builder methods to configure deployment settings
  /// // (Full deployment API is available through the builder)
  /// # Ok( () )
  /// # }
  /// ```
  #[ must_use ]
  #[ inline ]
  pub fn deploy( &self ) -> crate::models::model_deployment::DeploymentBuilder< '_ >
  {
    crate ::models::model_deployment::DeploymentBuilder::new( self )
  }

  /// Counts the number of tokens in the provided content.
  ///
  /// This method helps you understand token usage before making generation requests,
  /// allowing you to stay within model limits and estimate costs. It supports counting
  /// tokens for both simple content and full generation requests.
  ///
  /// # Arguments
  ///
  /// * `request` - The token counting request containing content to analyze
  ///
  /// # Returns
  ///
  /// Returns a [`CountTokensResponse`] containing:
  /// - `total_tokens`: Total number of tokens in the input
  /// - `cached_content_token_count`: Number of cached content tokens (if applicable)
  ///
  /// # Errors
  ///
  /// This method returns an error in the following cases:
  /// - [`Error::InvalidArgument`] - Empty content or invalid request structure
  /// - [`Error::NetworkError`] - Network connectivity issues or timeout
  /// - [`Error::AuthenticationError`] - Invalid or missing API key
  /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
  /// - [`Error::DeserializationError`] - Failed to parse the API response
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # use api_gemini::models::{ CountTokensRequest, Content, Part };
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
  /// let model = models_api.by_name( "gemini-2.5-flash" );
  ///
  /// // Count tokens for simple text
  /// let request = CountTokensRequest {
  ///   contents : vec![ Content {
  ///     parts : vec![ Part {
  ///       text : Some( "How many tokens is this?".to_string() ),
  ///       ..Default::default()
  ///     } ],
  ///     role : "user".to_string(),
  ///   } ],
  ///   ..Default::default()
  /// };
  ///
  /// let response = model.count_tokens( &request ).await?;
  /// println!( "This text contains {} tokens", response.total_tokens );
  ///
  /// // Use the count to check against model limits
  /// let model_info = model.get().await?;
  /// if let Some( limit ) = model_info.input_token_limit {
  ///   if response.total_tokens > limit {
  ///     println!( "Warning : Input exceeds model limit of {} tokens", limit );
  ///   }
  /// }
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn count_tokens
  (
    &self,
    request : &crate::models::CountTokensRequest,
  )
  ->
  Result< crate::models::CountTokensResponse, Error >
  {
    // Validate request before sending
    if request.contents.is_empty()
    {
      return Err( Error::InvalidArgument(
        "Count tokens request cannot have empty contents. Please provide at least one content item.".to_string()
      ) );
    }

    let url = format!(
      "{}/v1beta/models/{}:countTokens",
      self.client.base_url,
      self.model_id
    );

    http ::execute_with_optional_retries
    (
      self.client,
      Method::POST,
      &url,
      &self.client.api_key,
      Some( request ),
    )
    .await
    .map_err( |e| self.enhance_model_operation_error( "count tokens", e ) )
  }

  /// Enhance errors from model operations with model-specific context.
  pub( super ) fn enhance_model_operation_error( &self, operation : &str, error : Error ) -> Error
  {
    match error
    {
      Error::NetworkError( ref msg ) =>
        Error::NetworkError(
          format!( "Failed to {operation} for model '{}': {msg}. Please check your internet connection.", self.model_id )
        ),
      Error::AuthenticationError( ref msg ) =>
        Error::AuthenticationError(
          format!( "Authentication failed while trying to {operation} for model '{}': {msg}. Please verify your API key.", self.model_id )
        ),
      Error::ServerError( ref msg ) if msg.contains( "404" ) || msg.contains( "not found" ) =>
        Error::InvalidArgument(
          format!( "Model '{}' not found while trying to {operation}. Please check the model ID.", self.model_id )
        ),
      Error::ServerError( ref msg ) =>
        Error::ServerError(
          format!( "Gemini API server error for model '{}' while trying to {operation}: {msg}", self.model_id )
        ),
      other => other,
    }
  }
}

//! Client implementation
//!
//! Main Client struct and its implementation.

#[ allow( clippy::missing_inline_in_public_items ) ]
mod private
{
  use super::super::types::orphan::*;
  use crate::Secret;
  #[ cfg( feature = "error-handling" ) ]
  use crate::error::{ AnthropicError, AnthropicResult };

  /// Anthropic API client
  #[ derive( Debug, Clone ) ]
  pub struct Client
  {
    secret : Secret,
    config : ClientConfig,
    http : reqwest::Client,
    // Automatic retry configuration removed per governing principle - use explicit retry methods
    // Automatic circuit breaker removed per governing principle - use explicit health monitoring methods
    // Automatic rate limiting removed per governing principle - use explicit rate limit information access
  }

  impl Client
  {
    /// Create new client with API key and default configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::{ Client, Secret };
    ///
    /// // Create a client with default configuration
    /// let secret = Secret::new( "sk-ant-api03-example".to_string() ).unwrap();
    /// let client = Client::new( secret );
    /// ```
    #[ inline ]
    #[ must_use ]
    pub fn new( secret : Secret ) -> Self
    {
      Self::with_config( secret, ClientConfig::recommended() )
    }

    /// Create new client with API key and custom configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::{ Client, Secret, ClientConfig };
    /// use std::time::Duration;
    ///
    /// // Create a client with custom configuration
    /// let secret = Secret::new( "sk-ant-api03-example".to_string() ).unwrap();
    /// let config = ClientConfig::recommended()
    ///   .with_timeout( Duration::from_secs( 30 ) )
    ///   .with_base_url( "https://api.anthropic.com".to_string() );
    /// let client = Client::with_config( secret, config );
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if HTTP client fails to build
    #[ inline ]
    #[ must_use ]
    pub fn with_config( secret : Secret, config : ClientConfig ) -> Self
    {
      let http_client = reqwest::Client::builder()
        .timeout( config.request_timeout )
        .user_agent( &config.user_agent )
        .build()
        .expect( "Failed to build HTTP client" );

      Self
      {
        secret,
        config,
        http : http_client,
        // retry_config field removed per governing principle
        // circuit_breaker field removed per governing principle
        // rate_limiter field removed per governing principle
      }
    }

    /// Create client from environment variable
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable is not set or contains an invalid API key
    #[ inline ]
    pub fn from_env() -> AnthropicResult< Self >
    {
      let secret = Secret::load_from_env( "ANTHROPIC_API_KEY" )
        .map_err( | e | AnthropicError::MissingEnvironment( e.to_string() ) )?;
      
      Ok( Self::new( secret ) )
    }

    /// Create client from workspace secrets
    ///
    /// # Errors
    ///
    /// Returns an error if workspace loading fails or the API key is invalid
    #[ inline ]
    pub fn from_workspace() -> AnthropicResult< Self >
    {
      let secret = Secret::load_from_workspace( "ANTHROPIC_API_KEY", "-secrets.sh" )
        .map_err( | e | AnthropicError::MissingEnvironment( e.to_string() ) )?;
      
      Ok( Self::new( secret ) )
    }

    /// Set custom base URL
    #[ inline ]
    #[ must_use ]
    pub fn with_base_url( mut self, base_url : String ) -> Self
    {
      self.config.base_url = base_url;
      self
    }

    /// Get API key
    #[ inline ]
    #[ must_use ]
    pub fn secret( &self ) -> &Secret
    {
      &self.secret
    }

    /// Get base URL
    #[ inline ]
    #[ must_use ]
    pub fn base_url( &self ) -> &str
    {
      &self.config.base_url
    }

    /// Get client configuration
    #[ inline ]
    #[ must_use ]
    pub fn config( &self ) -> &ClientConfig
    {
      &self.config
    }

    // Automatic retry configuration methods removed per governing principle
    // Use explicit retry methods on individual requests instead

    // Automatic circuit breaker configuration methods removed per governing principle
    // Use explicit health monitoring methods instead

    /// Get HTTP client for direct API calls
    #[ cfg( feature = "model-management" ) ]
    pub fn http( &self ) -> &reqwest::Client
    {
      &self.http
    }

    /// Create a message using Claude
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use api_claude::{ Client, Secret, CreateMessageRequest, Message, Role, Content };
    ///
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// // Create a client
    /// let secret = Secret::new( "sk-ant-api03-example".to_string() ).unwrap();
    /// let client = Client::new( secret );
    ///
    /// // Create a message request
    /// let request = CreateMessageRequest::builder()
    ///   .model( "claude-sonnet-4-6".to_string() )
    ///   .max_tokens( 1000 )
    ///   .messages( vec![
    ///     Message {
    ///       role : Role::User,
    ///       content : vec![ Content::Text {
    ///         r#type : "text".to_string(),
    ///         text : "Hello, Claude!".to_string()
    ///       } ],
    ///       cache_control : None,
    ///     }
    ///   ] )
    ///   .build();
    ///
    /// // Send the request
    /// let response = client.create_message( request ).await?;
    /// println!( "Response : {:?}", response.content );
    /// # Ok( () )
    /// # }
    /// ```
    ///
    /// # Governing Principle Compliance
    ///
    /// This method follows the "Thin Client, Rich API" principle by:
    /// - **Direct API Mapping**: One-to-one correspondence with Anthropic's `/v1/messages` endpoint
    /// - **Zero Client Intelligence**: Sends requests immediately without automatic behaviors or magic thresholds
    /// - **Explicit Control**: All retry logic, rate limiting, and circuit breaking require explicit configuration
    /// - **Transparent Operations**: Returns all API errors without client-side filtering or modification
    /// - **Information vs Action**: Provides message creation without imposing application patterns
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response is invalid
    ///
    /// # Panics
    ///
    /// Panics if the response cannot be unwrapped during cache storage (this should not happen due to the `is_ok` check)
    #[ inline ]
    pub async fn create_message( &self, request : CreateMessageRequest ) -> AnthropicResult< CreateMessageResponse >
    {

      // Automatic circuit breaker checks removed per governing principle - use explicit health monitoring methods

      let url = format!( "{}/v1/messages", self.config.base_url );

      let headers = build_headers( &self.secret, &self.config );

      let response = self.http
        .post( &url )
        .headers( headers )
        .json( &request )
        .send()
        .await
        .map_err( AnthropicError::from )?;

      let result = handle_response::< CreateMessageResponse >( response ).await;

      // Automatic circuit breaker recording removed per governing principle - use explicit health monitoring methods

      result
    }

    /// Count tokens in a message without sending it
    ///
    /// This method allows pre-calculating token usage for cost estimation without making actual API calls.
    /// It uses the `/v1/messages/count_tokens` endpoint.
    ///
    /// # Governing Principle Compliance
    ///
    /// This method follows the "Thin Client, Rich API" principle by:
    /// - **Direct API Mapping**: One-to-one correspondence with Anthropic's `/v1/messages/count_tokens` endpoint
    /// - **Zero Client Intelligence**: Performs token counting without automatic behaviors
    /// - **Transparent Operations**: Returns all API errors without client-side filtering
    /// - **Information Provision**: Provides token counts for cost estimation
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response is invalid
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// use api_claude::{ Client, Secret, CountMessageTokensRequest, Message };
    ///
    /// let secret = Secret::from_workspace()?;
    /// let client = Client::new( secret );
    ///
    /// let request = CountMessageTokensRequest
    /// {
    ///   model : "claude-sonnet-4-6".to_string(),
    ///   messages : vec![ Message::user( "Hello, Claude!".to_string() ) ],
    ///   system : None,
    ///   tools : None,
    /// };
    ///
    /// let response = client.count_message_tokens( request ).await?;
    /// println!( "Input tokens : {}", response.input_tokens );
    /// # Ok( () )
    /// # }
    /// ```
    #[ inline ]
    pub async fn count_message_tokens( &self, request : CountMessageTokensRequest ) -> AnthropicResult< CountMessageTokensResponse >
    {
      // Validate request
      request.validate()?;

      let url = format!( "{}/v1/messages/count_tokens", self.config.base_url );

      let headers = build_headers( &self.secret, &self.config );

      let response = self.http
        .post( &url )
        .headers( headers )
        .json( &request )
        .send()
        .await
        .map_err( AnthropicError::from )?;

      handle_response::< CountMessageTokensResponse >( response ).await
    }

    /// Create messages in batch
    ///
    /// Submits multiple message requests for asynchronous batch processing.
    /// Batches can contain up to 100,000 requests with a maximum size of 256 MB.
    /// Results are retrieved asynchronously when processing completes.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, validation errors occur, or the API returns an error
    #[ cfg( all( feature = "batch-processing", feature = "error-handling" ) ) ]
    #[ inline ]
    pub async fn create_messages_batch( &self, batch_request : crate::CreateBatchRequest ) -> AnthropicResult< crate::BatchResponse >
    {
      // Validate batch request
      batch_request.validate()?;

      let url = format!( "{}/v1/messages/batches", self.config.base_url );
      let headers = build_headers( &self.secret, &self.config );

      let response = self.http
        .post( &url )
        .headers( headers )
        .json( &batch_request )
        .send()
        .await
        .map_err( AnthropicError::from )?;

      handle_response::< crate::BatchResponse >( response ).await
    }

    /// Retrieve batch status and information
    ///
    /// Fetches the current status of a batch by its ID, including processing status,
    /// request counts, and `results_url` when available.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or batch ID is invalid
    #[ cfg( all( feature = "batch-processing", feature = "error-handling" ) ) ]
    #[ inline ]
    pub async fn retrieve_batch( &self, batch_id : &str ) -> AnthropicResult< crate::BatchResponse >
    {
      if batch_id.is_empty()
      {
        return Err( AnthropicError::InvalidArgument( "batch_id cannot be empty".to_string() ) );
      }

      let url = format!( "{}/v1/messages/batches/{}", self.config.base_url, batch_id );
      let headers = build_headers( &self.secret, &self.config );

      let response = self.http
        .get( &url )
        .headers( headers )
        .send()
        .await
        .map_err( AnthropicError::from )?;

      handle_response::< crate::BatchResponse >( response ).await
    }

    /// List all batches with optional pagination
    ///
    /// Returns a paginated list of batches. Use `before_id`/`after_id` for cursor-based pagination.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    #[ cfg( all( feature = "batch-processing", feature = "error-handling" ) ) ]
    #[ inline ]
    pub async fn list_batches( &self, before_id : Option< &str >, after_id : Option< &str >, limit : Option< u32 > ) -> AnthropicResult< crate::BatchListResponse >
    {
      let mut url = format!( "{}/v1/messages/batches", self.config.base_url );
      let mut query_params = vec![];

      if let Some( id ) = before_id
      {
        query_params.push( format!( "before_id={id}" ) );
      }
      if let Some( id ) = after_id
      {
        query_params.push( format!( "after_id={id}" ) );
      }
      if let Some( lim ) = limit
      {
        query_params.push( format!( "limit={lim}" ) );
      }

      if !query_params.is_empty()
      {
        url.push( '?' );
        url.push_str( &query_params.join( "&" ) );
      }

      let headers = build_headers( &self.secret, &self.config );

      let response = self.http
        .get( &url )
        .headers( headers )
        .send()
        .await
        .map_err( AnthropicError::from )?;

      handle_response::< crate::BatchListResponse >( response ).await
    }

    /// Cancel a batch that is in progress
    ///
    /// Attempts to cancel a batch that is currently processing. Batches that have
    /// already completed cannot be canceled.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or batch cannot be canceled
    #[ cfg( all( feature = "batch-processing", feature = "error-handling" ) ) ]
    #[ inline ]
    pub async fn cancel_batch( &self, batch_id : &str ) -> AnthropicResult< crate::BatchResponse >
    {
      if batch_id.is_empty()
      {
        return Err( AnthropicError::InvalidArgument( "batch_id cannot be empty".to_string() ) );
      }

      let url = format!( "{}/v1/messages/batches/{}/cancel", self.config.base_url, batch_id );
      let headers = build_headers( &self.secret, &self.config );

      let response = self.http
        .post( &url )
        .headers( headers )
        .send()
        .await
        .map_err( AnthropicError::from )?;

      handle_response::< crate::BatchResponse >( response ).await
    }

    /// Create a message with context for error tracking
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response is invalid
    #[ cfg( feature = "error-handling" ) ]
    #[ inline ]
    pub async fn create_message_with_context( &self, request : CreateMessageRequest, context : crate::RequestContext ) -> AnthropicResult< CreateMessageResponse >
    {
      // Call the regular create_message method but enhance any errors with context
      match self.create_message( request ).await
      {
        Ok( response ) => Ok( response ),
        Err( basic_error ) =>
        {
          // Convert basic error to enhanced error with context
          let request_id = format!( "req_{}_{}", context.correlation_id(), context.request_sequence() );
          let error_context = crate::ErrorContext::new(
            "create_message_with_context".to_string(),
            request_id.clone(),
            std::collections::HashMap::new()
          );

          let enhanced = crate::EnhancedAnthropicError::new(
            crate::ErrorType::InvalidRequest,
            format!( "create_message_with_context : {basic_error}" ),
            Some( error_context )
          )
          .with_stack_trace( vec![ "create_message_with_context".to_string() ] )
          .with_request_id( Some( request_id ) )
          .with_correlation_id( Some( context.correlation_id().to_string() ) );

          // Return enhanced error wrapped in AnthropicError::Enhanced variant
          Err( crate::AnthropicError::Enhanced( Box::new( enhanced ) ) )
        }
      }
    }

    /// Check if client supports embeddings functionality
    ///
    /// # Returns
    ///
    /// Returns true when embeddings feature is enabled
    #[ cfg( feature = "embeddings" ) ]
    #[ inline ]
    #[ must_use ]
    pub fn supports_embeddings( &self ) -> bool
    {
      true // When embeddings feature is enabled, client supports it
    }

    /// Check if client supports embeddings functionality (feature disabled)
    ///
    /// # Returns
    ///
    /// Returns false when embeddings feature is disabled
    #[ cfg( not( feature = "embeddings" ) ) ]
    #[ inline ]
    #[ must_use ]
    pub fn supports_embeddings( &self ) -> bool
    {
      false
    }

    /// Create text embeddings (placeholder for future Anthropic embeddings API)
    ///
    /// # Errors
    ///
    /// Currently returns "not supported" error since Anthropic doesn't offer embeddings yet
    #[ cfg( feature = "embeddings" ) ]
    #[ inline ]
    pub fn create_embedding( &self, request : &crate::EmbeddingRequest ) -> AnthropicResult< crate::EmbeddingResponse >
    {
      // Validate request first
      request.validate()?;

      // Since Anthropic doesn't support embeddings yet, return appropriate error
      #[ cfg( feature = "error-handling" ) ]
      return Err( AnthropicError::NotImplemented(
        "Embeddings API not yet supported by Anthropic. This is a placeholder for future functionality.".to_string()
      ) );

      #[ cfg( not( feature = "error-handling" ) ) ]
      return Err( error_tools::Error::msg(
        "Embeddings API not yet supported by Anthropic. This is a placeholder for future functionality."
      ) );
    }

    /// Batch create text embeddings (placeholder for future functionality)
    ///
    /// # Errors
    ///
    /// Currently returns "not supported" error since Anthropic doesn't offer embeddings yet
    #[ cfg( feature = "embeddings" ) ]
    #[ inline ]
    pub fn create_embeddings_batch( &self, requests : &[crate::EmbeddingRequest] ) -> AnthropicResult< Vec< crate::EmbeddingResponse > >
    {
      // Validate all requests
      for ( index, request ) in requests.iter().enumerate()
      {
        request.validate().map_err( | e |
        {
          #[ cfg( feature = "error-handling" ) ]
          return AnthropicError::InvalidArgument( format!( "Request at index {index} invalid : {e}" ) );
          #[ cfg( not( feature = "error-handling" ) ) ]
          return error_tools::Error::msg( format!( "Request at index {index} invalid : {e}" ) );
        } )?;
      }

      // Since Anthropic doesn't support embeddings yet, return appropriate error
      #[ cfg( feature = "error-handling" ) ]
      return Err( AnthropicError::NotImplemented(
        "Batch embeddings API not yet supported by Anthropic. This is a placeholder for future functionality.".to_string()
      ) );

      #[ cfg( not( feature = "error-handling" ) ) ]
      return Err( error_tools::Error::msg(
        "Batch embeddings API not yet supported by Anthropic. This is a placeholder for future functionality."
      ) );
    }

    // CURL Diagnostics functionality

    /// Get the API key for diagnostics (with curl-diagnostics feature)
    ///
    /// # Returns
    ///
    /// The API key secret for use in cURL command generation
    #[ cfg( feature = "curl-diagnostics" ) ]
    #[ inline ]
    #[ must_use ]
    pub fn api_key( &self ) -> Option< &Secret >
    {
      Some( &self.secret )
    }

    /// Get the API key for diagnostics (feature disabled)
    #[ cfg( not( feature = "curl-diagnostics" ) ) ]
    #[ inline ]
    #[ must_use ]
    pub fn api_key( &self ) -> Option< &Secret >
    {
      None
    }

  }
}

crate::mod_interface!
{
  exposed use Client;
}

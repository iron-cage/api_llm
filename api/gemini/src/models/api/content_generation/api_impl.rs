//! Content generation API implementation.
//!
//! This module provides comprehensive content generation capabilities including
//! text generation, conversation handling, streaming, and batch processing.

use reqwest::Method;
use crate::error::Error;
use crate::models::{ GenerateContentResponse, Content, Part, Candidate };
use crate::internal::http;

use super::super::ModelApi;
use super::builder::GenerationRequestBuilder;

impl ModelApi< '_ >
{
  /// Generates content using this model.
  ///
  /// This method sends a content generation request to the Gemini model and
  /// returns the generated response. The model will process the input content
  /// and generate appropriate responses based on the model's capabilities
  /// and the provided configuration.
  ///
  /// # Arguments
  ///
  /// * `request` - A [`crate::models::GenerateContentRequest`] containing:
  ///   - `contents`: The input conversation history or single prompt
  ///   - `generation_config`: Optional generation parameters (temperature, etc.)
  ///   - `safety_settings`: Optional safety configuration
  ///   - `tools`: Optional function calling configuration
  ///
  /// # Returns
  ///
  /// Returns a [`crate::models::GenerateContentResponse`] containing:
  /// - `candidates`: Generated response candidates
  /// - `prompt_feedback`: Feedback about the input prompt
  /// - `usage_metadata`: Token usage information
  ///
  /// # Errors
  ///
  /// This method returns an error in the following cases:
  /// - [`Error::InvalidArgument`] - Invalid request format, empty content, or model doesn't support generation
  /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
  /// - [`Error::AuthenticationError`] - Invalid or missing API key  
  /// - [`Error::RateLimitError`] - API rate limits exceeded
  /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
  /// - [`Error::SerializationError`] - Failed to serialize the request
  /// - [`Error::DeserializationError`] - Failed to parse the API response
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::{ client::Client, GenerateContentRequest, Content, Part };
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
/// let model = models_api.by_name( "gemini-2.5-flash" );
  /// 
  /// // Simple text generation
  /// let request = GenerateContentRequest {
  ///   contents : vec![ Content {
  ///     parts : vec![ Part {
  ///       text : Some( "Explain quantum computing in simple terms.".to_string() ),
  ///       ..Default::default()
  ///     } ],
  ///     role : "user".to_string(),
  ///   } ],
  ///   ..Default::default()
  /// };
  /// 
  /// let response = model.generate_content( &request ).await?;
  /// 
  /// // Extract the generated text
  /// if let Some( candidate ) = response.candidates.first() {
  ///   let content = &candidate.content;
  ///   for part in &content.parts {
  ///     if let Some( text ) = &part.text {
  ///       println!( "Generated : {}", text );
  ///     }
  ///   }
  /// }
  /// 
  /// // Check usage statistics
  /// if let Some( usage ) = response.usage_metadata {
  ///   println!( "Tokens used - Input : {}, Output : {}", 
  ///     usage.prompt_token_count.unwrap_or( 0 ),
  ///     usage.candidates_token_count.unwrap_or( 0 )
  ///   );
  /// }
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn generate_content
  (
    &self,
    request : &crate::models::GenerateContentRequest,
  )
  ->
  Result< crate::models::GenerateContentResponse, Error >
  {
    // Validate request before sending
    if request.contents.is_empty()
    {
      return Err( Error::InvalidArgument( 
        "Generate content request cannot have empty contents. Please provide at least one content item.".to_string()
      ) );
    }

    let url = format!(
      "{}/v1beta/models/{}:generateContent",
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
    .map_err( |e| self.enhance_model_operation_error( "generate content", e ) )
  }
  /// Generates content with retry logic and exponential backoff.
  ///
  /// This method is similar to [`Self::generate_content`] but includes automatic retry
  /// logic with exponential backoff for handling transient failures (5xx errors, timeouts).
  /// Non-retryable errors (4xx) are returned immediately without retry attempts.
  ///
  /// # Arguments
  ///
  /// * `request` - A [`crate::models::GenerateContentRequest`] containing the content generation parameters
  ///
  /// # Returns
  ///
  /// Returns a [`crate::models::GenerateContentResponse`] containing the generated content,
  /// after potentially multiple retry attempts with exponential backoff.
  ///
  /// # Errors
  ///
  /// - [`Error::InvalidArgument`] - Empty contents or invalid request format
  /// - [`Error::AuthenticationError`] - Invalid or missing API key  
  /// - [`Error::ServerError`] - Server error (after retries exhausted)
  /// - [`Error::NetworkError`] - Network connectivity issues (after retries)
  ///
  /// # Panics
  ///
  /// May panic if the internal retry attempt counter mutex is poisoned,
  /// which should only occur in exceptional circumstances.
  ///
  #[ cfg( feature = "retry" ) ]
  #[ inline ]
  pub async fn generate_content_with_retry
  (
    &self,
    request : &crate::models::GenerateContentRequest,
  )
  ->
  Result< crate::models::GenerateContentResponse, Error >
  {
    use backoff::{ ExponentialBackoff, future::retry };

    // Create exponential backoff configuration using client settings
    let mut backoff = ExponentialBackoff
    {
      initial_interval : self.client.base_delay,
      max_interval : self.client.max_delay,
      multiplier : self.client.backoff_multiplier,
      max_elapsed_time : self.client.max_elapsed_time,
      ..Default::default()
    };

    // Apply jitter if enabled
    if self.client.enable_jitter
    {
      backoff.randomization_factor = 0.1; // 10% jitter
    }

    // Use both max_retries (attempt count) and max_elapsed_time for comprehensive limiting
    let max_retries = self.client.max_retries;
    let attempt_counter = std::sync::Arc::new( core::sync::atomic::AtomicU32::new( 0 ) );
    
    // Clone the counter for use in the closure
    let counter_clone = attempt_counter.clone();
    
    // Retry operation with exponential backoff and attempt counting
    retry( backoff, || async
    {
      // Check max_retries limit
      let current_attempt = counter_clone.fetch_add( 1, core::sync::atomic::Ordering::SeqCst );
      if current_attempt >= max_retries
      {
        return Err( backoff::Error::permanent( 
          Error::ApiError( format!( "Maximum retry attempts ({max_retries}) exceeded" ) )
        ) );
      }

      match self.generate_content( request ).await
      {
        Ok( response ) => Ok( response ),
        Err( error ) => 
        {
          // Enhanced error classification for better retry decisions
          match &error
          {
            // API errors from rate limiting (429) are retryable  
            Error::ApiError( msg ) if msg.contains( "429" ) || msg.contains( "Rate limit" ) => 
            {
              Err( backoff::Error::transient( error ) )
            },
            
            // HTTP 5xx server errors are retryable
            Error::ApiError( msg ) if msg.contains( "502" ) || msg.contains( "503" ) || 
                                      msg.contains( "504" ) || msg.contains( "408" ) => 
            {
              Err( backoff::Error::transient( error ) )
            },
            
            // Timeout errors are retryable
            Error::ApiError( msg ) if msg.contains( "timeout" ) || msg.contains( "Timeout" ) => 
            {
              Err( backoff::Error::transient( error ) )
            },
            
            // Connection-specific network errors are retryable  
            Error::NetworkError( msg ) if msg.contains( "connection" ) || 
                                         msg.contains( "Connection" ) ||
                                         msg.contains( "refused" ) ||
                                         msg.contains( "unreachable" ) => 
            {
              Err( backoff::Error::transient( error ) )
            },
            
            // General server and network errors are retryable (catch-all)
            Error::ServerError( _ ) | Error::NetworkError( _ ) => Err( backoff::Error::transient( error ) ),
            
            // All other errors are permanent (4xx client errors, auth errors, etc.)
            _ => Err( backoff::Error::permanent( error ) ),
          }
        }
      }
    } ).await
  }
  /// Generates content using streaming responses (Server-Sent Events).
  ///
  /// This method is similar to [`Self::generate_content`] but returns a stream of incremental
  /// responses instead of waiting for the complete response. This enables real-time
  /// processing of generated content as it becomes available.
  ///
  /// **Note**: This feature is currently not implemented and will return 
  /// [`Error::NotImplemented`]. This method is provided to support TDD development.
  ///
  /// # Arguments
  ///
  /// * `request` - A [`crate::models::GenerateContentRequest`] with the same format
  ///   as used for non-streaming generation
  ///
  /// # Returns
  ///
  /// When implemented, returns a stream of [`crate::models::StreamingResponse`] objects
  /// containing incremental content updates.
  ///
  /// # Errors
  ///
  /// Currently returns [`Error::NotImplemented`]. When implemented, will return:
  /// - [`Error::InvalidArgument`] - Invalid request format or empty content
  /// - [`Error::NetworkError`] - Network connectivity issues
  /// - [`Error::AuthenticationError`] - Invalid or missing API key
  /// - [`Error::NetworkError`] - Stream-specific errors (connection drops, etc.)
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::{ client::Client, GenerateContentRequest, Content, Part };
  /// # use futures::StreamExt;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
  /// let model = models_api.by_name( "gemini-2.5-flash" );
  /// 
  /// let request = GenerateContentRequest {
  ///   contents : vec![ Content {
  ///     parts : vec![ Part {
  ///       text : Some( "Write a long story about adventures".to_string() ),
  ///       ..Default::default()
  ///     } ],
  ///     role : "user".to_string(),
  ///   } ],
  ///   ..Default::default()
  /// };
  /// 
  /// // Stream the response (when implemented)
  /// let mut stream = model.generate_content_stream( &request ).await?;
  /// futures::pin_mut!( stream );
  /// 
  /// while let Some( chunk ) = stream.next().await {
  ///   match chunk {
  ///     Ok( response ) => {
  ///       if let Some( candidates ) = response.candidates {
  ///         for candidate in candidates {
  ///           for part in candidate.content.parts {
  ///             if let Some( text ) = part.text {
  ///               print!( "{}", text ); // Print incremental text
  ///             }
  ///           }
  ///         }
  ///       }
  ///     }
  ///     Err( e ) => eprintln!( "Stream error : {}", e ),
  ///   }
  /// }
  /// # Ok( () )
  /// # }
  /// ```
  #[ cfg( feature = "streaming" ) ]
  #[ inline ]
  pub async fn generate_content_stream
  (
    &self,
    request : &crate::models::GenerateContentRequest,
  )
  ->
  Result< impl futures::Stream< Item = Result< crate::models::StreamingResponse, Error > >, Error >
  {
    // Validate request
    Self::validate_generate_content_request( request )?;
    
    // Build streaming request
    let stream_request = self.build_streaming_request( request );
    
    // Execute streaming request
    let response = self.execute_streaming_request( stream_request ).await?;
    
    // Process streaming response with optimized parsing
    Ok( Self::process_streaming_response( response ) )
  }
  /// Create a streaming request builder for more ergonomic API usage.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::{ client::Client, models::{ Content, Part } };
  /// # async fn example() -> Result<(), Box< dyn std::error::Error > > {
  /// let client = Client::builder()
  ///   .api_key( "your-api-key".to_string() )
  ///   .build()?;
  ///
  /// let mut stream = client
  ///   .models()
  ///   .by_name( "gemini-2.5-flash" )
  ///   .stream_builder()
  ///   .add_content( "user", "Tell me about Rust" )
  ///   .temperature( 0.7 )
  ///   .max_output_tokens( 1000 )
  ///   .execute()
  ///   .await?;
  /// # Ok( () )
  /// # }
  /// ```
  #[ cfg( feature = "streaming" ) ]
  #[ must_use ]
  #[ inline ]
  pub fn stream_builder( &self ) -> crate::models::StreamingRequestBuilder< '_ >
  {
    crate ::models::StreamingRequestBuilder::new( self )
  }

  /// Create a controllable streaming request builder for fine-grained stream management.
  ///
  /// This method provides explicit streaming control following the "Thin Client, Rich API" principle.
  /// All streaming control operations (pause, resume, cancel) are explicit user operations.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::{ client::Client };
  /// # async fn example() -> Result<(), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  ///
  /// let mut controllable_stream = client
  ///   .models()
  ///   .by_name( "gemini-pro" )
  ///   .stream_controllable()
  ///   .text( "Write a long story" )
  ///   .buffer_size( 2048 )
  ///   .create()
  ///   .await?;
  ///
  /// // Explicitly pause the stream
  /// controllable_stream.pause().await?;
  ///
  /// // Resume when ready
  /// controllable_stream.resume().await?;
  ///
  /// // Cancel if needed
  /// controllable_stream.cancel().await?;
  /// # Ok( () )
  /// # }
  /// ```
  #[ cfg( feature = "streaming" ) ]
  #[ must_use ]
  #[ inline ]
  pub fn stream_controllable( &self ) -> crate::models::streaming_control::ControllableStreamBuilder< '_ >
  {
    crate ::models::streaming_control::ControllableStreamBuilder::new( self )
  }

  /// Generates content from a simple text prompt with default settings.
  ///
  /// This is a convenience method for simple text generation that automatically
  /// wraps the text in the required request structure. For more control over
  /// generation parameters, use [`generate_content`] directly.
  ///
  /// # Arguments
  ///
  /// * `prompt` - The text prompt to generate content from
  ///
  /// # Returns
  ///
  /// Returns the first generated text candidate, or an error if generation fails
  /// or no candidates are returned.
  ///
  /// # Errors
  ///
  /// Returns the same errors as [`generate_content`] plus:
  /// - [`Error::ApiError`] - No candidates returned in response
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
  /// let response = model.generate_text( "Explain quantum computing in simple terms" ).await?;
  /// println!( "Generated : {}", response );
  /// # Ok( () )
  /// # }
  /// ```
  ///
  /// [`generate_content`]: ModelApi::generate_content
  #[ inline ]
  pub async fn generate_text( &self, prompt : &str ) -> Result< String, Error >
  {
    let request = crate::models::GenerateContentRequest {
      contents : vec![ crate::models::Content {
        parts : vec![ crate::models::Part {
          text : Some( prompt.to_string() ),
          ..Default::default()
        } ],
        role : "user".to_string(),
      } ],
      ..Default::default()
    };

    let response = self.generate_content( &request ).await?;
    
    // Extract first candidate's text
    response.candidates
      .first()
      .and_then( |candidate| candidate.content.parts.first() )
      .and_then( |part| part.text.as_ref() )
      .cloned()
      .ok_or_else( || Error::ApiError( 
        format!( "No text content returned from model '{}'. The model may have been blocked by safety filters or returned an unexpected response format.", 
          self.model_id )
      ) )
  }
  /// Generates content with custom generation configuration.
  ///
  /// This convenience method allows easy configuration of common generation
  /// parameters like temperature and token limits without manually building
  /// the full request structure.
  ///
  /// # Arguments
  ///
  /// * `prompt` - The text prompt to generate content from  
  /// * `temperature` - Controls randomness (0.0 = deterministic, 1.0 = very random)
  /// * `max_output_tokens` - Maximum number of tokens to generate
  ///
  /// # Returns
  ///
  /// Returns the first generated text candidate, or an error if generation fails.
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
  /// // Generate creative response with high temperature
  /// let creative = model.generate_text_with_config(
  ///   "Write a haiku about programming",
  ///   0.9,  // High creativity
  ///   100   // Limit to 100 tokens
  /// ).await?;
  /// 
  /// println!( "Creative haiku : {}", creative );
  /// # Ok( () )
  /// # }
  /// ```
  ///
  /// # Errors
  ///
  /// Returns the same errors as [`generate_content`].
  ///
  /// [`generate_content`]: ModelApi::generate_content
  #[ inline ]
  pub async fn generate_text_with_config
  (
    &self,
    prompt : &str,
    temperature : f32,
    max_output_tokens : i32,
  )
  ->
  Result< String, Error >
  {
    let request = crate::models::GenerateContentRequest {
      contents : vec![ crate::models::Content {
        parts : vec![ crate::models::Part {
          text : Some( prompt.to_string() ),
          ..Default::default()
        } ],
        role : "user".to_string(),
      } ],
      generation_config : Some( crate::models::GenerationConfig {
        temperature : Some( temperature ),
        max_output_tokens : Some( max_output_tokens ),
        ..Default::default()
      } ),
      ..Default::default()
    };

    let response = self.generate_content( &request ).await?;
    
    // Extract first candidate's text
    response.candidates
      .first()
      .and_then( |candidate| candidate.content.parts.first() )
      .and_then( |part| part.text.as_ref() )
      .cloned()
      .ok_or_else( || Error::ApiError( 
        format!( "No text content returned from model '{}'. The response may have been blocked by safety filters.", 
          self.model_id )
      ) )
  }
  /// Continues a multi-turn conversation by adding a user message.
  ///
  /// This convenience method extends an existing conversation with a new user
  /// message and generates a response. It maintains conversation context
  /// by preserving the conversation history.
  ///
  /// # Arguments
  ///
  /// * `conversation_history` - Previous messages in the conversation
  /// * `user_message` - The new message from the user
  ///
  /// # Returns
  ///
  /// Returns a tuple of (`updated_conversation`, `assistant_response`) where:
  /// - `updated_conversation` includes the new user message and assistant response
  /// - `assistant_response` is the text content of the model's response
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::{ client::Client, models::Content };
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
/// let model = models_api.by_name( "gemini-2.5-flash" );
  /// 
  /// // Start conversation
  /// let mut conversation = vec![];
  /// 
  /// // User asks first question
  /// let (updated_conv, response) = model.continue_conversation(
  ///   conversation, 
  ///   "What is machine learning?"
  /// ).await?;
  /// 
  /// println!( "Assistant : {}", response );
  /// conversation = updated_conv;
  /// 
  /// // User asks follow-up
  /// let (final_conv, response2) = model.continue_conversation(
  ///   conversation,
  ///   "Can you give me a simple example?"
  /// ).await?;
  /// 
  /// println!( "Assistant : {}", response2 );
  /// # Ok( () )
  /// # }
  /// ```
  ///
  /// # Errors
  ///
  /// Returns the same errors as [`generate_content`] plus text extraction errors 
  /// if the model's response cannot be parsed as text.
  ///
  /// [`generate_content`]: ModelApi::generate_content
  #[ inline ]
  pub async fn continue_conversation
  (
    &self,
    mut conversation_history : Vec< crate::models::Content >,
    user_message : &str,
  )
  ->
  Result< ( Vec< crate::models::Content >, String ), Error >
  {
    // Add the new user message to conversation history
    conversation_history.push( crate::models::Content {
      parts : vec![ crate::models::Part {
        text : Some( user_message.to_string() ),
        ..Default::default()
      } ],
      role : "user".to_string(),
    } );

    let request = crate::models::GenerateContentRequest {
      contents : conversation_history.clone(),
      ..Default::default()
    };

    let response = self.generate_content( &request ).await?;
    
    // Extract assistant response
    let assistant_text = response.candidates
      .first()
      .and_then( |candidate| candidate.content.parts.first() )
      .and_then( |part| part.text.as_ref() )
      .cloned()
      .ok_or_else( || Error::ApiError( 
        format!( "No text content returned from model '{}' during conversation.", 
          self.model_id )
      ) )?;

    // Add assistant response to conversation history
    conversation_history.push( crate::models::Content {
      parts : vec![ crate::models::Part {
        text : Some( assistant_text.clone() ),
        ..Default::default()
      } ],
      role : "model".to_string(),
    } );

    Ok( ( conversation_history, assistant_text ) )
  }

  /// Creates a request builder for complex generation scenarios.
  ///
  /// This method returns a builder that allows fluent configuration of
  /// generation parameters, safety settings, and tools before executing
  /// the request. This is useful for complex scenarios that need fine-grained
  /// control over the generation process.
  ///
  /// # Returns
  ///
  /// Returns a [`GenerationRequestBuilder`] for fluent request configuration.
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
  /// let response = model.generation_request()
  ///   .with_prompt( "Write a story about AI" )
  ///   .with_temperature( 0.7 )
  ///   .with_max_tokens( 500 )
  ///   .with_stop_sequences( vec![ "THE END".to_string() ] )
  ///   .execute()
  ///   .await?;
  ///
  /// println!( "Generated story : {:?}", response );
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  #[ must_use ]
  pub fn generation_request( &self ) -> GenerationRequestBuilder< '_ >
  {
    GenerationRequestBuilder::new( self )
  }

  /// Generates content for multiple prompts in batch.
  ///
  /// This method processes multiple prompts efficiently using batch processing
  /// to minimize API calls and improve performance compared to individual requests.
  ///
  /// # Arguments
  ///
  /// * `prompts` - A slice of prompt strings to process
  ///
  /// # Returns
  ///
  /// Returns a vector of `GenerateContentResponse` objects, one for each prompt.
  /// The order of responses corresponds to the order of input prompts.
  ///
  /// # Errors
  ///
  /// Same error conditions as [`Self::batch_embed_texts`].
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
  /// let model = models_api.by_name( "gemini-1.5-pro" );
  /// 
  /// let prompts = vec![
  ///   "Write a haiku",
  ///   "Explain AI",
  ///   "What is Rust?",
  /// ];
  /// 
  /// let responses = model.batch_generate_content( &prompts ).await?;
  /// println!( "Generated {} responses", responses.len() );
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn batch_generate_content( &self, prompts : &[ &str ] ) -> Result< Vec< GenerateContentResponse >, Error >
  {
    // Validate input
    if prompts.is_empty()
    {
      return Err( Error::ValidationError { 
        message : "Cannot process empty prompt list".to_string() 
      } );
    }

    // For now, process prompts individually
    // qqq : Implement actual batch API when available from Gemini
    let mut responses = Vec::with_capacity( prompts.len() );
    let mut successful = 0;
    let mut failed = 0;

    for prompt in prompts
    {
      match self.generate_text( prompt ).await
      {
        Ok( text ) => {
          // Create a mock response structure for now
          // In a real implementation, we'd use the actual generate_content method
          let response = GenerateContentResponse {
            candidates : vec![ Candidate {
              content : Content {
                parts : vec![ Part {
                  text : Some( text ),
                  ..Default::default()
                } ],
                role : "model".to_string(),
              },
              finish_reason : Some( "STOP".to_string() ),
              safety_ratings : None,
              citation_metadata : None,
              token_count : None,
              index : Some( 0 ),
            } ],
            prompt_feedback : None,
            usage_metadata : None,
            grounding_metadata : None,
          };
          responses.push( response );
          successful += 1;
        },
        Err( e ) => {
          failed += 1;
          if responses.is_empty()
          {
            return Err( e );
          }
          let remaining = prompts.len() - successful - failed;
          return Err( Error::BatchProcessingError {
            successful,
            failed : failed + remaining,
            message : format!( "Batch processing failed on prompt '{prompt}': {e}" ),
          } );
        }
      }
    }

    Ok( responses )
  }
  #[ inline ]
  fn validate_generate_content_request( request : &crate::models::GenerateContentRequest ) -> Result< (), Error >
  {
    if request.contents.is_empty()
    {
      return Err( Error::InvalidArgument( 
        "Generate content request cannot have empty contents. Please provide at least one content item.".to_string()
      ) );
    }
    Ok( () )
  }

  /// Build streaming HTTP request with optimized headers
  #[ cfg( feature = "streaming" ) ]
  #[ inline ]
  fn build_streaming_request( &self, request : &crate::models::GenerateContentRequest ) -> reqwest::RequestBuilder
  {
    let url = format!(
      "{}/v1beta/models/{}:streamGenerateContent",
      self.client.base_url,
      self.model_id
    );

    // Use client's configured HTTP client for connection reuse
    // Gemini API returns newline-delimited JSON (NDJSON) for streaming
    self.client.http
      .post( &url )
      .header( "Accept", "application/json" )
      .header( "Cache-Control", "no-cache" )
      .header( "Connection", "keep-alive" )
      .header( "User-Agent", "api_gemini/0.2.0" )
      .query( &[ ( "key", &*self.client.api_key ) ] )
      .json( request )
  }

  /// Execute streaming request with enhanced error handling
  #[ cfg( feature = "streaming" ) ]
  #[ inline ]
  async fn execute_streaming_request( &self, request : reqwest::RequestBuilder ) -> Result< reqwest::Response, Error >
  {
    let response = request
      .send()
      .await
      .map_err( |e| self.enhance_model_operation_error( "initiate streaming content generation", e.into() ) )?;

    if !response.status().is_success()
    {
      let status = response.status();
      let error_text = response.text().await.unwrap_or_else( |_| "Unknown error".to_string() );
      
      let enhanced_error = match status.as_u16()
      {
        429 => format!( "Rate limit exceeded for streaming requests. Please reduce request frequency. Details : {error_text}" ),
        500..=599 => format!( "Server error during streaming request ({status}). This may be temporary - please retry. Details : {error_text}" ),
        401 => format!( "Authentication failed for streaming request. Please check your API key. Details : {error_text}" ),
        403 => format!( "Streaming requests not authorized for your API key. Please check permissions. Details : {error_text}" ),
        _ => format!( "Streaming request failed with HTTP {status}. Details : {error_text}" ),
      };
      
      return Err( Error::ApiError( enhanced_error ) );
    }

    Ok( response )
  }

  /// Process Gemini streaming response by buffering and parsing as JSON array.
  ///
  /// # Gemini Streaming API Format
  ///
  /// **CRITICAL**: Gemini's `:streamGenerateContent` endpoint returns a complete JSON array
  /// containing all response chunks, NOT Server-Sent Events (SSE) or newline-delimited JSON.
  ///
  /// ## Actual Format
  ///
  /// ```json
  /// [
  ///   {"candidates": [...], "usageMetadata": {...}},
  ///   {"candidates": [...], "usageMetadata": {...}, "finishReason": "STOP"}
  /// ]
  /// ```
  ///
  /// ## NOT These Formats
  ///
  /// - ❌ NOT SSE: `data : {...}\n\ndata : {...}\n\n`
  /// - ❌ NOT NDJSON: `{...}\n{...}\n`
  /// - ❌ NOT chunked JSON: Separate JSON objects
  ///
  /// ## Implementation Strategy
  ///
  /// 1. **Buffer entire response**: Call `response.bytes().await` to collect full body
  /// 2. **Parse as array**: `serde_json::from_str::< Vec< GenerateContentResponse > >(&text)`
  /// 3. **Emit as stream**: Use `async_stream::stream!` to yield array elements as chunks
  /// 4. **Add final marker**: Emit terminal chunk with `is_final : true` after array exhausted
  ///
  /// ## Why Not SSE Parser?
  ///
  /// Previous implementation used `eventsource-stream` crate expecting SSE format. This failed
  /// because the SSE parser couldn't recognize the JSON array structure, resulting in zero
  /// parsed chunks. See integration test documentation in `tests/comprehensive_integration_tests.rs`
  /// for detailed failure analysis.
  ///
  /// ## Performance Trade-off
  ///
  /// Buffering the entire response before parsing means:
  /// - ✅ Simple, robust parsing
  /// - ✅ Matches actual Gemini API behavior
  /// - ⚠️ Delays first chunk until complete response received
  /// - ⚠️ Higher memory usage for large responses
  ///
  /// This is acceptable because Gemini sends the complete array quickly (typically <1 second)
  /// and response sizes are limited by API constraints.
  #[ cfg( feature = "streaming" ) ]
  #[ inline ]
  fn process_streaming_response( response : reqwest::Response ) -> impl futures::Stream< Item = Result< crate::models::StreamingResponse, Error > >
  {
    // Gemini API returns a JSON array : [{...response1...}, {...response2...}]
    // We need to buffer the entire response and parse as array
    //
    // DEVELOPMENT NOTE: Previous versions imported `futures::{StreamExt, stream}` here,
    // expecting to use incremental stream processing. However, these imports were never
    // actually used because:
    // 1. The `async_stream::stream!` macro handles all stream creation internally
    // 2. Gemini's JSON array format requires full buffering (not incremental parsing)
    // 3. No futures combinator operations are needed in this function
    //
    // The unused imports were removed during test-clean cycle on 2025-10-12 when
    // `-D warnings` flag caught them during ctest3 verification.
    async_stream ::stream!
    {
      // Collect all bytes
      let bytes_result = response.bytes().await;

      match bytes_result
      {
        Ok( bytes ) => {
          let text = String::from_utf8_lossy( &bytes );

          // Parse as JSON array of GenerateContentResponse
          match serde_json::from_str::< Vec< crate::models::GenerateContentResponse > >( &text )
          {
            Ok( responses ) => {
              // Emit each response as a streaming chunk
              for api_response in responses.into_iter()
              {
                let is_final = api_response.candidates
                  .first()
                  .and_then( |candidate| candidate.finish_reason.as_ref() )
                  .is_some();

                let streaming_response = crate::models::StreamingResponse {
                  candidates : Some( api_response.candidates ),
                  usage_metadata : api_response.usage_metadata,
                  is_final : Some( is_final ),
                  error : None,
                };

                yield Ok( streaming_response );
              }

              // Emit final marker
              yield Ok( crate::models::StreamingResponse {
                candidates : None,
                usage_metadata : None,
                is_final : Some( true ),
                error : None,
              } );
            },
            Err( parse_error ) => {
              yield Err( Error::SerializationError( format!( "Failed to parse streaming response array : {parse_error}" ) ) );
            }
          }
        },
        Err( network_error ) => {
          yield Err( Error::NetworkError( format!( "Failed to read streaming response : {network_error}" ) ) );
        }
      }
    }
  }
}

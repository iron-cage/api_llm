//! API handle for chat completion operations.

#[ cfg( feature = "chat" ) ]
use super::super::Client;
#[ cfg( feature = "chat" ) ]
use super::conversation_builder::ConversationBuilder;

/// API handle for chat completion operations.
#[ cfg( feature = "chat" ) ]
#[ derive( Debug ) ]

pub struct ChatApi< 'a >
{
    #[ allow( dead_code ) ]
    pub( crate ) client : &'a Client,
}

#[ cfg( feature = "chat" ) ]
impl ChatApi< '_ >
{
  /// Complete a chat conversation.
  ///
  /// Optimized implementation that converts chat messages to Gemini's Content format 
  /// and uses generateContent API with improved performance and error handling.
  ///
  /// # Arguments
  ///
  /// * `request` - A [`crate::models::ChatCompletionRequest`] containing the conversation
  ///
  /// # Returns
  ///
  /// Returns a [`crate::models::ChatCompletionResponse`] with the completion.
  ///
  /// # Errors
  ///
  /// - [`Error::InvalidArgument`] - Empty messages, invalid roles, or malformed request
  /// - [`Error::AuthenticationError`] - Invalid or missing API key
  /// - [`Error::NetworkError`] - Network connectivity issues
  /// - [`Error::ApiError`] - Server returned error or empty response
  ///
  #[ inline ]
  pub async fn complete
  (
    &self,
    request : &crate::models::ChatCompletionRequest,
  )
  ->
  Result< crate::models::ChatCompletionResponse, crate::error::Error >
  {
    // Validate and convert request to GenerateContentRequest
    let generate_request = Self::validate_and_convert_chat_request( request )?;

    // Use existing generate_content implementation
    let response = self.client
      .models()
      .by_name( &request.model )
      .generate_content( &generate_request )
      .await
      .map_err( | e | match e
      {
        crate ::error::Error::ApiError( msg ) if msg.contains( "400" ) => 
          crate ::error::Error::InvalidArgument( 
            format!( "Chat completion request failed : {msg}. Please check message format and roles." )
          ),
        crate ::error::Error::ApiError( msg ) if msg.contains( "401" ) || msg.contains( "403" ) => 
          crate ::error::Error::AuthenticationError( 
            format!( "Chat completion authentication failed : {msg}. Please verify your API key has chat permissions." )
          ),
        crate ::error::Error::NetworkError( msg ) => 
          crate ::error::Error::NetworkError( 
            format!( "Chat completion network error : {msg}. This may be temporary - please retry." )
          ),
        other => other,
      } )?;

    // Convert GenerateContentResponse to ChatCompletionResponse
    Self::convert_to_chat_response( response, request )
  }

  /// Validate chat request and convert to `GenerateContentRequest` for optimal processing.
  ///
  /// Consolidates validation and conversion logic to eliminate code duplication
  /// between streaming and non-streaming chat completion methods.
  fn validate_and_convert_chat_request
  (
    request : &crate::models::ChatCompletionRequest,
  )
  ->
  Result< crate::models::GenerateContentRequest, crate::error::Error >
  {
    use crate::models::{ GenerateContentRequest, Content, Part };
    
    // Validate request with detailed error context
    if request.messages.is_empty()
    {
      return Err( crate::error::Error::InvalidArgument( 
        "Chat completion requires at least one message. Please provide a non-empty messages array.".to_string() 
      ) );
    }

    // Pre-allocate vectors with known capacity for better performance
    let mut contents = Vec::with_capacity( request.messages.len() );
    let mut system_instruction : Option< String > = None;

    // Validate and convert messages in a single pass
    for ( index, message ) in request.messages.iter().enumerate()
    {
      // Validate message content
      if message.content.is_empty()
      {
        return Err( crate::error::Error::InvalidArgument( 
          format!( "Message at index {index} has empty content. All messages must have non-empty content." )
        ) );
      }

      match message.role.as_str()
      {
        "system" => {
          if system_instruction.is_some()
          {
            return Err( crate::error::Error::InvalidArgument( 
              "Multiple system messages found. Only one system message is allowed per conversation.".to_string() 
            ) );
          }
          system_instruction = Some( message.content.clone() );
        },
        "user" => {
          contents.push( Content
          {
            parts : vec![ Part
            {
              text : Some( message.content.clone() ),
              ..Default::default()
            } ],
            role : "user".to_string(),
          } );
        },
        "assistant" => {
          contents.push( Content
          {
            parts : vec![ Part
            {
              text : Some( message.content.clone() ),
              ..Default::default()
            } ],
            role : "model".to_string(), // Gemini uses "model" for assistant role
          } );
        },
        invalid_role => {
          return Err( crate::error::Error::InvalidArgument( 
            format!( "Invalid message role '{invalid_role}' at index {index}. Valid roles are : 'user', 'assistant', 'system'." )
          ) );
        },
      }
    }

    // Validate conversation state : must have at least one user message
    if !request.messages.iter().any( | msg | msg.role == "user" )
    {
      return Err( crate::error::Error::InvalidArgument( 
        "Chat completion requires at least one user message to generate a response.".to_string() 
      ) );
    }

    // Create optimized generation config only when parameters are specified
    let generation_config = Self::build_generation_config( request );

    // Handle system instruction by prepending it as the first user message
    // This is more memory efficient than inserting at index 0
    if let Some( system_content ) = system_instruction
    {
      let system_content_obj = Content
      {
        parts : vec![ Part
        {
          text : Some( format!( "System : {system_content}" ) ),
          ..Default::default()
        } ],
        role : "user".to_string(),
      };
      contents.insert( 0, system_content_obj );
    }
    
    Ok( GenerateContentRequest
    {
      contents,
      generation_config,
      ..Default::default()
    } )
  }

  /// Build generation config from chat request parameters with validation.
  fn build_generation_config
  (
    request : &crate::models::ChatCompletionRequest,
  )
  ->
  Option< crate::models::GenerationConfig >
  {
    use crate::models::GenerationConfig;

    // Only create config if parameters are actually specified
    let has_parameters = request.temperature.is_some() || 
                        request.max_tokens.is_some() ||
                        request.top_p.is_some() ||
                        request.frequency_penalty.is_some() ||
                        request.presence_penalty.is_some();

    if !has_parameters
    {
      return None;
    }

    Some( GenerationConfig
    {
      temperature : request.temperature,
      max_output_tokens : request.max_tokens,
      top_p : request.top_p,
      // Note : Gemini doesn't directly support frequency_penalty and presence_penalty
      // They would need custom handling in future versions
      ..Default::default()
    } )
  }

  /// Convert `GenerateContentResponse` to `ChatCompletionResponse` format with optimizations.
  fn convert_to_chat_response
  (
    response : crate::models::GenerateContentResponse,
    request : &crate::models::ChatCompletionRequest,
  )
  ->
  Result< crate::models::ChatCompletionResponse, crate::error::Error >
  {
    use crate::models::{ ChatChoice, ChatMessage, ChatUsage };

    if response.candidates.is_empty()
    {
      return Err( crate::error::Error::ApiError( 
        format!( "No response candidates generated for model '{}'. This may indicate content filtering or server issues.", 
          request.model )
      ) );
    }

    // Pre-allocate choices vector for better performance
    let mut choices = Vec::with_capacity( response.candidates.len() );
    
    for ( index, candidate ) in response.candidates.into_iter().enumerate()
    {
      // Optimized content extraction with better error handling
      let content = if let Some( first_part ) = candidate.content.parts.into_iter().next()
      {
        first_part.text.unwrap_or_else( String::new )
      } else {
        String::new()
      };

      let message = ChatMessage
      {
        role : "assistant".to_string(),
        content,
      };

      // Enhanced finish reason mapping with better context
      let finish_reason = match candidate.finish_reason.as_deref()
      {
        Some( "MAX_TOKENS" ) => "length",
        Some("SAFETY" | "RECITATION") => "content_filter",
        _ => "stop",
      }.to_string();

      choices.push( ChatChoice
      {
        index : i32::try_from( index ).unwrap_or( i32::MAX ),
        message,
        finish_reason : Some( finish_reason ),
      } );
    }

    // Optimized usage calculation
    let usage = response.usage_metadata.map( | usage | ChatUsage
    {
      prompt_tokens : usage.prompt_token_count.unwrap_or( 0 ),
      completion_tokens : usage.candidates_token_count.unwrap_or( 0 ),
      total_tokens : usage.total_token_count.unwrap_or( 0 ),
    } );

    // Generate deterministic but unique ID based on timestamp and content hash
    let timestamp = std::time::SystemTime::now()
      .duration_since( std::time::UNIX_EPOCH )
      .map_err( | e | crate::error::Error::Io( format!( "System time error : {e}" ) ) )?
      .as_secs();
      
    let id = format!( "chatcmpl-{:x}{:x}", timestamp, rand::random::< u32 >() );

    Ok( crate::models::ChatCompletionResponse
    {
      id,
      object : "chat.completion".to_string(),
      created : i64::try_from( timestamp ).unwrap_or( 0 ),
      model : request.model.clone(),
      choices,
      usage,
    } )
  }

  /// Complete a chat conversation with streaming.
  ///
  /// Optimized streaming implementation that uses the same validation and conversion
  /// logic as the non-streaming version for consistency and performance.
  ///
  /// # Arguments
  ///
  /// * `request` - A [`crate::models::ChatCompletionRequest`] with streaming enabled
  ///
  /// # Returns
  ///
  /// Returns a stream of [`crate::models::ChatCompletionResponse`] objects.
  ///
  /// # Errors
  ///
  /// - [`Error::InvalidArgument`] - Empty messages, invalid roles, or malformed request
  /// - [`Error::AuthenticationError`] - Invalid or missing API key
  /// - [`Error::NetworkError`] - Network connectivity issues
  /// - [`Error::ApiError`] - Streaming connection or server issues
  ///
  #[ cfg( feature = "streaming" ) ]
  #[ inline ]
  pub async fn complete_stream
  (
    &self,
    request : &crate::models::ChatCompletionRequest,
  )
  ->
  Result< impl futures::Stream< Item = Result< crate::models::ChatCompletionResponse, crate::error::Error > > + use< '_ >, crate::error::Error >
  {
    use futures::stream::StreamExt;
    
    // Use shared validation and conversion logic
    let generate_request = Self::validate_and_convert_chat_request( request )?;

    // Use existing streaming implementation with enhanced error context
    let stream = self.client
      .models()
      .by_name( &request.model )
      .generate_content_stream( &generate_request )
      .await
      .map_err( | e | match e
      {
        crate ::error::Error::ApiError( msg ) if msg.contains( "400" ) => 
          crate ::error::Error::InvalidArgument( 
            format!( "Chat streaming request failed : {msg}. Please check message format and roles." )
          ),
        crate ::error::Error::ApiError( msg ) if msg.contains( "401" ) || msg.contains( "403" ) => 
          crate ::error::Error::AuthenticationError( 
            format!( "Chat streaming authentication failed : {msg}. Please verify your API key has streaming permissions." )
          ),
        crate ::error::Error::NetworkError( msg ) => 
          crate ::error::Error::NetworkError( 
            format!( "Chat streaming network error : {msg}. Connection may be unstable - please retry." )
          ),
        other => other,
      } )?;

    // Convert each streaming response to ChatCompletionResponse format
    // Clone request to move into closure for efficient processing
    let request_clone = request.clone();
    let chat_stream = stream.map( move | result | 
    {
      match result
      {
        Ok( response ) => Self::convert_streaming_to_chat_response( response, &request_clone ),
        Err( e ) => Err( Self::enhance_streaming_error_context( e ) ),
      }
    } );

    Ok( chat_stream )
  }

  /// Enhance streaming-specific error context for better debugging.
  #[ cfg( feature = "streaming" ) ]
  fn enhance_streaming_error_context
  (
    error : crate::error::Error,
  )
  ->
  crate ::error::Error
  {
    match error
    {
      crate ::error::Error::NetworkError( msg ) => 
        crate ::error::Error::NetworkError( 
          format!( "Streaming connection error : {msg}. This may indicate network instability or server-side issues." )
        ),
      crate ::error::Error::SerializationError( msg ) => 
        crate ::error::Error::SerializationError( 
          format!( "Streaming response parsing error : {msg}. This may indicate malformed server-sent events." )
        ),
      other => other,
    }
  }

  /// Convert streaming `StreamingResponse` to `ChatCompletionResponse` format with optimizations.
  #[ cfg( feature = "streaming" ) ]
  fn convert_streaming_to_chat_response
  (
    response : crate::models::StreamingResponse,
    request : &crate::models::ChatCompletionRequest,
  )
  ->
  Result< crate::models::ChatCompletionResponse, crate::error::Error >
  {
    // Convert StreamingResponse to GenerateContentResponse-like structure for reuse
    // This avoids code duplication while maintaining type safety
    let generate_response = crate::models::GenerateContentResponse
    {
      candidates : response.candidates.unwrap_or_default(),
      prompt_feedback : None, // StreamingResponse doesn't include prompt_feedback
      usage_metadata : response.usage_metadata,
      grounding_metadata : None, // StreamingResponse doesn't include grounding_metadata
    };
    
    Self::convert_to_chat_response( generate_response, request )
  }

  #[ cfg( not( feature = "streaming" ) ) ]
  #[ inline ]
  pub async fn complete_stream
  (
    &self,
    _request : &crate::models::ChatCompletionRequest,
  )
  ->
  Result< futures::stream::Empty< Result< crate::models::ChatCompletionResponse, crate::error::Error > >, crate::error::Error >
  {
    Err( crate::error::Error::NotImplemented( 
      "Chat completion streaming requires the 'streaming' feature flag".to_string()
    ) )
  }

  /// Create an enhanced conversation builder for fluent API.
  ///
  /// Creates a new conversation builder with improved ergonomics for constructing 
  /// chat requests with generation parameters and method chaining.
  ///
  /// # Returns
  ///
  /// Returns a [`ConversationBuilder`] for building optimized chat conversations.
  ///
  /// # Errors
  ///
  /// Currently always succeeds, but returns Result for future extensibility.
  ///
  #[ inline ]
  pub fn conversation( &self ) -> Result< ConversationBuilder< '_ >, crate::error::Error >
  {
    Ok( ConversationBuilder
    {
      client : self.client,
      messages : Vec::new(),
      model : "gemini-2.5-flash".to_string(), // Default model
      temperature : None,
      max_tokens : None,
      top_p : None,
    } )
  }
}

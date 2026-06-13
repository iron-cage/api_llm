//! Content generation functionality for Anthropic API
//!
//! This module provides a dedicated interface for content generation operations,
//! refactored from the core client to improve modularity and maintainability.

// Allow missing inline attributes for content generation module
#[ allow( clippy::missing_inline_in_public_items ) ]
mod private
{
  use crate::{ 
    error::{ AnthropicError, AnthropicResult }, 
    client::{ CreateMessageRequest, Client },
    messages::Message,
  };
  use serde::{ Serialize, Deserialize };
  use core::time::Duration;

  /// Content generation configuration and parameters
  #[ derive( Debug, Clone, Serialize ) ]
  pub struct ContentGenerationRequest
  {
    /// Model to use for generation
    pub model : String,
    /// Maximum tokens to generate
    pub max_tokens : u32,
    /// Input messages or prompt
    pub messages : Vec< Message >,
    /// System prompt for behavior guidance
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub system : Option< String >,
    /// Temperature for creativity control (0.0-1.0)
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub temperature : Option< f32 >,
    /// Custom generation settings
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub settings : Option< GenerationSettings >,
  }

  /// Advanced content generation settings
  #[ derive( Debug, Clone, Serialize, Deserialize, Default ) ]
  pub struct GenerationSettings
  {
    /// Stop sequences to terminate generation
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub stop_sequences : Option< Vec< String > >,
    /// Top-p nucleus sampling parameter
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub top_p : Option< f32 >,
    /// Top-k sampling parameter
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub top_k : Option< u32 >,
    /// Presence penalty
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub presence_penalty : Option< f32 >,
    /// Frequency penalty
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub frequency_penalty : Option< f32 >,
  }


  /// Builder for content generation requests
  #[ derive( Debug, Default ) ]
  pub struct ContentGenerationRequestBuilder
  {
    model : Option< String >,
    max_tokens : Option< u32 >,
    messages : Vec< Message >,
    system : Option< String >,
    temperature : Option< f32 >,
    settings : Option< GenerationSettings >,
  }


  impl ContentGenerationRequestBuilder
  {
    /// Create a new builder
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Set the model to use
    #[ must_use ]
    pub fn model< S : Into< String > >( mut self, model : S ) -> Self
    {
      self.model = Some( model.into() );
      self
    }

    /// Set maximum tokens to generate
    #[ must_use ]
    pub fn max_tokens( mut self, max_tokens : u32 ) -> Self
    {
      self.max_tokens = Some( max_tokens );
      self
    }

    /// Add a message to the conversation
    #[ must_use ]
    pub fn message( mut self, message : Message ) -> Self
    {
      self.messages.push( message );
      self
    }

    /// Add multiple messages to the conversation
    #[ must_use ]
    pub fn messages( mut self, messages : Vec< Message > ) -> Self
    {
      self.messages.extend( messages );
      self
    }

    /// Set system prompt
    #[ must_use ]
    pub fn system< S : Into< String > >( mut self, system : S ) -> Self
    {
      self.system = Some( system.into() );
      self
    }

    /// Set temperature for creativity control
    #[ must_use ]
    pub fn temperature( mut self, temperature : f32 ) -> Self
    {
      self.temperature = Some( temperature );
      self
    }

    /// Set advanced generation settings
    #[ must_use ]
    pub fn settings( mut self, settings : GenerationSettings ) -> Self
    {
      self.settings = Some( settings );
      self
    }

    /// Build the content generation request
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing
    pub fn build( self ) -> AnthropicResult< ContentGenerationRequest >
    {
      let model = self.model.ok_or_else( || 
        AnthropicError::InvalidArgument( "Model is required".to_string() )
      )?;
      
      // Validate model is not empty
      if model.trim().is_empty()
      {
        return Err( AnthropicError::InvalidArgument( "Model cannot be empty".to_string() ) );
      }

      let max_tokens = self.max_tokens.ok_or_else( || 
        AnthropicError::InvalidArgument( "Max tokens is required".to_string() )
      )?;

      if self.messages.is_empty()
      {
        return Err( AnthropicError::InvalidArgument( "At least one message is required".to_string() ) );
      }

      Ok( ContentGenerationRequest
      {
        model,
        max_tokens,
        messages : self.messages,
        system : self.system,
        temperature : self.temperature,
        settings : self.settings,
      })
    }
  }

  impl ContentGenerationRequest
  {
    /// Create a new builder
    #[ must_use ]
    pub fn builder() -> ContentGenerationRequestBuilder
    {
      ContentGenerationRequestBuilder::new()
    }

    /// Convert to `CreateMessageRequest` for API compatibility
    #[ must_use ]
    pub fn to_message_request( &self ) -> CreateMessageRequest
    {
      CreateMessageRequest
      {
        model : self.model.clone(),
        max_tokens : self.max_tokens,
        messages : self.messages.clone(),
        system : self.system.as_ref().map( | s | vec![ crate::SystemContent::text( s.as_str() ) ] ),
        temperature : self.temperature,
        stream : None,
        tools : None,
        tool_choice : None,
      }
    }

    /// Validate the content generation request
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails
    pub fn validate( &self ) -> AnthropicResult< () >
    {
      // Validate model name
      if self.model.trim().is_empty()
      {
        return Err( AnthropicError::InvalidArgument( "Model cannot be empty".to_string() ) );
      }

      // Validate max_tokens range
      if self.max_tokens == 0 || self.max_tokens > 200_000
      {
        return Err( AnthropicError::InvalidArgument( 
          "Max tokens must be between 1 and 200,000".to_string() 
        ));
      }

      // Validate temperature range
      if let Some( temp ) = self.temperature
      {
        if !(0.0..=1.0).contains( &temp )
        {
          return Err( AnthropicError::InvalidArgument( 
            "Temperature must be between 0.0 and 1.0".to_string() 
          ));
        }
      }

      // Validate messages
      if self.messages.is_empty()
      {
        return Err( AnthropicError::InvalidArgument( "At least one message is required".to_string() ) );
      }

      Ok( () )
    }
  }

  /// Content generation response with additional metadata
  #[ derive( Debug, Clone, Deserialize ) ]
  pub struct ContentGenerationResponse
  {
    /// Generated content
    pub content : String,
    /// Model used for generation
    pub model : String,
    /// Token usage statistics
    pub usage : crate::client::Usage,
    /// Stop reason
    pub stop_reason : Option< String >,
    /// Response metadata
    pub metadata : GenerationMetadata,
  }

  /// Metadata about content generation
  #[ derive( Debug, Clone, Deserialize, Default ) ]
  pub struct GenerationMetadata
  {
    /// Generation time in milliseconds
    pub generation_time_ms : Option< u64 >,
    /// Model version used
    pub model_version : Option< String >,
    /// Content safety assessment
    pub safety_assessment : Option< String >,
  }

  /// Content generator with advanced capabilities
  #[ derive( Debug, Clone ) ]
  pub struct ContentGenerator
  {
    /// HTTP client for API requests
    client : Client,
    /// Default model to use
    default_model : String,
    /// Default max tokens
    default_max_tokens : u32,
    /// Generation timeout
    timeout : Duration,
  }

  impl ContentGenerator
  {
    /// Create a new content generator
    #[ must_use ]
    pub fn new( client : Client ) -> Self
    {
      Self
      {
        client,
        default_model : "claude-sonnet-4-6".to_string(),
        default_max_tokens : 1000,
        timeout : Duration::from_secs( 60 ),
      }
    }

    /// Set default model for generation
    #[ must_use ]
    pub fn with_default_model< S : Into< String > >( mut self, model : S ) -> Self
    {
      self.default_model = model.into();
      self
    }

    /// Set default max tokens
    #[ must_use ]
    pub fn with_default_max_tokens( mut self, max_tokens : u32 ) -> Self
    {
      self.default_max_tokens = max_tokens;
      self
    }

    /// Set generation timeout
    #[ must_use ]
    pub fn with_timeout( mut self, timeout : Duration ) -> Self
    {
      self.timeout = timeout;
      self
    }

    /// Generate content using the refactored interface
    ///
    /// # Errors
    ///
    /// Returns an error if generation fails or request is invalid
    pub async fn generate( &self, request : ContentGenerationRequest ) -> AnthropicResult< ContentGenerationResponse >
    {
      // Validate request
      request.validate()?;

      // Convert to message request for API compatibility
      let message_request = request.to_message_request();

      // Generate content using the underlying client
      let start_time = std::time::Instant::now();
      let response = self.client.create_message( message_request ).await?;
      let generation_time = start_time.elapsed();

      // Extract content from response
      let content = response.text().unwrap_or( "" ).to_string();

      Ok( ContentGenerationResponse
      {
        content,
        model : response.model,
        usage : response.usage,
        stop_reason : response.stop_reason,
        metadata : GenerationMetadata
        {
          generation_time_ms : Some( generation_time.as_millis().try_into().unwrap_or( u64::MAX ) ),
          model_version : None,
          safety_assessment : None,
        },
      })
    }

    /// Generate content with simple text input
    ///
    /// # Errors
    ///
    /// Returns an error if generation fails
    pub async fn generate_text( &self, prompt : &str ) -> AnthropicResult< String >
    {
      let request = ContentGenerationRequest::builder()
        .model( &self.default_model )
        .max_tokens( self.default_max_tokens )
        .message( Message::user( prompt.to_string() ) )
        .build()?;

      let response = self.generate( request ).await?;
      Ok( response.content )
    }

    /// Generate content with system prompt and user input
    ///
    /// # Errors
    ///
    /// Returns an error if generation fails
    pub async fn generate_with_system( &self, system : &str, prompt : &str ) -> AnthropicResult< String >
    {
      let request = ContentGenerationRequest::builder()
        .model( &self.default_model )
        .max_tokens( self.default_max_tokens )
        .system( system )
        .message( Message::user( prompt.to_string() ) )
        .build()?;

      let response = self.generate( request ).await?;
      Ok( response.content )
    }
  }

  /// Extension methods for Client to provide content generation capabilities
  impl Client
  {
    /// Create a content generator with this client
    #[ must_use ]
    pub fn content_generator( &self ) -> ContentGenerator
    {
      ContentGenerator::new( self.clone() )
    }

    /// Generate content directly using the refactored interface
    ///
    /// # Errors
    ///
    /// Returns an error if generation fails
    pub async fn generate_content( &self, request : ContentGenerationRequest ) -> AnthropicResult< ContentGenerationResponse >
    {
      let generator = self.content_generator();
      generator.generate( request ).await
    }
  }
}

#[ cfg( feature = "content-generation" ) ]
crate::mod_interface!
{
  exposed use
  {
    ContentGenerationRequest,
    ContentGenerationRequestBuilder,
    ContentGenerationResponse,
    GenerationSettings,
    GenerationMetadata,
    ContentGenerator,
  };
}

#[ cfg( not( feature = "content-generation" ) ) ]
crate::mod_interface!
{
  // Empty when content-generation feature is disabled
}
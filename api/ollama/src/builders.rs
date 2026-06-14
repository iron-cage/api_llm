//! Request builder patterns for Ollama API.
//!
//! Fluent builder APIs for ChatRequest, GenerateRequest, and EmbeddingsRequest.

#[ cfg( feature = "enabled" ) ]
mod private
{
  use crate::{ OllamaResult, ChatRequest, GenerateRequest };
  #[ cfg( feature = "vision_support" ) ]
  use crate::{ ChatMessage, MessageRole };
  #[ cfg( feature = "embeddings" ) ]
  use crate::{ EmbeddingsRequest };
  use error_tools::format_err;

  /// Builder for `ChatRequest` with fluent API
  #[ cfg( feature = "builder_patterns" ) ]
  #[ derive( Debug, Clone ) ]
  #[ must_use ]
  pub struct ChatRequestBuilder
  {
    model : Option< String >,
    #[ cfg( feature = "vision_support" ) ]
    messages : Vec< ChatMessage >,
    #[ cfg( not( feature = "vision_support" ) ) ]
    messages : Vec< Message >,
    stream : Option< bool >,
    options : Option< serde_json::Value >,
  }

  /// Builder for `GenerateRequest` with fluent API
  #[ cfg( feature = "builder_patterns" ) ]
  #[ derive( Debug, Clone ) ]
  #[ must_use ]
  pub struct GenerateRequestBuilder
  {
    model : Option< String >,
    prompt : Option< String >,
    stream : Option< bool >,
    options : Option< serde_json::Value >,
  }

  /// Builder for `EmbeddingsRequest` with fluent API
  #[ cfg( all( feature = "builder_patterns", feature = "embeddings" ) ) ]
  #[ derive( Debug, Clone ) ]
  #[ must_use ]
  pub struct EmbeddingsRequestBuilder
  {
    model : Option< String >,
    prompt : Option< String >,
    options : Option< std::collections::HashMap<  String, serde_json::Value  > >,
  }

  // Builder implementations will be inserted here by bash script
  impl ChatRequestBuilder
  {
    /// Create a new `ChatRequestBuilder`
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        model : None,
        messages : Vec::new(),
        stream : Some( false ), // Default to non-streaming for compatibility
        options : None,
      }
    }
    
    /// Helper method to set an option value
    fn set_option( &mut self, key : &str, value : serde_json::Value )
    {
      if self.options.is_none()
      {
        self.options = Some( serde_json::Value::Object( serde_json::Map::new() ) );
      }
      if let Some( serde_json::Value::Object( ref mut map ) ) = self.options
      {
        map.insert( key.to_string(), value );
      }
    }

    /// Set the model name
    #[ inline ]
    #[ must_use ]
    pub fn model( mut self, model : &str ) -> Self
    {
      self.model = Some( model.to_string() );
      self
    }

    /// Add a system message to the conversation
    #[ inline ]
    #[ must_use ]
    pub fn system_message( mut self, content : &str ) -> Self
    {
      #[ cfg( feature = "vision_support" ) ]
      {
        self.messages.push( ChatMessage
        {
          role : MessageRole::System,
          content : content.to_string(),
          images : None,
          #[ cfg( feature = "tool_calling" ) ]
          tool_calls : None,
        });
      }
      #[ cfg( not( feature = "vision_support" ) ) ]
      {
        self.messages.push( Message
        {
          role : "system".to_string(),
          content : content.to_string(),
        });
      }
      self
    }

    /// Add a user message to the conversation
    #[ inline ]
    #[ must_use ]
    pub fn user_message( mut self, content : &str ) -> Self
    {
      #[ cfg( feature = "vision_support" ) ]
      {
        self.messages.push( ChatMessage
        {
          role : MessageRole::User,
          content : content.to_string(),
          images : None,
          #[ cfg( feature = "tool_calling" ) ]
          tool_calls : None,
        });
      }
      #[ cfg( not( feature = "vision_support" ) ) ]
      {
        self.messages.push( Message
        {
          role : "user".to_string(),
          content : content.to_string(),
        });
      }
      self
    }

    /// Add an assistant message to the conversation
    #[ inline ]
    #[ must_use ]
    pub fn assistant_message( mut self, content : &str ) -> Self
    {
      #[ cfg( feature = "vision_support" ) ]
      {
        self.messages.push( ChatMessage
        {
          role : MessageRole::Assistant,
          content : content.to_string(),
          images : None,
          #[ cfg( feature = "tool_calling" ) ]
          tool_calls : None,
        });
      }
      #[ cfg( not( feature = "vision_support" ) ) ]
      {
        self.messages.push( Message
        {
          role : "assistant".to_string(),
          content : content.to_string(),
        });
      }
      self
    }

    /// Enable or disable streaming
    #[ inline ]
    #[ must_use ]
    pub fn streaming( mut self, enable : bool ) -> Self
    {
      self.stream = Some( enable );
      self
    }

    /// Set temperature parameter
    #[ inline ]
    #[ must_use ]
    pub fn temperature( mut self, temp : f64 ) -> Self
    {
      self.set_option( "temperature", serde_json::Value::from( temp ) );
      self
    }

    /// Set `top_p` parameter
    #[ inline ]
    #[ must_use ]
    pub fn top_p( mut self, top_p : f64 ) -> Self
    {
      self.set_option( "top_p", serde_json::Value::from( top_p ) );
      self
    }

    /// Set `num_predict` (max tokens to generate) parameter.
    ///
    /// Fix(issue-ollama-max-tokens-007): Uses `num_predict` (the valid Ollama API field) not
    /// `max_tokens` (an OpenAI concept Ollama does not recognize as an option).
    /// Root cause: `max_tokens` in Ollama options is silently ignored — the model generates
    ///   until EOS with no token limit, causing multi-minute inference in integration tests.
    /// Pitfall: Never use `"max_tokens"` in Ollama options; always use `"num_predict"`.
    #[ inline ]
    #[ must_use ]
    pub fn max_tokens( mut self, max_tokens : u32 ) -> Self
    {
      self.set_option( "num_predict", serde_json::Value::from( max_tokens ) );
      self
    }

    /// Set custom options
    #[ inline ]
    #[ must_use ]
    pub fn options( mut self, options : std::collections::HashMap<  String, serde_json::Value  > ) -> Self
    {
      self.options = Some( serde_json::Value::Object( options.into_iter().collect() ) );
      self
    }

    /// Build the `ChatRequest`
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing or invalid
    #[ inline ]
    pub fn build( self ) -> OllamaResult< ChatRequest >
    {
      let model = self.model.ok_or_else( || format_err!( "Model is required" ) )?;
      if model.is_empty()
      {
        return Err( format_err!( "Model cannot be empty" ) );
      }

      if self.messages.is_empty()
      {
        return Err( format_err!( "At least one message is required" ) );
      }

      // Validate messages
      for message in &self.messages
      {
        if message.content.is_empty()
        {
          return Err( format_err!( "Message content cannot be empty" ) );
        }
      }

      Ok( ChatRequest
      {
        model,
        messages : self.messages,
        stream : self.stream,
        options : self.options,
        #[ cfg( feature = "tool_calling" ) ]
        tools : None,
        #[ cfg( feature = "tool_calling" ) ]
        tool_messages : None,
      })
    }
  }

  #[ cfg( feature = "builder_patterns" ) ]
  impl Default for ChatRequestBuilder
  {
    #[ inline ]
    fn default() -> Self
    {
      Self::new()
    }
  }

  #[ cfg( feature = "builder_patterns" ) ]
  impl GenerateRequestBuilder
  {
    /// Create a new `GenerateRequestBuilder`
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        model : None,
        prompt : None,
        stream : Some( false ), // Default to non-streaming for compatibility
        options : None,
      }
    }
    
    /// Helper method to set an option value
    fn set_option( &mut self, key : &str, value : serde_json::Value )
    {
      if self.options.is_none()
      {
        self.options = Some( serde_json::Value::Object( serde_json::Map::new() ) );
      }
      if let Some( serde_json::Value::Object( ref mut map ) ) = self.options
      {
        map.insert( key.to_string(), value );
      }
    }

    /// Set the model name
    #[ inline ]
    #[ must_use ]
    pub fn model( mut self, model : &str ) -> Self
    {
      self.model = Some( model.to_string() );
      self
    }

    /// Set the prompt text
    #[ inline ]
    #[ must_use ]
    pub fn prompt( mut self, prompt : &str ) -> Self
    {
      self.prompt = Some( prompt.to_string() );
      self
    }

    /// Enable or disable streaming
    #[ inline ]
    #[ must_use ]
    pub fn streaming( mut self, enable : bool ) -> Self
    {
      self.stream = Some( enable );
      self
    }

    /// Set temperature parameter
    #[ inline ]
    #[ must_use ]
    pub fn temperature( mut self, temp : f64 ) -> Self
    {
      self.set_option( "temperature", serde_json::Value::from( temp ) );
      self
    }

    /// Set `num_predict` (max tokens to generate) parameter.
    /// Fix(issue-ollama-max-tokens-007): Uses `num_predict` not `max_tokens` (see ChatRequestBuilder).
    #[ inline ]
    #[ must_use ]
    pub fn max_tokens( mut self, max_tokens : u32 ) -> Self
    {
      self.set_option( "num_predict", serde_json::Value::from( max_tokens ) );
      self
    }

    /// Set stop sequences
    #[ inline ]
    #[ must_use ]
    pub fn stop_sequences( mut self, stops : &[ &str ] ) -> Self
    {
      let stop_values : Vec< serde_json::Value > = stops.iter().map( | s | serde_json::Value::from( *s ) ).collect();
      self.set_option( "stop", serde_json::Value::from( stop_values ) );
      self
    }

    /// Build the `GenerateRequest`
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing or invalid
    #[ inline ]
    pub fn build( self ) -> OllamaResult< GenerateRequest >
    {
      let model = self.model.ok_or_else( || format_err!( "Model is required" ) )?;
      if model.is_empty()
      {
        return Err( format_err!( "Model cannot be empty" ) );
      }

      let prompt = self.prompt.ok_or_else( || format_err!( "Prompt is required" ) )?;
      if prompt.is_empty()
      {
        return Err( format_err!( "Prompt cannot be empty" ) );
      }

      Ok( GenerateRequest
      {
        model,
        prompt,
        stream : self.stream,
        options : self.options,
      })
    }
  }

  #[ cfg( feature = "builder_patterns" ) ]
  impl Default for GenerateRequestBuilder
  {
    #[ inline ]
    fn default() -> Self
    {
      Self::new()
    }
  }

  #[ cfg( all( feature = "builder_patterns", feature = "embeddings" ) ) ]
  impl EmbeddingsRequestBuilder
  {
    /// Create a new `EmbeddingsRequestBuilder`
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        model : None,
        prompt : None,
        options : None,
      }
    }
    
    /// Helper method to set an option value
    fn set_option( &mut self, key : &str, value : serde_json::Value )
    {
      if self.options.is_none()
      {
        self.options = Some( std::collections::HashMap::new() );
      }
      if let Some( ref mut options ) = self.options
      {
        options.insert( key.to_string(), value );
      }
    }

    /// Set the model name
    #[ inline ]
    #[ must_use ]
    pub fn model( mut self, model : &str ) -> Self
    {
      self.model = Some( model.to_string() );
      self
    }

    /// Set the prompt text
    #[ inline ]
    #[ must_use ]
    pub fn prompt( mut self, prompt : &str ) -> Self
    {
      self.prompt = Some( prompt.to_string() );
      self
    }

    /// Set temperature parameter
    #[ inline ]
    #[ must_use ]
    pub fn temperature( mut self, temp : f64 ) -> Self
    {
      self.set_option( "temperature", serde_json::Value::from( temp ) );
      self
    }

    /// Set dimension parameter (hint for embedding dimensions)
    #[ inline ]
    #[ must_use ]
    pub fn dimension( mut self, dim : u32 ) -> Self
    {
      self.set_option( "dimension", serde_json::Value::from( dim ) );
      self
    }

    /// Build the `EmbeddingsRequest`
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing or invalid
    #[ inline ]
    pub fn build( self ) -> OllamaResult< EmbeddingsRequest >
    {
      let model = self.model.ok_or_else( || format_err!( "Model is required" ) )?;
      if model.is_empty()
      {
        return Err( format_err!( "Model cannot be empty" ) );
      }

      let prompt = self.prompt.ok_or_else( || format_err!( "Prompt is required" ) )?;
      if prompt.is_empty()
      {
        return Err( format_err!( "Prompt cannot be empty" ) );
      }

      Ok( EmbeddingsRequest
      {
        model,
        prompt,
        options : self.options,
      })
    }
  }

  #[ cfg( all( feature = "builder_patterns", feature = "embeddings" ) ) ]
  impl Default for EmbeddingsRequestBuilder
  {
    #[ inline ]
    fn default() -> Self
    {
      Self::new()
    }
  }
}

#[ cfg( feature = "enabled" ) ]
crate ::mod_interface!
{
  #[ cfg( feature = "builder_patterns" ) ]
  exposed use
  {
    ChatRequestBuilder,
    GenerateRequestBuilder,
  };
  #[ cfg( all( feature = "builder_patterns", feature = "embeddings" ) ) ]
  exposed use
  {
    EmbeddingsRequestBuilder,
  };
}

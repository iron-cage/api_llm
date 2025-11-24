//! Enhanced conversation builder for chat API.

#[ cfg( feature = "chat" ) ]
use super::super::Client;

/// Enhanced builder for creating chat conversations with fluent API and generation parameters.
///
/// Provides a more ergonomic interface for constructing complex chat requests with
/// generation configuration, conversation state management, and method chaining.
#[ cfg( feature = "chat" ) ]
#[ derive( Debug ) ]
pub struct ConversationBuilder< 'a >
{
    pub( crate ) client : &'a Client,
    pub( crate ) messages : Vec< crate::models::ChatMessage >,
    pub( crate ) model : String,
    pub( crate ) temperature : Option< f32 >,
    pub( crate ) max_tokens : Option< i32 >,
    pub( crate ) top_p : Option< f32 >,
}

#[ cfg( feature = "chat" ) ]
impl ConversationBuilder< '_ >
{
  /// Add a system message to the conversation with validation.
  ///
  /// System messages provide instructions that guide the assistant's behavior.
  /// Only one system message is allowed per conversation.
  ///
  /// # Panics
  ///
  /// Panics if a system message was already added to this conversation.
  #[ inline ]
  #[ must_use ]
  pub fn system( mut self, content : &str ) -> Self
  {
    // Validate that no system message already exists
    assert!( !self.messages.iter().any( | msg | msg.role == "system" ), 
            "Only one system message is allowed per conversation. Use a single system() call." );
    
    assert!( !content.is_empty(), "System message content cannot be empty." );

    self.messages.push( crate::models::ChatMessage
    {
      role : "system".to_string(),
      content : content.to_string(),
    } );
    self
  }

  /// Add a user message to the conversation with validation.
  ///
  /// User messages represent input from the human user.
  ///
  /// # Panics
  ///
  /// Panics if content is empty.
  #[ inline ]
  #[ must_use ]
  pub fn user( mut self, content : &str ) -> Self
  {
    assert!( !content.is_empty(), "User message content cannot be empty." );

    self.messages.push( crate::models::ChatMessage
    {
      role : "user".to_string(),
      content : content.to_string(),
    } );
    self
  }

  /// Add an assistant message to the conversation with validation.
  ///
  /// Assistant messages represent previous responses from the AI assistant,
  /// useful for maintaining conversation context.
  ///
  /// # Panics
  ///
  /// Panics if content is empty.
  #[ inline ]
  #[ must_use ]
  pub fn assistant( mut self, content : &str ) -> Self
  {
    assert!( !content.is_empty(), "Assistant message content cannot be empty." );

    self.messages.push( crate::models::ChatMessage
    {
      role : "assistant".to_string(),
      content : content.to_string(),
    } );
    self
  }

  /// Set the model to use for completion.
  ///
  /// # Arguments
  ///
  /// * `model` - Model identifier (e.g., "gemini-2.5-flash", "gemini-1.5-pro")
  ///
  /// # Panics
  ///
  /// Panics if model name is empty.
  #[ inline ]
  #[ must_use ]
  pub fn model( mut self, model : &str ) -> Self
  {
    assert!( !model.is_empty(), "Model name cannot be empty." );
    self.model = model.to_string();
    self
  }

  /// Set the creativity/randomness of responses (0.0 to 2.0).
  ///
  /// Higher values make responses more creative but less focused.
  /// Lower values make responses more deterministic and focused.
  ///
  /// # Arguments
  ///
  /// * `temperature` - Temperature value between 0.0 and 2.0
  ///
  /// # Panics
  ///
  /// Panics if temperature is not between 0.0 and 2.0.
  #[ inline ]
  #[ must_use ]
  pub fn temperature( mut self, temperature : f32 ) -> Self
  {
    assert!( ( 0.0..=2.0 ).contains( &temperature ), "Temperature must be between 0.0 and 2.0, got : {temperature}" );
    self.temperature = Some( temperature );
    self
  }

  /// Set the maximum number of tokens to generate.
  ///
  /// # Arguments
  ///
  /// * `max_tokens` - Maximum tokens to generate (must be positive)
  ///
  /// # Panics
  ///
  /// Panics if `max_tokens` is not positive.
  #[ inline ]
  #[ must_use ]
  pub fn max_tokens( mut self, max_tokens : i32 ) -> Self
  {
    assert!( max_tokens > 0, "max_tokens must be positive, got : {max_tokens}" );
    self.max_tokens = Some( max_tokens );
    self
  }

  /// Set nucleus sampling parameter (0.0 to 1.0).
  ///
  /// Controls diversity by limiting token selection to the top cumulative probability mass.
  ///
  /// # Arguments
  ///
  /// * `top_p` - Top-p value between 0.0 and 1.0
  ///
  /// # Panics
  ///
  /// Panics if `top_p` is not between 0.0 and 1.0.
  #[ inline ]
  #[ must_use ]
  pub fn top_p( mut self, top_p : f32 ) -> Self
  {
    assert!( ( 0.0..=1.0 ).contains( &top_p ), "top_p must be between 0.0 and 1.0, got : {top_p}" );
    self.top_p = Some( top_p );
    self
  }

  /// Complete the conversation using configured parameters.
  ///
  /// # Errors
  ///
  /// Returns an error if the chat completion fails or if the underlying API call fails.
  #[ inline ]
  pub async fn complete( self ) -> Result< crate::models::ChatCompletionResponse, crate::error::Error >
  {
    let request = crate::models::ChatCompletionRequest
    {
      messages : self.messages,
      model : self.model,
      temperature : self.temperature,
      max_tokens : self.max_tokens,
      top_p : self.top_p,
      ..Default::default()
    };

    self.client.chat().complete( &request ).await
  }

  /// Build the request for use with the `ChatApi.complete_stream` method.
  ///
  /// For streaming functionality, build the request and pass it to the `ChatApi` directly.
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// # async fn example( client : &api_gemini::client::Client ) -> Result< (), Box< dyn std::error::Error > > {
  /// let request = client.chat().conversation()?
  ///   .user( "Hello!" )
  ///   .build_request();
  /// let chat_api = client.chat();
  /// let stream = chat_api.complete_stream( &request ).await?;
  /// # Ok( () )
  /// # }
  /// ```
  #[ must_use ]
  #[ inline ]
  pub fn build_request( self ) -> crate::models::ChatCompletionRequest
  {
    crate ::models::ChatCompletionRequest
    {
      messages : self.messages,
      model : self.model,
      temperature : self.temperature,
      max_tokens : self.max_tokens,
      top_p : self.top_p,
      ..Default::default()
    }
  }

  /// Get a summary of the current conversation state.
  ///
  /// Useful for debugging and understanding the conversation structure.
  #[ must_use ]
  #[ inline ]
  pub fn summary( &self ) -> ConversationSummary
  {
    let message_counts = self.messages.iter().fold( 
      ( 0, 0, 0 ), 
      | ( mut system, mut user, mut assistant ), msg | {
        match msg.role.as_str()
        {
          "system" => system += 1,
          "user" => user += 1,
          "assistant" => assistant += 1,
          _ => {},
        }
        ( system, user, assistant )
      }
    );

    ConversationSummary
    {
      total_messages : self.messages.len(),
      system_messages : message_counts.0,
      user_messages : message_counts.1,
      assistant_messages : message_counts.2,
      model : self.model.clone(),
      has_temperature : self.temperature.is_some(),
      has_max_tokens : self.max_tokens.is_some(),
      has_top_p : self.top_p.is_some(),
    }
  }
}

/// Summary of conversation state for debugging and introspection.
#[ cfg( feature = "chat" ) ]
#[ derive( Debug, Clone ) ]
pub struct ConversationSummary
{
  /// Total number of messages in the conversation
  pub total_messages : usize,
  /// Number of system messages (should be 0 or 1)
  pub system_messages : usize,
  /// Number of user messages
  pub user_messages : usize,
  /// Number of assistant messages
  pub assistant_messages : usize,
  /// Model configured for completion
  pub model : String,
  /// Whether temperature is configured
  pub has_temperature : bool,
  /// Whether `max_tokens` is configured
  pub has_max_tokens : bool,
  /// Whether `top_p` is configured
  pub has_top_p : bool,
}

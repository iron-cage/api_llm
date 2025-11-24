//! Chat completion types for the Gemini API.

use serde::{ Deserialize, Serialize };

/// Request for chat completion functionality.
#[ cfg( feature = "chat" ) ]
#[ derive( Debug, Clone, Serialize, Deserialize, Default ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct ChatCompletionRequest
{
  /// List of messages comprising the conversation so far.
  pub messages : Vec< ChatMessage >,

  /// ID of the model to use.
  pub model : String,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Maximum number of tokens to generate.
  pub max_tokens : Option< i32 >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Sampling temperature between 0 and 1.
  pub temperature : Option< f32 >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Nucleus sampling parameter.
  pub top_p : Option< f32 >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Frequency penalty between -2.0 and 2.0.
  pub frequency_penalty : Option< f32 >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Presence penalty between -2.0 and 2.0.
  pub presence_penalty : Option< f32 >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Whether to stream back partial progress.
  pub stream : Option< bool >,
}

#[ cfg( feature = "chat" ) ]
/// Builder for creating `ChatCompletionRequest` with fluent API and validation.
///
/// Provides a more ergonomic way to construct chat completion requests with
/// parameter validation and method chaining for complex configurations.
#[ derive( Debug, Clone ) ]
pub struct ChatCompletionRequestBuilder
{
  messages : Vec< ChatMessage >,
  model : String,
  max_tokens : Option< i32 >,
  temperature : Option< f32 >,
  top_p : Option< f32 >,
  frequency_penalty : Option< f32 >,
  presence_penalty : Option< f32 >,
  stream : Option< bool >,
}

#[ cfg( feature = "chat" ) ]
impl ChatCompletionRequestBuilder
{
  /// Create a new `ChatCompletionRequestBuilder` with default model.
  #[ must_use ]
  #[ inline ]
  pub fn new() -> Self
  {
    Self
    {
      messages : Vec::new(),
      model : "gemini-2.5-flash".to_string(),
      max_tokens : None,
      temperature : None,
      top_p : None,
      frequency_penalty : None,
      presence_penalty : None,
      stream : None,
    }
  }

  /// Create a new `ChatCompletionRequestBuilder` with specific model.
  #[ must_use ]
  #[ inline ]
  pub fn with_model( model : &str ) -> Self
  {
    let mut builder = Self::new();
    builder.model = model.to_string();
    builder
  }

  /// Add a system message to the request.
  ///
  /// # Panics
  ///
  /// Panics if a system message already exists or if content is empty.
  #[ must_use ]
  #[ inline ]
  pub fn system( mut self, content : &str ) -> Self
  {
    assert!( !self.messages.iter().any( | msg | msg.role == "system" ), "Only one system message is allowed per request." );
    assert!( !content.is_empty(), "System message content cannot be empty." );

    self.messages.push( ChatMessage
    {
      role : "system".to_string(),
      content : content.to_string(),
    } );
    self
  }

  /// Add a user message to the request.
  ///
  /// # Panics
  ///
  /// Panics if content is empty.
  #[ must_use ]
  #[ inline ]
  pub fn user( mut self, content : &str ) -> Self
  {
    assert!( !content.is_empty(), "User message content cannot be empty." );

    self.messages.push( ChatMessage
    {
      role : "user".to_string(),
      content : content.to_string(),
    } );
    self
  }

  /// Add an assistant message to the request.
  ///
  /// # Panics
  ///
  /// Panics if content is empty.
  #[ must_use ]
  #[ inline ]
  pub fn assistant( mut self, content : &str ) -> Self
  {
    assert!( !content.is_empty(), "Assistant message content cannot be empty." );

    self.messages.push( ChatMessage
    {
      role : "assistant".to_string(),
      content : content.to_string(),
    } );
    self
  }

  /// Set the model for the request.
  ///
  /// # Panics
  ///
  /// Panics if model name is empty.
  #[ must_use ]
  #[ inline ]
  pub fn model( mut self, model : &str ) -> Self
  {
    assert!( !model.is_empty(), "Model name cannot be empty." );
    self.model = model.to_string();
    self
  }

  /// Set the maximum tokens to generate.
  ///
  /// # Panics
  ///
  /// Panics if `max_tokens` is not positive.
  #[ must_use ]
  #[ inline ]
  pub fn max_tokens( mut self, max_tokens : i32 ) -> Self
  {
    assert!( max_tokens > 0, "max_tokens must be positive, got : {max_tokens}" );
    self.max_tokens = Some( max_tokens );
    self
  }

  /// Set the temperature for response creativity.
  ///
  /// # Panics
  ///
  /// Panics if temperature is not between 0.0 and 2.0.
  #[ must_use ]
  #[ inline ]
  pub fn temperature( mut self, temperature : f32 ) -> Self
  {
    assert!( ( 0.0..=2.0 ).contains( &temperature ), "Temperature must be between 0.0 and 2.0, got : {temperature}" );
    self.temperature = Some( temperature );
    self
  }

  /// Set the nucleus sampling parameter.
  ///
  /// # Panics
  ///
  /// Panics if `top_p` is not between 0.0 and 1.0.
  #[ must_use ]
  #[ inline ]
  pub fn top_p( mut self, top_p : f32 ) -> Self
  {
    assert!( ( 0.0..=1.0 ).contains( &top_p ), "top_p must be between 0.0 and 1.0, got : {top_p}" );
    self.top_p = Some( top_p );
    self
  }

  /// Set the frequency penalty.
  ///
  /// # Panics
  ///
  /// Panics if `frequency_penalty` is not between -2.0 and 2.0.
  #[ must_use ]
  #[ inline ]
  pub fn frequency_penalty( mut self, frequency_penalty : f32 ) -> Self
  {
    assert!( ( -2.0..=2.0 ).contains( &frequency_penalty ), "frequency_penalty must be between -2.0 and 2.0, got : {frequency_penalty}" );
    self.frequency_penalty = Some( frequency_penalty );
    self
  }

  /// Set the presence penalty.
  ///
  /// # Panics
  ///
  /// Panics if `presence_penalty` is not between -2.0 and 2.0.
  #[ must_use ]
  #[ inline ]
  pub fn presence_penalty( mut self, presence_penalty : f32 ) -> Self
  {
    assert!( ( -2.0..=2.0 ).contains( &presence_penalty ), "presence_penalty must be between -2.0 and 2.0, got : {presence_penalty}" );
    self.presence_penalty = Some( presence_penalty );
    self
  }

  /// Enable streaming responses.
  #[ must_use ]
  #[ inline ]
  pub fn stream( mut self ) -> Self
  {
    self.stream = Some( true );
    self
  }

  /// Build the final `ChatCompletionRequest`.
  ///
  /// # Panics
  ///
  /// Panics if no messages have been added or if there are no user messages.
  #[ must_use ]
  #[ inline ]
  pub fn build( self ) -> ChatCompletionRequest
  {
    assert!( !self.messages.is_empty(), "At least one message is required to build a ChatCompletionRequest." );
    assert!( self.messages.iter().any( | msg | msg.role == "user" ), "At least one user message is required to build a ChatCompletionRequest." );

    ChatCompletionRequest
    {
      messages : self.messages,
      model : self.model,
      max_tokens : self.max_tokens,
      temperature : self.temperature,
      top_p : self.top_p,
      frequency_penalty : self.frequency_penalty,
      presence_penalty : self.presence_penalty,
      stream : self.stream,
    }
  }
}

#[ cfg( feature = "chat" ) ]
impl Default for ChatCompletionRequestBuilder
{
  #[ inline ]
  fn default() -> Self
  {
    Self::new()
  }
}

#[ cfg( feature = "chat" ) ]
/// A single message in a chat completion request.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct ChatMessage
{
  /// Role of the message author.
  pub role : String,

  /// Content of the message.
  pub content : String,
}

#[ cfg( feature = "chat" ) ]
/// Response from chat completion API.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct ChatCompletionResponse
{
  /// Unique identifier for the chat completion.
  pub id : String,

  /// Object type, always "chat.completion".
  pub object : String,

  /// Unix timestamp of when the completion was created.
  pub created : i64,

  /// Model used for completion.
  pub model : String,

  /// List of completion choices.
  pub choices : Vec< ChatChoice >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Usage statistics for the completion request.
  pub usage : Option< ChatUsage >,
}

#[ cfg( feature = "chat" ) ]
/// A single choice in a chat completion response.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct ChatChoice
{
  /// Index of this choice.
  pub index : i32,

  /// Generated message.
  pub message : ChatMessage,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Reason why completion finished.
  pub finish_reason : Option< String >,
}

#[ cfg( feature = "chat" ) ]
/// Usage statistics for chat completion.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct ChatUsage
{
  /// Tokens in the prompt.
  pub prompt_tokens : i32,

  /// Tokens in the completion.
  pub completion_tokens : i32,

  /// Total tokens used.
  pub total_tokens : i32,
}

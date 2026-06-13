//! Request Templates for Common Use Cases
//!
//! Pre-configured request templates for common AI tasks.

mod private
{
  use super::super::{ CreateMessageRequest, Message, Role, Content };

  /// Request template for common use cases
  #[ derive( Debug, Clone ) ]
  pub struct RequestTemplate
  {
    model : String,
    max_tokens : u32,
    temperature : Option< f32 >,
    system_prompt : Option< String >,
  }

  impl RequestTemplate
  {
    /// Create a chat conversation template
    ///
    /// Optimized for natural, conversational interactions
    #[ must_use ]
    pub fn chat( model : impl Into< String > ) -> Self
    {
      Self
      {
        model : model.into(),
        max_tokens : 4096,
        temperature : Some( 1.0 ),
        system_prompt : Some( "You are a helpful, friendly, and knowledgeable AI assistant.".to_string() ),
      }
    }

    /// Create a code generation template
    ///
    /// Optimized for generating clean, well-documented code
    #[ must_use ]
    pub fn code_generation( model : impl Into< String > ) -> Self
    {
      Self
      {
        model : model.into(),
        max_tokens : 4096,
        temperature : Some( 0.2 ),
        system_prompt : Some( "You are an expert software engineer. Generate clean, well-documented, and efficient code.".to_string() ),
      }
    }

    /// Create a creative writing template
    ///
    /// Optimized for creative, imaginative content
    #[ must_use ]
    pub fn creative_writing( model : impl Into< String > ) -> Self
    {
      Self
      {
        model : model.into(),
        max_tokens : 4096,
        temperature : Some( 1.2 ),
        system_prompt : Some( "You are a creative writer with a vivid imagination. Write engaging, original content.".to_string() ),
      }
    }

    /// Create a factual Q&A template
    ///
    /// Optimized for accurate, factual responses
    #[ must_use ]
    pub fn factual_qa( model : impl Into< String > ) -> Self
    {
      Self
      {
        model : model.into(),
        max_tokens : 2048,
        temperature : Some( 0.3 ),
        system_prompt : Some( "You are a knowledgeable assistant focused on providing accurate, factual information. Be precise and cite sources when appropriate.".to_string() ),
      }
    }

    /// Create a summarization template
    ///
    /// Optimized for concise summaries
    #[ must_use ]
    pub fn summarization( model : impl Into< String > ) -> Self
    {
      Self
      {
        model : model.into(),
        max_tokens : 1024,
        temperature : Some( 0.5 ),
        system_prompt : Some( "You are an expert at creating clear, concise summaries. Extract key points and main ideas.".to_string() ),
      }
    }

    /// Set custom prompt
    #[ must_use ]
    pub fn with_prompt( mut self, prompt : impl Into< String > ) -> Self
    {
      self.system_prompt = Some( prompt.into() );
      self
    }

    /// Set custom temperature
    #[ must_use ]
    pub fn with_temperature( mut self, temperature : f32 ) -> Self
    {
      self.temperature = Some( temperature );
      self
    }

    /// Set custom max tokens
    #[ must_use ]
    pub fn with_max_tokens( mut self, max_tokens : u32 ) -> Self
    {
      self.max_tokens = max_tokens;
      self
    }

    /// Build a `CreateMessageRequest` with user message
    #[ must_use ]
    pub fn build( self, user_message : impl Into< String > ) -> CreateMessageRequest
    {
      CreateMessageRequest
      {
        model : self.model,
        max_tokens : self.max_tokens,
        messages : vec![ Message
        {
          role : Role::User,
          content : vec![ Content::Text
          {
            r#type : "text".to_string(),
            text : user_message.into(),
          } ],
          cache_control : None,
        } ],
        system : self.system_prompt.map( | text | vec![ crate::client::SystemContent
        {
          r#type : "text".to_string(),
          text,
          cache_control : None,
        } ] ),
        temperature : self.temperature,
        stream : None,
        #[ cfg( feature = "tools" ) ]
        tools : None,
        #[ cfg( feature = "tools" ) ]
        tool_choice : None,
      }
    }
  }

  #[ cfg( test ) ]
  mod tests
  {
    use super::*;

    #[ test ]
    fn test_chat_template()
    {
      let template = RequestTemplate::chat( "claude-sonnet-4-6" );
      let request = template.build( "Hello!" );

      assert_eq!( request.model, "claude-sonnet-4-6" );
      assert_eq!( request.max_tokens, 4096 );
      assert_eq!( request.temperature, Some( 1.0 ) );
      assert!( request.system.is_some() );
    }

    #[ test ]
    fn test_code_generation_template()
    {
      let template = RequestTemplate::code_generation( "claude-sonnet-4-6" );
      let request = template.build( "Write a function" );

      assert_eq!( request.temperature, Some( 0.2 ) );
    }

    #[ test ]
    fn test_creative_writing_template()
    {
      let template = RequestTemplate::creative_writing( "claude-sonnet-4-6" );
      let request = template.build( "Write a story" );

      assert_eq!( request.temperature, Some( 1.2 ) );
    }

    #[ test ]
    fn test_factual_qa_template()
    {
      let template = RequestTemplate::factual_qa( "claude-sonnet-4-6" );
      let request = template.build( "What is the capital of France?" );

      assert_eq!( request.temperature, Some( 0.3 ) );
      assert_eq!( request.max_tokens, 2048 );
    }

    #[ test ]
    fn test_summarization_template()
    {
      let template = RequestTemplate::summarization( "claude-sonnet-4-6" );
      let request = template.build( "Summarize this text" );

      assert_eq!( request.temperature, Some( 0.5 ) );
      assert_eq!( request.max_tokens, 1024 );
    }

    #[ test ]
    fn test_with_prompt()
    {
      let template = RequestTemplate::chat( "claude-sonnet-4-6" )
        .with_prompt( "Custom system prompt" );
      let request = template.build( "Hello!" );

      assert!( request.system.is_some() );
      let system_text = &request.system.unwrap()[ 0 ].text;
      assert_eq!( system_text, "Custom system prompt" );
    }

    #[ test ]
    fn test_with_temperature()
    {
      let template = RequestTemplate::chat( "claude-sonnet-4-6" )
        .with_temperature( 0.7 );
      let request = template.build( "Hello!" );

      assert_eq!( request.temperature, Some( 0.7 ) );
    }

    #[ test ]
    fn test_with_max_tokens()
    {
      let template = RequestTemplate::chat( "claude-sonnet-4-6" )
        .with_max_tokens( 2000 );
      let request = template.build( "Hello!" );

      assert_eq!( request.max_tokens, 2000 );
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    RequestTemplate,
  };
}

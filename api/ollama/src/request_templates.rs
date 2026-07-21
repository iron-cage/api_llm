//! Request Templates for Common Use Cases
//!
//! Pre-configured request templates for common AI tasks.

#[ cfg( feature = "enabled" ) ]
mod private
{
  use super::super::{ ChatRequest, messages::ChatMessage };
  #[ cfg( not( feature = "vision_support" ) ) ]
  use super::super::messages::Message;

  /// Request template for common use cases
  #[ derive( Debug, Clone ) ]
  pub struct RequestTemplate
  {
    model : String,
    system_prompt : Option< String >,
    temperature : Option< f32 >,
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
        system_prompt : Some( "You are a helpful, friendly, and knowledgeable AI assistant.".to_string() ),
        temperature : Some( 1.0 ),
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
        system_prompt : Some( "You are an expert software engineer. Generate clean, well-documented, and efficient code.".to_string() ),
        temperature : Some( 0.2 ),
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
        system_prompt : Some( "You are a creative writer with a vivid imagination. Write engaging, original content.".to_string() ),
        temperature : Some( 1.2 ),
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
        system_prompt : Some( "You are a knowledgeable assistant focused on providing accurate, factual information. Be precise and cite sources when appropriate.".to_string() ),
        temperature : Some( 0.3 ),
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
        system_prompt : Some( "You are an expert at creating clear, concise summaries. Extract key points and main ideas.".to_string() ),
        temperature : Some( 0.5 ),
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

    /// Build a ChatRequest with user message
    #[ must_use ]
    pub fn build( self, user_message : impl Into< String > ) -> ChatRequest
    {
      let mut messages = Vec::new();

      // Add system message if present
      if let Some( system_content ) = self.system_prompt
      {
        #[ cfg( feature = "vision_support" ) ]
        messages.push( ChatMessage
        {
          role : super::super::messages::MessageRole::System,
          content : system_content,
          images : None,
          #[ cfg( feature = "tool_calling" ) ]
          tool_calls : None,
        } );

        #[ cfg( not( feature = "vision_support" ) ) ]
        messages.push( Message
        {
          role : "system".to_string(),
          content : system_content,
        } );
      }

      // Add user message
      #[ cfg( feature = "vision_support" ) ]
      messages.push( ChatMessage
      {
        role : super::super::messages::MessageRole::User,
        content : user_message.into(),
        images : None,
        #[ cfg( feature = "tool_calling" ) ]
        tool_calls : None,
      } );

      #[ cfg( not( feature = "vision_support" ) ) ]
      messages.push( Message
      {
        role : "user".to_string(),
        content : user_message.into(),
      } );

      let mut options = serde_json::json!( {} );
      if let Some( temp ) = self.temperature
      {
        options[ "temperature" ] = serde_json::json!( temp );
      }

      ChatRequest
      {
        model : self.model,
        messages,
        stream : None,
        think : None,
        options : Some( options ),
        #[ cfg( feature = "tool_calling" ) ]
        tools : None,
        #[ cfg( feature = "tool_calling" ) ]
        tool_messages : None,
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
      let template = RequestTemplate::chat( "llama3.2" );
      let request = template.build( "Hello!" );

      assert_eq!( request.model, "llama3.2" );
      assert_eq!( request.messages.len(), 2 ); // system + user
    }

    #[ test ]
    fn test_code_generation_template()
    {
      let template = RequestTemplate::code_generation( "codellama" );
      let request = template.build( "Write a function" );

      assert!( request.options.is_some() );
    }

    #[ test ]
    fn test_creative_writing_template()
    {
      let template = RequestTemplate::creative_writing( "llama3.2" );
      let request = template.build( "Write a story" );

      assert_eq!( request.model, "llama3.2" );
    }

    #[ test ]
    fn test_factual_qa_template()
    {
      let template = RequestTemplate::factual_qa( "llama3.2" );
      let request = template.build( "What is the capital of France?" );

      assert_eq!( request.model, "llama3.2" );
    }

    #[ test ]
    fn test_summarization_template()
    {
      let template = RequestTemplate::summarization( "llama3.2" );
      let request = template.build( "Summarize this text" );

      assert_eq!( request.model, "llama3.2" );
    }

    #[ test ]
    fn test_with_prompt()
    {
      let template = RequestTemplate::chat( "llama3.2" )
        .with_prompt( "Custom system prompt" );
      let request = template.build( "Hello!" );

      #[ cfg( feature = "vision_support" ) ]
      assert_eq!( request.messages[ 0 ].content, "Custom system prompt" );
      #[ cfg( not( feature = "vision_support" ) ) ]
      assert_eq!( request.messages[ 0 ].content, "Custom system prompt" );
    }

    #[ test ]
    fn test_with_temperature()
    {
      let template = RequestTemplate::chat( "llama3.2" )
        .with_temperature( 0.7 );
      let request = template.build( "Hello!" );

      assert!( request.options.is_some() );
    }
  }
}

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{
  exposed use
  {
    RequestTemplate,
  };
}

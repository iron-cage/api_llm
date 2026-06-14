//! Chat completion types for Ollama API.
//!
//! Provides request and response structures for the chat completion endpoint,
//! including support for vision models and tool calling.

#[ cfg( feature = "enabled" ) ]
mod private
{
  use serde::{ Serialize, Deserialize };
  use core::hash::{ Hash, Hasher };
  use crate::messages::{ ChatMessage, ToolDefinition, ToolMessage };
  #[ cfg( not( feature = "vision_support" ) ) ]
  use crate::messages::Message;

  /// Chat completion request
  #[ derive( Debug, Clone, Serialize ) ]
  pub struct ChatRequest
  {
    /// Model name to use for completion
    pub model : String,
    /// Messages in the conversation
    #[ cfg( feature = "vision_support" ) ]
    pub messages : Vec< ChatMessage >,
    /// Messages in the conversation (non-vision)
    #[ cfg( not( feature = "vision_support" ) ) ]
    pub messages : Vec< Message >,
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    /// Whether to stream the response
    pub stream : Option< bool >,
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    /// Additional model parameters
    pub options : Option< serde_json::Value >,
    /// Available tools for function calling
    #[ cfg( feature = "tool_calling" ) ]
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub tools : Option< Vec< ToolDefinition > >,
    /// Tool response messages
    #[ cfg( feature = "tool_calling" ) ]
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub tool_messages : Option< Vec< ToolMessage > >,
  }

  #[ cfg( feature = "request_caching" ) ]
  impl Hash for ChatRequest
  {
    #[ inline ]
    fn hash< H : Hasher >( &self, state : &mut H )
    {
      self.model.hash( state );
      self.messages.hash( state );
      self.stream.hash( state );
      if let Some( ref options ) = self.options
      {
        options.to_string().hash( state );
      }
      #[ cfg( feature = "tool_calling" ) ]
      {
        self.tools.hash( state );
        self.tool_messages.hash( state );
      }
    }
  }

  /// Chat completion response
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ChatResponse
  {
    /// Generated message from the model
    // Fix(BUG-013): Added #[serde(default)] to allow streaming "done" chunks with no message field.
    // Root cause: Ollama's final streaming chunk omits `message`; without default, serde fails.
    // Pitfall: ChatMessage derives Default (role=User, content=""), so absent field → empty message.
    #[ cfg( feature = "vision_support" ) ]
    #[ serde( default ) ]
    pub message : ChatMessage,
    #[ serde( default ) ]
    /// Generated message from the model (non-vision)
    #[ cfg( not( feature = "vision_support" ) ) ]
    pub message : Option< Message >,
    #[ serde( default ) ]
    /// Whether generation is complete
    pub done : bool,
    #[ serde( default ) ]
    /// Reason for completion (e.g., "stop")
    pub done_reason : Option< String >,
    #[ serde( default ) ]
    /// Model name used for generation
    pub model : Option< String >,
    #[ serde( default ) ]
    /// Timestamp of response creation
    pub created_at : Option< String >,
    #[ serde( default ) ]
    /// Total time taken for generation in nanoseconds
    pub total_duration : Option< u64 >,
    #[ serde( default ) ]
    /// Time taken to load the model in nanoseconds
    pub load_duration : Option< u64 >,
    #[ serde( default ) ]
    /// Number of tokens in the prompt
    pub prompt_eval_count : Option< u32 >,
    #[ serde( default ) ]
    /// Time taken for prompt evaluation in nanoseconds
    pub prompt_eval_duration : Option< u64 >,
    #[ serde( default ) ]
    /// Number of tokens generated
    pub eval_count : Option< u32 >,
    #[ serde( default ) ]
    /// Time taken for evaluation in nanoseconds
    pub eval_duration : Option< u64 >,
  }
}

#[ cfg( feature = "enabled" ) ]
crate ::mod_interface!
{
  exposed use
  {
    ChatRequest,
    ChatResponse,
  };
}

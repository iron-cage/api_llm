//! Message types for chat conversations.
//!
//! Provides message structures for Ollama chat API, including support for
//! vision-enabled models and tool calling functionality.

#[ cfg( feature = "enabled" ) ]
mod private
{
  use serde::{Serialize, Deserialize, Serializer};
  use core::hash::{ Hash, Hasher };
  use serde::ser::SerializeStruct;
  use serde_json::json;

  /// Message in chat conversation
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  #[ cfg_attr( feature = "request_caching", derive( Hash ) ) ]
  pub struct Message
  {
    /// Role of the message sender (e.g., "user", "assistant")
    pub role : String,
    /// Content of the message
    pub content : String,
  }

  /// Message roles for vision-enabled chat
  #[ cfg( feature = "vision_support" ) ]
  #[ derive( Debug, Clone, Serialize, Deserialize, Default, PartialEq ) ]
  #[ cfg_attr( feature = "request_caching", derive( Hash ) ) ]
  #[ serde( rename_all = "lowercase" ) ]
  pub enum MessageRole
  {
    /// User message
    #[ default ]
    User,
    /// Assistant message
    Assistant,
    /// System message
    System,
    /// Tool response message
    #[ cfg( feature = "tool_calling" ) ]
    Tool,
  }

  /// Enhanced message with vision support
  #[ cfg( feature = "vision_support" ) ]
  #[ derive( Debug, Clone, Serialize, Deserialize, Default ) ]
  #[ cfg_attr( feature = "request_caching", derive( Hash ) ) ]
  pub struct ChatMessage
  {
    /// Role of the message sender
    pub role : MessageRole,
    /// Content of the message
    pub content : String,
    /// Optional base64-encoded images for vision models
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub images : Option< Vec< String > >,
    /// Optional tool calls made by the assistant
    #[ cfg( feature = "tool_calling" ) ]
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub tool_calls : Option< Vec< ToolCall > >,
  }

  /// Tool definition for function calling
  #[ cfg( feature = "tool_calling" ) ]
  #[ derive( Debug, Clone, Deserialize ) ]
  pub struct ToolDefinition
  {
    /// Name of the tool/function
    pub name : String,
    /// Description of what the tool does
    pub description : String,
    /// JSON schema defining the function parameters
    pub parameters : serde_json::Value,
  }

  #[ cfg( feature = "tool_calling" ) ]
  impl Serialize for ToolDefinition
  {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer
    {
      let mut st = serializer.serialize_struct("ToolDefinition", 2)?;
      st.serialize_field("type", "function")?;
      st.serialize_field("function", &json!({
        "name": self.name.clone(),
        "description": self.description.clone(),
        "parameters": self.parameters.clone()
      }))?;
      st.end()
    }
  }

  #[ cfg( all( feature = "tool_calling", feature = "request_caching" ) ) ]
  impl Hash for ToolDefinition
  {
    #[ inline ]
    fn hash< H : Hasher >( &self, state : &mut H )
    {
      self.name.hash( state );
      self.description.hash( state );
      self.parameters.to_string().hash( state );
    }
  }

  /// Tool call information
  #[ cfg( feature = "tool_calling" ) ]
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ToolCall
  {
    /// Unique identifier for this tool call
    pub id : String,
    /// Function call details
    pub function : serde_json::Value,
  }

  #[ cfg( all( feature = "tool_calling", feature = "request_caching" ) ) ]
  impl Hash for ToolCall
  {
    #[ inline ]
    fn hash< H : Hasher >( &self, state : &mut H )
    {
      self.id.hash( state );
      self.function.to_string().hash( state );
    }
  }

  /// Tool message for function responses
  #[ cfg( feature = "tool_calling" ) ]
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  #[ cfg_attr( feature = "request_caching", derive( Hash ) ) ]
  pub struct ToolMessage
  {
    /// Role should be "tool"
    pub role : MessageRole,
    /// Result content from tool execution
    pub content : String,
    /// ID linking this response to the tool call
    pub tool_call_id : String,
  }
}

#[ cfg( feature = "enabled" ) ]
crate ::mod_interface!
{
  exposed use
  {
    Message,
  };

  #[ cfg( feature = "vision_support" ) ]
  exposed use
  {
    MessageRole,
    ChatMessage,
  };

  #[ cfg( feature = "tool_calling" ) ]
  exposed use
  {
    ToolDefinition,
    ToolCall,
    ToolMessage,
  };
}

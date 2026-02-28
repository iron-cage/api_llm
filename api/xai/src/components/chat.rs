//! Chat completion wire types for the `api_xai` crate.
//!
//! Re-exported from `api_openai_compatible` to eliminate wire-type duplication.
//! Streaming types (`ChatCompletionChunk`, `ChunkChoice`, `Delta`) are available
//! when the `streaming` feature is active.

mod private
{
  pub use api_openai_compatible::
  {
    Message,
    ToolCall,
    FunctionCall,
    ChatCompletionRequest,
    Tool,
    Function,
    ChatCompletionResponse,
    Choice,
  };

  #[ cfg( feature = "streaming" ) ]
  pub use api_openai_compatible::{ ChatCompletionChunk, ChunkChoice, Delta };
}

crate::mod_interface!
{
  exposed use
  {
    Message,
    ToolCall,
    FunctionCall,
    ChatCompletionRequest,
    Tool,
    Function,
    ChatCompletionResponse,
    Choice,
  };

  #[ cfg( feature = "streaming" ) ]
  exposed use
  {
    ChatCompletionChunk,
    ChunkChoice,
    Delta,
  };
}

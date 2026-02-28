//! Wire types for `OpenAI` Server-Sent Events streaming completions.
//!
//! These types represent the incremental chunks delivered over a streaming
//! chat completion response. Each chunk carries partial content via a `Delta`.

mod private
{
  use serde::{ Serialize, Deserialize };
  use crate::{ Role, ToolCall };

  /// A single Server-Sent Events chunk from a streaming completion.
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ChatCompletionChunk
  {
    /// Opaque completion identifier (same across all chunks for one request).
    pub id : String,

    /// Always `"chat.completion.chunk"`.
    pub object : String,

    /// Unix timestamp of this chunk.
    pub created : u64,

    /// Model that generated this chunk.
    pub model : String,

    /// One or more delta choices in this chunk.
    pub choices : Vec< ChunkChoice >,
  }

  /// One delta choice within a streaming chunk.
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ChunkChoice
  {
    /// Zero-based index.
    pub index : u32,

    /// Incremental content update.
    pub delta : Delta,

    /// Set only in the final chunk; `None` for all intermediate chunks.
    pub finish_reason : Option< String >,
  }

  /// Partial message update delivered in a streaming chunk.
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq, Default ) ]
  pub struct Delta
  {
    /// Role — present only in the first chunk of a response.
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub role : Option< Role >,

    /// Partial text to append to the running response.
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub content : Option< String >,

    /// Partial tool calls (for streaming function calling).
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub tool_calls : Option< Vec< ToolCall > >,
  }
}

crate::mod_interface!
{
  exposed use
  {
    ChatCompletionChunk,
    ChunkChoice,
    Delta,
  };
}

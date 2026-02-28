//! OpenAI-compatible wire-type components.
//!
//! Re-exports all wire types for chat completions and optional streaming.

mod private
{
}

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{
  /// Chat completion request, response, and message types.
  layer chat;

  /// Server-Sent Events streaming types.
  #[ cfg( feature = "streaming" ) ]
  layer streaming;
}

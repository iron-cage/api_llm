//! Diagnostics module for debugging and development tools
//!
//! This module provides diagnostic utilities to help developers debug API requests
//! and responses. It defines the `AsCurl` trait to enable generating executable
//! curl commands from request objects.
//!
//! # Features
//!
//! - **curl Command Generation**: Convert API requests to executable curl commands
//! - **Pretty-formatted Output**: Readable curl commands with proper JSON formatting
//! - **Authorization Header Support**: Uses Bearer token authentication for `HuggingFace` Router API
//! - **Error Resilience**: Robust handling of serialization failures
//!
//! # Usage Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use api_huggingface::
//! {
//!   components::inference_shared::{ ChatCompletionRequest, ChatMessage },
//!   diagnostics::AsCurl,
//! };
//!
//! let request = ChatCompletionRequest
//! {
//!   messages : vec![
//!     ChatMessage
//!     {
//!       role : "user".to_string(),
//!       content : "What is 2+2?".to_string(),
//!       tool_calls : None,
//!       tool_call_id : None,
//!     }
//!   ],
//!   model : "moonshotai/Kimi-K2-Instruct-0905:groq".to_string(),
//!   temperature : Some( 0.7 ),
//!   max_tokens : Some( 100 ),
//!   top_p : Some( 0.9 ),
//!   stream : Some( false ),
//!   tools : None,
//!   tool_choice : None,
//! };
//!
//! let curl_command = request.as_curl();
//! println!( "Debug curl command:\n{}", curl_command );
//! ```
//!
//! ## Pretty-formatted Output
//!
//! ```no_run
//! use api_huggingface::
//! {
//!   components::inference_shared::{ ChatCompletionRequest, ChatMessage },
//!   diagnostics::{ AsCurl, CurlOptions },
//! };
//!
//! let request = ChatCompletionRequest
//! {
//!   messages : vec![ ChatMessage
//!   {
//!     role : "user".to_string(),
//!     content : "Hello".to_string(),
//!     tool_calls : None,
//!     tool_call_id : None,
//!   } ],
//!   model : "moonshotai/Kimi-K2-Instruct-0905:groq".to_string(),
//!   temperature : Some( 0.7 ),
//!   max_tokens : Some( 100 ),
//!   top_p : Some( 0.9 ),
//!   stream : Some( false ),
//!   tools : None,
//!   tool_choice : None,
//! };
//!
//! let curl_command = request.as_curl_with_options( &CurlOptions::pretty() );
//! println!( "Readable curl command:\n{}", curl_command );
//! ```
//!
//! ## Adding API Key
//!
//! The generated curl commands include a placeholder for the API key:
//!
//! ```bash
//! curl -X POST "https://router.huggingface.co/v1/chat/completions" \
//!   -H "Authorization : Bearer YOUR_API_KEY_HERE" \
//!   -H "Content-Type : application/json" \
//!   -d '{...}'
//! ```

mod private
{

/// Trait for converting API request objects into executable curl commands
///
/// This trait allows you to convert API request objects into executable curl
/// commands for debugging purposes.
pub trait AsCurl
{
  /// Convert the request object to an executable curl command string
  ///
  /// This method generates a basic curl command with compact JSON formatting.
  /// For more control over the output format, use `as_curl_with_options`.
  fn as_curl( &self ) -> String;

  /// Convert the request object to a curl command with custom formatting options
  ///
  /// # Arguments
  ///
  /// * `options` - Configuration for curl command generation
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use api_huggingface::
  /// {
  ///   components::inference_shared::{ ChatCompletionRequest, ChatMessage },
  ///   diagnostics::{ AsCurl, CurlOptions },
  /// };
  ///
  /// let request = ChatCompletionRequest
  /// {
  ///   messages : vec![ ChatMessage
  ///   {
  ///     role : "user".to_string(),
  ///     content : "Hello".to_string(),
  ///     tool_calls : None,
  ///     tool_call_id : None,
  ///   } ],
  ///   model : "moonshotai/Kimi-K2-Instruct-0905:groq".to_string(),
  ///   temperature : Some( 0.7 ),
  ///   max_tokens : Some( 100 ),
  ///   top_p : Some( 0.9 ),
  ///   stream : Some( false ),
  ///   tools : None,
  ///   tool_choice : None,
  /// };
  ///
  /// let curl_command = request.as_curl_with_options( &CurlOptions::pretty() );
  /// ```
  fn as_curl_with_options( &self, options : &CurlOptions ) -> String;
}

use crate::components::inference_shared::ChatCompletionRequest;

/// Configuration options for curl command generation
///
/// This struct provides fine-grained control over how curl commands are formatted
/// and what additional information is included.
#[ derive( Debug, Clone ) ]
pub struct CurlOptions
{
  /// Whether to format JSON with pretty printing (indentation and line breaks)
  pub pretty_json : bool,
  /// Whether to include the auth header placeholder
  pub include_auth_header : bool,
  /// Whether to use multi-line formatting for better readability
  pub multiline_format : bool,
  /// Custom API key to include (if provided, overrides placeholder)
  pub api_key : Option< String >,
}

impl CurlOptions
{
  /// Create default curl options (compact format)
  #[ inline ]
  #[ must_use ]
  pub fn new() -> Self
  {
  Self
  {
      pretty_json : false,
      include_auth_header : true,
      multiline_format : false,
      api_key : None,
  }
  }

  /// Create options for pretty-formatted output
  ///
  /// This generates curl commands that are more readable, with:
  /// - Pretty-formatted JSON with indentation
  /// - Multi-line formatting with line continuations
  /// - Authorization header placeholder included
  #[ inline ]
  #[ must_use ]
  pub fn pretty() -> Self
  {
  Self
  {
      pretty_json : true,
      include_auth_header : true,
      multiline_format : true,
      api_key : None,
  }
  }

  /// Create options with a specific API key
  ///
  /// # Arguments
  ///
  /// * `api_key` - The API key to include in the curl command
  ///
  /// # Security Note
  ///
  /// Be careful when using this method in production code, as it will
  /// include your actual API key in the curl command string.
  #[ inline ]
  pub fn with_api_key< S : Into< String > >( api_key : S ) -> Self
  {
  Self
  {
      pretty_json : false,
      include_auth_header : true,
      multiline_format : false,
      api_key : Some( api_key.into() ),
  }
  }

  /// Enable pretty JSON formatting
  #[ inline ]
  #[ must_use ]
  pub fn pretty_json( mut self ) -> Self
  {
  self.pretty_json = true;
  self
  }

  /// Enable multi-line formatting
  #[ inline ]
  #[ must_use ]
  pub fn multiline( mut self ) -> Self
  {
  self.multiline_format = true;
  self
  }
}

impl Default for CurlOptions
{
  #[ inline ]
  fn default() -> Self
  {
  Self::new()
  }
}

/// Helper functions for generating curl commands
mod curl_helpers
{
  use super::*;

  /// Generate a curl command for a given URL and JSON body
  pub fn generate_curl_command(
  url : &str,
  json_body : &str,
  options : &CurlOptions,
  ) -> String
  {
  let formatted_json = if options.pretty_json
  {
      // Pretty-format the JSON if requested
      match serde_json::from_str::< serde_json::Value >( json_body )
      {
  Ok( value ) => serde_json::to_string_pretty( &value )
          .unwrap_or_else( |_| json_body.to_string() ),
  Err( _ ) => json_body.to_string(),
      }
  }
  else
  {
      json_body.to_string()
  };

  // Generate authorization header
  let auth_header = if let Some( ref api_key ) = options.api_key
  {
      format!( "Authorization : Bearer {api_key}" )
  }
  else if options.include_auth_header
  {
      "Authorization : Bearer YOUR_API_KEY_HERE".to_string()
  }
  else
  {
      String::new()
  };

  if options.multiline_format
  {
      // Multi-line format for better readability
      let json_escaped = formatted_json.replace( '"', "\\\"" );

      if options.include_auth_header || options.api_key.is_some()
      {
  format!(
          "curl -X POST \\\n  \"{url}\" \\\n  -H \"{auth_header}\" \\\n  -H \"Content-Type : application/json\" \\\n  -d \"{json_escaped}\""
  )
      }
      else
      {
  format!(
          "curl -X POST \\\n  \"{url}\" \\\n  -H \"Content-Type : application/json\" \\\n  -d \"{json_escaped}\""
  )
      }
  }
  else
  {
      // Single-line format
      let json_escaped = formatted_json.replace( '\'', "'\"'\"'" );

      if options.include_auth_header || options.api_key.is_some()
      {
  format!(
          "curl -X POST \"{url}\" -H \"{auth_header}\" -H \"Content-Type : application/json\" -d '{json_escaped}'"
  )
      }
      else
      {
  format!(
          "curl -X POST \"{url}\" -H \"Content-Type : application/json\" -d '{json_escaped}'"
  )
      }
  }
  }

  /// Handle JSON serialization with error recovery
  pub fn safe_json_serialize< T : serde::Serialize >( value : &T ) -> String
  {
  serde_json::to_string( value ).unwrap_or_else( |err| {
      format!( "{{\"error\": \"Failed to serialize request : {err}\"}}" )
  })
  }
}

impl AsCurl for ChatCompletionRequest
{
  #[ inline ]
  fn as_curl( &self ) -> String
  {
  self.as_curl_with_options( &CurlOptions::default() )
  }

  #[ inline ]
  fn as_curl_with_options( &self, options : &CurlOptions ) -> String
  {
  let url = "https://router.huggingface.co/v1/chat/completions";
  let json_body = curl_helpers::safe_json_serialize( self );
  curl_helpers::generate_curl_command( url, &json_body, options )
  }
}

/// Convenience functions for common debugging scenarios
///
/// These functions provide quick access to common curl generation patterns
/// without needing to construct request objects manually.
pub mod debug_helpers
{
  use super::*;
  use crate::components::inference_shared::ChatMessage;

  /// Generate a curl command for a simple chat completion request
  ///
  /// # Arguments
  ///
  /// * `message` - The message text to send
  /// * `options` - Optional curl formatting options
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use api_huggingface::diagnostics::debug_helpers;
  ///
  /// let curl_cmd = debug_helpers::simple_chat_curl(
  ///   "What is the capital of France?",
  ///   None
  /// );
  /// println!( "{}", curl_cmd );
  /// ```
  #[ inline ]
  #[ must_use ]
  pub fn simple_chat_curl( message : &str, options : Option< &CurlOptions > ) -> String
  {
  let request = ChatCompletionRequest
  {
      messages : vec![ ChatMessage
      {
  role : "user".to_string(),
  content : message.to_string(),
  tool_calls : None,
  tool_call_id : None,
      } ],
      model : "moonshotai/Kimi-K2-Instruct-0905:groq".to_string(),
      temperature : Some( 0.7 ),
      max_tokens : Some( 150 ),
      top_p : Some( 0.9 ),
      stream : Some( false ),
      tools : None,
      tool_choice : None,
  };

  match options
  {
      Some( opts ) => request.as_curl_with_options( opts ),
      None => request.as_curl(),
  }
  }

  /// Generate a curl command for a system + user message conversation
  ///
  /// # Arguments
  ///
  /// * `system_message` - The system prompt
  /// * `user_message` - The user message
  /// * `options` - Optional curl formatting options
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use api_huggingface::diagnostics::{ debug_helpers, CurlOptions };
  ///
  /// let curl_cmd = debug_helpers::chat_with_system_curl(
  ///   "You are a helpful assistant",
  ///   "Hello",
  ///   Some( &CurlOptions::pretty() )
  /// );
  /// println!( "{}", curl_cmd );
  /// ```
  #[ inline ]
  #[ must_use ]
  pub fn chat_with_system_curl(
  system_message : &str,
  user_message : &str,
  options : Option< &CurlOptions >
  ) -> String
  {
  let request = ChatCompletionRequest
  {
      messages : vec![
  ChatMessage
  {
          role : "system".to_string(),
          content : system_message.to_string(),
          tool_calls : None,
          tool_call_id : None,
  },
  ChatMessage
  {
          role : "user".to_string(),
          content : user_message.to_string(),
          tool_calls : None,
          tool_call_id : None,
  }
      ],
      model : "moonshotai/Kimi-K2-Instruct-0905:groq".to_string(),
      temperature : Some( 0.7 ),
      max_tokens : Some( 150 ),
      top_p : Some( 0.9 ),
      stream : Some( false ),
      tools : None,
      tool_choice : None,
  };

  match options
  {
      Some( opts ) => request.as_curl_with_options( opts ),
      None => request.as_curl(),
  }
  }
}
} // end mod private

crate::mod_interface!
{
  exposed use private::
  {
  AsCurl,
  CurlOptions,
  debug_helpers,
  };
}

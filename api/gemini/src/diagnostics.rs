//! Diagnostics module for debugging and development tools
//! 
//! This module provides diagnostic utilities to help developers debug API requests
//! and responses. It defines the AsCurl trait to enable generating executable 
//! curl commands from request objects.
//!
//! # Features
//! 
//! - **curl Command Generation**: Convert API requests to executable curl commands
//! - **Pretty-formatted Output**: Readable curl commands with proper JSON formatting
//! - **API Key Placeholders**: Clear indication of where API keys should be added
//! - **Error Resilience**: Robust handling of serialization failures
//! 
//! # Usage
//!
//! Convert API request objects to executable curl commands using the `AsCurl` trait.
//! Use `as_curl()` for compact output or `as_curl_with_options()` for customized formatting.
//! 
//! ## Adding API Key
//! 
//! The generated curl commands include a placeholder for the API key. Replace 
//! `YOUR_API_KEY_HERE` with your actual Gemini API key:
//! 
//! ```bash
//! curl -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key=YOUR_API_KEY_HERE" \
//!   -H "Content-Type : application/json" \
//!   -d '{...}'
//! ```

/// Trait for converting API request objects into executable curl commands
/// 
/// When the `diagnostics_curl` feature is enabled, this trait allows you to convert
/// API request objects into executable curl commands for debugging purposes.
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
  fn as_curl_with_options( &self, options : &CurlOptions ) -> String;
}

use crate::models::{ GenerateContentRequest, EmbedContentRequest, Blob };

/// Type alias for compatibility with test code
pub type InlineData = Blob;

/// Configuration options for curl command generation
/// 
/// This struct provides fine-grained control over how curl commands are formatted
/// and what additional information is included.
#[ derive( Debug, Clone ) ]
pub struct CurlOptions
{
  /// Whether to format JSON with pretty printing (indentation and line breaks)
  pub pretty_json : bool,
  /// Whether to include the API key placeholder in the URL
  pub include_api_key_placeholder : bool,
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
    Self {
      pretty_json : false,
      include_api_key_placeholder : true,
      multiline_format : false,
      api_key : None,
    }
  }
  
  /// Create options for pretty-formatted output
  /// 
  /// This generates curl commands that are more readable, with:
  /// - Pretty-formatted JSON with indentation
  /// - Multi-line formatting with line continuations
  /// - API key placeholder included
  #[ inline ]
  #[ must_use ]
  pub fn pretty() -> Self
  {
    Self {
      pretty_json : true,
      include_api_key_placeholder : true,
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
  pub fn with_api_key< S: Into< String > >( api_key : S ) -> Self
  {
    Self {
      pretty_json : false,
      include_api_key_placeholder : false,
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
    } else {
      json_body.to_string()
    };
    
    let url_with_key = if let Some( ref api_key ) = options.api_key
    {
      format!( "{url}?key={api_key}" )
    } else if options.include_api_key_placeholder
    {
      format!( "{url}?key=YOUR_API_KEY_HERE" )
    } else {
      url.to_string()
    };
    
    if options.multiline_format
    {
      // Multi-line format for better readability
      let json_escaped = formatted_json.replace( '"', "\\\"" );
      format!(
        "curl -X POST \\\n  \"{url_with_key}\" \\\n  -H \"Content-Type : application/json\" \\\n  -d \"{json_escaped}\""
      )
    } else {
      // Single-line format
      let json_escaped = formatted_json.replace( '\'', "'\"'\"'" );
      format!(
        "curl -X POST \"{url_with_key}\" -H \"Content-Type : application/json\" -d '{json_escaped}'"
      )
    }
  }
  
  /// Handle JSON serialization with error recovery
  pub fn safe_json_serialize< T: serde::Serialize >( value : &T ) -> String
  {
    serde_json ::to_string( value ).unwrap_or_else( |err| {
      format!( "{{\"error\": \"Failed to serialize request : {err}\"}}" )
    })
  }
}

impl AsCurl for GenerateContentRequest
{
  #[ inline ]
  fn as_curl( &self ) -> String
  {
    self.as_curl_with_options( &CurlOptions::default() )
  }
  
  #[ inline ]
  fn as_curl_with_options( &self, options : &CurlOptions ) -> String
  {
    let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent";
    let json_body = curl_helpers::safe_json_serialize( self );
    curl_helpers ::generate_curl_command( url, &json_body, options )
  }
}

impl AsCurl for EmbedContentRequest
{
  #[ inline ]
  fn as_curl( &self ) -> String
  {
    self.as_curl_with_options( &CurlOptions::default() )
  }
  
  #[ inline ]  
  fn as_curl_with_options( &self, options : &CurlOptions ) -> String
  {
    let url = "https://generativelanguage.googleapis.com/v1beta/models/text-embedding-004:embedContent";
    let json_body = curl_helpers::safe_json_serialize( self );
    curl_helpers ::generate_curl_command( url, &json_body, options )
  }
}

/// Convenience functions for common debugging scenarios
/// 
/// These functions provide quick access to common curl generation patterns
/// without needing to construct request objects manually.
pub mod debug_helpers
{
  use super::*;
  
  /// Generate a curl command for a simple text generation request
  ///
  /// # Arguments
  ///
  /// * `text` - The input text to send to the model
  /// * `options` - Optional curl formatting options
  #[ inline ]
  #[ must_use ]
  pub fn simple_generate_curl( text : &str, options : Option< &CurlOptions > ) -> String
  {
    let request = GenerateContentRequest {
      contents : vec![ crate::models::Content {
        parts : vec![ crate::models::Part {
          text : Some( text.to_string() ),
          ..Default::default()
        } ],
        role : "user".to_string(),
      } ],
      ..Default::default()
    };
    
    match options
    {
      Some( opts ) => request.as_curl_with_options( opts ),
      None => request.as_curl(),
    }
  }
  
  /// Generate a curl command for a simple embedding request
  ///
  /// # Arguments
  ///
  /// * `text` - The text to embed
  /// * `options` - Optional curl formatting options
  #[ inline ]
  #[ must_use ]
  pub fn simple_embed_curl( text : &str, options : Option< &CurlOptions > ) -> String
  {
    let request = EmbedContentRequest {
      content : crate::models::Content {
        parts : vec![ crate::models::Part {
          text : Some( text.to_string() ),
          ..Default::default()
        } ],
        role : "user".to_string(),
      },
      task_type : None,
      title : None,
      output_dimensionality : None,
    };
    
    match options
    {
      Some( opts ) => request.as_curl_with_options( opts ),
      None => request.as_curl(),
    }
  }
}
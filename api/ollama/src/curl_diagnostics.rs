//! CURL Diagnostics for Debugging
//!
//! Generate equivalent curl commands for API requests to aid in debugging.

#[ cfg( feature = "enabled" ) ]
mod private
{
  use super::super::{ ChatRequest, GenerateRequest };
  use serde::Serialize;

  /// CURL command generator
  #[ derive( Debug ) ]
  pub struct CurlGenerator;

  impl CurlGenerator
  {
    /// Generate curl command for a chat request
    #[ must_use ]
    pub fn for_chat( base_url : &str, request : &ChatRequest ) -> String
    {
      let url = format!( "{}/api/chat", base_url );
      let json = serde_json::to_string_pretty( request ).unwrap_or_else( | _ | "{}".to_string() );

      format!(
        "curl -X POST '{}' \\\n  -H 'Content-Type: application/json' \\\n  -d '{}'",
        url,
        json.replace( '\'', "'\\''" )
      )
    }

    /// Generate curl command for a generate request
    #[ must_use ]
    pub fn for_generate( base_url : &str, request : &GenerateRequest ) -> String
    {
      let url = format!( "{}/api/generate", base_url );
      let json = serde_json::to_string_pretty( request ).unwrap_or_else( | _ | "{}".to_string() );

      format!(
        "curl -X POST '{}' \\\n  -H 'Content-Type: application/json' \\\n  -d '{}'",
        url,
        json.replace( '\'', "'\\''" )
      )
    }

    /// Generate curl command for any endpoint with generic request
    #[ must_use ]
    pub fn for_endpoint< T : Serialize >( base_url : &str, endpoint : &str, request : &T ) -> String
    {
      let url = format!( "{}{}", base_url, endpoint );
      let json = serde_json::to_string_pretty( request ).unwrap_or_else( | _ | "{}".to_string() );

      format!(
        "curl -X POST '{}' \\\n  -H 'Content-Type: application/json' \\\n  -d '{}'",
        url,
        json.replace( '\'', "'\\''" )
      )
    }

    /// Generate curl command for GET request
    #[ must_use ]
    pub fn for_get( url : &str ) -> String
    {
      format!( "curl -X GET '{}'", url )
    }
  }

  /// CURL options for customizing generated commands
  #[ derive( Debug, Clone ) ]
  pub struct CurlOptions
  {
    /// Include verbose output
    pub verbose : bool,
    /// Include timing information
    pub include_timing : bool,
    /// Pretty print JSON
    pub pretty_print : bool,
  }

  impl Default for CurlOptions
  {
    fn default() -> Self
    {
      Self
      {
        verbose : false,
        include_timing : false,
        pretty_print : true,
      }
    }
  }

  impl CurlOptions
  {
    /// Create new CURL options
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Enable verbose output
    #[ must_use ]
    pub fn with_verbose( mut self ) -> Self
    {
      self.verbose = true;
      self
    }

    /// Enable timing information
    #[ must_use ]
    pub fn with_timing( mut self ) -> Self
    {
      self.include_timing = true;
      self
    }

    /// Apply options to curl command
    #[ must_use ]
    pub fn apply( &self, mut command : String ) -> String
    {
      if self.verbose
      {
        command.push_str( " \\\n  -v" );
      }
      if self.include_timing
      {
        command.push_str( " \\\n  -w '\\nTime: %{time_total}s\\n'" );
      }
      command
    }
  }

  #[ cfg( test ) ]
  mod tests
  {
    use super::*;

    #[ test ]
    fn test_curl_for_chat()
    {
      use crate::messages::{ ChatMessage, MessageRole };

      let request = ChatRequest
      {
        model : "llama3.2".to_string(),
        messages : vec![ ChatMessage
        {
          role : MessageRole::User,
          content : "Hello!".to_string(),
          images : None,
          #[ cfg( feature = "tool_calling" ) ]
          tool_calls : None,
        } ],
        stream : None,
        think : None,
        options : None,
        #[ cfg( feature = "tool_calling" ) ]
        tools : None,
        #[ cfg( feature = "tool_calling" ) ]
        tool_messages : None,
      };

      let curl = CurlGenerator::for_chat( "http://localhost:11434", &request );

      assert!( curl.contains( "curl -X POST" ) );
      assert!( curl.contains( "/api/chat" ) );
      assert!( curl.contains( "llama3.2" ) );
    }

    #[ test ]
    fn test_curl_for_get()
    {
      let curl = CurlGenerator::for_get( "http://localhost:11434/api/tags" );

      assert!( curl.contains( "curl -X GET" ) );
      assert!( curl.contains( "/api/tags" ) );
    }

    #[ test ]
    fn test_curl_options()
    {
      let options = CurlOptions::new()
        .with_verbose()
        .with_timing();

      let command = options.apply( "curl test".to_string() );

      assert!( command.contains( "-v" ) );
      assert!( command.contains( "time_total" ) );
    }
  }
}

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{
  exposed use
  {
    CurlGenerator,
    CurlOptions,
  };
}

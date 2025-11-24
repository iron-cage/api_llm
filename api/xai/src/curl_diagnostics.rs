mod private
{
  //! CURL command generation for debugging XAI API requests.
  //!
  //! This module provides utilities to convert XAI API requests into
  //! equivalent CURL commands for debugging, testing, and sharing.
  //!
  //! # Use Cases
  //!
  //! 1. **Debugging**: Test requests directly with CURL
  //! 2. **Reproduction**: Share reproducible API calls with support
  //! 3. **Documentation**: Generate example requests for docs
  //! 4. **Learning**: Understand the underlying HTTP requests
  //!
  //! # Design Decisions
  //!
  //! ## Why CURL?
  //!
  //! CURL is the universal HTTP debugging tool:
  //!
  //! 1. **Ubiquity**: Available on all platforms
  //! 2. **Simplicity**: Easy to understand and modify
  //! 3. **Portability**: Works in any terminal
  //! 4. **Standard**: Industry-standard HTTP debugging
  //!
  //! ## Command Format
  //!
  //! Generated commands use:
  //!
  //! - **Readable formatting**: Multi-line with backslash continuation
  //! - **Environment variables**: `$XAI_API_KEY` for security
  //! - **Pretty JSON**: Indented request body for readability
  //! - **Explicit headers**: All required headers shown
  //!
  //! ## Security
  //!
  //! **IMPORTANT**: Generated commands use `$XAI_API_KEY` environment
  //! variable instead of embedding the actual key. This prevents
  //! accidental key exposure when sharing commands.

  use crate::ChatCompletionRequest;

  /// Converts a chat completion request to a CURL command.
  ///
  /// Generates a ready-to-execute CURL command that makes the same
  /// API request. The API key is referenced via `$XAI_API_KEY`
  /// environment variable for security.
  ///
  /// # Arguments
  ///
  /// * `request` - The chat completion request to convert
  ///
  /// # Returns
  ///
  /// A CURL command string that can be executed in a shell.
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "curl_diagnostics") ]
  /// # {
  /// use api_xai::{ to_curl, ChatCompletionRequest, Message };
  ///
  /// let request = ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "Hello!" ) ] )
  ///   .temperature( 0.7 )
  ///   .form();
  ///
  /// let curl = to_curl( &request );
  /// println!( "{}", curl );
  ///
  /// // Output:
  /// // curl -X POST https://api.x.ai/v1/chat/completions \
  /// //   -H "Authorization : Bearer $XAI_API_KEY" \
  /// //   -H "Content-Type : application/json" \
  /// //   -d '{
  /// //   "model": "grok-2-1212",
  /// //   "messages": [{"role": "user", "content": "Hello!"}],
  /// //   "temperature": 0.7
  /// // }'
  /// # }
  /// ```
  #[ cfg( feature = "curl_diagnostics" ) ]
  pub fn to_curl( request : &ChatCompletionRequest ) -> String
  {
    let json = serde_json::to_string_pretty( request )
      .unwrap_or_else( | _ | "{}".to_string() );

    format!
    (
      "curl -X POST https://api.x.ai/v1/chat/completions \\\n  \
       -H \"Authorization : Bearer $XAI_API_KEY\" \\\n  \
       -H \"Content-Type : application/json\" \\\n  \
       -d '{json}'"
    )
  }

  /// Converts a chat completion request to a CURL command with a custom API key.
  ///
  /// **WARNING**: This function embeds the API key directly in the command.
  /// Only use for testing. Never share the output.
  ///
  /// # Arguments
  ///
  /// * `request` - The chat completion request to convert
  /// * `api_key` - The API key to embed (WARNING: visible in output)
  ///
  /// # Returns
  ///
  /// A CURL command string with embedded API key.
  ///
  /// # Security Warning
  ///
  /// The generated command contains your API key in plain text.
  /// Do not share this command or save it in version control.
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "curl_diagnostics") ]
  /// # {
  /// use api_xai::{ to_curl_with_key, ChatCompletionRequest, Message };
  ///
  /// let request = ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "Hello!" ) ] )
  ///   .form();
  ///
  /// // WARNING: This embeds your API key in the output!
  /// let curl = to_curl_with_key( &request, "xai-your-key-here" );
  ///
  /// // Only use for immediate testing, never share
  /// println!( "{}", curl );
  /// # }
  /// ```
  #[ cfg( feature = "curl_diagnostics" ) ]
  pub fn to_curl_with_key( request : &ChatCompletionRequest, api_key : &str ) -> String
  {
    let json = serde_json::to_string_pretty( request )
      .unwrap_or_else( | _ | "{}".to_string() );

    format!
    (
      "curl -X POST https://api.x.ai/v1/chat/completions \\\n  \
       -H \"Authorization : Bearer {api_key}\" \\\n  \
       -H \"Content-Type : application/json\" \\\n  \
       -d '{json}'"
    )
  }

  /// Converts a chat completion request to a CURL command with custom endpoint.
  ///
  /// Useful for testing with proxy servers or alternative endpoints.
  ///
  /// # Arguments
  ///
  /// * `request` - The chat completion request to convert
  /// * `endpoint` - The custom endpoint URL (must include full path)
  ///
  /// # Returns
  ///
  /// A CURL command string targeting the custom endpoint.
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "curl_diagnostics") ]
  /// # {
  /// use api_xai::{ to_curl_with_endpoint, ChatCompletionRequest, Message };
  ///
  /// let request = ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "Hello!" ) ] )
  ///   .form();
  ///
  /// // Test with local proxy
  /// let curl = to_curl_with_endpoint
  /// (
  ///   &request,
  ///   "http://localhost:8080/v1/chat/completions"
  /// );
  ///
  /// println!( "{}", curl );
  /// # }
  /// ```
  #[ cfg( feature = "curl_diagnostics" ) ]
  pub fn to_curl_with_endpoint
  (
    request : &ChatCompletionRequest,
    endpoint : &str
  )
  -> String
  {
    let json = serde_json::to_string_pretty( request )
      .unwrap_or_else( | _ | "{}".to_string() );

    format!
    (
      "curl -X POST {endpoint} \\\n  \
       -H \"Authorization : Bearer $XAI_API_KEY\" \\\n  \
       -H \"Content-Type : application/json\" \\\n  \
       -d '{json}'"
    )
  }

  /// Converts a chat completion request to a compact CURL command (single line).
  ///
  /// Generates a single-line CURL command without pretty formatting.
  /// Useful for copy-paste into logs or when space is limited.
  ///
  /// # Arguments
  ///
  /// * `request` - The chat completion request to convert
  ///
  /// # Returns
  ///
  /// A compact CURL command string (single line).
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "curl_diagnostics") ]
  /// # {
  /// use api_xai::{ to_curl_compact, ChatCompletionRequest, Message };
  ///
  /// let request = ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "Hello!" ) ] )
  ///   .form();
  ///
  /// let curl = to_curl_compact( &request );
  /// println!( "{}", curl );
  ///
  /// // Output (single line):
  /// // curl -X POST https://api.x.ai/v1/chat/completions -H "Authorization : Bearer $XAI_API_KEY" -H "Content-Type : application/json" -d '{"model":"grok-2-1212","messages":[...]}'
  /// # }
  /// ```
  #[ cfg( feature = "curl_diagnostics" ) ]
  pub fn to_curl_compact( request : &ChatCompletionRequest ) -> String
  {
    let json = serde_json::to_string( request )
      .unwrap_or_else( | _ | "{}".to_string() );

    format!
    (
      "curl -X POST https://api.x.ai/v1/chat/completions -H \"Authorization : Bearer $XAI_API_KEY\" -H \"Content-Type : application/json\" -d '{json}'"
    )
  }
}

#[ cfg( feature = "curl_diagnostics" ) ]
crate::mod_interface!
{
  exposed use
  {
    to_curl,
    to_curl_with_key,
    to_curl_with_endpoint,
    to_curl_compact,
  };
}

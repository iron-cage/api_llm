//! Client configuration and message types
//!
//! Request/response types and configuration for the Anthropic API client.

#[ allow( clippy::missing_inline_in_public_items ) ]
mod private
{
  #[ cfg( feature = "error-handling" ) ]
  use crate::error::{ AnthropicError, AnthropicResult };
  
  #[ cfg( not( feature = "error-handling" ) ) ]
  use error_tools::{ err, Result as AnthropicResult };
  
  #[ cfg( not( feature = "error-handling" ) ) ]
  type AnthropicError = error_tools::Error;
  
  use crate::{ secret::Secret, messages::Message };
  
  #[ cfg( feature = "tools" ) ]
  use crate::messages::{ ToolDefinition, ToolChoice };
  use serde::{ Serialize, Deserialize };
  use std::time::Duration;
  
  /// Standard base URL for Anthropic API (no longer a magic default)
  pub const ANTHROPIC_API_BASE_URL : &str = "https://api.anthropic.com";
  /// Standard API version header value (no longer a magic default)
  pub const ANTHROPIC_API_VERSION : &str = "2023-06-01";
  /// Standard user agent string (no longer a magic default)
  pub const ANTHROPIC_USER_AGENT : &str = "anthropic-rust-client/0.1.0";
  /// Current recommended model (no longer a magic default)
  pub const RECOMMENDED_MODEL : &str = "claude-sonnet-4-5-20250929";
  /// Minimum allowed `max_tokens` value
  pub const MIN_MAX_TOKENS : u32 = 1;
  /// Maximum allowed `max_tokens` value
  pub const MAX_MAX_TOKENS : u32 = 200_000;
  /// Minimum allowed temperature value
  pub const MIN_TEMPERATURE : f32 = 0.0;
  /// Maximum allowed temperature value
  pub const MAX_TEMPERATURE : f32 = 1.0;

  /// Configuration for Anthropic API client
  #[ derive( Debug, Clone ) ]
  pub struct ClientConfig
  {
    /// Base URL for API requests
    pub base_url : String,
    /// API version to use
    pub api_version : String,
    /// Request timeout
    pub request_timeout : Duration,
    /// User agent string
    pub user_agent : String,
  }

  // No Default implementation - explicit configuration required

  impl ClientConfig
  {
    /// Create a new configuration builder requiring explicit configuration
    ///
    /// # Governing Principle Compliance
    ///
    /// This builder follows the "Thin Client, Rich API" principle by:
    /// - **Explicit Configuration**: All client behaviors require explicit configuration
    /// - **Zero Magic**: No automatic behaviors or hidden defaults that affect API communication
    /// - **Transparent Control**: Each configuration option clearly indicates its purpose and impact
    pub fn builder() -> ClientConfigBuilder
    {
      ClientConfigBuilder::new()
    }

    /// Create configuration with recommended values for typical usage
    ///
    /// # Governing Principle Compliance
    ///
    /// This provides recommended values without making them implicit defaults.
    /// Developers must explicitly choose to use these recommended values.
    pub fn recommended() -> Self
    {
      Self
      {
        base_url : ANTHROPIC_API_BASE_URL.to_string(),
        api_version : ANTHROPIC_API_VERSION.to_string(),
        request_timeout : Duration::from_secs( 60 ), // Recommended for most use cases
        user_agent : ANTHROPIC_USER_AGENT.to_string(),
      }
    }

    /// Create configuration with explicit values (no recommendations)
    pub fn with_explicit_config(
      base_url : String,
      api_version : String,
      request_timeout : Duration,
      user_agent : String,
    ) -> Self
    {
      Self
      {
        base_url,
        api_version,
        request_timeout,
        user_agent,
      }
    }

    /// Set base URL
    #[ must_use ]
    pub fn with_base_url( mut self, base_url : String ) -> Self
    {
      self.base_url = base_url;
      self
    }

    /// Set request timeout
    #[ must_use ]
    pub fn with_timeout( mut self, timeout : Duration ) -> Self
    {
      self.request_timeout = timeout;
      self
    }
  }

  /// Builder for client configuration requiring explicit values
  #[ derive( Debug ) ]
  pub struct ClientConfigBuilder
  {
    base_url : Option< String >,
    api_version : Option< String >,
    request_timeout : Option< Duration >,
    user_agent : Option< String >,
  }

  impl Default for ClientConfigBuilder 
  {
    fn default() -> Self 
    {
      Self::new()
    }
  }

  // Builder implementation in types_builders.rs
  include!( "types_builders.rs" );

  /// Cache control configuration for prompt caching
  ///
  /// Anthropic Prompt Caching allows caching of large context (system prompts, documents, etc.)
  /// to reduce costs (~90% savings on cached tokens) and improve latency.
  #[ derive( Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
  pub struct CacheControl
  {
    /// Cache type - currently only "ephemeral" is supported (5-minute TTL)
    #[ serde( rename = "type" ) ]
    pub cache_type : String,
  }

  impl CacheControl
  {
    /// Create an ephemeral cache control (5-minute TTL)
    pub fn ephemeral() -> Self
    {
      Self { cache_type : "ephemeral".to_string() }
    }
  }

  impl From< String > for SystemPrompt
  {
    fn from( text : String ) -> Self
    {
      Self { text, cache_control : None }
    }
  }

  impl From< &str > for SystemPrompt
  {
    fn from( text : &str ) -> Self
    {
      Self { text : text.to_string(), cache_control : None }
    }
  }

  /// System prompt with optional cache control
  ///
  /// Replaces simple String system prompts to support caching large system contexts.
  #[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
  pub struct SystemPrompt
  {
    /// System prompt text
    pub text : String,
    /// Optional cache control for this system prompt
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub cache_control : Option< CacheControl >,
  }

  /// Request to create a message
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct CreateMessageRequest
  {
    /// Model to use for generation
    pub model : String,
    /// Max tokens to generate
    pub max_tokens : u32,
    /// Messages in conversation
    pub messages : Vec< Message >,
    /// System prompt blocks with optional cache control
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub system : Option< Vec< SystemContent > >,
    /// Temperature for sampling
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub temperature : Option< f32 >,
    /// Whether to stream the response
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub stream : Option< bool >,
    /// Tools available for the model to use
    #[ cfg( feature = "tools" ) ]
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub tools : Option< Vec< ToolDefinition > >,
    /// How the model should use tools
    #[ cfg( feature = "tools" ) ]
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub tool_choice : Option< ToolChoice >,
  }

  impl CreateMessageRequest
  {
    /// Create a new builder for `CreateMessageRequest`
    ///
    /// # Governing Principle Compliance
    ///
    /// This builder follows the "Thin Client, Rich API" principle by:
    /// - **Explicit Parameter Setting**: All message parameters must be explicitly configured
    /// - **Direct API Mapping**: Builder fields correspond directly to Anthropic API parameters
    /// - **No Automatic Behaviors**: No hidden logic that modifies or interprets developer input
    /// - **Transparent Validation**: Clear error messages for invalid parameter combinations
    pub fn builder() -> CreateMessageRequestBuilder
    {
      CreateMessageRequestBuilder::default()
    }

    /// Validate the request parameters
    ///
    /// # Governing Principle Compliance
    ///
    /// This validation method follows the "Thin Client, Rich API" principle by:
    /// - **Transparent Error Reporting**: All validation errors expose exact parameter violations
    /// - **API Constraint Enforcement**: Validates only constraints required by Anthropic's API
    /// - **Zero Client Intelligence**: No automatic correction of invalid parameters
    /// - **Explicit Error Messages**: Clear, actionable error descriptions for developers
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails for any parameter
    ///
    /// # Panics
    ///
    /// Panics if `tool_choice` is specific but `tools` is None (internal consistency error)
    pub fn validate( &self ) -> AnthropicResult< () >
    {
      if self.model.trim().is_empty()
      {
        return Err( AnthropicError::InvalidRequest( "Model cannot be empty".to_string() ) );
      }

      if self.max_tokens < MIN_MAX_TOKENS || self.max_tokens > MAX_MAX_TOKENS
      {
        return Err( AnthropicError::InvalidRequest( 
          format!( "max_tokens must be between {MIN_MAX_TOKENS} and {MAX_MAX_TOKENS}" ) 
        ) );
      }

      if self.messages.is_empty()
      {
        return Err( AnthropicError::InvalidRequest( "At least one message is required".to_string() ) );
      }

      if let Some( temp ) = self.temperature
      {
        if !( MIN_TEMPERATURE..=MAX_TEMPERATURE ).contains( &temp )
        {
          return Err( AnthropicError::InvalidRequest( 
            format!( "Temperature must be between {MIN_TEMPERATURE} and {MAX_TEMPERATURE}" ) 
          ) );
        }
      }

      #[ cfg( feature = "tools" ) ]
      {
        // Validate tool-related parameters
        if let Some( ref tool_choice ) = self.tool_choice
        {
          if self.tools.is_none()
          {
            return Err( AnthropicError::InvalidRequest( 
              "tool_choice specified but no tools provided".to_string() 
            ) );
          }
          
          // Validate specific tool choice references an existing tool
          if tool_choice.is_specific()
          {
            if let Some( tool_name ) = tool_choice.tool_name()
            {
              let tools = self.tools.as_ref().unwrap();
              if !tools.iter().any( | tool | tool.name == tool_name )
              {
                return Err( AnthropicError::InvalidRequest( 
                  format!( "tool_choice references unknown tool : '{tool_name}'" )
                ) );
              }
            }
          }
        }
        
        // Validate tool definitions
        if let Some( ref tools ) = self.tools
        {
          if tools.is_empty()
          {
            return Err( AnthropicError::InvalidRequest( 
              "tools array cannot be empty - use None instead".to_string() 
            ) );
          }
          
          // Check for duplicate tool names
          let mut seen_names = std::collections::HashSet::new();
          for tool in tools
          {
            if tool.name.trim().is_empty()
            {
              return Err( AnthropicError::InvalidRequest( 
                "tool name cannot be empty".to_string() 
              ) );
            }
            
            if !seen_names.insert( &tool.name )
            {
              return Err( AnthropicError::InvalidRequest( 
                format!( "duplicate tool name : '{}'", tool.name )
              ) );
            }
            
            if tool.description.trim().is_empty()
            {
              return Err( AnthropicError::InvalidRequest( 
                format!( "tool '{}' description cannot be empty", tool.name )
              ) );
            }
          }
          
          // Validate tool limit
          if tools.len() > 64
          {
            return Err( AnthropicError::InvalidRequest( 
              "maximum of 64 tools allowed per request".to_string() 
            ) );
          }
        }
      }

      Ok( () )
    }
  }

  /// Builder for `CreateMessageRequest`
  #[ derive( Debug, Default ) ]
  pub struct CreateMessageRequestBuilder
  {
    model : Option< String >,
    max_tokens : Option< u32 >,
    messages : Vec< Message >,
    system : Option< Vec< SystemContent > >,
    temperature : Option< f32 >,
    stream : Option< bool >,
    #[ cfg( feature = "tools" ) ]
    tools : Option< Vec< ToolDefinition > >,
    #[ cfg( feature = "tools" ) ]
    tool_choice : Option< ToolChoice >,
  }


  /// Response from create message API
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct CreateMessageResponse
  {
    /// Response ID
    pub id : String,
    /// Response type
    pub r#type : String,
    /// Role of the response
    pub role : String,
    /// Content of the response
    pub content : Vec< ResponseContent >,
    /// Model used
    pub model : String,
    /// Stop reason
    pub stop_reason : Option< String >,
    /// Stop sequence
    pub stop_sequence : Option< String >,
    /// Usage statistics
    pub usage : Usage,
  }

  impl CreateMessageResponse
  {
    /// Get the first text content from the response
    pub fn text( &self ) -> Option< &str >
    {
      self.content
        .iter()
        .find_map( | content | match content {
            ResponseContent::Text { text } => Some(text.as_str()),
            _ => None,
        })
    }

    /// Check if the response was truncated due to `max_tokens`
    pub fn is_truncated( &self ) -> bool
    {
      self.stop_reason.as_deref() == Some( "max_tokens" )
    }
  }

  /// Content in response
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  #[serde(tag = "type")]
  pub enum ResponseContent
  {
    /// Text response type.
    #[serde(rename = "text")]
    Text {
        /// Text response content.
        text: String
    },

    /// Tool use response type.
    #[serde(rename = "tool_use")]
    ToolUse {
        /// Tool use ID.
        id: String,
        /// Name of tool called.
        name: String,
        /// Tool use content, corresponding to user-provided schema.
        input: serde_json::Value,
    },

    /// Other response types without present support.
    #[serde(other)]
    Unsupported,
  }

  /// Usage statistics
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct Usage
  {
    /// Input tokens
    pub input_tokens : u32,
    /// Output tokens
    pub output_tokens : u32,
    /// Cache creation input tokens (when cache is first created)
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub cache_creation_input_tokens : Option< u32 >,
    /// Cache read input tokens (when reading from existing cache)
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub cache_read_input_tokens : Option< u32 >,
  }

  impl Usage
  {
    /// Get total tokens used
    pub fn total_tokens( &self ) -> u32
    {
      self.input_tokens + self.output_tokens
    }
  }

  /// System content block for count tokens endpoint
  ///
  /// The count tokens endpoint expects system as an array of content blocks
  #[ derive( Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
  pub struct SystemContent
  {
    /// Type - always "text"
    #[ serde( rename = "type" ) ]
    pub r#type : String,
    /// Text content
    pub text : String,
    /// Optional cache control
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub cache_control : Option< CacheControl >,
  }

  impl SystemContent
  {
    /// Create a new system content block from text
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::SystemContent;
    ///
    /// let content = SystemContent::text( "You are a helpful assistant" );
    /// assert_eq!( content.text, "You are a helpful assistant" );
    /// ```
    pub fn text< S : Into< String > >( text : S ) -> Self
    {
      Self
      {
        r#type : "text".to_string(),
        text : text.into(),
        cache_control : None,
      }
    }

    /// Set cache control for this system content (builder pattern)
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::{ SystemContent, CacheControl };
    ///
    /// let content = SystemContent::text( "Knowledge base" )
    ///   .with_cache_control( CacheControl::ephemeral() );
    ///
    /// assert!( content.cache_control.is_some() );
    /// ```
    #[ must_use ]
    pub fn with_cache_control( mut self, cache_control : CacheControl ) -> Self
    {
      self.cache_control = Some( cache_control );
      self
    }

    /// Validate the system content
    ///
    /// Checks that:
    /// - Text is not empty
    /// - Type is set correctly
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::SystemContent;
    ///
    /// let content = SystemContent::text( "Valid content" );
    /// assert!( content.validate().is_ok() );
    ///
    /// let empty = SystemContent
    /// {
    ///   r#type : "text".to_string(),
    ///   text : "".to_string(),
    ///   cache_control : None,
    /// };
    /// assert!( empty.validate().is_err() );
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the text is empty or the type is not "text"
    pub fn validate( &self ) -> Result< (), String >
    {
      if self.text.is_empty()
      {
        return Err( "System content text cannot be empty".to_string() );
      }

      if self.r#type != "text"
      {
        return Err( format!( "Invalid system content type : {}", self.r#type ) );
      }

      Ok( () )
    }

    /// Check if this system content has cache control enabled
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::{ SystemContent, CacheControl };
    ///
    /// let cached = SystemContent::text( "Cached" )
    ///   .with_cache_control( CacheControl::ephemeral() );
    /// assert!( cached.has_cache_control() );
    ///
    /// let not_cached = SystemContent::text( "Not cached" );
    /// assert!( !not_cached.has_cache_control() );
    /// ```
    pub fn has_cache_control( &self ) -> bool
    {
      self.cache_control.is_some()
    }
  }

  impl From< &str > for SystemContent
  {
    fn from( text : &str ) -> Self
    {
      Self::text( text )
    }
  }

  impl From< String > for SystemContent
  {
    fn from( text : String ) -> Self
    {
      Self::text( text )
    }
  }

  /// Builder for composing multi-part system instructions
  ///
  /// Provides a convenient API for building structured system prompts
  /// with multiple content blocks, optional caching, and validation.
  ///
  /// # Examples
  ///
  /// ```
  /// use api_claude::{ SystemInstructions, CacheControl };
  ///
  /// let instructions = SystemInstructions::new()
  ///   .add_text( "You are a helpful assistant." )
  ///   .add_cached_text( "Knowledge base : Large corpus of information..." )
  ///   .add_text( "Help the user with their questions." )
  ///   .build();
  ///
  /// assert_eq!( instructions.len(), 3 );
  /// ```
  #[ derive( Debug, Clone, Default ) ]
  pub struct SystemInstructions
  {
    parts : Vec< SystemContent >,
  }

  impl SystemInstructions
  {
    /// Create a new empty system instructions builder
    pub fn new() -> Self
    {
      Self
      {
        parts : Vec::new(),
      }
    }

    /// Add a text instruction
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::SystemInstructions;
    ///
    /// let instructions = SystemInstructions::new()
    ///   .add_text( "You are a helpful assistant" )
    ///   .build();
    ///
    /// assert_eq!( instructions.len(), 1 );
    /// ```
    #[ must_use ]
    pub fn add_text< S : Into< String > >( mut self, text : S ) -> Self
    {
      self.parts.push( SystemContent::text( text ) );
      self
    }

    /// Add a cached text instruction
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::SystemInstructions;
    ///
    /// let instructions = SystemInstructions::new()
    ///   .add_cached_text( "Large knowledge base" )
    ///   .build();
    ///
    /// assert!( instructions[ 0 ].has_cache_control() );
    /// ```
    #[ must_use ]
    pub fn add_cached_text< S : Into< String > >( mut self, text : S ) -> Self
    {
      let content = SystemContent::text( text )
        .with_cache_control( CacheControl::ephemeral() );
      self.parts.push( content );
      self
    }

    /// Add a custom system content block
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::{ SystemInstructions, SystemContent, CacheControl };
    ///
    /// let custom = SystemContent::text( "Custom instruction" )
    ///   .with_cache_control( CacheControl::ephemeral() );
    ///
    /// let instructions = SystemInstructions::new()
    ///   .add( custom )
    ///   .build();
    ///
    /// assert_eq!( instructions.len(), 1 );
    /// ```
    #[ must_use ]
    #[ allow( clippy::should_implement_trait ) ]
    pub fn add( mut self, content : SystemContent ) -> Self
    {
      self.parts.push( content );
      self
    }

    /// Build the final vector of system content blocks
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::SystemInstructions;
    ///
    /// let instructions = SystemInstructions::new()
    ///   .add_text( "Part 1" )
    ///   .add_text( "Part 2" )
    ///   .build();
    ///
    /// assert_eq!( instructions.len(), 2 );
    /// ```
    #[ must_use ]
    pub fn build( self ) -> Vec< SystemContent >
    {
      self.parts
    }

    /// Validate all system content blocks
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::SystemInstructions;
    ///
    /// let instructions = SystemInstructions::new()
    ///   .add_text( "Valid instruction" );
    ///
    /// assert!( instructions.validate().is_ok() );
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if any content block fails validation
    pub fn validate( &self ) -> Result< (), String >
    {
      if self.parts.is_empty()
      {
        return Err( "System instructions cannot be empty".to_string() );
      }

      for ( idx, content ) in self.parts.iter().enumerate()
      {
        content.validate()
          .map_err( |e| format!( "Invalid content at index {idx}: {e}" ) )?;
      }

      Ok( () )
    }

    /// Get the number of content blocks
    pub fn len( &self ) -> usize
    {
      self.parts.len()
    }

    /// Check if there are no content blocks
    pub fn is_empty( &self ) -> bool
    {
      self.parts.is_empty()
    }
  }

  /// Request to count tokens in a message
  ///
  /// This allows pre-calculating token usage for cost estimation without sending actual requests.
  #[ cfg( feature = "count-tokens" ) ]
  #[ derive( Debug, Clone, Serialize ) ]
  pub struct CountMessageTokensRequest
  {
    /// Model to use for token counting
    pub model : String,
    /// Messages in conversation
    pub messages : Vec< Message >,
    /// System prompt blocks with optional cache control
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub system : Option< Vec< SystemContent > >,
    /// Tools available for the model to use
    #[ cfg( feature = "tools" ) ]
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub tools : Option< Vec< ToolDefinition > >,
  }

  #[ cfg( feature = "count-tokens" ) ]
  impl CountMessageTokensRequest
  {
    /// Validate the request parameters
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails for any parameter
    pub fn validate( &self ) -> AnthropicResult< () >
    {
      if self.model.trim().is_empty()
      {
        return Err( AnthropicError::InvalidRequest( "Model cannot be empty".to_string() ) );
      }

      if self.messages.is_empty()
      {
        return Err( AnthropicError::InvalidRequest( "At least one message is required".to_string() ) );
      }

      Ok( () )
    }
  }

  /// Response from token counting endpoint
  #[ cfg( feature = "count-tokens" ) ]
  #[ derive( Debug, Clone, Deserialize ) ]
  pub struct CountMessageTokensResponse
  {
    /// Number of input tokens
    pub input_tokens : u32,
  }

  /// Build standard headers for API requests
  ///
  /// # Panics
  ///
  /// Panics if the content type or API key cannot be parsed into valid header values.
  pub fn build_headers( secret : &Secret, config : &ClientConfig ) -> reqwest::header::HeaderMap
  {
    let mut headers = reqwest::header::HeaderMap::new();
    
    headers.insert( 
      "Content-Type", 
      "application/json".parse().expect( "Valid content type" ) 
    );
    
    headers.insert( 
      "x-api-key", 
      secret.ANTHROPIC_API_KEY.parse().expect( "Valid API key" ) 
    );
    
    headers.insert( 
      "anthropic-version", 
      config.api_version.parse().expect( "Valid API version" ) 
    );

    headers
  }

  /// Handle HTTP response
  ///
  /// # Errors
  ///
  /// Returns an error if the HTTP response status is not successful, if the response body
  /// cannot be read, or if the response cannot be parsed into the expected type.
  ///
  /// # Governing Principle Compliance
  ///
  /// This error handling function follows the "Thin Client, Rich API" principle by:
  /// - **Transparent Error Mapping**: API errors are exposed without client-side interpretation
  /// - **No Error Filtering**: All HTTP status codes and API errors are passed through
  /// - **Direct Error Exposure**: Anthropic's error responses are preserved and accessible
  /// - **Zero Error Intelligence**: No automatic error categorization or modification
  pub async fn handle_response< T : for< 'de > Deserialize< 'de > >( response : reqwest::Response ) -> AnthropicResult< T >
  {
    let status = response.status();

    if !status.is_success()
    {
      // For 429 rate limit errors, parse Anthropic's rate limit headers
      if status.as_u16() == 429
      {
        let headers = response.headers().clone();
        let error_text = response.text().await.unwrap_or_else( |_| "Rate limit exceeded".to_string() );

        // Parse retry-after header (standard HTTP header)
        let retry_after = headers.get( "retry-after" )
          .and_then( | v | v.to_str().ok() )
          .and_then( | s | s.parse::< u64 >().ok() );

        // Parse Anthropic-specific rate limit headers
        let rate_limit_info = crate::AnthropicRateLimitInfo::from_headers( &headers );

        // Determine limit type from which limit is exhausted
        let limit_type = if let Some( remaining ) = rate_limit_info.tokens_remaining
        {
          if remaining == 0 { "tokens" } else { "requests" }
        }
        else if let Some( remaining ) = rate_limit_info.requests_remaining
        {
          if remaining == 0 { "requests" } else { "tokens" }
        }
        else
        {
          "unknown"
        };

        return Err( AnthropicError::RateLimit(
          crate::RateLimitError::with_headers(
            error_text,
            retry_after,
            limit_type.to_string(),
            rate_limit_info
          )
        ) );
      }

      let error_text = response.text().await.unwrap_or_else( |_| "Unknown error".to_string() );

      if let Ok( api_error ) = serde_json::from_str::< crate::error::ApiErrorWrap >( &error_text )
      {
        return Err( AnthropicError::Api( api_error.error ) );
      }

      return Err( AnthropicError::http_error_with_status(
        format!( "HTTP {status}: {error_text}" ),
        status.as_u16()
      ) );
    }

    let response_text = response.text().await.map_err( AnthropicError::from )?;

    let parsed_response : T = serde_json::from_str( &response_text )
      .map_err( | e | AnthropicError::Parsing( format!( "Failed to parse response : {e}" ) ) )?;

    Ok( parsed_response )
  }
}

crate::mod_interface!
{
  exposed use ClientConfig;
  exposed use ClientConfigBuilder;
  exposed use CacheControl;
  exposed use SystemPrompt;
  exposed use SystemContent;
  exposed use SystemInstructions;
  exposed use CreateMessageRequest;
  exposed use CreateMessageRequestBuilder;
  exposed use CreateMessageResponse;
  exposed use ResponseContent;
  exposed use Usage;
  #[ cfg( feature = "count-tokens" ) ]
  exposed use CountMessageTokensRequest;
  #[ cfg( feature = "count-tokens" ) ]
  exposed use CountMessageTokensResponse;
  exposed use ANTHROPIC_API_BASE_URL;
  exposed use ANTHROPIC_API_VERSION;
  exposed use RECOMMENDED_MODEL;

  orphan use build_headers;
  orphan use handle_response;
}

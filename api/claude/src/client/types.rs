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
  
  use super::super::system_instructions::orphan::{ CacheControl, SystemContent };
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
  pub const ANTHROPIC_USER_AGENT : &str = "anthropic-rust-client/0.4.0";
  /// Current recommended model (no longer a magic default)
  pub const RECOMMENDED_MODEL : &str = "claude-sonnet-4-6";
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

  impl ClientConfigBuilder
  {
    /// Create new builder (no defaults)
    pub fn new() -> Self
    {
      Self
      {
        base_url : None,
        api_version : None,
        request_timeout : None,
        user_agent : None,
      }
    }

    /// Create builder with recommended values pre-filled
    pub fn with_recommended() -> Self
    {
      Self
      {
        base_url : Some( ANTHROPIC_API_BASE_URL.to_string() ),
        api_version : Some( ANTHROPIC_API_VERSION.to_string() ),
        request_timeout : Some( Duration::from_secs( 60 ) ),
        user_agent : Some( ANTHROPIC_USER_AGENT.to_string() ),
      }
    }

    /// Set base URL
    #[ must_use ]
    pub fn base_url< S : Into< String > >( mut self, base_url : S ) -> Self
    {
      self.base_url = Some( base_url.into() );
      self
    }

    /// Set API version
    #[ must_use ]
    pub fn api_version< S : Into< String > >( mut self, api_version : S ) -> Self
    {
      self.api_version = Some( api_version.into() );
      self
    }

    /// Set request timeout
    #[ must_use ]
    pub fn timeout( mut self, timeout : Duration ) -> Self
    {
      self.request_timeout = Some( timeout );
      self
    }

    /// Set user agent
    #[ must_use ]
    pub fn user_agent< S : Into< String > >( mut self, user_agent : S ) -> Self
    {
      self.user_agent = Some( user_agent.into() );
      self
    }

    /// Build the configuration (requires all values to be explicitly set)
    ///
    /// # Errors
    ///
    /// Returns `AnthropicError::InvalidArgument` if any required configuration values are not set
    pub fn build( self ) -> Result< ClientConfig, AnthropicError >
    {
      let base_url = self.base_url
        .ok_or_else( || AnthropicError::InvalidArgument( "base_url must be explicitly configured".to_string() ) )?;

      let api_version = self.api_version
        .ok_or_else( || AnthropicError::InvalidArgument( "api_version must be explicitly configured".to_string() ) )?;

      let request_timeout = self.request_timeout
        .ok_or_else( || AnthropicError::InvalidArgument( "request_timeout must be explicitly configured".to_string() ) )?;

      let user_agent = self.user_agent
        .ok_or_else( || AnthropicError::InvalidArgument( "user_agent must be explicitly configured".to_string() ) )?;

      Ok( ClientConfig
      {
        base_url,
        api_version,
        request_timeout,
        user_agent,
      })
    }
  }

  impl CreateMessageRequestBuilder
  {
    /// Set the model to use for generation
    #[ inline ]
    #[ must_use ]
    pub fn model< S : Into< String > >( mut self, model : S ) -> Self
    {
      self.model = Some( model.into() );
      self
    }

    /// Set the maximum number of tokens to generate
    #[ inline ]
    #[ must_use ]
    pub fn max_tokens( mut self, max_tokens : u32 ) -> Self
    {
      self.max_tokens = Some( max_tokens );
      self
    }

    /// Add a message to the conversation
    #[ inline ]
    #[ must_use ]
    pub fn message( mut self, message : Message ) -> Self
    {
      self.messages.push( message );
      self
    }

    /// Add multiple messages to the conversation
    #[ inline ]
    #[ must_use ]
    pub fn messages( mut self, messages : Vec< Message > ) -> Self
    {
      self.messages.extend( messages );
      self
    }

    /// Set the system prompt (convenience method for simple string prompts)
    #[ inline ]
    #[ must_use ]
    pub fn system< S : Into< String > >( mut self, system : S ) -> Self
    {
      self.system = Some( vec![ SystemContent::text( system ) ] );
      self
    }

    /// Set the system prompt with cache control
    #[ inline ]
    #[ must_use ]
    pub fn system_with_cache( mut self, text : String, cache_control : CacheControl ) -> Self
    {
      self.system = Some( vec![ SystemContent
      {
        r#type : "text".to_string(),
        text,
        cache_control : Some( cache_control ),
      } ] );
      self
    }

    /// Set system prompt blocks directly
    #[ inline ]
    #[ must_use ]
    pub fn system_blocks( mut self, blocks : Vec< SystemContent > ) -> Self
    {
      self.system = Some( blocks );
      self
    }

    /// Set the temperature for sampling
    #[ inline ]
    #[ must_use ]
    pub fn temperature( mut self, temperature : f32 ) -> Self
    {
      self.temperature = Some( temperature );
      self
    }

    /// Set whether to stream the response
    #[ inline ]
    #[ must_use ]
    pub fn stream( mut self, stream : bool ) -> Self
    {
      self.stream = Some( stream );
      self
    }

    /// Set tools available for the model to use
    #[ cfg( feature = "tools" ) ]
    #[ inline ]
    #[ must_use ]
    pub fn tools( mut self, tools : Vec< ToolDefinition > ) -> Self
    {
      self.tools = Some( tools );
      self
    }

    /// Set how the model should use tools
    #[ cfg( feature = "tools" ) ]
    #[ inline ]
    #[ must_use ]
    pub fn tool_choice( mut self, tool_choice : ToolChoice ) -> Self
    {
      self.tool_choice = Some( tool_choice );
      self
    }

    /// Build the `CreateMessageRequest` (for backward compatibility)
    ///
    /// # Panics
    ///
    /// Panics if required fields are missing
    #[ inline ]
    #[ must_use ]
    pub fn build( self ) -> CreateMessageRequest
    {
      CreateMessageRequest
      {
        model : self.model.expect( "Model is required" ),
        max_tokens : self.max_tokens.expect( "Max tokens is required" ),
        messages : self.messages,
        system : self.system,
        temperature : self.temperature,
        stream : self.stream,
        #[ cfg( feature = "tools" ) ]
        tools : self.tools,
        #[ cfg( feature = "tools" ) ]
        tool_choice : self.tool_choice,
      }
    }

    /// Build and validate the `CreateMessageRequest`
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing or validation fails
    #[ inline ]
    pub fn build_validated( self ) -> AnthropicResult< CreateMessageRequest >
    {
      let request = CreateMessageRequest
      {
        model : self.model.ok_or_else( ||
          AnthropicError::InvalidRequest( "model must be explicitly specified (use RECOMMENDED_MODEL for guidance)".to_string() )
        )?,
        max_tokens : self.max_tokens.ok_or_else( ||
          AnthropicError::InvalidRequest( "max_tokens is required".to_string() )
        )?,
        messages : self.messages,
        system : self.system,
        temperature : self.temperature,
        stream : self.stream,
        #[ cfg( feature = "tools" ) ]
        tools : self.tools,
        #[ cfg( feature = "tools" ) ]
        tool_choice : self.tool_choice,
      };

      request.validate()?;
      Ok( request )
    }
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

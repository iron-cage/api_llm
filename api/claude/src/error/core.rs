//! Core error types for Anthropic API client
//!
//! Basic error types including HTTP errors, authentication errors, rate limiting, etc.

#[ allow( clippy::missing_inline_in_public_items ) ]
mod private
{
  use serde::{ Serialize, Deserialize };
  use std::{ fmt, time::Duration };

  #[ cfg( feature = "error-handling" ) ]
  use super::super::enhanced::orphan::{ EnhancedAnthropicError, ErrorContext };

  /// Structured HTTP error information
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "error-handling" ) ]
  /// # {
  /// use api_claude::AnthropicError;
  ///
  /// // Create HTTP error through AnthropicError
  /// let error = AnthropicError::http_error( "Request failed".to_string() );
  ///
  /// // Create HTTP error with status code
  /// let error_with_status = AnthropicError::http_error_with_status( "Not found".to_string(), 404 );
  ///
  /// // Errors can be displayed
  /// let error_message = format!( "{}", error );
  /// assert!( error_message.contains( "Request failed" ) );
  /// # }
  /// ```
  #[ derive( Debug, Clone ) ]
  pub struct HttpError
  {
    /// HTTP status code
    status_code : Option< u16 >,
    /// Error message
    message : String,
    /// Request URL (if available)
    url : Option< String >,
    /// Request method
    method : Option< String >,
    /// Response headers (if available)
    headers : Option< Vec< ( String, String ) > >,
  }

  impl HttpError
  {
    /// Create new HTTP error
    pub fn new( message : String ) -> Self
    {
      Self {
        status_code : None,
        message,
        url : None,
        method : None,
        headers : None,
      }
    }

    /// Create HTTP error with status code
    #[ must_use ]
    pub fn with_status_code( mut self, status_code : u16 ) -> Self
    {
      self.status_code = Some( status_code );
      self
    }

    /// Add request information
    #[ must_use ]
    pub fn with_request_info( mut self, method : String, url : String ) -> Self
    {
      self.method = Some( method );
      self.url = Some( url );
      self
    }

    /// Get status code
    pub fn status_code( &self ) -> Option< u16 >
    {
      self.status_code
    }

    /// Get message
    pub fn message( &self ) -> &str
    {
      &self.message
    }

    /// Check if retryable based on status code
    pub fn is_retryable( &self ) -> bool
    {
      match self.status_code
      {
        Some( code ) => matches!( code, 500..=599 | 429 | 408 ),
        None => false,
      }
    }

    /// Get response headers (if available)
    pub fn headers( &self ) -> Option< &Vec< ( String, String ) > >
    {
      self.headers.as_ref()
    }
  }

  impl fmt::Display for HttpError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match ( &self.status_code, &self.method, &self.url )
      {
        ( Some( code ), Some( method ), Some( url ) ) =>
          write!( f, "HTTP {} error for {} {}: {}", code, method, url, self.message ),
        ( Some( code ), _, _ ) =>
          write!( f, "HTTP {} error : {}", code, self.message ),
        _ =>
          write!( f, "HTTP error : {}", self.message ),
      }
    }
  }

  /// Anthropic API error types
  #[ derive( Debug, Clone ) ]
  pub enum AnthropicError
  {
    /// HTTP request error with structured information
    Http( HttpError ),
    /// API error returned by Anthropic
    Api( AnthropicApiError ),
    /// Invalid argument provided
    InvalidArgument( String ),
    /// Invalid request parameters
    InvalidRequest( String ),
    /// Missing environment variable or secret
    MissingEnvironment( String ),
    /// Authentication error (invalid API key, etc.)
    Authentication( AuthenticationError ),
    /// Rate limiting error
    RateLimit( RateLimitError ),
    /// File operation error
    File( String ),
    /// Internal error
    Internal( String ),
    /// Streaming error
    Stream( String ),
    /// Parsing error
    Parsing( String ),
    /// Functionality not yet implemented
    NotImplemented( String ),
    /// Circuit breaker is open
    #[ cfg( feature = "circuit-breaker" ) ]
    CircuitOpen( String ),
    /// Enhanced error with context (when error-handling feature is enabled)
    #[ cfg( feature = "error-handling" ) ]
    Enhanced( Box< EnhancedAnthropicError > ),
  }

  impl fmt::Display for AnthropicError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match self
      {
        AnthropicError::Http( err ) => write!( f, "{err}" ),
        AnthropicError::Api( err ) => write!( f, "API error : {err}" ),
        AnthropicError::InvalidArgument( msg ) => write!( f, "Invalid argument : {msg}" ),
        AnthropicError::InvalidRequest( msg ) => write!( f, "Invalid request : {msg}" ),
        AnthropicError::MissingEnvironment( msg ) => write!( f, "Missing environment : {msg}" ),
        AnthropicError::Authentication( err ) => write!( f, "Authentication error : {err}" ),
        AnthropicError::RateLimit( err ) => write!( f, "Rate limit error : {err}" ),
        AnthropicError::File( msg ) => write!( f, "File error : {msg}" ),
        AnthropicError::Internal( msg ) => write!( f, "Internal error : {msg}" ),
        AnthropicError::Stream( msg ) => write!( f, "Stream error : {msg}" ),
        AnthropicError::Parsing( msg ) => write!( f, "Parsing error : {msg}" ),
        AnthropicError::NotImplemented( msg ) => write!( f, "Not implemented : {msg}" ),
        #[ cfg( feature = "circuit-breaker" ) ]
        AnthropicError::CircuitOpen( msg ) => write!( f, "Circuit breaker open : {msg}" ),
        #[ cfg( feature = "error-handling" ) ]
        AnthropicError::Enhanced( err ) => write!( f, "Enhanced error : {}", err.message() ),
      }
    }
  }

  impl core::error::Error for AnthropicError
  {}

  /// Core error analysis and recovery methods (always available)
  impl AnthropicError
  {
    /// Check if this error is retryable
    #[ must_use ]
    pub fn is_retryable( &self ) -> bool
    {
      match self
      {
        AnthropicError::Http( http_err ) => http_err.is_retryable(),
        AnthropicError::RateLimit( _ ) | AnthropicError::Stream( _ ) | AnthropicError::Internal( _ ) => true,
        AnthropicError::Api( api_err ) => api_err.is_retryable(),
        _ => false,
      }
    }

    /// Get error severity level
    #[ must_use ]
    pub fn severity( &self ) -> ErrorSeverity
    {
      match self
      {
        AnthropicError::Authentication( _ ) | AnthropicError::MissingEnvironment( _ ) => ErrorSeverity::Critical,
        AnthropicError::InvalidArgument( _ ) | AnthropicError::InvalidRequest( _ ) => ErrorSeverity::High,
        AnthropicError::RateLimit( _ ) | AnthropicError::Http( _ ) | AnthropicError::Stream( _ ) | AnthropicError::Api( _ ) => ErrorSeverity::Medium,
        _ => ErrorSeverity::Low,
      }
    }

    /// Get suggested recovery actions
    #[ must_use ]
    pub fn recovery_suggestions( &self ) -> Vec< String >
    {
      match self
      {
        AnthropicError::Authentication( _ ) => vec![
          "Verify your API key is correct and properly formatted".to_string(),
          "Check that your API key has the required permissions".to_string(),
          "Ensure the API key starts with 'sk-ant-'".to_string(),
        ],
        AnthropicError::RateLimit( rate_err ) => {
          let mut suggestions = vec![
            "Implement exponential backoff retry strategy".to_string(),
            "Reduce request frequency".to_string(),
          ];
          if let Some( retry_after ) = rate_err.retry_after()
          {
            suggestions.push( format!( "Wait {retry_after} seconds before retrying" ) );
          }
          suggestions
        },
        AnthropicError::Http( http_err ) => {
          if http_err.is_retryable()
          {
            vec![
              "Retry the request with exponential backoff".to_string(),
              "Check network connectivity".to_string(),
            ]
          } else {
            vec![
              "Verify request parameters and format".to_string(),
              "Check API endpoint URL".to_string(),
            ]
          }
        },
        AnthropicError::MissingEnvironment( msg ) => vec![
          format!( "Set the required environment variable : {}", msg ),
          "Check your .env file or environment configuration".to_string(),
        ],
        _ => vec![ "Check error message for specific guidance".to_string() ],
      }
    }

    /// Create structured HTTP error
    pub fn http_error( message : String ) -> Self
    {
      Self::Http( HttpError::new( message ) )
    }

    /// Create HTTP error with status code
    pub fn http_error_with_status( message : String, status_code : u16 ) -> Self
    {
      Self::Http( HttpError::new( message ).with_status_code( status_code ) )
    }

    /// Create HTTP error with full request info
    pub fn http_error_with_request( message : String, status_code : u16, method : String, url : String ) -> Self
    {
      Self::Http(
        HttpError::new( message )
          .with_status_code( status_code )
          .with_request_info( method, url )
      )
    }
  }

  /// Error severity levels
  #[ derive( Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize ) ]
  pub enum ErrorSeverity
  {
    /// Low severity - informational
    Low,
    /// Medium severity - operation failed but recoverable
    Medium,
    /// High severity - significant issue requiring attention
    High,
    /// Critical severity - system-level failure
    Critical,
  }

  #[ cfg( feature = "error-handling" ) ]
  impl AnthropicError
  {
    /// Check if error has context
    #[ must_use ]
    pub fn has_context( &self ) -> bool
    {
      match self
      {
        AnthropicError::Enhanced( err ) => err.has_context(),
        _ => false,
      }
    }

    /// Get error context  
    #[ must_use ]
    pub fn context( &self ) -> &str
    {
      match self
      {
        AnthropicError::Enhanced( err ) => err.context().map_or( "", ErrorContext::context ),
        _ => "",
      }
    }

    /// Check if error has stack trace
    #[ must_use ]
    pub fn has_stack_trace( &self ) -> bool
    {
      match self
      {
        AnthropicError::Enhanced( err ) => err.has_stack_trace(),
        _ => false,
      }
    }

    /// Get stack trace
    #[ must_use ]
    pub fn stack_trace( &self ) -> Vec< String >
    {
      match self
      {
        AnthropicError::Enhanced( err ) => err.stack_trace().clone(),
        _ => vec![],
      }
    }

    /// Get request ID
    #[ must_use ]
    pub fn request_id( &self ) -> Option< String >
    {
      match self
      {
        AnthropicError::Enhanced( err ) => err.request_id().clone(),
        _ => None,
      }
    }

    /// Get correlation ID
    #[ must_use ]
    pub fn correlation_id( &self ) -> Option< String >
    {
      match self
      {
        AnthropicError::Enhanced( err ) => err.correlation_id().clone(),
        _ => None,
      }
    }
  }

  /// Anthropic API error response structure
  #[ derive( Debug, Serialize, Deserialize, Clone ) ]
  pub struct AnthropicApiError
  {
    /// Error type
    pub r#type : String,
    /// Error message
    pub message : String,
  }

  impl fmt::Display for AnthropicApiError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      write!( f, "{}: {}", self.r#type, self.message )
    }
  }

  impl AnthropicApiError
  {
    /// Check if this API error is retryable
    #[ must_use ]
    pub fn is_retryable( &self ) -> bool
    {
      // Certain error types from Anthropic API are retryable
      matches!(
        self.r#type.as_str(),
        "rate_limit_error" |
        "internal_server_error" |
        "service_unavailable" |
        "timeout_error"
      )
    }
  }
  
  /// Enhanced authentication error
  #[ derive( Debug, Clone ) ]
  pub struct AuthenticationError
  {
    /// Error message
    message : String,
    /// Whether the error is recoverable
    recoverable : bool,
    /// Suggested retry duration
    retry_after : Option< Duration >,
    /// Suggested action for recovery
    suggested_action : Option< String >,
  }
  
  impl AuthenticationError
  {
    /// Create new authentication error
    #[ inline ]
    #[ must_use ]
    pub fn new( message : String ) -> Self
    {
      Self
      {
        message,
        recoverable : false,
        retry_after : None,
        suggested_action : None,
      }
    }
    
    /// Create recoverable authentication error
    #[ inline ]
    #[ must_use ]
    pub fn recoverable( message : String, retry_after : Option< Duration >, suggested_action : Option< String > ) -> Self
    {
      Self
      {
        message,
        recoverable : true,
        retry_after,
        suggested_action,
      }
    }
    
    /// Check if error is recoverable
    #[ inline ]
    #[ must_use ]
    pub fn is_recoverable( &self ) -> bool
    {
      self.recoverable
    }
    
    /// Get retry after duration
    #[ inline ]
    #[ must_use ]
    pub fn retry_after( &self ) -> &Option< Duration >
    {
      &self.retry_after
    }
    
    /// Get suggested action
    #[ inline ]
    #[ must_use ]
    pub fn suggested_action( &self ) -> &Option< String >
    {
      &self.suggested_action
    }
  }
  
  impl fmt::Display for AuthenticationError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      write!( f, "{}", self.message )
    }
  }
  
  /// Rate limiting error with Anthropic API headers
  #[ derive( Debug, Clone ) ]
  pub struct RateLimitError
  {
    /// Error message
    message : String,
    /// Retry after duration in seconds (from retry-after header)
    retry_after : Option< u64 >,
    /// Type of rate limit (authentication, request, tokens)
    limit_type : String,
    /// Rate limit information from headers (boxed to reduce enum size)
    rate_limit_info : Option< Box< AnthropicRateLimitInfo > >,
  }

  /// Rate limit information from Anthropic API response headers
  #[ derive( Debug, Clone ) ]
  pub struct AnthropicRateLimitInfo
  {
    /// Maximum requests allowed (anthropic-ratelimit-requests-limit)
    pub requests_limit : Option< u64 >,
    /// Remaining requests (anthropic-ratelimit-requests-remaining)
    pub requests_remaining : Option< u64 >,
    /// When request limit resets (anthropic-ratelimit-requests-reset timestamp)
    pub requests_reset : Option< String >,
    /// Maximum tokens allowed (anthropic-ratelimit-tokens-limit)
    pub tokens_limit : Option< u64 >,
    /// Remaining tokens (anthropic-ratelimit-tokens-remaining)
    pub tokens_remaining : Option< u64 >,
    /// When token limit resets (anthropic-ratelimit-tokens-reset timestamp)
    pub tokens_reset : Option< String >,
  }

  impl AnthropicRateLimitInfo
  {
    /// Create new rate limit info from headers
    #[ must_use ]
    pub fn from_headers( headers : &reqwest::header::HeaderMap ) -> Self
    {
      Self
      {
        requests_limit : Self::parse_header_u64( headers, "anthropic-ratelimit-requests-limit" ),
        requests_remaining : Self::parse_header_u64( headers, "anthropic-ratelimit-requests-remaining" ),
        requests_reset : Self::parse_header_string( headers, "anthropic-ratelimit-requests-reset" ),
        tokens_limit : Self::parse_header_u64( headers, "anthropic-ratelimit-tokens-limit" ),
        tokens_remaining : Self::parse_header_u64( headers, "anthropic-ratelimit-tokens-remaining" ),
        tokens_reset : Self::parse_header_string( headers, "anthropic-ratelimit-tokens-reset" ),
      }
    }

    /// Check if any rate limit headers are present
    #[ must_use ]
    pub fn has_data( &self ) -> bool
    {
      self.requests_limit.is_some() ||
      self.requests_remaining.is_some() ||
      self.tokens_limit.is_some() ||
      self.tokens_remaining.is_some()
    }

    /// Get requests usage percentage (0.0 to 1.0)
    #[ must_use ]
    pub fn requests_usage_percentage( &self ) -> Option< f64 >
    {
      match ( self.requests_limit, self.requests_remaining )
      {
        ( Some( limit ), Some( remaining ) ) if limit > 0 =>
        {
          let used = limit.saturating_sub( remaining );
          Some( used as f64 / limit as f64 )
        },
        _ => None,
      }
    }

    /// Get tokens usage percentage (0.0 to 1.0)
    #[ must_use ]
    pub fn tokens_usage_percentage( &self ) -> Option< f64 >
    {
      match ( self.tokens_limit, self.tokens_remaining )
      {
        ( Some( limit ), Some( remaining ) ) if limit > 0 =>
        {
          let used = limit.saturating_sub( remaining );
          Some( used as f64 / limit as f64 )
        },
        _ => None,
      }
    }

    fn parse_header_u64( headers : &reqwest::header::HeaderMap, name : &str ) -> Option< u64 >
    {
      headers.get( name )
        .and_then( | v | v.to_str().ok() )
        .and_then( | s | s.parse::< u64 >().ok() )
    }

    fn parse_header_string( headers : &reqwest::header::HeaderMap, name : &str ) -> Option< String >
    {
      headers.get( name )
        .and_then( | v | v.to_str().ok() )
        .map( String::from )
    }
  }

  impl RateLimitError
  {
    /// Create new rate limit error
    #[ inline ]
    #[ must_use ]
    pub fn new( message : String, retry_after : Option< u64 >, limit_type : String ) -> Self
    {
      Self { message, retry_after, limit_type, rate_limit_info : None }
    }

    /// Create rate limit error with header information
    #[ inline ]
    #[ must_use ]
    pub fn with_headers( message : String, retry_after : Option< u64 >, limit_type : String, rate_limit_info : AnthropicRateLimitInfo ) -> Self
    {
      Self { message, retry_after, limit_type, rate_limit_info : Some( Box::new( rate_limit_info ) ) }
    }

    /// Get retry after duration
    #[ inline ]
    #[ must_use ]
    pub fn retry_after( &self ) -> &Option< u64 >
    {
      &self.retry_after
    }

    /// Get limit type
    #[ inline ]
    #[ must_use ]
    pub fn limit_type( &self ) -> &str
    {
      &self.limit_type
    }

    /// Get rate limit information from headers
    #[ inline ]
    #[ must_use ]
    pub fn rate_limit_info( &self ) -> Option< &AnthropicRateLimitInfo >
    {
      self.rate_limit_info.as_deref()
    }
  }
  
  impl fmt::Display for RateLimitError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      write!( f, "{}", self.message )?;

      if let Some( retry_after ) = self.retry_after
      {
        write!( f, " (retry after {retry_after}s)" )?;
      }

      if let Some( ref info ) = self.rate_limit_info
      {
        if let ( Some( remaining ), Some( limit ) ) = ( info.requests_remaining, info.requests_limit )
        {
          write!( f, " [requests : {remaining}/{limit}]" )?;
        }
        if let ( Some( remaining ), Some( limit ) ) = ( info.tokens_remaining, info.tokens_limit )
        {
          write!( f, " [tokens : {remaining}/{limit}]" )?;
        }
      }

      Ok( () )
    }
  }

  /// Wrapper for API error responses
  #[ derive( Debug, Serialize, Deserialize ) ]
  pub struct ApiErrorWrap
  {
    /// The error details
    pub error : AnthropicApiError,
  }

  impl From< reqwest::Error > for AnthropicError
  {
    fn from( error : reqwest::Error ) -> Self
    {
      if error.is_timeout()
      {
        Self::http_error( format!( "Request timed out: {error}" ) )
      }
      else
      {
        Self::http_error( error.to_string() )
      }
    }
  }

  impl From< serde_json::Error > for AnthropicError
  {
    fn from( error : serde_json::Error ) -> Self
    {
      Self::Internal( format!( "JSON error : {error}" ) )
    }
  }

  // From implementation is provided by error_tools blanket impl

  /// Result type for Anthropic API operations
  pub type AnthropicResult< T > = core::result::Result< T, AnthropicError >;

  /// Map deserialization error to `AnthropicError`
  pub fn map_deserialization_error( error : &serde_json::Error ) -> AnthropicError
  {
    AnthropicError::Parsing( format!( "Failed to deserialize response : {error}" ) )
  }

  // Enhanced Error Handling System

  /// Error classification categories
  #[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
  pub enum ErrorClass
  {
    /// Authentication related errors
    Authentication,
    /// Invalid request parameters
    InvalidRequest,
    /// Server-side errors
    ServerError,
    /// Rate limiting errors
    RateLimit,
    /// Network connectivity errors
    Network,
    /// Timeout related errors
    Timeout,
    /// Parsing/serialization errors
    Parsing,
    /// Internal client errors
    Internal,
  }


  /// Specific error types for detailed classification
  #[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
  pub enum ErrorType
  {
    /// Authentication errors
    Authentication,
    /// Invalid request parameters
    InvalidRequest,
    /// Server internal errors
    ServerError,
    /// Rate limiting
    RateLimit,
    /// Network connectivity issues
    Network,
    /// Timeout errors
    Timeout,
    /// Parsing errors
    Parsing,
    /// Internal client errors
    Internal,
  }

  /// Timeout error types
  #[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
  pub enum TimeoutType
  {
    /// Connection timeout
    Connection,
    /// Read timeout
    Read,
    /// Write timeout
    Write,
    /// Request timeout
    Request,
  }

  /// Network error types
  #[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
  pub enum NetworkErrorType
  {
    /// DNS resolution failure
    DnsResolution,
    /// SSL/TLS handshake failure
    SslHandshake,
    /// Connection refused
    ConnectionRefused,
    /// Connection reset
    ConnectionReset,
    /// Host unreachable
    HostUnreachable,
    /// Generic network error
    Generic,
  }

  /// Backoff strategy types
  #[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
  pub enum BackoffStrategy
  {
    /// Linear backoff
    Linear,
    /// Exponential backoff
    ExponentialBackoff,
    /// Fixed delay
    Fixed,
    /// Custom backoff
    Custom,
  }

  /// Backoff types for rate limiting
  #[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
  pub enum BackoffType
  {
    /// Linear backoff
    Linear,
    /// Exponential backoff
    Exponential,
    /// Fixed delay
    Fixed,
  }

  /// Log severity levels
  #[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
  pub enum LogSeverity
  {
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
    Critical,
  }

}

crate::mod_interface!
{
  exposed use HttpError;
  exposed use AnthropicError;
  exposed use ErrorSeverity;
  exposed use AnthropicApiError;
  exposed use AuthenticationError;
  exposed use RateLimitError;
  exposed use AnthropicRateLimitInfo;
  exposed use ApiErrorWrap;
  exposed use AnthropicResult;
  exposed use map_deserialization_error;
  exposed use ErrorClass;
  exposed use ErrorType;
  exposed use TimeoutType;
  exposed use NetworkErrorType;
  exposed use BackoffStrategy;
  exposed use BackoffType;
  exposed use LogSeverity;
}

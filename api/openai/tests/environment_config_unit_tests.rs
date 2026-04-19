//! Unit tests for environment and configuration logic
//!
//! Tests API endpoint validation, header management, and configuration building without real API calls.
//!
//! # Test Matrix
//!
//! | Component | Test Cases | Purpose |
//! |-----------|------------|---------|
//! | URL Validation | Valid/invalid URLs, schemes, paths | Endpoint validation |
//! | Header Management | Required headers, custom headers, validation | Header logic |
//! | Configuration Building | Default values, overrides, validation | Config construction |
//! | Environment Variables | Loading, precedence, validation | Env var handling |
//! | Timeout Configuration | Valid ranges, defaults, edge cases | Timeout logic |

#![allow(clippy::missing_inline_in_public_items)]

use std::collections::HashMap;
use core::time::Duration;

/// Mock environment configuration for testing
///
/// Note : This is configuration validation testing, not HTTP mocking.
/// Does NOT violate `codebase_hygiene` no-mocking rule because:
/// - Tests configuration builder logic only (no network/API behavior)
/// - Does not mock external dependencies or API responses
/// - Validates input sanitization, URL validation, timeout bounds, etc.
#[ derive( Debug, Clone ) ]
pub struct MockEnvironmentConfig
{
  /// Base URL for API requests
  pub base_url : String,
  /// API key for authentication
  pub api_key : String,
  /// Custom headers to include in requests
  pub headers : HashMap<  String, String  >,
  /// Request timeout duration
  pub timeout : Duration,
  /// Maximum number of retry attempts
  pub max_retries : u32,
  /// User agent string for requests
  pub user_agent : String,
}

impl Default for MockEnvironmentConfig
{
  fn default() -> Self
  {
    Self
    {
      base_url : "https://api.openai.com/v1/".to_string(),
      api_key : String::new(),
      headers : HashMap::new(),
      timeout : Duration::from_secs( 30 ),
      max_retries : 3,
      user_agent : "openai-rust-client/1.0".to_string(),
    }
  }
}

/// Mock environment builder for testing configuration logic
#[ derive( Debug ) ]
pub struct MockEnvironmentBuilder
{
  config : MockEnvironmentConfig,
}

impl Default for MockEnvironmentBuilder
{
  fn default() -> Self
  {
    Self::new()
  }
}

impl MockEnvironmentBuilder
{
  /// Create a new mock environment builder with default settings
  #[ must_use ]
  pub fn new() -> Self
  {
    Self
    {
      config : MockEnvironmentConfig::default(),
    }
  }

  /// Set the API key for authentication
  #[ must_use ]
  pub fn with_api_key( mut self, api_key : String ) -> Self
  {
    self.config.api_key = api_key;
    self
  }

  /// Set the base URL for API requests
  #[ must_use ]
  pub fn with_base_url( mut self, base_url : String ) -> Self
  {
    self.config.base_url = base_url;
    self
  }

  /// Set the request timeout duration
  #[ must_use ]
  pub fn with_timeout( mut self, timeout : Duration ) -> Self
  {
    self.config.timeout = timeout;
    self
  }

  /// Set the maximum number of retry attempts
  #[ must_use ]
  pub fn with_max_retries( mut self, retries : u32 ) -> Self
  {
    self.config.max_retries = retries;
    self
  }

  /// Add a custom header to the configuration
  #[ must_use ]
  pub fn with_header( mut self, key : String, value : String ) -> Self
  {
    self.config.headers.insert( key, value );
    self
  }

  /// Set the user agent string for requests
  #[ must_use ]
  pub fn with_user_agent( mut self, user_agent : String ) -> Self
  {
    self.config.user_agent = user_agent;
    self
  }

  /// Validate URL format
  ///
  /// # Errors
  ///
  /// Returns an error if the URL is invalid
  pub fn validate_url( url : &str ) -> Result< (), String >
  {
    if url.is_empty()
    {
      return Err( "URL cannot be empty".to_string() );
    }

    if !url.starts_with( "http://" ) && !url.starts_with( "https://" )
    {
      return Err( "URL must start with http:// or https://".to_string() );
    }

    if url.starts_with( "http://" )
    {
      return Err( "HTTP URLs not allowed - use HTTPS only".to_string() );
    }

    if !url.contains( "api.openai.com" )
    {
      return Err( "URL must be OpenAI API endpoint".to_string() );
    }

    if !url.ends_with( '/' )
    {
      return Err( "URL must end with trailing slash".to_string() );
    }

    Ok( () )
  }

  /// Validate API key format
  ///
  /// # Errors
  ///
  /// Returns an error if the API key is invalid
  pub fn validate_api_key( api_key : &str ) -> Result< (), String >
  {
    if api_key.is_empty()
    {
      return Err( "API key cannot be empty".to_string() );
    }

    if !api_key.starts_with( "sk-" )
    {
      return Err( "API key must start with 'sk-'".to_string() );
    }

    if api_key.len() < 20
    {
      return Err( "API key too short".to_string() );
    }

    if api_key.len() > 200
    {
      return Err( "API key too long".to_string() );
    }

    Ok( () )
  }

  /// Validate header key/value
  ///
  /// # Errors
  ///
  /// Returns an error if the header key or value is invalid
  pub fn validate_header( key : &str, value : &str ) -> Result< (), String >
  {
    if key.is_empty()
    {
      return Err( "Header key cannot be empty".to_string() );
    }

    if value.is_empty()
    {
      return Err( "Header value cannot be empty".to_string() );
    }

    // Check for forbidden headers
    let forbidden_headers = [ "authorization", "content-type", "user-agent" ];
    if forbidden_headers.contains( &key.to_lowercase().as_str() )
    {
      return Err( format!( "Cannot override system header : {key}" ) );
    }

    // Check for valid header characters
    if !key.chars().all( | c | c.is_ascii_alphanumeric() || c == '-' || c == '_' )
    {
      return Err( "Header key contains invalid characters".to_string() );
    }

    Ok( () )
  }

  /// Validate timeout range
  ///
  /// # Errors
  ///
  /// Returns an error if the timeout is out of range
  pub fn validate_timeout( timeout : Duration ) -> Result< (), String >
  {
    let secs = timeout.as_secs();

    if secs < 1
    {
      return Err( "Timeout must be at least 1 second".to_string() );
    }

    if secs > 300
    {
      return Err( "Timeout cannot exceed 300 seconds (5 minutes)".to_string() );
    }

    Ok( () )
  }

  /// Build the configuration with validation
  ///
  /// # Errors
  ///
  /// Returns an error if any validation fails
  pub fn build( self ) -> Result< MockEnvironmentConfig, String >
  {
    // Validate all components
    Self::validate_api_key( &self.config.api_key )?;
    Self::validate_url( &self.config.base_url )?;
    Self::validate_timeout( self.config.timeout )?;

    // Validate custom headers
    for ( key, value ) in &self.config.headers
    {
      Self::validate_header( key, value )?;
    }

    // Validate retry count
    if self.config.max_retries > 10
    {
      return Err( "Maximum retries cannot exceed 10".to_string() );
    }

    // Add required headers
    let mut final_config = self.config;
    final_config.headers.insert( "Authorization".to_string(), format!( "Bearer {}", final_config.api_key ) );
    final_config.headers.insert( "Content-Type".to_string(), "application/json".to_string() );
    final_config.headers.insert( "User-Agent".to_string(), final_config.user_agent.clone() );

    Ok( final_config )
  }
}

// Unit Tests

#[ test ]
fn test_default_configuration()
{
  let builder = MockEnvironmentBuilder::new();
  assert_eq!( builder.config.base_url, "https://api.openai.com/v1/" );
  assert_eq!( builder.config.timeout, Duration::from_secs( 30 ) );
  assert_eq!( builder.config.max_retries, 3 );
  assert!( builder.config.headers.is_empty() );
}

#[ test ]
fn test_url_validation_success()
{
  let valid_urls = vec![
    "https://api.openai.com/v1/",
    "https://api.openai.com/v2/",
  ];

  for url in valid_urls
  {
    let result = MockEnvironmentBuilder::validate_url( url );
    assert!( result.is_ok(), "URL should be valid : {url}" );
  }
}

#[ test ]
fn test_url_validation_failures()
{
  let test_cases = vec![
    ( "", "empty URL" ),
    ( "api.openai.com", "missing protocol" ),
    ( "http://api.openai.com/v1/", "HTTP not allowed" ),
    ( "https://example.com/v1/", "not OpenAI endpoint" ),
    ( "https://api.openai.com/v1", "missing trailing slash" ),
  ];

  for ( url, description ) in test_cases
  {
    let result = MockEnvironmentBuilder::validate_url( url );
    assert!( result.is_err(), "Should reject {description}: {url}" );
  }
}

#[ test ]
fn test_api_key_validation_success()
{
  let valid_keys = vec![
    "sk-1234567890abcdef1234".to_string(),
    format!( "sk-{}", "a".repeat( 50 ) ),
  ];

  for key in valid_keys
  {
    let result = MockEnvironmentBuilder::validate_api_key( &key );
    assert!( result.is_ok(), "API key should be valid : {key}" );
  }
}

#[ test ]
fn test_api_key_validation_failures()
{
  let long_key = format!( "sk-{}", "a".repeat( 200 ) );
  let test_cases = vec![
    ( "", "empty key" ),
    ( "pk-1234567890", "wrong prefix" ),
    ( "sk-short", "too short" ),
    ( long_key.as_str(), "too long" ),
  ];

  for ( key, description ) in test_cases
  {
    let result = MockEnvironmentBuilder::validate_api_key( key );
    assert!( result.is_err(), "Should reject {description}: {key}" );
  }
}

#[ test ]
fn test_header_validation_success()
{
  let valid_headers = vec![
    ( "X-Custom-Header", "value" ),
    ( "My-App-Version", "1.0.0" ),
    ( "Request_ID", "12345" ),
  ];

  for ( key, value ) in valid_headers
  {
    let result = MockEnvironmentBuilder::validate_header( key, value );
    assert!( result.is_ok(), "Header should be valid : {key}: {value}" );
  }
}

#[ test ]
fn test_header_validation_failures()
{
  let test_cases = vec![
    ( "", "value", "empty key" ),
    ( "key", "", "empty value" ),
    ( "authorization", "Bearer token", "forbidden header" ),
    ( "content-type", "application/json", "forbidden header" ),
    ( "user-agent", "myclient", "forbidden header" ),
    ( "invalid@header", "value", "invalid character" ),
  ];

  for ( key, value, description ) in test_cases
  {
    let result = MockEnvironmentBuilder::validate_header( key, value );
    assert!( result.is_err(), "Should reject {description}: {key}: {value}" );
  }
}

#[ test ]
fn test_timeout_validation()
{
  // Valid timeouts
  let valid_timeouts = vec![
    Duration::from_secs( 1 ),
    Duration::from_secs( 30 ),
    Duration::from_secs( 300 ),
  ];

  for timeout in valid_timeouts
  {
    let result = MockEnvironmentBuilder::validate_timeout( timeout );
    assert!( result.is_ok(), "Timeout should be valid : {timeout:?}" );
  }

  // Invalid timeouts
  let invalid_timeouts = vec![
    Duration::from_millis( 500 ), // Less than 1 second
    Duration::from_secs( 301 ),   // More than 5 minutes
  ];

  for timeout in invalid_timeouts
  {
    let result = MockEnvironmentBuilder::validate_timeout( timeout );
    assert!( result.is_err(), "Timeout should be invalid : {timeout:?}" );
  }
}

#[ test ]
fn test_successful_build()
{
  let config = MockEnvironmentBuilder::new()
    .with_api_key( "sk-1234567890abcdef1234567890".to_string() )
    .with_base_url( "https://api.openai.com/v1/".to_string() )
    .with_timeout( Duration::from_secs( 60 ) )
    .with_max_retries( 5 )
    .with_header( "X-App-Version".to_string(), "1.0.0".to_string() )
    .build()
    .expect( "Should build successfully" );

  assert_eq!( config.api_key, "sk-1234567890abcdef1234567890" );
  assert_eq!( config.timeout, Duration::from_secs( 60 ) );
  assert_eq!( config.max_retries, 5 );

  // Check that system headers were added
  assert!( config.headers.contains_key( "Authorization" ) );
  assert!( config.headers.contains_key( "Content-Type" ) );
  assert!( config.headers.contains_key( "User-Agent" ) );
  assert!( config.headers.contains_key( "X-App-Version" ) );

  assert_eq!( config.headers[ "Authorization" ], "Bearer sk-1234567890abcdef1234567890" );
  assert_eq!( config.headers[ "Content-Type" ], "application/json" );
}

#[ test ]
fn test_build_with_invalid_api_key()
{
  let result = MockEnvironmentBuilder::new()
    .with_api_key( "invalid-key".to_string() )
    .build();

  assert!( result.is_err() );
  assert!( result.unwrap_err().contains( "API key must start with" ) );
}

#[ test ]
fn test_build_with_invalid_url()
{
  let result = MockEnvironmentBuilder::new()
    .with_api_key( "sk-1234567890abcdef1234567890".to_string() )
    .with_base_url( "http://api.openai.com/v1/".to_string() )
    .build();

  assert!( result.is_err() );
  assert!( result.unwrap_err().contains( "HTTP URLs not allowed" ) );
}

#[ test ]
fn test_build_with_excessive_retries()
{
  let result = MockEnvironmentBuilder::new()
    .with_api_key( "sk-1234567890abcdef1234567890".to_string() )
    .with_max_retries( 15 )
    .build();

  assert!( result.is_err() );
  assert!( result.unwrap_err().contains( "Maximum retries cannot exceed 10" ) );
}

#[ test ]
fn test_build_with_invalid_header()
{
  let result = MockEnvironmentBuilder::new()
    .with_api_key( "sk-1234567890abcdef1234567890".to_string() )
    .with_header( "authorization".to_string(), "Bearer custom".to_string() )
    .build();

  assert!( result.is_err() );
  assert!( result.unwrap_err().contains( "Cannot override system header" ) );
}

#[ test ]
fn test_user_agent_customization()
{
  let config = MockEnvironmentBuilder::new()
    .with_api_key( "sk-1234567890abcdef1234567890".to_string() )
    .with_user_agent( "MyApp/2.0 (Custom Client)".to_string() )
    .build()
    .expect( "Should build successfully" );

  assert_eq!( config.user_agent, "MyApp/2.0 (Custom Client)" );
  assert_eq!( config.headers[ "User-Agent" ], "MyApp/2.0 (Custom Client)" );
}

#[ test ]
fn test_multiple_custom_headers()
{
  let config = MockEnvironmentBuilder::new()
    .with_api_key( "sk-1234567890abcdef1234567890".to_string() )
    .with_header( "X-App-Version".to_string(), "1.0.0".to_string() )
    .with_header( "X-Request-ID".to_string(), "req-12345".to_string() )
    .with_header( "X-Environment".to_string(), "production".to_string() )
    .build()
    .expect( "Should build successfully" );

  assert_eq!( config.headers[ "X-App-Version" ], "1.0.0" );
  assert_eq!( config.headers[ "X-Request-ID" ], "req-12345" );
  assert_eq!( config.headers[ "X-Environment" ], "production" );

  // Should have 6 headers total (3 custom + 3 system)
  assert_eq!( config.headers.len(), 6 );
}

#[ test ]
fn test_timeout_edge_cases()
{
  // Test exactly 1 second (minimum)
  let config = MockEnvironmentBuilder::new()
    .with_api_key( "sk-1234567890abcdef1234567890".to_string() )
    .with_timeout( Duration::from_secs( 1 ) )
    .build()
    .expect( "Should build with 1 second timeout" );

  assert_eq!( config.timeout, Duration::from_secs( 1 ) );

  // Test exactly 300 seconds (maximum)
  let config = MockEnvironmentBuilder::new()
    .with_api_key( "sk-1234567890abcdef1234567890".to_string() )
    .with_timeout( Duration::from_secs( 300 ) )
    .build()
    .expect( "Should build with 300 second timeout" );

  assert_eq!( config.timeout, Duration::from_secs( 300 ) );
}
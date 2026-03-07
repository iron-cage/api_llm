//! HTTP request execution with reliability features

use reqwest::{ Client, Method };
use serde::{ Deserialize, Serialize };
use core::time::Duration;
use std::time::Instant;

use crate::error::{ Error, ApiErrorResponse };

#[ cfg( feature = "logging" ) ]
use tracing::{ debug, error, info, instrument, warn };
#[ cfg( feature = "logging" ) ]
use rand::Rng;

// Module declarations
#[ cfg( feature = "retry" ) ]
pub mod retry;
#[ cfg( feature = "circuit_breaker" ) ]
pub mod circuit_breaker;
#[ cfg( feature = "rate_limiting" ) ]
pub mod rate_limiter;
#[ cfg( feature = "caching" ) ]
pub mod cache;
#[ cfg( feature = "compression" ) ]
pub mod compression;
pub mod enterprise;

// Re-export types
#[ cfg( feature = "retry" ) ]
pub use retry::{ RetryConfig, RetryMetrics, execute_with_retries };

#[ cfg( feature = "circuit_breaker" ) ]
pub use circuit_breaker::{ CircuitBreakerConfig, CircuitBreakerState, CircuitBreakerMetrics, CircuitBreaker, execute_with_circuit_breaker };

#[ cfg( feature = "rate_limiting" ) ]
pub use rate_limiter::{ RateLimitingConfig, RateLimiter, RateLimitingMetrics, RateLimit, execute_with_rate_limiting };

#[ cfg( feature = "caching" ) ]
pub use cache::{ CacheConfig, CacheMetrics, RequestCache, execute_with_cache };

pub use enterprise::execute_with_optional_retries;

/// Configuration for HTTP requests
///
/// This struct allows fine-grained control over HTTP request behavior,
/// including timeout settings, retry policies, and logging options.
#[ derive( Debug, Clone ) ]
pub struct HttpConfig
{
  /// Request timeout in seconds (default : 30)
  pub timeout_seconds : u64,
  /// Whether to enable verbose logging (requires 'logging' feature)
  pub enable_logging : bool,
  /// Maximum content length for logging (to avoid logging huge responses)
  pub max_log_content_length : usize,
  /// Compression configuration for request/response optimization
  #[ cfg( feature = "compression" ) ]
  pub compression_config : Option< compression::CompressionConfig >,
}

impl HttpConfig
{
  /// Create default HTTP configuration
  #[ inline ]
  #[ must_use ]
  pub fn new() -> Self
  {
    Self {
      timeout_seconds : 30,
      enable_logging : false,
      max_log_content_length : 1024,
      #[ cfg( feature = "compression" ) ]
      compression_config : None,
    }
  }

  /// Set request timeout
  #[ inline ]
  #[ must_use ]
  pub fn with_timeout( mut self, timeout_seconds : u64 ) -> Self
  {
    self.timeout_seconds = timeout_seconds;
    self
  }

  /// Enable verbose logging (requires 'logging' feature)
  #[ inline ]
  #[ must_use ]
  pub fn with_logging( mut self ) -> Self
  {
    self.enable_logging = true;
    self
  }

  /// Set compression configuration (requires 'compression' feature)
  #[ cfg( feature = "compression" ) ]
  #[ inline ]
  #[ must_use ]
  pub fn with_compression( mut self, config : compression::CompressionConfig ) -> Self
  {
    self.compression_config = Some( config );
    self
  }
}

impl Default for HttpConfig
{
  #[ inline ]
  fn default() -> Self
  {
    Self::new()
  }
}

/// Execute an HTTP request with JSON serialization/deserialization
///
/// This function handles the complete HTTP request lifecycle with enhanced
/// error handling, performance monitoring, and optional structured logging.
///
/// # Features
///
/// - Automatic JSON serialization and deserialization
/// - Comprehensive error handling with specific error types
/// - Performance monitoring with request timing
/// - Structured logging (when 'logging' feature is enabled)
/// - Enhanced error messages with context
/// - Proper handling of API key authentication
///
/// # Performance
///
/// This function is optimized for low overhead:
/// - Minimal allocations during the request lifecycle
/// - Efficient JSON handling with streaming where possible
/// - Request timing to monitor API performance
///
/// # Errors
///
/// This function returns specific error types for different failure scenarios:
/// - [`Error::SerializationError`] - Request body serialization failed
/// - [`Error::NetworkError`] - Network connectivity issues
/// - [`Error::AuthenticationError`] - API key or permission issues (401/403)
/// - [`Error::InvalidArgument`] - Invalid request parameters (400)
/// - [`Error::RateLimitError`] - Rate limiting applied (429)
/// - [`Error::ServerError`] - Server-side issues (5xx)
/// - [`Error::DeserializationError`] - Response parsing failed
/// - [`Error::RequestBuilding`] - Invalid URL or request configuration
#[ cfg_attr( feature = "logging", instrument(
  skip( client, api_key, body ),
  fields(
    method = %method,
    url = url,
    has_body = body.is_some(),
  )
) ) ]
#[ inline ]
pub async fn execute< T, R >
(
  client : &Client,
  method : Method,
  url : &str,
  api_key : &str,
  body : Option< &T >,
  config : &HttpConfig,
)
->
Result< R, Error >
where
  T : Serialize,
  R : for< 'de > Deserialize< 'de >,
{
  let start_time = Instant::now();

  // Generate request ID for correlation
  #[ cfg( feature = "logging" ) ]
  let request_id = if config.enable_logging
  {
    format!( "req-{:08x}", rand::rng().random::< u32 >() )
  } else {
    String::new()
  };

  #[ cfg( feature = "logging" ) ]
  if config.enable_logging
  {
    info!(
      url = %url,
      method = %method,
      request_id = %request_id,
      "Starting HTTP request"
    );
  }

  // Build the request with enhanced configuration
  let request = build_request( client, method, url, api_key, body, config )?;

  // Execute the request with timing
  let response = send_request( client, request, config ).await?;

  // Capture response metadata before processing
  let status_code = response.status().as_u16();
  let response_size = response.content_length().unwrap_or( 0 );

  // Process the response with comprehensive error handling
  let result = process_response::< R >( response, config ).await;

  let elapsed = start_time.elapsed();
  let duration_ms = elapsed.as_secs_f64() * 1000.0;

  // Suppress unused variable warnings when logging is disabled
  #[ cfg( not( feature = "logging" ) ) ]
  {
    let _ = ( status_code, response_size, duration_ms );
  }

  #[ cfg( feature = "logging" ) ]
  if config.enable_logging
  {
    // Extract operation from URL path for better monitoring
    let operation = extract_operation_from_url( url );

    match &result
    {
      Ok( _ ) => info!(
        request_id = %request_id,
        duration_ms = duration_ms,
        status_code = status_code,
        response_size_bytes = response_size,
        operation = %operation,
        "HTTP request completed successfully"
      ),
      Err( error ) => {
        let error_type = match error
        {
          Error::ApiError( _ ) => "ApiError",
          Error::AuthenticationError( _ ) => "AuthenticationError",
          Error::NetworkError( _ ) => "NetworkError",
          Error::SerializationError( _ ) => "SerializationError",
          Error::DeserializationError( _ ) => "DeserializationError",
          Error::InvalidArgument( _ ) => "InvalidArgument",
          Error::RateLimitError( _ ) => "RateLimitError",
          Error::ServerError( _ ) => "ServerError",
          Error::RequestBuilding( _ ) => "RequestBuilding",
          _ => "UnknownError",
        };

        error!(
          request_id = %request_id,
          duration_ms = duration_ms,
          error_type = error_type,
          error_message = %error,
          url = %url,
          operation = %operation,
          "HTTP request failed"
        );
      },
    }
  }

  // Log performance warning for slow requests
  if elapsed > Duration::from_millis( 5000 )
  {
    #[ cfg( feature = "logging" ) ]
    if config.enable_logging
    {
      warn!(
        url = %url,
        duration_ms = duration_ms,
        "Slow HTTP request detected"
      );
    }
  }

  result
}

/// Build an HTTP request with proper configuration and error handling
///
/// This function handles request construction including:
/// - URL validation and formatting
/// - Header configuration
/// - JSON body serialization
/// - API key parameter injection
fn build_request< T >
(
  client : &Client,
  method : Method,
  url : &str,
  api_key : &str,
  body : Option< &T >,
  config : &HttpConfig,
)
-> Result< reqwest::Request, Error >
where
  T : Serialize,
{
  #[ cfg( feature = "logging" ) ]
  if config.enable_logging
  {
    debug!( "Building {} request to {}", method, url );
  }

  // Validate URL format
  if !url.starts_with( "http" )
  {
    return Err( Error::RequestBuilding(
      format!( "Invalid URL format '{url}': URL must start with http:// or https://" )
    ) );
  }

  // Create request builder - only apply timeout if not already set on client
  let mut request_builder = client
    .request( method, url )
    .query( &[ ( "key", api_key ) ] )
    .header( "Content-Type", "application/json" )
    .header( "User-Agent", "api-gemini-rust/0.2.0" );

  // Only set timeout if it's different from default (indicating explicit config)
  if config.timeout_seconds != 30
  {
    request_builder = request_builder.timeout( Duration::from_secs( config.timeout_seconds ) );
  }

  // Serialize and attach body if provided
  if let Some( body ) = body
  {
    let json_body = serde_json::to_string( body )
      .map_err( |e| Error::SerializationError(
        format!( "Failed to serialize request body : {e}" )
      ) )?;

    #[ cfg( feature = "logging" ) ]
    if config.enable_logging
    {
      let log_content = if json_body.len() > config.max_log_content_length
      {
        format!( "{}... ({} bytes total)", &json_body[..config.max_log_content_length], json_body.len() )
      } else {
        json_body.clone()
      };
      debug!( "Request body : {}", log_content );
    }

    // Apply compression if configured
    #[ cfg( feature = "compression" ) ]
    let ( final_body, is_compressed ) = if let Some( ref compression_cfg ) = config.compression_config
    {
      match compression::compress( json_body.as_bytes(), compression_cfg )
      {
        Ok( compressed ) => {
          // Check if compression actually helped
          if compressed.len() < json_body.len()
          {
            #[ cfg( feature = "logging" ) ]
            if config.enable_logging
            {
              debug!(
                "Compressed request body : {} bytes -> {} bytes ({}% reduction)",
                json_body.len(),
                compressed.len(),
                100 - ( compressed.len() * 100 / json_body.len() )
              );
            }
            ( compressed, true )
          }
          else
          {
            ( json_body.into_bytes(), false )
          }
        },
        Err( e ) => {
          #[ cfg( feature = "logging" ) ]
          if config.enable_logging
          {
            warn!( "Compression failed, using uncompressed body : {}", e );
          }
          ( json_body.into_bytes(), false )
        }
      }
    }
    else
    {
      ( json_body.into_bytes(), false )
    };

    #[ cfg( not( feature = "compression" ) ) ]
    let ( final_body, _is_compressed ) = ( json_body.into_bytes(), false );

    // Add compression headers if body was compressed
    #[ cfg( feature = "compression" ) ]
    if is_compressed
    {
      if let Some( ref compression_cfg ) = config.compression_config
      {
        if let Some( encoding ) = compression_cfg.algorithm.content_encoding()
        {
          request_builder = request_builder.header( "Content-Encoding", encoding );
        }
      }
    }

    request_builder = request_builder.body( final_body );
  }

  request_builder.build()
    .map_err( |e| Error::RequestBuilding(
      format!( "Failed to build HTTP request : {e}" )
    ) )
}

/// Send an HTTP request with comprehensive error handling
///
/// This function handles the actual request transmission and captures
/// various types of network and protocol errors.
async fn send_request
(
  client : &Client,
  request : reqwest::Request,
  config : &HttpConfig,
)
-> Result< reqwest::Response, Error >
{
  let url = request.url().to_string();

  #[ cfg( feature = "logging" ) ]
  if config.enable_logging
  {
    debug!( "Sending HTTP request" );
  }

  client
    .execute( request )
    .await
    .map_err( |e| {
      #[ cfg( feature = "logging" ) ]
      error!( "Network error during request to {}: {}", url, e );

      // Enhanced error classification
      if e.is_timeout()
      {
        Error::NetworkError( format!( "Request timeout after {}s : {}", config.timeout_seconds, e ) )
      } else if e.is_connect()
      {
        Error::NetworkError( format!( "Connection failed to {url}: {e}" ) )
      } else if e.is_request()
      {
        Error::RequestBuilding( format!( "Request configuration error : {e}" ) )
      } else {
        Error::NetworkError( format!( "Network error : {e}" ) )
      }
    } )
}

/// Process HTTP response with comprehensive error handling and deserialization
///
/// This function handles response processing including:
/// - Status code analysis and error classification
/// - API error response parsing
/// - JSON deserialization with error context
/// - Structured logging of response details
async fn process_response< R >
(
  response : reqwest::Response,
  #[ allow( unused_variables ) ]
  config : &HttpConfig,
)
-> Result< R, Error >
where
  R : for< 'de > Deserialize< 'de >,
{
  let status = response.status();
  let status_code = status.as_u16();

  #[ cfg( feature = "logging" ) ]
  if config.enable_logging
  {
    debug!( "Received response with status : {}", status );
  }

  // Get response body text for processing
  let response_text = response.text().await
    .map_err( |e| Error::NetworkError(
      format!( "Failed to read response body : {e}" )
    ) )?;

  #[ cfg( feature = "logging" ) ]
  if config.enable_logging
  {
    let log_content = if response_text.len() > config.max_log_content_length
    {
      format!( "{}... ({} bytes total)", &response_text[..config.max_log_content_length], response_text.len() )
    } else {
      response_text.clone()
    };
    debug!( "Response body : {}", log_content );
  }

  if status.is_success()
  {
    // Successful response - deserialize JSON
    serde_json ::from_str( &response_text )
      .map_err( |e| {
        #[ cfg( feature = "logging" ) ]
        error!( "Failed to deserialize successful response : {}", e );

        Error::DeserializationError(
          format!( "Failed to parse successful response as JSON: {}. Response content : {}",
            e,
            if response_text.len() > 200
            {
              format!( "{}...", &response_text[..200] )
            } else {
              response_text
            }
          )
        )
      } )
  }
  else
  {
    // Error response - attempt structured error parsing
    classify_error_response( status_code, &response_text ).map( |_| {
      // This will never be reached since classify_error_response always returns an error
      unreachable!("classify_error_response should never return Ok")
    } )
  }
}

/// Classify and create appropriate error types from HTTP error responses
///
/// This function provides comprehensive error classification based on:
/// - HTTP status codes
/// - API error response structure
/// - Error message content analysis
/// - Authentication and authorization patterns
fn classify_error_response( status_code : u16, response_text : &str ) -> Result< never, Error >
{
  #[ cfg( feature = "logging" ) ]
  debug!( "Classifying error response : HTTP {}", status_code );

  // Try to parse as structured API error response first
  if let Ok( api_error ) = serde_json::from_str::< ApiErrorResponse >( response_text )
  {
    let error_message = format!( "HTTP {}: {}", status_code, api_error.error.message );

    #[ cfg( feature = "logging" ) ]
    debug!( "Parsed structured API error : {}", api_error.error.message );

    // Classify based on message content and status code
    if is_authentication_error( &api_error.error.message ) || matches!( status_code, 401 | 403 )
    {
      Err( Error::AuthenticationError( error_message ) )
    }
    else
    {
      match status_code
      {
        400 => Err( Error::InvalidArgument( error_message ) ),
        429 => Err( Error::RateLimitError( error_message ) ),
        500..=599 => Err( Error::ServerError( error_message ) ),
        _ => Err( Error::ApiError( error_message ) ),
      }
    }
  }
  else
  {
    // Fallback to plain text error response analysis
    let error_message = format!( "HTTP {status_code}: {response_text}" );

    #[ cfg( feature = "logging" ) ]
    debug!( "Using fallback error classification for non-JSON response" );

    if is_authentication_error( response_text ) || matches!( status_code, 401 | 403 )
    {
      Err( Error::AuthenticationError( error_message ) )
    }
    else
    {
      match status_code
      {
        400 => Err( Error::InvalidArgument( error_message ) ),
        429 => Err( Error::RateLimitError( error_message ) ),
        500..=599 => Err( Error::ServerError( error_message ) ),
        _ => Err( Error::ApiError( error_message ) ),
      }
    }
  }
}

/// Determine if an error message indicates an authentication or authorization issue
///
/// This function analyzes error messages for common authentication-related patterns
/// to provide better error classification regardless of HTTP status codes.
#[ inline ]
fn is_authentication_error( message : &str ) -> bool
{
  let msg_lower = message.to_lowercase();

  msg_lower.contains( "api key" ) ||
  msg_lower.contains( "authentication" ) ||
  msg_lower.contains( "unauthorized" ) ||
  msg_lower.contains( "forbidden" ) ||
  msg_lower.contains( "permission" ) ||
  msg_lower.contains( "access denied" ) ||
  msg_lower.contains( "invalid key" ) ||
  msg_lower.contains( "expired key" ) ||
  msg_lower.contains( "quota exceeded" )
}

/// Backward compatibility wrapper for the original execute function
///
/// This function maintains the original API while internally using the new
/// enhanced execute function with a test-friendly timeout configuration.
/// Uses a 10-second timeout to work well with test environments.
///
/// # Errors
///
/// Returns the same errors as [`execute`] - see that function's documentation
/// for detailed error information.
#[ inline ]
pub async fn execute_legacy< T, R >
(
  client : &Client,
  method : Method,
  url : &str,
  api_key : &str,
  body : Option< &T >,
)
->
Result< R, Error >
where
  T : Serialize,
  R : for< 'de > Deserialize< 'de >,
{
  // Use enhanced config with logging enabled when the logging feature is available
  #[ cfg( feature = "logging" ) ]
  let config = HttpConfig::default().with_logging();
  #[ cfg( not( feature = "logging" ) ) ]
  let config = HttpConfig::default();

  execute( client, method, url, api_key, body, &config ).await
}

/// Extract operation name from URL for monitoring purposes
#[ cfg( feature = "logging" ) ]
fn extract_operation_from_url( url : &str ) -> String
{
  if let Some( path_start ) = url.find( "/v1beta/" )
  {
    let path = &url[path_start + 8..]; // Skip "/v1beta/"

    // Extract meaningful operation name
    if path.starts_with( "models/" )
    {
      if path.contains( ":embedContent" )
      {
        "embed_content".to_string()
      } else if path.contains( ":generateContent" )
      {
        "generate_content".to_string()
      } else if path.contains( ":streamGenerateContent" )
      {
        "stream_generate_content".to_string()
      } else if path.ends_with( "/models" ) || path == "models"
      {
        "list_models".to_string()
      } else {
        "get_model".to_string()
      }
    } else {
      // Fallback to first path segment
      path.split( '/' ).next().unwrap_or( "unknown" ).to_string()
    }
  } else {
    "unknown".to_string()
  }
}

/// Execute an HTTP request and return the raw response without deserialization
///
/// This function provides a low-level interface for cases where the caller needs
/// to handle the response directly (e.g., checking status codes, reading raw text).
///
/// # Errors
///
/// Returns the same network and request building errors as [`execute`] but does not
/// return deserialization errors since no deserialization is performed.
#[ inline ]
pub async fn execute_raw< T >
(
  client : &Client,
  method : Method,
  url : &str,
  api_key : &str,
  body : Option< &T >,
)
->
Result< reqwest::Response, Error >
where
  T : Serialize,
{
  let config = HttpConfig::default();

  // Build and send the request
  let request = build_request( client, method, url, api_key, body, &config )?;
  send_request( client, request, &config ).await
}

// Type alias for never type until it's stabilized
#[ allow( non_camel_case_types ) ]
type never = core::convert::Infallible;

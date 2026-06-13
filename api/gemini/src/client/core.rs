//! Core Client struct and implementation.

use core::time::Duration;
use reqwest;
use crate::error::Error;
use super::builder::ClientBuilder;
use super::config::{ ClientConfig, ClientConfigFormer };
use super::sync::SyncClientBuilder;

  /// The main client for interacting with the Gemini API.
  ///
  /// ## Design Principle : Thin Client
  ///
  /// This client provides transparent access to Gemini API endpoints without
  /// client-side intelligence or automatic behaviors. All operations are explicit
  /// HTTP calls to the server with no hidden logic or magic thresholds.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use api_gemini::client::Client;
  ///
  /// #[ tokio::main ]
  /// async fn main() -> Result< (), Box< dyn std::error::Error > >
  /// {
  ///   // Create client from environment variable
  ///   let client = Client::new()?;
  ///
  ///   // Or use builder pattern
  ///   let client = Client::builder()
  ///     .api_key( "your-api-key".to_string() )
  ///     .build()?;
  ///
  ///   // List available models
  ///   let models = client.models().list().await?;
  ///   println!( "Available models : {}", models.models.len() );
  ///
  ///   Ok( () )
  /// }
  /// ```
  #[ derive( Debug, Clone ) ]
  #[ allow( clippy::struct_excessive_bools ) ] // Configuration struct with feature flags
  pub struct Client
  {
    pub( crate ) api_key : String,
    pub( crate ) base_url : String,
    pub( crate ) http : reqwest::Client,
    pub( crate ) timeout : Duration,
    #[ cfg( feature = "retry" ) ]
    pub( crate ) max_retries : u32,
    #[ cfg( feature = "retry" ) ]
    pub( crate ) base_delay : Duration,
    #[ cfg( feature = "retry" ) ]
    pub( crate ) max_delay : Duration,
    #[ cfg( feature = "retry" ) ]
    pub( crate ) enable_jitter : bool,
    #[ cfg( feature = "retry" ) ]
    // xxx : @team : Implement per-request timeout override mechanism (task/unverified/007)
    // Currently client-level timeout applies to all requests uniformly
    #[ allow( dead_code ) ]
    pub( crate ) request_timeout : Option< Duration >,
    #[ cfg( feature = "retry" ) ]
    pub( crate ) backoff_multiplier : f64,
    #[ cfg( feature = "retry" ) ]
    // xxx : @team : Implement retry metrics collection and aggregation
    // Track retry attempts, backoff timing, success/failure rates per endpoint
    #[ allow( dead_code ) ]
    pub( crate ) enable_retry_metrics : bool,
    #[ cfg( feature = "retry" ) ]
    pub( crate ) max_elapsed_time : Option< Duration >,
    #[ cfg( feature = "circuit_breaker" ) ]
    // xxx : @team : Integrate circuit breaker from internal/http.rs into Client API (task/unverified/007, task/verified/003)
    // Circuit breaker is fully implemented in internal::http::CircuitBreaker
    // Need to expose it through Client methods (execute_with_circuit_breaker)
    #[ allow( dead_code ) ]
    pub( crate ) enable_circuit_breaker : bool,
    #[ cfg( feature = "circuit_breaker" ) ]
    pub( crate ) circuit_breaker_failure_threshold : u32,
    #[ cfg( feature = "circuit_breaker" ) ]
    pub( crate ) circuit_breaker_success_threshold : u32,
    #[ cfg( feature = "circuit_breaker" ) ]
    pub( crate ) circuit_breaker_timeout : Duration,
    #[ cfg( feature = "circuit_breaker" ) ]
    // xxx : @team : Expose circuit breaker metrics through Client::get_circuit_breaker_metrics()
    pub( crate ) enable_circuit_breaker_metrics : bool,
    #[ cfg( feature = "circuit_breaker" ) ]
    // xxx : @team : Implement Arc-based shared circuit breaker state across client instances
    #[ allow( dead_code ) ]
    pub( crate ) circuit_breaker_shared_state : bool,
    #[ cfg( feature = "caching" ) ]
    // xxx : @team : Implement general HTTP response caching layer (task/unverified/007)
    // Domain-specific caches exist (WorkspaceCache, SemanticCache, MediaCache)
    // Need general request/response cache with LRU eviction for all API calls
    #[ allow( dead_code ) ]
    pub( crate ) enable_request_cache : bool,
    #[ cfg( feature = "caching" ) ]
    #[ allow( dead_code ) ]
    pub( crate ) cache_ttl : Duration,
    #[ cfg( feature = "caching" ) ]
    #[ allow( dead_code ) ]
    pub( crate ) cache_max_size : usize,
    #[ cfg( feature = "caching" ) ]
    // xxx : @team : Track cache hit/miss rates, eviction statistics
    #[ allow( dead_code ) ]
    pub( crate ) enable_cache_metrics : bool,
    #[ cfg( feature = "caching" ) ]
    /// General HTTP request cache instance
    pub( crate ) request_cache : Option< std::sync::Arc< crate::internal::http::RequestCache > >,
    #[ cfg( feature = "rate_limiting" ) ]
    // xxx : @team : Integrate rate limiter from internal/http.rs into Client API (task/unverified/007)
    // Rate limiter is fully implemented in internal::http::RateLimiter
    // Need to expose it through Client methods (execute_with_rate_limiting)
    #[ allow( dead_code ) ]
    pub( crate ) enable_rate_limiting : bool,
    #[ cfg( feature = "rate_limiting" ) ]
    pub( crate ) rate_limit_requests_per_second : f64,
    #[ cfg( feature = "rate_limiting" ) ]
    pub( crate ) rate_limit_algorithm : String,
    #[ cfg( feature = "rate_limiting" ) ]
    pub( crate ) rate_limit_bucket_size : usize,
    #[ cfg( feature = "rate_limiting" ) ]
    // xxx : @team : Expose rate limiting metrics through Client::get_rate_limiter_metrics()
    pub( crate ) enable_rate_limiting_metrics : bool,
    #[ cfg( feature = "compression" ) ]
    /// Compression configuration for request/response optimization
    pub( crate ) compression_config : Option< crate::internal::http::compression::CompressionConfig >,
  }

  impl Client
  {
      /// Create a new client builder using the `former` crate
    #[ must_use ]
    #[ inline ]
    pub fn former() -> ClientConfigFormer
    {
        ClientConfig::former()
    }
      /// Create a new client builder
    #[ must_use ]
    #[ inline ]
    pub fn builder() -> ClientBuilder
    {
        ClientBuilder::new()
    }

    /// Create a new sync client builder
    #[ must_use ]
    #[ inline ]
    pub fn sync_builder() -> SyncClientBuilder
    {
        SyncClientBuilder::new()
    }

      /// Create a new client using the `GEMINI_API_KEY` from workspace secrets or environment.
      ///
      /// This method attempts to load the API key in the following order:
      /// 1. Workspace secrets file : `secret/-secrets.sh` (using `workspace_tools` 0.6.0)
      /// 2. Environment variable : `GEMINI_API_KEY`
      ///
      /// **Note**: `workspace_tools` 0.6.0 uses `secret/` (visible directory, NO dot prefix).
      ///
      /// # Errors
      ///
      /// Returns an error with detailed path information if the `GEMINI_API_KEY` cannot be loaded
      /// from either workspace secrets or environment variable, or if the client cannot be built.
      ///
      /// # Examples
      ///
      /// ```rust,no_run
      /// use api_gemini::client::Client;
      ///
      /// // Will try workspace secrets first, then environment variable
      /// let client = Client::new()?;
      /// # Ok::<(), Box< dyn std::error::Error > >(())
      /// ```
    #[ inline ]
    pub fn new() -> Result< Client, Error >
    {
        // First try to load from secret/-secrets.sh file
        let api_key = match Self::load_api_key_from_secret_file()
        {
          Ok( key ) => key,
          Err( secret_err ) => {
            // Fallback to environment variable
            match std::env::var( "GEMINI_API_KEY" )
            {
              Ok( key ) if !key.is_empty() => key,
              _ => {
                return Err( Error::AuthenticationError(
                  format!(
                    "GEMINI_API_KEY not found. Tried:\n  \
                    1. Workspace secrets : secret/-secrets.sh ({})\n  \
                    2. Environment variable : GEMINI_API_KEY (not set or empty)\n\n  \
                    Setup instructions:\n  \
                    - Add to workspace secrets : echo 'export GEMINI_API_KEY=\"your-key\"' > > secret/-secrets.sh\n  \
                    - Or set environment : export GEMINI_API_KEY=\"your-key\"\n  \
                    - Note : workspace_tools 0.6.0 uses secret/ (visible directory, NO dot prefix)\n  \
                    - See tests/readme.md for detailed setup guide",
                    secret_err
                  )
                ) );
              }
            }
          }
        };

        Self::builder()
          .api_key( api_key )
          .build()
    }

      /// Load API key from workspace `secret/-secrets.sh` file using `workspace_tools`.
      ///
      /// This method uses `workspace_tools` to properly locate and load secrets from the
      /// workspace root's secret directory, following the Secret Directory Policy.
      ///
      /// # Errors
      ///
      /// Returns an error if workspace cannot be resolved or if `GEMINI_API_KEY` is not found
      /// in the -secrets.sh file.
    fn load_api_key_from_secret_file() -> Result< String, Error >
    {
        use workspace_tools as workspace;

        // Use workspace_tools to properly load secrets from workspace
        let ws = workspace::workspace()
          .map_err( | e | Error::Io( format!( "Failed to resolve workspace : {e}" ) ) )?;

        // Load GEMINI_API_KEY from -secrets.sh file in secret directory
        let api_key = ws.load_secret_key( "GEMINI_API_KEY", "-secrets.sh" )
          .map_err( | e | Error::AuthenticationError( format!( "key not found or file unreadable : {e}" ) ) )?;

        Ok( api_key )
    }

      /// Send a GET request to the specified URL with API key authentication
      ///
      /// # Errors
      ///
      /// Returns an error if the request fails or if there are network issues.
    #[ inline ]
    pub async fn send_get_request( &self, url : &str ) -> Result< reqwest::Response, Error >
    {
        let url_with_key = self.add_api_key_to_url( url );
        
        let response = self.http
          .get( &url_with_key )
          .header( "Content-Type", "application/json" )
          .send()
          .await?;
          
        Ok( response )
    }

      /// Send a POST request to the specified URL with JSON body and API key authentication
      ///
      /// # Errors
      ///
      /// Returns an error if the request fails, if serialization fails, or if there are network issues.
    #[ inline ]
    pub async fn send_post_request( &self, url : &str, body : &serde_json::Value ) -> Result< reqwest::Response, Error >
    {
        let url_with_key = self.add_api_key_to_url( url );
        let json_body = self.serialize_request_body( body )?;
        
        let response = self.http
          .post( &url_with_key )
          .header( "Content-Type", "application/json" )
          .body( json_body )
          .send()
          .await?;
          
        Ok( response )
    }

      /// Serialize request body to JSON string
      ///
      /// # Errors
      ///
      /// Returns an error if JSON serialization fails.
    #[ inline ]
    pub fn serialize_request_body( &self, body : &serde_json::Value ) -> Result< String, Error >
    {
        serde_json ::to_string( body )
          .map_err( | e | Error::SerializationError( format!( "Failed to serialize request body : {e}" ) ) )
    }

      /// Deserialize response text to the specified type
      ///
      /// # Errors
      ///
      /// Returns an error if JSON deserialization fails.
    #[ inline ]
    pub fn deserialize_response< T >( &self, response_text : &str ) -> Result< T, Error >
    where
      T : for< 'de > serde::Deserialize< 'de >,
    {
        serde_json ::from_str( response_text )
          .map_err( | e | Error::DeserializationError( format!( "Failed to deserialize response : {e}" ) ) )
    }

      /// Add API key as query parameter to URL
      ///
      /// Handles URLs that already have query parameters by appending with &
    #[ must_use ]
    #[ inline ]
    pub fn add_api_key_to_url( &self, base_url : &str ) -> String
    {
        if base_url.contains( '?' )
        {
          { let encoded_key = urlencoding::encode( &self.api_key ); format!( "{base_url}&key={encoded_key}" ) }
        }
        else
        {
          { let encoded_key = urlencoding::encode( &self.api_key ); format!( "{base_url}?key={encoded_key}" ) }
        }
    }

      /// Handle API error responses and convert to appropriate Error types
      ///
      /// # Errors
      ///
      /// Always returns an error based on the provided status code and message.
    #[ inline ]
    pub fn handle_response_error( &self, status : u16, status_text : &str, message : &str ) -> Result< (), Error >
    {
        let error_message = if message.is_empty() 
        {
          format!( "{status} {status_text}" )
        }
        else
        {
          format!( "{status} {status_text}: {message}" )
        };

        match status
        {
          500..=599 => Err( Error::ServerError( error_message ) ),
          _ => Err( Error::ApiError( error_message ) ),
        }
    }

    /// Convert client retry configuration into HTTP layer `RetryConfig`
    #[ cfg( feature = "retry" ) ]
    pub( crate ) fn to_retry_config( &self ) -> Option< crate::internal::http::RetryConfig >
    {
      if self.max_retries == 0
      {
        None
      } else {
        Some( crate::internal::http::RetryConfig {
          max_retries : self.max_retries,
          base_delay : self.base_delay,
          max_delay : self.max_delay,
          backoff_multiplier : self.backoff_multiplier,
          enable_jitter : self.enable_jitter,
          max_elapsed_time : self.max_elapsed_time,
        } )
      }
    }

    /// Convert client circuit breaker configuration into HTTP layer `CircuitBreakerConfig`
    #[ cfg( feature = "circuit_breaker" ) ]
    pub( crate ) fn to_circuit_breaker_config( &self ) -> Option< crate::internal::http::CircuitBreakerConfig >
    {
      if self.circuit_breaker_failure_threshold == 0
      {
        None
      } else {
        Some( crate::internal::http::CircuitBreakerConfig {
          failure_threshold : self.circuit_breaker_failure_threshold,
          timeout : self.circuit_breaker_timeout,
          success_threshold : self.circuit_breaker_success_threshold,
          enable_metrics : self.enable_circuit_breaker_metrics,
        } )
      }
    }

    /// Convert client rate limiting configuration into HTTP layer `RateLimitingConfig`
    #[ cfg( feature = "rate_limiting" ) ]
    pub( crate ) fn to_rate_limiting_config( &self ) -> Option< crate::internal::http::RateLimitingConfig >
    {
      if self.rate_limit_requests_per_second <= 0.0
      {
        None
      } else {
        Some( crate::internal::http::RateLimitingConfig {
          requests_per_second : self.rate_limit_requests_per_second,
          bucket_size : self.rate_limit_bucket_size,
          algorithm : self.rate_limit_algorithm.clone(),
          enable_metrics : self.enable_rate_limiting_metrics,
        } )
      }
    }
  }

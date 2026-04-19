//! Client configuration types.

use core::time::Duration;
use former::Former;
use crate::error::Error;
use super::Client;

  /// Configuration struct using the `former` crate for builder pattern generation.
  ///
  /// This provides a `former`-based builder pattern as an alternative to the manual `ClientBuilder`.
  /// Users can choose between the comprehensive manual builder or this derive-based approach.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// use api_gemini::client::ClientConfig;
  ///
  /// // Using the former-based builder
  /// let client = ClientConfig::former()
  ///   .api_key( "your-api-key".to_string() )
  ///   .timeout( std::time::Duration::from_secs( 60 ) )
  ///   .form()
  ///   .build()?;
  /// # Ok( () )
  /// # }
  /// ```
  #[ allow( clippy::struct_excessive_bools ) ]
  #[ derive( Debug, Clone, Former ) ]
  pub struct ClientConfig
  {
    /// API key for authentication with Gemini API
    pub api_key : String,

    /// Base URL for API requests
    #[ former( default = "https://generativelanguage.googleapis.com".to_string() ) ]
    pub base_url : String,

    /// Request timeout duration
    #[ former( default = Duration::from_secs( 30 ) ) ]
    pub timeout : Duration,

    // Retry configuration (feature-gated)
    #[ cfg( feature = "retry" ) ]
    #[ former( default = 3_u32 ) ]
    /// Maximum number of retry attempts for failed requests
    pub max_retries : u32,

    #[ cfg( feature = "retry" ) ]
    #[ former( default = Duration::from_millis( 100 ) ) ]
    /// Initial delay before first retry attempt
    pub base_delay : Duration,

    #[ cfg( feature = "retry" ) ]
    #[ former( default = Duration::from_secs( 10 ) ) ]
    /// Maximum delay between retry attempts
    pub max_delay : Duration,

    #[ cfg( feature = "retry" ) ]
    #[ former( default = true ) ]
    /// Whether to add random jitter to retry delays
    pub enable_jitter : bool,

    #[ cfg( feature = "retry" ) ]
    #[ former( default = 2.0 ) ]
    /// Multiplier for exponential backoff between retries
    pub backoff_multiplier : f64,

    #[ cfg( feature = "retry" ) ]
    #[ former( default = false ) ]
    /// Whether to collect retry-related metrics
    pub enable_retry_metrics : bool,

    #[ cfg( feature = "retry" ) ]
    /// Maximum total elapsed time for all retry attempts
    pub max_elapsed_time : Option< Duration >,

    // Circuit breaker configuration (feature-gated)
    #[ cfg( feature = "circuit_breaker" ) ]
    #[ former( default = false ) ]
    /// Whether to enable circuit breaker pattern
    pub enable_circuit_breaker : bool,

    #[ cfg( feature = "circuit_breaker" ) ]
    #[ former( default = 5_u32 ) ]
    /// Number of failures before circuit breaker opens
    pub circuit_breaker_failure_threshold : u32,

    #[ cfg( feature = "circuit_breaker" ) ]
    #[ former( default = 3_u32 ) ]
    /// Number of successes needed to close circuit breaker
    pub circuit_breaker_success_threshold : u32,

    #[ cfg( feature = "circuit_breaker" ) ]
    #[ former( default = Duration::from_secs( 60 ) ) ]
    /// Timeout before circuit breaker attempts to close
    pub circuit_breaker_timeout : Duration,

    // Caching configuration (feature-gated)
    #[ cfg( feature = "caching" ) ]
    #[ former( default = false ) ]
    /// Whether to enable request caching
    pub enable_request_cache : bool,

    #[ cfg( feature = "caching" ) ]
    #[ former( default = Duration::from_secs( 300 ) ) ]
    /// Time-to-live for cached responses
    pub cache_ttl : Duration,

    #[ cfg( feature = "caching" ) ]
    #[ former( default = 1000_usize ) ]
    /// Maximum number of entries in cache
    pub cache_max_size : usize,

    // Rate limiting configuration (feature-gated)
    #[ cfg( feature = "rate_limiting" ) ]
    #[ former( default = false ) ]
    /// Whether to enable rate limiting
    pub enable_rate_limiting : bool,

    #[ cfg( feature = "rate_limiting" ) ]
    #[ former( default = 10.0 ) ]
    /// Maximum requests per second allowed
    pub rate_limit_requests_per_second : f64,

    #[ cfg( feature = "rate_limiting" ) ]
    #[ former( default = "token_bucket".to_string() ) ]
    /// Rate limiting algorithm to use
    pub rate_limit_algorithm : String,

    #[ cfg( feature = "rate_limiting" ) ]
    #[ former( default = 100_usize ) ]
    /// Size of rate limiting token bucket
    pub rate_limit_bucket_size : usize,
  }

  impl ClientConfig
  {
    /// Build a `Client` from this configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or if the HTTP client cannot be created.
    #[ inline ]
    pub fn build( &self ) -> Result< Client, Error >
    {
      // Validate API key
      if self.api_key.is_empty()
      {
        return Err( Error::AuthenticationError( "API key cannot be empty".to_string() ) );
      }

      // Validate retry configuration
      #[ cfg( feature = "retry" ) ]
      {
        if self.backoff_multiplier <= 1.0
        {
          return Err( Error::InvalidArgument(
            format!( "Backoff multiplier must be greater than 1.0, got : {0}", self.backoff_multiplier )
          ) );
        }

        if self.base_delay >= self.max_delay
        {
          return Err( Error::InvalidArgument(
            "Base delay must be less than max delay".to_string()
          ) );
        }
      }

      // Build HTTP client
      let http = reqwest::Client::builder()
        .timeout( self.timeout )
        .build()
        .map_err( | e | Error::NetworkError( format!( "Failed to create HTTP client : {e}" ) ) )?;

      // Create request cache if caching is enabled
      #[ cfg( feature = "caching" ) ]
      let request_cache = if self.enable_request_cache
      {
        let cache_config = crate::internal::http::CacheConfig {
          max_size : self.cache_max_size,
          ttl : self.cache_ttl,
          enable_metrics : false, // Simplified for former version
        };
        Some( std::sync::Arc::new( crate::internal::http::RequestCache::new( cache_config ) ) )
      } else {
        None
      };

      // Create the client instance
      Ok( Client
      {
        api_key : self.api_key.clone(),
        base_url : self.base_url.clone(),
        http,
        timeout : self.timeout,
        #[ cfg( feature = "retry" ) ]
        max_retries : self.max_retries,
        #[ cfg( feature = "retry" ) ]
        base_delay : self.base_delay,
        #[ cfg( feature = "retry" ) ]
        max_delay : self.max_delay,
        #[ cfg( feature = "retry" ) ]
        enable_jitter : self.enable_jitter,
        #[ cfg( feature = "retry" ) ]
        request_timeout : None, // Not configurable in former version for simplicity
        #[ cfg( feature = "retry" ) ]
        backoff_multiplier : self.backoff_multiplier,
        #[ cfg( feature = "retry" ) ]
        enable_retry_metrics : self.enable_retry_metrics,
        #[ cfg( feature = "retry" ) ]
        max_elapsed_time : self.max_elapsed_time,
        #[ cfg( feature = "circuit_breaker" ) ]
        enable_circuit_breaker : self.enable_circuit_breaker,
        #[ cfg( feature = "circuit_breaker" ) ]
        circuit_breaker_failure_threshold : self.circuit_breaker_failure_threshold,
        #[ cfg( feature = "circuit_breaker" ) ]
        circuit_breaker_success_threshold : self.circuit_breaker_success_threshold,
        #[ cfg( feature = "circuit_breaker" ) ]
        circuit_breaker_timeout : self.circuit_breaker_timeout,
        #[ cfg( feature = "circuit_breaker" ) ]
        enable_circuit_breaker_metrics : false, // Simplified for former version
        #[ cfg( feature = "circuit_breaker" ) ]
        circuit_breaker_shared_state : false, // Simplified for former version
        #[ cfg( feature = "caching" ) ]
        enable_request_cache : self.enable_request_cache,
        #[ cfg( feature = "caching" ) ]
        cache_ttl : self.cache_ttl,
        #[ cfg( feature = "caching" ) ]
        cache_max_size : self.cache_max_size,
        #[ cfg( feature = "caching" ) ]
        enable_cache_metrics : false, // Simplified for former version
        #[ cfg( feature = "caching" ) ]
        request_cache,
        #[ cfg( feature = "rate_limiting" ) ]
        enable_rate_limiting : self.enable_rate_limiting,
        #[ cfg( feature = "rate_limiting" ) ]
        rate_limit_requests_per_second : self.rate_limit_requests_per_second,
        #[ cfg( feature = "rate_limiting" ) ]
        rate_limit_algorithm : self.rate_limit_algorithm.clone(),
        #[ cfg( feature = "rate_limiting" ) ]
        rate_limit_bucket_size : self.rate_limit_bucket_size,
        #[ cfg( feature = "rate_limiting" ) ]
        enable_rate_limiting_metrics : false, // Simplified for former version
        #[ cfg( feature = "compression" ) ]
        compression_config : None, // Not configurable in former version for simplicity
      } )
    }
  }


  /// Handle for the dynamic configuration watcher, keeping it alive for the client's lifetime.
  #[ cfg( feature = "dynamic_configuration" ) ]
  #[ derive( Debug ) ]
  pub struct ConfigWatchHandle
  {
    pub( super ) _handle : std::sync::Arc< () >,
  }

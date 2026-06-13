//! Client builder for configuring Gemini API client.

use core::time::Duration;
use reqwest;
use crate::error::Error;
use super::Client;

mod setters_core;
#[ cfg( feature = "retry" ) ]
mod setters_retry;
#[ cfg( feature = "circuit_breaker" ) ]
mod setters_circuit_breaker;
#[ cfg( feature = "caching" ) ]
mod setters_caching;
#[ cfg( feature = "rate_limiting" ) ]
mod setters_rate_limiting;
#[ cfg( feature = "compression" ) ]
mod setters_compression;
#[ cfg( feature = "builder_patterns" ) ]
mod presets;

  /// Builder for configuring and constructing a `Client` instance.
  #[ derive( Debug ) ]
  #[ allow( clippy::struct_excessive_bools ) ] // Configuration struct with feature flags
  pub struct ClientBuilder
  {
    base_url : String,
    api_key : Option< String >,
    timeout : Duration,
    #[ cfg( feature = "retry" ) ]
    max_retries : u32,
    #[ cfg( feature = "retry" ) ]
    base_delay : Duration,
    #[ cfg( feature = "retry" ) ]
    max_delay : Duration,
    #[ cfg( feature = "retry" ) ]
    enable_jitter : bool,
    #[ cfg( feature = "retry" ) ]
    request_timeout : Option< Duration >,
    #[ cfg( feature = "retry" ) ]
    backoff_multiplier : f64,
    #[ cfg( feature = "retry" ) ]
    enable_retry_metrics : bool,
    #[ cfg( feature = "retry" ) ]
    max_elapsed_time : Option< Duration >,
    #[ cfg( feature = "circuit_breaker" ) ]
    enable_circuit_breaker : bool,
    #[ cfg( feature = "circuit_breaker" ) ]
    circuit_breaker_failure_threshold : u32,
    #[ cfg( feature = "circuit_breaker" ) ]
    circuit_breaker_success_threshold : u32,
    #[ cfg( feature = "circuit_breaker" ) ]
    circuit_breaker_timeout : Duration,
    #[ cfg( feature = "circuit_breaker" ) ]
    enable_circuit_breaker_metrics : bool,
    #[ cfg( feature = "circuit_breaker" ) ]
    circuit_breaker_shared_state : bool,
    #[ cfg( feature = "caching" ) ]
    enable_request_cache : bool,
    #[ cfg( feature = "caching" ) ]
    cache_ttl : Duration,
    #[ cfg( feature = "caching" ) ]
    cache_max_size : usize,
    #[ cfg( feature = "caching" ) ]
    enable_cache_metrics : bool,
    #[ cfg( feature = "rate_limiting" ) ]
    enable_rate_limiting : bool,
    #[ cfg( feature = "rate_limiting" ) ]
    rate_limit_requests_per_second : f64,
    #[ cfg( feature = "rate_limiting" ) ]
    rate_limit_algorithm : String,
    #[ cfg( feature = "rate_limiting" ) ]
    rate_limit_bucket_size : usize,
    #[ cfg( feature = "rate_limiting" ) ]
    enable_rate_limiting_metrics : bool,
    #[ cfg( feature = "compression" ) ]
    compression_config : Option< crate::internal::http::compression::CompressionConfig >,
  }

  impl Default for ClientBuilder
  {
    #[ inline ]
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl ClientBuilder
  {
      /// Creates a new `ClientBuilder` with default settings.
    #[ must_use ]
    #[ inline ]
    pub fn new() -> Self
    {
        ClientBuilder
        {
          base_url : "https://generativelanguage.googleapis.com".to_string(),
          api_key : None,
          timeout : Duration::from_secs( 30 ),
          #[ cfg( feature = "retry" ) ]
          max_retries : 3,
          #[ cfg( feature = "retry" ) ]
          base_delay : Duration::from_millis( 100 ),
          #[ cfg( feature = "retry" ) ]
          max_delay : Duration::from_secs( 10 ),
          #[ cfg( feature = "retry" ) ]
          enable_jitter : true,
          #[ cfg( feature = "retry" ) ]
          request_timeout : None,
          #[ cfg( feature = "retry" ) ]
          backoff_multiplier : 2.0,
          #[ cfg( feature = "retry" ) ]
          enable_retry_metrics : false,
          #[ cfg( feature = "retry" ) ]
          max_elapsed_time : Some( Duration::from_secs( 60 ) ),
          #[ cfg( feature = "circuit_breaker" ) ]
          enable_circuit_breaker : false,
          #[ cfg( feature = "circuit_breaker" ) ]
          circuit_breaker_failure_threshold : 5,
          #[ cfg( feature = "circuit_breaker" ) ]
          circuit_breaker_success_threshold : 2,
          #[ cfg( feature = "circuit_breaker" ) ]
          circuit_breaker_timeout : Duration::from_secs( 60 ),
          #[ cfg( feature = "circuit_breaker" ) ]
          enable_circuit_breaker_metrics : false,
          #[ cfg( feature = "circuit_breaker" ) ]
          circuit_breaker_shared_state : false,
          #[ cfg( feature = "caching" ) ]
          enable_request_cache : false,
          #[ cfg( feature = "caching" ) ]
          cache_ttl : Duration::from_secs( 300 ),
          #[ cfg( feature = "caching" ) ]
          cache_max_size : 1000,
          #[ cfg( feature = "caching" ) ]
          enable_cache_metrics : false,
          #[ cfg( feature = "rate_limiting" ) ]
          enable_rate_limiting : false,
          #[ cfg( feature = "rate_limiting" ) ]
          rate_limit_requests_per_second : 10.0,
          #[ cfg( feature = "rate_limiting" ) ]
          rate_limit_algorithm : "token_bucket".to_string(),
          #[ cfg( feature = "rate_limiting" ) ]
          rate_limit_bucket_size : 10,
          #[ cfg( feature = "rate_limiting" ) ]
          enable_rate_limiting_metrics : false,
          #[ cfg( feature = "compression" ) ]
          compression_config : None,
        }
    }

      /// Builds the `Client` with the configured settings.
      ///
      /// # Errors
      ///
      /// Returns an error if the API key is missing or empty.
    #[ allow( clippy::too_many_lines ) ]
    #[ inline ]
    pub fn build( self ) -> Result< Client, Error >
    {
        let api_key = self.api_key
          .ok_or_else( || Error::AuthenticationError( "API key is required".to_string() ) )?;

        // Reject both empty and whitespace-only keys — neither is a valid credential.
        // Root cause of previous bug: `is_empty()` passed "   " (spaces) to the HTTP layer
        // where it caused an unhelpful authentication failure with no client-side context.
        if api_key.trim().is_empty()
        {
          return Err( Error::AuthenticationError( "API key cannot be empty or blank".to_string() ) );
        }

        // Validate retry configuration when retry feature is enabled
        #[ cfg( feature = "retry" ) ]
        {
          // Validate backoff multiplier
          if self.backoff_multiplier <= 1.0
          {
            return Err( Error::InvalidArgument(
              format!( "Backoff multiplier must be greater than 1.0, got : {0}", self.backoff_multiplier )
            ) );
          }

          // Validate delay ranges
          if self.base_delay >= self.max_delay
          {
            return Err( Error::InvalidArgument(
              "Base delay must be less than max delay".to_string()
            ) );
          }
        }

        // Validate circuit breaker configuration when circuit breaker feature is enabled
        #[ cfg( feature = "circuit_breaker" ) ]
        {
          if self.enable_circuit_breaker
          {
            // Validate failure threshold
            if self.circuit_breaker_failure_threshold == 0
            {
              return Err( Error::InvalidArgument(
                "Circuit breaker failure threshold must be greater than 0".to_string()
              ) );
            }

            // Validate success threshold
            if self.circuit_breaker_success_threshold == 0
            {
              return Err( Error::InvalidArgument(
                "Circuit breaker success threshold must be greater than 0".to_string()
              ) );
            }

            // Validate timeout
            if self.circuit_breaker_timeout.is_zero()
            {
              return Err( Error::InvalidArgument(
                "Circuit breaker timeout must be greater than 0".to_string()
              ) );
            }
          }
        }

        // Validate caching configuration when caching feature is enabled
        #[ cfg( feature = "caching" ) ]
        {
          if self.enable_request_cache
          {
            // Validate cache TTL
            if self.cache_ttl.is_zero()
            {
              return Err( Error::InvalidArgument(
                "Cache TTL must be greater than 0".to_string()
              ) );
            }

            // Validate cache max size
            if self.cache_max_size == 0
            {
              return Err( Error::InvalidArgument(
                "Cache max size must be greater than 0".to_string()
              ) );
            }
          }
        }

        // Validate rate limiting configuration when rate limiting feature is enabled
        #[ cfg( feature = "rate_limiting" ) ]
        {
          if self.enable_rate_limiting
          {
            // Validate requests per second
            if self.rate_limit_requests_per_second <= 0.0
            {
              return Err( Error::InvalidArgument(
                "Rate limit requests per second must be greater than 0.0".to_string()
              ) );
            }

            // Validate bucket size
            if self.rate_limit_bucket_size == 0
            {
              return Err( Error::InvalidArgument(
                "Rate limit bucket size must be greater than 0".to_string()
              ) );
            }

            // Validate algorithm
            match self.rate_limit_algorithm.as_str()
            {
              "token_bucket" | "sliding_window" | "adaptive" => {},
              invalid => {
                return Err( Error::InvalidArgument(
                  format!( "Invalid rate limiting algorithm '{invalid}'. Valid options : 'token_bucket', 'sliding_window', 'adaptive'" )
                ) );
              }
            }
          }
        }

        let http_client = reqwest::Client::builder()
          .timeout( self.timeout )
          .build()
          .map_err( |e| Error::NetworkError( format!( "Failed to create HTTP client : {e}" ) ) )?;

        // Create request cache if caching is enabled
        #[ cfg( feature = "caching" ) ]
        let request_cache = if self.enable_request_cache
        {
          let cache_config = crate::internal::http::CacheConfig {
            max_size : self.cache_max_size,
            ttl : self.cache_ttl,
            enable_metrics : self.enable_cache_metrics,
          };
          Some( std::sync::Arc::new( crate::internal::http::RequestCache::new( cache_config ) ) )
        } else {
          None
        };

        Ok( Client
        {
          api_key,
          base_url : self.base_url,
          http : http_client,
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
          request_timeout : self.request_timeout,
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
          enable_circuit_breaker_metrics : self.enable_circuit_breaker_metrics,
          #[ cfg( feature = "circuit_breaker" ) ]
          circuit_breaker_shared_state : self.circuit_breaker_shared_state,
          #[ cfg( feature = "caching" ) ]
          enable_request_cache : self.enable_request_cache,
          #[ cfg( feature = "caching" ) ]
          cache_ttl : self.cache_ttl,
          #[ cfg( feature = "caching" ) ]
          cache_max_size : self.cache_max_size,
          #[ cfg( feature = "caching" ) ]
          enable_cache_metrics : self.enable_cache_metrics,
          #[ cfg( feature = "caching" ) ]
          request_cache,
          #[ cfg( feature = "rate_limiting" ) ]
          enable_rate_limiting : self.enable_rate_limiting,
          #[ cfg( feature = "rate_limiting" ) ]
          rate_limit_requests_per_second : self.rate_limit_requests_per_second,
          #[ cfg( feature = "rate_limiting" ) ]
          rate_limit_algorithm : self.rate_limit_algorithm,
          #[ cfg( feature = "rate_limiting" ) ]
          rate_limit_bucket_size : self.rate_limit_bucket_size,
          #[ cfg( feature = "rate_limiting" ) ]
          enable_rate_limiting_metrics : self.enable_rate_limiting_metrics,
          #[ cfg( feature = "compression" ) ]
          compression_config : self.compression_config,
        } )
    }
  }

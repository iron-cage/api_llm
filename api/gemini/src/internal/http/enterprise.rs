//! Enterprise reliability features combining retry, circuit breaker, rate limiting, and caching

use reqwest::Method;
use serde::{ Serialize, Deserialize };

use crate::error::Error;
use super::HttpConfig;

#[ cfg( feature = "retry" ) ]
use super::retry::{ RetryConfig, is_retryable_error, calculate_retry_delay };

#[ cfg( feature = "circuit_breaker" ) ]
use super::circuit_breaker::{ CircuitBreaker, is_circuit_breaker_error };

#[ cfg( feature = "rate_limiting" ) ]
use super::rate_limiter::RateLimit;

#[ cfg( feature = "caching" ) ]
use super::cache::{ RequestCache, execute_with_cache };

/// Execute an HTTP request with optional retry, circuit breaker, and rate limiting logic based on client configuration
/// This function provides a unified interface that automatically uses enterprise reliability features
/// when available, falling back to legacy execution when features are disabled or unavailable
pub async fn execute_with_optional_retries< T, R >
(
  full_client : &crate::client::Client,
  method : Method,
  url : &str,
  api_key : &str,
  body : Option< &T >,
)
-> Result< R, Error >
where
  T: Serialize,
  R: Serialize + for< 'de > Deserialize< 'de >,
{
  // Use HTTP config with logging enabled in test environment
  #[ allow(unused_mut) ] // May not be mutated depending on feature flags
  let mut http_config = HttpConfig::new();

  // Enable logging during tests when logging feature is active
  #[ cfg( all( feature = "logging", test ) ) ]
  {
    http_config.enable_logging = true;
  }

  // Also enable logging if GEMINI_ENABLE_HTTP_LOGGING environment variable is set
  #[ cfg( feature = "logging" ) ]
  if std::env::var( "GEMINI_ENABLE_HTTP_LOGGING" ).is_ok()
  {
    http_config.enable_logging = true;
  }

  // Add compression configuration if available
  #[ cfg( feature = "compression" ) ]
  {
    http_config.compression_config = full_client.compression_config.clone();
  }

  // Create instances for each configured feature
  #[ cfg( feature = "rate_limiting" ) ]
  let rate_limiter = full_client.to_rate_limiting_config().map( RateLimit::new );
  #[ cfg( not( feature = "rate_limiting" ) ) ]
  let rate_limiter : Option< () > = None;

  #[ cfg( feature = "circuit_breaker" ) ]
  let circuit_breaker = full_client.to_circuit_breaker_config().map( CircuitBreaker::new );
  #[ cfg( not( feature = "circuit_breaker" ) ) ]
  let circuit_breaker : Option< () > = None;

  #[ cfg( feature = "retry" ) ]
  let retry_config = full_client.to_retry_config();
  #[ cfg( not( feature = "retry" ) ) ]
  let retry_config : Option< () > = None;

  #[ cfg( feature = "caching" ) ]
  let cache = full_client.request_cache.as_ref().map( |arc| arc.as_ref() );
  #[ cfg( not( feature = "caching" ) ) ]
  let cache : Option< &() > = None;

  // Execute with the configured features
  execute_with_enterprise_features(
    &full_client.http,
    method,
    url,
    api_key,
    body,
    &http_config,
    rate_limiter.as_ref(),
    circuit_breaker.as_ref(),
    retry_config.as_ref(),
    cache,
  ).await
}

/// Execute an HTTP request with enterprise reliability features (rate limiting, circuit breaker, retry, caching)
// too_many_arguments: each parameter corresponds to one distinct enterprise feature; no natural grouping possible
#[ allow( clippy::too_many_arguments ) ]
pub( crate ) async fn execute_with_enterprise_features< T, R >
(
  client : &reqwest::Client,
  method : Method,
  url : &str,
  api_key : &str,
  body : Option< &T >,
  config : &HttpConfig,
  #[ cfg( feature = "rate_limiting" ) ]
  rate_limiter : Option< &RateLimit >,
  #[ cfg( not( feature = "rate_limiting" ) ) ]
  _rate_limiter : Option< &() >,
  #[ cfg( feature = "circuit_breaker" ) ]
  circuit_breaker : Option< &CircuitBreaker >,
  #[ cfg( not( feature = "circuit_breaker" ) ) ]
  _circuit_breaker : Option< &() >,
  #[ cfg( feature = "retry" ) ]
  retry_config : Option< &RetryConfig >,
  #[ cfg( not( feature = "retry" ) ) ]
  _retry_config : Option< &() >,
  #[ cfg( feature = "caching" ) ]
  cache : Option< &RequestCache >,
  #[ cfg( not( feature = "caching" ) ) ]
  _cache : Option< &() >,
)
-> Result< R, Error >
where
  T: Serialize,
  R: Serialize + for< 'de > Deserialize< 'de >,
{
  // Helper function to execute one attempt with rate limiting and circuit breaker
  let execute_single_attempt = || async {
    // Check rate limiting first
    #[ cfg( feature = "rate_limiting" ) ]
    if let Some( rl ) = rate_limiter
    {
      if !rl.should_allow_request().await
      {
        return Err( Error::RateLimited( "Rate limit exceeded".to_string() ) );
      }
    }

    // Then check circuit breaker
    #[ cfg( feature = "circuit_breaker" ) ]
    if let Some( cb ) = circuit_breaker
    {
      if !cb.should_allow_request()
      {
        return Err( Error::CircuitBreakerOpen( "Circuit breaker is open".to_string() ) );
      }
    }

    // Execute the actual request with caching if available
    #[ cfg( feature = "caching" ) ]
    let result = execute_with_cache( client, method.clone(), url, api_key, body, config, cache ).await;
    #[ cfg( not( feature = "caching" ) ) ]
    let result = super::execute( client, method.clone(), url, api_key, body, config ).await;

    // Record circuit breaker results
    #[ cfg( feature = "circuit_breaker" ) ]
    if let Some( cb ) = circuit_breaker
    {
      match &result
      {
        Ok( _ ) => cb.record_success(),
        Err( error ) if is_circuit_breaker_error( error ) => cb.record_failure(),
        _ => {} // Dont count non-circuit-breaker errors
      }
    }

    result
  };

  // Use retry logic if available
  #[ cfg( feature = "retry" ) ]
  {
    if let Some( retry_cfg ) = retry_config
    {
      let start_time = std::time::Instant::now();
      let mut attempt = 1;

      loop
      {
        match execute_single_attempt().await
        {
          Ok( response ) => return Ok( response ),
          Err( error ) => {
            // Check if we should retry this error
            if !is_retryable_error( &error ) || attempt > retry_cfg.max_retries
            {
              return Err( error );
            }

            // Check max elapsed time
            if let Some( max_elapsed ) = retry_cfg.max_elapsed_time
            {
              if start_time.elapsed() >= max_elapsed
              {
                return Err( error );
              }
            }

            // Calculate and apply retry delay
            let delay = calculate_retry_delay( attempt, retry_cfg );
            tokio ::time::sleep( delay ).await;
            attempt += 1;
          }
        }
      }
    } else {
      execute_single_attempt().await
    }
  }

  #[ cfg( not( feature = "retry" ) ) ]
  {
    execute_single_attempt().await
  }
}

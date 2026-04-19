//! Enhanced retry logic implementation for HTTP requests

use core::time::Duration;
use std::time::Instant;
use reqwest::{ Client, Method };
use serde::Serialize;
use serde::Deserialize;

use crate::error::Error;

#[ cfg( feature = "logging" ) ]
use tracing::{ warn, debug };
use rand::RngExt;

/// Retry configuration extracted from client for HTTP layer usage
#[ derive( Debug, Clone ) ]
pub struct RetryConfig
{
  /// Maximum number of retry attempts
  pub max_retries : u32,
  /// Base delay between retries
  pub base_delay : Duration,
  /// Maximum delay between retries
  pub max_delay : Duration,
  /// Multiplier for exponential backoff
  pub backoff_multiplier : f64,
  /// Whether to add jitter to delays
  pub enable_jitter : bool,
  /// Maximum total elapsed time for all retries
  pub max_elapsed_time : Option< Duration >,
}

/// Retry metrics for tracking retry behavior
#[ derive( Debug, Clone, Default ) ]
pub struct RetryMetrics
{
  /// Total number of retry attempts made
  pub total_retries : u32,
  /// Total time spent on retries
  pub total_retry_time : Duration,
  /// Number of successful retry attempts
  pub successful_retries : u32,
  /// Number of failed retry attempts
  pub failed_retries : u32,
}

/// Determines if an error is retryable based on error type
pub fn is_retryable_error( error : &Error ) -> bool
{
  match error
  {
    // Retryable errors (transient failures)
    Error::NetworkError( _ ) => true,
    Error::ServerError( _ ) => true,
    Error::TimeoutError( _ ) => true,
    Error::RateLimitError( _ ) => true,

    // Non-retryable errors (permanent failures)
    Error::AuthenticationError( _ ) => false,
    Error::InvalidArgument( _ ) => false,
    Error::DeserializationError( _ ) => false,
    Error::SerializationError( _ ) => false,
    Error::RequestBuilding( _ ) => false,
    Error::NotFound( _ ) => false,

    // API errors could be either, but typically should not be retried
    Error::ApiError( _ ) => false,

    // Unknown errors and other types default to non-retryable for safety
    _ => false,
  }
}

/// Calculate retry delay with exponential backoff and optional jitter
pub fn calculate_retry_delay(
  attempt : u32,
  config : &RetryConfig
) -> Duration
{
  // Exponential backoff : base_delay * multiplier^(attempt-1)
  let base_delay_ms = config.base_delay.as_millis() as f64;
  let multiplier = config.backoff_multiplier;
  let backoff_delay_ms = base_delay_ms * multiplier.powi( ( attempt - 1 ) as i32 );

  let mut delay_ms = backoff_delay_ms as u64;

  // Apply jitter if enabled : add random variation up to 50% of delay
  if config.enable_jitter && delay_ms > 0
  {
    let jitter_range = delay_ms / 2; // 50% jitter
    let jitter = rand::rng().random_range( 0..=jitter_range );
    delay_ms += jitter;
  }

  // Cap at max_delay
  let max_delay_ms = config.max_delay.as_millis() as u64;
  delay_ms = delay_ms.min( max_delay_ms );

  Duration::from_millis( delay_ms )
}

/// Execute HTTP request with retry logic
pub async fn execute_with_retries< T, R >
(
  client : &Client,
  method : Method,
  url : &str,
  api_key : &str,
  body : Option< &T >,
  config : &super::HttpConfig,
  retry_config : Option< &RetryConfig >,
)
-> Result< R, Error >
where
  T: Serialize,
  R: for< 'de > Deserialize< 'de >,
{
  let Some( retry_config ) = retry_config else {
    // No retry config - fall back to regular execution
    return super::execute( client, method.clone(), url, api_key, body, config ).await;
  };

  let start_time = Instant::now();
  let mut attempt = 1;
  let mut _last_error = None;

  loop
  {
    #[ cfg( feature = "logging" ) ]
    if config.enable_logging && attempt > 1
    {
      debug!(
        attempt = attempt,
        url = %url,
        "Retry attempt"
      );
    }

    // Execute the request
    match super::execute( client, method.clone(), url, api_key, body, config ).await
    {
      Ok( response ) => {
        #[ cfg( feature = "logging" ) ]
        if config.enable_logging && attempt > 1
        {
          debug!(
            attempt = attempt,
            url = %url,
            "Request succeeded after retries"
          );
        }
        return Ok( response );
      },
      Err( error ) => {
        _last_error = Some( error.clone() );

        // Check if we should retry this error
        if !is_retryable_error( &error )
        {
          #[ cfg( feature = "logging" ) ]
          if config.enable_logging
          {
            debug!(
              error = %error,
              url = %url,
              "Non-retryable error encountered"
            );
          }
          return Err( error );
        }

        // Check if we've exceeded max attempts
        if attempt > retry_config.max_retries  // First attempt isnt counted as a retry
        {
          #[ cfg( feature = "logging" ) ]
          if config.enable_logging
          {
            warn!(
              max_retries = retry_config.max_retries,
              url = %url,
              "Max retry attempts exceeded"
            );
          }
          return Err( error );
        }

        // Check if we've exceeded max elapsed time
        if let Some( max_elapsed ) = retry_config.max_elapsed_time
        {
          if start_time.elapsed() >= max_elapsed
          {
            #[ cfg( feature = "logging" ) ]
            if config.enable_logging
            {
              warn!(
                elapsed_ms = start_time.elapsed().as_millis(),
                max_elapsed_ms = max_elapsed.as_millis(),
                url = %url,
                "Max elapsed time exceeded"
              );
            }
            return Err( error );
          }
        }

        // Calculate and apply retry delay
        let delay = calculate_retry_delay( attempt, retry_config );

        #[ cfg( feature = "logging" ) ]
        if config.enable_logging
        {
          debug!(
            attempt = attempt,
            delay_ms = delay.as_millis(),
            error = %error,
            url = %url,
            "Retrying after delay"
          );
        }

        tokio ::time::sleep( delay ).await;
        attempt += 1;
      }
    }
  }
}

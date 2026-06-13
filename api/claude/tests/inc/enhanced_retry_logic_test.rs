//! Enhanced Retry Logic Tests with Anthropic Header Parsing
//!
//! # Knowledge : Anthropic Rate Limit Header Integration
//!
//! ## Why 6 Headers Matter
//!
//! Anthropic provides 6 rate limit headers in HTTP 429 responses that enable intelligent retry:
//! - `anthropic-ratelimit-requests-limit`: Maximum requests allowed in the time window
//! - `anthropic-ratelimit-requests-remaining`: Requests left before rate limiting
//! - `anthropic-ratelimit-requests-reset`: When request quota resets (RFC3339 timestamp)
//! - `anthropic-ratelimit-tokens-limit`: Maximum tokens allowed in the time window
//! - `anthropic-ratelimit-tokens-remaining`: Tokens left before rate limiting
//! - `anthropic-ratelimit-tokens-reset`: When token quota resets (RFC3339 timestamp)
//!
//! ## Usage Percentage Calculations
//!
//! We calculate usage percentages to make proactive decisions:
//! - `requests_usage_percentage = (limit - remaining) / limit`
//! - `tokens_usage_percentage = (limit - remaining) / limit`
//!
//! **Example:** If limit=1000, remaining=250:
//! - Used : 750 requests (75% of quota)
//! - This signals approaching rate limit, can trigger warnings/throttling
//!
//! ## Server-Provided retry-after
//!
//! The `retry-after` value in HTTP 429 responses tells us EXACTLY when to retry.
//! This is more accurate than exponential backoff guessing because:
//! - Server knows its rate limit window reset time
//! - Prevents unnecessary early retries (wasting requests)
//! - Prevents late retries (missing available quota)
//!
//! ## Integration with Retry Strategy
//!
//! Our retry strategy respects server guidance:
//! 1. If `retry-after` present : use that duration (server knows best)
//! 2. If rate limit headers present : calculate optimal delay from reset times
//! 3. Fallback to exponential backoff only when no server guidance available
//!
//! ## Zero Overhead Design
//!
//! Header parsing is only triggered on HTTP 429 responses.
//! Normal successful requests have zero parsing overhead.

#[ cfg( all( feature = "retry-logic", feature = "error-handling" ) ) ]
mod enhanced_retry_tests
{
  use crate::inc::the_module;

  #[ test ]
  fn test_anthropic_rate_limit_info_from_headers()
  {
    use reqwest::header::{ HeaderMap, HeaderValue };

    let mut headers = HeaderMap::new();
    headers.insert( "anthropic-ratelimit-requests-limit", HeaderValue::from_static( "1000" ) );
    headers.insert( "anthropic-ratelimit-requests-remaining", HeaderValue::from_static( "50" ) );
    headers.insert( "anthropic-ratelimit-requests-reset", HeaderValue::from_static( "2024-01-01T00:00:00Z" ) );
    headers.insert( "anthropic-ratelimit-tokens-limit", HeaderValue::from_static( "100000" ) );
    headers.insert( "anthropic-ratelimit-tokens-remaining", HeaderValue::from_static( "25000" ) );
    headers.insert( "anthropic-ratelimit-tokens-reset", HeaderValue::from_static( "2024-01-01T00:05:00Z" ) );

    let info = the_module::AnthropicRateLimitInfo::from_headers( &headers );

    assert_eq!( info.requests_limit, Some( 1000 ) );
    assert_eq!( info.requests_remaining, Some( 50 ) );
    assert_eq!( info.requests_reset, Some( "2024-01-01T00:00:00Z".to_string() ) );
    assert_eq!( info.tokens_limit, Some( 100_000 ) );
    assert_eq!( info.tokens_remaining, Some( 25000 ) );
    assert_eq!( info.tokens_reset, Some( "2024-01-01T00:05:00Z".to_string() ) );

    assert!( info.has_data() );
  }

  #[ test ]
  fn test_anthropic_rate_limit_info_empty_headers()
  {
    use reqwest::header::HeaderMap;

    let headers = HeaderMap::new();
    let info = the_module::AnthropicRateLimitInfo::from_headers( &headers );

    assert_eq!( info.requests_limit, None );
    assert_eq!( info.requests_remaining, None );
    assert_eq!( info.tokens_limit, None );
    assert_eq!( info.tokens_remaining, None );

    assert!( !info.has_data() );
  }

  #[ test ]
  fn test_anthropic_rate_limit_info_partial_headers()
  {
    use reqwest::header::{ HeaderMap, HeaderValue };

    let mut headers = HeaderMap::new();
    headers.insert( "anthropic-ratelimit-requests-limit", HeaderValue::from_static( "1000" ) );
    headers.insert( "anthropic-ratelimit-requests-remaining", HeaderValue::from_static( "100" ) );
    // Missing token headers

    let info = the_module::AnthropicRateLimitInfo::from_headers( &headers );

    assert_eq!( info.requests_limit, Some( 1000 ) );
    assert_eq!( info.requests_remaining, Some( 100 ) );
    assert_eq!( info.tokens_limit, None );
    assert_eq!( info.tokens_remaining, None );

    assert!( info.has_data() );
  }

  #[ test ]
  fn test_anthropic_rate_limit_info_usage_percentage()
  {
    use reqwest::header::{ HeaderMap, HeaderValue };

    let mut headers = HeaderMap::new();
    headers.insert( "anthropic-ratelimit-requests-limit", HeaderValue::from_static( "1000" ) );
    headers.insert( "anthropic-ratelimit-requests-remaining", HeaderValue::from_static( "250" ) );
    headers.insert( "anthropic-ratelimit-tokens-limit", HeaderValue::from_static( "100000" ) );
    headers.insert( "anthropic-ratelimit-tokens-remaining", HeaderValue::from_static( "10000" ) );

    let info = the_module::AnthropicRateLimitInfo::from_headers( &headers );

    // 750/1000 = 0.75 (75% used)
    let requests_usage = info.requests_usage_percentage().unwrap();
    assert!( ( requests_usage - 0.75 ).abs() < 0.01 );

    // 90000/100000 = 0.9 (90% used)
    let tokens_usage = info.tokens_usage_percentage().unwrap();
    assert!( ( tokens_usage - 0.9 ).abs() < 0.01 );
  }

  #[ test ]
  fn test_rate_limit_error_with_headers()
  {
    use reqwest::header::{ HeaderMap, HeaderValue };

    let mut headers = HeaderMap::new();
    headers.insert( "anthropic-ratelimit-requests-limit", HeaderValue::from_static( "1000" ) );
    headers.insert( "anthropic-ratelimit-requests-remaining", HeaderValue::from_static( "0" ) );

    let info = the_module::AnthropicRateLimitInfo::from_headers( &headers );
    let error = the_module::RateLimitError::with_headers(
      "Rate limit exceeded".to_string(),
      Some( 60 ),
      "requests".to_string(),
      info
    );

    assert_eq!( error.retry_after(), &Some( 60 ) );
    assert_eq!( error.limit_type(), "requests" );
    assert!( error.rate_limit_info().is_some() );

    let rate_info = error.rate_limit_info().unwrap();
    assert_eq!( rate_info.requests_limit, Some( 1000 ) );
    assert_eq!( rate_info.requests_remaining, Some( 0 ) );
  }

  #[ test ]
  fn test_rate_limit_error_display_with_headers()
  {
    use reqwest::header::{ HeaderMap, HeaderValue };

    let mut headers = HeaderMap::new();
    headers.insert( "anthropic-ratelimit-requests-limit", HeaderValue::from_static( "1000" ) );
    headers.insert( "anthropic-ratelimit-requests-remaining", HeaderValue::from_static( "50" ) );
    headers.insert( "anthropic-ratelimit-tokens-limit", HeaderValue::from_static( "100000" ) );
    headers.insert( "anthropic-ratelimit-tokens-remaining", HeaderValue::from_static( "25000" ) );

    let info = the_module::AnthropicRateLimitInfo::from_headers( &headers );
    let error = the_module::RateLimitError::with_headers(
      "Rate limit exceeded".to_string(),
      Some( 30 ),
      "requests".to_string(),
      info
    );

    let display = format!( "{error}" );
    assert!( display.contains( "Rate limit exceeded" ) );
    assert!( display.contains( "retry after 30s" ) );
    assert!( display.contains( "[requests : 50/1000]" ) );
    assert!( display.contains( "[tokens : 25000/100000]" ) );
  }

  #[ test ]
  fn test_rate_limit_error_without_headers()
  {
    let error = the_module::RateLimitError::new(
      "Rate limit exceeded".to_string(),
      Some( 60 ),
      "requests".to_string()
    );

    assert_eq!( error.retry_after(), &Some( 60 ) );
    assert_eq!( error.limit_type(), "requests" );
    assert!( error.rate_limit_info().is_none() );
  }

  #[ test ]
  fn test_retry_strategy_with_rate_limit_error()
  {
    use reqwest::header::{ HeaderMap, HeaderValue };

    let mut headers = HeaderMap::new();
    headers.insert( "anthropic-ratelimit-requests-remaining", HeaderValue::from_static( "0" ) );

    let info = the_module::AnthropicRateLimitInfo::from_headers( &headers );
    let rate_error = the_module::RateLimitError::with_headers(
      "Rate limit exceeded".to_string(),
      Some( 5 ), // 5 seconds retry-after
      "requests".to_string(),
      info
    );

    let config = the_module::RetryConfig::new()
      .with_max_attempts( 3 )
      .with_base_delay_ms( 1000 );

    let strategy = the_module::RetryStrategy::exponential_backoff_with_config( config );

    // Calculate delay for rate limit error - should respect retry-after
    let delay = strategy.calculate_delay_for_error( &rate_error, 1 );

    // Delay should be at least 5000ms (5 seconds from retry-after header)
    assert!( delay >= 5000 );
  }

  #[ test ]
  fn test_retry_strategy_error_classification()
  {
    use reqwest::header::HeaderMap;

    let info = the_module::AnthropicRateLimitInfo::from_headers( &HeaderMap::new() );
    let rate_error = the_module::AnthropicError::RateLimit(
      the_module::RateLimitError::with_headers(
        "Rate limit".to_string(),
        None,
        "requests".to_string(),
        info
      )
    );

    let config = the_module::RetryConfig::new();
    let strategy = the_module::RetryStrategy::exponential_backoff_with_config( config );

    // Rate limit errors should be retryable
    assert!( strategy.should_retry( &rate_error, 1 ) );

    // But not after max attempts
    assert!( !strategy.should_retry( &rate_error, 3 ) );
  }

  #[ test ]
  fn test_retry_config_validation()
  {
    let valid_config = the_module::RetryConfig::new()
      .with_max_attempts( 3 )
      .with_base_delay_ms( 1000 )
      .with_max_delay_ms( 60000 )
      .with_backoff_multiplier( 2.0 );

    assert!( valid_config.validate().is_ok() );
    assert!( valid_config.is_valid() );

    // Invalid : max_delay < base_delay
    let invalid_config = the_module::RetryConfig::new()
      .with_base_delay_ms( 5000 )
      .with_max_delay_ms( 1000 );

    assert!( invalid_config.validate().is_err() );
    assert!( !invalid_config.is_valid() );
  }

  #[ test ]
  fn test_exponential_backoff_calculation()
  {
    let config = the_module::RetryConfig::new()
      .with_max_attempts( 5 )
      .with_base_delay_ms( 1000 )
      .with_max_delay_ms( 60000 )
      .with_backoff_multiplier( 2.0 )
      .with_jitter( false );

    let strategy = the_module::RetryStrategy::exponential_backoff_with_config( config );

    // Attempt 1: 1000 * 2^0 = 1000ms
    let delay1 = strategy.calculate_delay_with_jitter_config( 1, None, None );
    assert_eq!( delay1, 1000 );

    // Attempt 2: 1000 * 2^1 = 2000ms
    let delay2 = strategy.calculate_delay_with_jitter_config( 2, None, None );
    assert_eq!( delay2, 2000 );

    // Attempt 3: 1000 * 2^2 = 4000ms
    let delay3 = strategy.calculate_delay_with_jitter_config( 3, None, None );
    assert_eq!( delay3, 4000 );

    // Attempt 4: 1000 * 2^3 = 8000ms
    let delay4 = strategy.calculate_delay_with_jitter_config( 4, None, None );
    assert_eq!( delay4, 8000 );
  }

  #[ test ]
  fn test_linear_backoff_calculation()
  {
    let config = the_module::RetryConfig::new()
      .with_max_attempts( 5 )
      .with_base_delay_ms( 1000 )
      .with_jitter( false );

    let strategy = the_module::RetryStrategy::linear_backoff_with_config( config );

    // Attempt 1: 1000 * 1 = 1000ms
    let delay1 = strategy.calculate_delay_with_jitter_config( 1, None, None );
    assert_eq!( delay1, 1000 );

    // Attempt 2: 1000 * 2 = 2000ms
    let delay2 = strategy.calculate_delay_with_jitter_config( 2, None, None );
    assert_eq!( delay2, 2000 );

    // Attempt 3: 1000 * 3 = 3000ms
    let delay3 = strategy.calculate_delay_with_jitter_config( 3, None, None );
    assert_eq!( delay3, 3000 );
  }

  #[ test ]
  fn test_max_delay_capping()
  {
    let config = the_module::RetryConfig::new()
      .with_max_attempts( 10 )
      .with_base_delay_ms( 1000 )
      .with_max_delay_ms( 5000 )
      .with_backoff_multiplier( 2.0 )
      .with_jitter( false );

    let strategy = the_module::RetryStrategy::exponential_backoff_with_config( config );

    // Attempt 5: 1000 * 2^4 = 16000ms, but should be capped at 5000ms
    let delay5 = strategy.calculate_delay_with_jitter_config( 5, None, None );
    assert_eq!( delay5, 5000 );

    // Attempt 10: would be huge, but capped at 5000ms
    let delay10 = strategy.calculate_delay_with_jitter_config( 10, None, None );
    assert_eq!( delay10, 5000 );
  }

  #[ test ]
  fn test_retry_metrics()
  {
    let mut metrics = the_module::RetryMetrics::new();

    // Record some attempts
    metrics.record_attempt( 1, 1000 );
    metrics.record_attempt( 2, 2000 );
    metrics.record_attempt( 3, 4000 );

    assert_eq!( metrics.total_attempts(), 3 );
    assert_eq!( metrics.total_delay_ms(), 7000 );

    // Record success on attempt 3
    metrics.record_success( 3 );
    assert_eq!( metrics.successful_retries(), 1 );

    // Record failure
    let error = the_module::AnthropicError::RateLimit(
      the_module::RateLimitError::new(
        "Rate limit".to_string(),
        None,
        "requests".to_string()
      )
    );
    metrics.record_failure( &error );
    assert_eq!( metrics.failed_attempts(), 1 );
  }

  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
async fn integration_retry_with_explicit_config()
  {
    use core::time::Duration;

    let client = the_module::Client::from_workspace()
      .expect( "Failed to create client from workspace secrets" );

    // Test explicit retry with manual configuration by making a simple API call
    // Note : This won't actually retry unless there's an error, but it tests the pattern
    let result = client
      .explicit_retry()
      .with_attempts( 3 )
      .with_delay( Duration::from_secs( 1 ) )
      .execute( | _client | async {
        // Simple operation that returns Ok immediately
        Ok( "success".to_string() )
      } )
      .await;

    assert!( result.is_ok() );
    assert_eq!( result.unwrap(), "success" );
  }

  #[ test ]
  fn test_anthropic_rate_limit_info_invalid_header_values()
  {
    use reqwest::header::{ HeaderMap, HeaderValue };

    let mut headers = HeaderMap::new();
    // Invalid number format
    headers.insert( "anthropic-ratelimit-requests-limit", HeaderValue::from_static( "not-a-number" ) );
    headers.insert( "anthropic-ratelimit-requests-remaining", HeaderValue::from_static( "also-invalid" ) );

    let info = the_module::AnthropicRateLimitInfo::from_headers( &headers );

    // Should gracefully handle invalid values by returning None
    assert_eq!( info.requests_limit, None );
    assert_eq!( info.requests_remaining, None );
    assert!( !info.has_data() );
  }
}

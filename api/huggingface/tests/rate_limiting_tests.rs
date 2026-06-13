//! Integration tests for Rate Limiting
//!
//! These tests use REAL `HuggingFace` API calls to verify rate limiting behavior.
//! NO MOCKING is used - all tests interact with actual endpoints.
//!
//! ## Test Strategy
//!
//! - Use real `HuggingFace` API endpoints
//! - Test actual rate limiting with real requests
//! - Test token refill mechanics with timing

#![ allow( clippy::doc_markdown ) ]
//! - Test all time windows ( per-second, per-minute, per-hour )
//!
//! ## Running Tests
//!
//! These tests require:
//! - HuggingFace API key ( `HUGGINGFACE_API_KEY` or `INFERENCE_API_KEY` env var )
//! - Network connectivity
//! - Real API quota/limits
//!
//! Run with:
//! ```bash
//! cargo test --test rate_limiting_tests --all-features -- --ignored
//! ```

mod inc;

use api_huggingface::reliability::{ RateLimiter, RateLimiterConfig };
use core::time::Duration;
use std::time::Instant;

#[ cfg( feature = "integration" ) ]
use api_huggingface::{
  Client,
  environment::HuggingFaceEnvironmentImpl,
  providers::ChatMessage,
  Secret,
};

#[ cfg( feature = "integration" ) ]
fn create_integration_client() -> Client< HuggingFaceEnvironmentImpl >
{
  let api_key = crate::inc::get_api_key_for_integration();
  let secret = Secret::new( api_key );
  let env = HuggingFaceEnvironmentImpl::build( secret, None )
    .expect( "Failed to build environment" );
  Client::build( env ).expect( "Failed to create client" )
}

// ============================================================================
// Basic Rate Limiting Tests
// ============================================================================

#[ tokio::test ]
async fn test_rate_limiter_per_second_limit() 
{
  let config = RateLimiterConfig {
  requests_per_second : Some( 2 ),
  requests_per_minute : None,
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  // First two requests should succeed
  assert!( limiter.try_acquire( ).await.is_ok( ));
  assert!( limiter.try_acquire( ).await.is_ok( ));

  // Third request should fail ( rate limited )
  assert!( limiter.try_acquire( ).await.is_err( ));
}

#[ tokio::test ]
async fn test_rate_limiter_per_minute_limit() 
{
  let config = RateLimiterConfig {
  requests_per_second : None,
  requests_per_minute : Some( 3 ),
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  // First three requests should succeed
  for _ in 0..3
  {
  assert!( limiter.try_acquire( ).await.is_ok( ));
  }

  // Fourth request should fail
  assert!( limiter.try_acquire( ).await.is_err( ));
}

#[ tokio::test ]
async fn test_rate_limiter_per_hour_limit() 
{
  let config = RateLimiterConfig {
  requests_per_second : None,
  requests_per_minute : None,
  requests_per_hour : Some( 5 ),
  };
  let limiter = RateLimiter::new( config );

  // First five requests should succeed
  for _ in 0..5
  {
  assert!( limiter.try_acquire( ).await.is_ok( ));
  }

  // Sixth request should fail
  assert!( limiter.try_acquire( ).await.is_err( ));
}

// ============================================================================
// Token Refill Tests
// ============================================================================

#[ tokio::test ]
async fn test_rate_limiter_token_refill() 
{
  let config = RateLimiterConfig {
  requests_per_second : Some( 10 ),
  requests_per_minute : None,
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  // Consume all tokens
  for _ in 0..10
  {
  assert!( limiter.try_acquire( ).await.is_ok( ));
  }

  // Should be rate limited
  assert!( limiter.try_acquire( ).await.is_err( ));

  // Wait for token refill ( 100ms = 1 token )
  tokio::time::sleep( Duration::from_millis( 150 )).await;

  // Should have at least one token now
  assert!( limiter.try_acquire( ).await.is_ok( ));
}

#[ tokio::test ]
async fn test_rate_limiter_gradual_refill() 
{
  let config = RateLimiterConfig {
  requests_per_second : Some( 10 ),
  requests_per_minute : None,
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  // Consume all tokens
  for _ in 0..10
  {
  limiter.try_acquire( ).await.unwrap( );
  }

  // Wait for 500ms ( should refill ~5 tokens )
  tokio::time::sleep( Duration::from_millis( 500 )).await;

  // Should be able to consume ~5 tokens
  let mut count = 0;
  for _ in 0..10
  {
  if limiter.try_acquire( ).await.is_ok( )
  {
      count += 1;
  } else {
      break;
  }
  }

  // Should have gotten at least 4 tokens ( accounting for timing variance )
  assert!( count >= 4, "Expected at least 4 tokens, got {count}" );
}

// ============================================================================
// Multiple Time Windows Tests
// ============================================================================

#[ tokio::test ]
async fn test_rate_limiter_multiple_windows() 
{
  let config = RateLimiterConfig {
  requests_per_second : Some( 5 ),
  requests_per_minute : Some( 10 ),
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  // Consume 5 requests ( hits per-second limit )
  for _ in 0..5
  {
  assert!( limiter.try_acquire( ).await.is_ok( ));
  }

  // Should be rate limited by per-second window
  let result = limiter.try_acquire( ).await;
  assert!( result.is_err( ));
}

#[ tokio::test ]
async fn test_rate_limiter_minute_limit_after_second_refill() 
{
  let config = RateLimiterConfig {
  requests_per_second : Some( 3 ),
  requests_per_minute : Some( 5 ),
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  // Consume 3 requests ( hits per-second limit )
  for _ in 0..3
  {
  limiter.try_acquire( ).await.unwrap( );
  }

  // Wait for per-second refill
  tokio::time::sleep( Duration::from_secs( 1 )).await;

  // Consume 2 more ( now at 5 total for minute )
  limiter.try_acquire( ).await.unwrap( );
  limiter.try_acquire( ).await.unwrap( );

  // Wait for per-second refill again
  tokio::time::sleep( Duration::from_secs( 1 )).await;

  // Try another - should be blocked by per-minute limit
  assert!( limiter.try_acquire( ).await.is_err( ));
}

// ============================================================================
// Blocking Acquire Tests
// ============================================================================

#[ tokio::test ]
async fn test_rate_limiter_acquire_blocks_and_succeeds() 
{
  let config = RateLimiterConfig {
  requests_per_second : Some( 2 ),
  requests_per_minute : None,
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  // Consume all tokens
  limiter.try_acquire( ).await.unwrap( );
  limiter.try_acquire( ).await.unwrap( );

  // This should block until token available
  let start = Instant::now( );
  limiter.acquire( ).await.unwrap( );
  let elapsed = start.elapsed( );

  // Should have waited at least 400ms ( allowing some margin )
  assert!( elapsed >= Duration::from_millis( 400 ), "Expected wait >=400ms, got {elapsed:?}" );
}

#[ tokio::test ]
async fn test_rate_limiter_acquire_with_available_tokens() 
{
  let config = RateLimiterConfig {
  requests_per_second : Some( 5 ),
  requests_per_minute : None,
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  // Acquire should not block when tokens available
  let start = Instant::now( );
  limiter.acquire( ).await.unwrap( );
  let elapsed = start.elapsed( );

  // Should be nearly instant
  assert!( elapsed < Duration::from_millis( 50 ), "Expected instant acquire, got {elapsed:?}" );
}

// ============================================================================
// Available Tokens Tests
// ============================================================================

#[ tokio::test ]
async fn test_rate_limiter_available_tokens_initial() 
{
  let config = RateLimiterConfig {
  requests_per_second : Some( 10 ),
  requests_per_minute : Some( 100 ),
  requests_per_hour : Some( 1000 ),
  };
  let limiter = RateLimiter::new( config );

  let tokens = limiter.available_tokens( ).await;
  assert_eq!( tokens.per_second, Some( 10 ));
  assert_eq!( tokens.per_minute, Some( 100 ));
  assert_eq!( tokens.per_hour, Some( 1000 ));
}

#[ tokio::test ]
async fn test_rate_limiter_available_tokens_after_consumption() 
{
  let config = RateLimiterConfig {
  requests_per_second : Some( 5 ),
  requests_per_minute : Some( 10 ),
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  // Consume 2 tokens
  limiter.try_acquire( ).await.unwrap( );
  limiter.try_acquire( ).await.unwrap( );

  let tokens = limiter.available_tokens( ).await;
  assert_eq!( tokens.per_second, Some( 3 ));
  assert_eq!( tokens.per_minute, Some( 8 ));
  assert_eq!( tokens.per_hour, None );
}

#[ tokio::test ]
async fn test_rate_limiter_available_tokens_tracks_refill() 
{
  let config = RateLimiterConfig {
  requests_per_second : Some( 10 ),
  requests_per_minute : None,
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  // Consume all tokens
  for _ in 0..10
  {
  limiter.try_acquire( ).await.unwrap( );
  }

  let tokens_before = limiter.available_tokens( ).await;
  assert_eq!( tokens_before.per_second, Some( 0 ));

  // Wait for refill
  tokio::time::sleep( Duration::from_millis( 200 )).await;

  let tokens_after = limiter.available_tokens( ).await;
  assert!( tokens_after.per_second.unwrap( ) >= 1 );
}

// ============================================================================
// Reset Tests
// ============================================================================

#[ tokio::test ]
async fn test_rate_limiter_reset() 
{
  let config = RateLimiterConfig {
  requests_per_second : Some( 3 ),
  requests_per_minute : Some( 5 ),
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  // Consume all tokens
  for _ in 0..3
  {
  limiter.try_acquire( ).await.unwrap( );
  }

  // Should be rate limited
  assert!( limiter.try_acquire( ).await.is_err( ));

  // Reset
  limiter.reset( ).await;

  // Should have full tokens again
  let tokens = limiter.available_tokens( ).await;
  assert_eq!( tokens.per_second, Some( 3 ));
  assert_eq!( tokens.per_minute, Some( 5 ));

  // Should be able to acquire
  assert!( limiter.try_acquire( ).await.is_ok( ));
}

// ============================================================================
// Real API Integration Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_rate_limiter_with_real_api_calls()
{
  let client = create_integration_client();
  // Use per-minute limit so the bucket does not refill during API call latency.
  let config = RateLimiterConfig {
  requests_per_second : None,
  requests_per_minute : Some( 2 ),
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  // Make 2 real API calls with rate limiting — both must succeed.
  for i in 0..2
  {
  limiter.acquire( ).await.unwrap( );

  let result = client.providers( ).chat_completion(
      "meta-llama/Llama-3.3-70B-Instruct",
      vec![ChatMessage {
  role : "user".to_string( ),
  content : format!( "Say the number {i}" ),
  tool_calls : None,
  tool_call_id : None,
      } ],
      Some( 10 ),
      None,
      None,
  ).await;

  assert!( result.is_ok( ), "API call {i} should succeed" );
  }

  // Per-minute bucket is now exhausted — immediate try_acquire must fail.
  let rate_limited = limiter.try_acquire( ).await;
  assert!( rate_limited.is_err( ), "Rate limiter must block after per-minute quota exhausted" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_rate_limiter_prevents_api_overload()
{
  let client = create_integration_client();
  let config = RateLimiterConfig {
  requests_per_second : Some( 3 ),
  requests_per_minute : Some( 5 ),
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  let mut successful_calls = 0;
  let mut rate_limited_calls = 0;

  // Try to make 10 calls rapidly
  for i in 0..10
  {
  if limiter.try_acquire( ).await.is_ok( )
  {
      let result = client.providers( ).chat_completion(
  "meta-llama/Llama-3.3-70B-Instruct",
  vec![ChatMessage {
          role : "user".to_string( ),
          content : format!( "Count {i}" ),
          tool_calls : None,
          tool_call_id : None,
  } ],
  Some( 5 ),
  None,
  None,
      ).await;

      if result.is_ok( )
      {
  successful_calls += 1;
      }
  } else {
      rate_limited_calls += 1;
  }
  }

  // Should have limited some calls
  assert!( rate_limited_calls > 0, "Expected some calls to be rate limited" );
  assert!( successful_calls <= 5, "Should not exceed minute limit" );
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[ tokio::test ]
async fn test_rate_limiter_concurrent_acquire() 
{
  let config = RateLimiterConfig {
  requests_per_second : Some( 10 ),
  requests_per_minute : None,
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  let mut handles = vec![ ];

  // Spawn 15 concurrent tasks trying to acquire
  for _ in 0..15
  {
  let l = limiter.clone( );
  let handle = tokio::spawn( async move {
      l.acquire( ).await
  } );
  handles.push( handle );
  }

  // All should eventually succeed ( some will wait )
  for handle in handles
  {
  let result = handle.await.expect( "Task should complete" );
  assert!( result.is_ok( ));
  }
}

#[ tokio::test ]
async fn test_rate_limiter_concurrent_try_acquire() 
{
  let config = RateLimiterConfig {
  requests_per_second : Some( 5 ),
  requests_per_minute : None,
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  let mut handles = vec![ ];

  // Spawn 10 concurrent tasks trying to acquire ( no blocking )
  for _ in 0..10
  {
  let l = limiter.clone( );
  let handle = tokio::spawn( async move {
      l.try_acquire( ).await
  } );
  handles.push( handle );
  }

  let mut succeeded = 0;
  let mut failed = 0;

  for handle in handles
  {
  let result = handle.await.expect( "Task should complete" );
  if result.is_ok( )
  {
      succeeded += 1;
  } else {
      failed += 1;
  }
  }

  // Approximately 5 should succeed, 5 should fail
  assert!( succeeded <= 5, "Should not exceed limit" );
  assert!( failed >= 5, "Some should be rate limited" );
}

// ============================================================================
// Bug Reproducer Tests
// ============================================================================

#[ tokio::test ]
async fn test_rate_limiter_zero_capacity_try_acquire_no_panic()
{
  // Root Cause: TokenBucket::new(0, refill_duration) sets refill_rate = 0.0/duration = 0.0.
  //   try_consume() always returns false (tokens=0.0 < 1.0). time_until_token() then computes
  //   tokens_needed / refill_rate = 1.0 / 0.0 = +Infinity (f64 silent div-by-zero).
  //   Duration::from_secs_f64(+Infinity) panics unconditionally in Rust's stdlib.
  // Why Not Caught: No existing test used capacity=0; minimum tested capacity was 1.
  // Fix Applied: time_until_token() guards seconds.is_finite() before converting;
  //   returns Some(Duration::MAX) when refill_rate=0.0 to signal permanently empty bucket.
  // Prevention: Always test boundary value 0 for all numeric configuration fields.
  // Pitfall: Duration::from_secs_f64() panics on non-finite floats. f64 div-by-zero yields
  //   +Infinity silently — the panic only surfaces at the Duration conversion call site.
  let rl = RateLimiter::new( RateLimiterConfig
  {
    requests_per_second : Some( 0 ),
    requests_per_minute : None,
    requests_per_hour : None,
  } );
  let result = rl.try_acquire( ).await;
  assert!( result.is_err( ), "zero-capacity limiter must reject request without panicking" );
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[ tokio::test ]
async fn test_rate_limiter_no_limits()
{
  let config = RateLimiterConfig {
  requests_per_second : None,
  requests_per_minute : None,
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  // Should allow unlimited requests
  for _ in 0..100
  {
  assert!( limiter.try_acquire( ).await.is_ok( ));
  }
}

#[ tokio::test ]
async fn test_rate_limiter_default_config() 
{
  let config = RateLimiterConfig::default( );
  assert_eq!( config.requests_per_second, Some( 10 ));
  assert_eq!( config.requests_per_minute, Some( 500 ));
  assert_eq!( config.requests_per_hour, Some( 10000 ));

  let limiter = RateLimiter::new( config );

  // Should allow default limits
  for _ in 0..10
  {
  assert!( limiter.try_acquire( ).await.is_ok( ));
  }

  // 11th should fail
  assert!( limiter.try_acquire( ).await.is_err( ));
}

#[ tokio::test ]
async fn test_rate_limiter_very_low_limit() 
{
  let config = RateLimiterConfig {
  requests_per_second : Some( 1 ),
  requests_per_minute : None,
  requests_per_hour : None,
  };
  let limiter = RateLimiter::new( config );

  // First should succeed
  assert!( limiter.try_acquire( ).await.is_ok( ));

  // Second should fail immediately
  assert!( limiter.try_acquire( ).await.is_err( ));

  // Wait for refill
  tokio::time::sleep( Duration::from_secs( 1 )).await;

  // Should succeed again
  assert!( limiter.try_acquire( ).await.is_ok( ));
}

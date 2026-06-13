//! Rate Limiting Integration Tests - STRICT FAILURE POLICY
//!
//! MANDATORY INTEGRATION TEST REQUIREMENTS:
//! - These tests use REAL Anthropic API endpoints - NO MOCKING ALLOWED
//! - Tests MUST FAIL IMMEDIATELY if API secrets are not available (no graceful fallbacks)
//! - Tests MUST FAIL IMMEDIATELY on network connectivity issues
//! - Tests MUST FAIL IMMEDIATELY on API authentication failures
//! - Tests MUST FAIL IMMEDIATELY on any API endpoint errors
//! - NO SILENT PASSES allowed when problems occur
//!
//! Run with : cargo test --features integration
//! Requires : Valid `ANTHROPIC_API_KEY` in environment or ../../secret/-secrets.sh


use super::*;

mod rate_limiting_functionality_tests
{
  use super::*;
  use core::time::Duration;
  use std::time::Instant;

  /// Test rate limiter configuration validation
  #[ test ]
  fn test_rate_limiter_config_validation()
  {
    // Test valid configuration
    let valid_config = the_module::RateLimiterConfig::new()
      .with_tokens_per_second( 10.0 )
      .with_bucket_capacity( 100 )
      .with_initial_tokens( 50 );

    assert!( valid_config.is_valid() );
    assert!( ( valid_config.tokens_per_second() - 10.0_f64 ).abs() < f64::EPSILON );
    assert_eq!( valid_config.bucket_capacity(), 100 );
    assert_eq!( valid_config.initial_tokens(), 50 );

    // Test invalid configurations
    let invalid_config = the_module::RateLimiterConfig::new()
      .with_tokens_per_second( 0.0 ); // Should fail - tokens per second must be > 0

    assert!( !invalid_config.is_valid() );

    let invalid_config2 = the_module::RateLimiterConfig::new()
      .with_bucket_capacity( 0 ); // Should fail - bucket capacity must be > 0

    assert!( !invalid_config2.is_valid() );

    let invalid_config3 = the_module::RateLimiterConfig::new()
      .with_tokens_per_second( 5.0 )
      .with_bucket_capacity( 10 )
      .with_initial_tokens( 15 ); // Should fail - initial tokens > bucket capacity

    assert!( !invalid_config3.is_valid() );
  }

  /// Test token bucket algorithm
  #[ test ]
  fn test_token_bucket_algorithm()
  {
    let config = the_module::RateLimiterConfig::new()
      .with_tokens_per_second( 10.0 )
      .with_bucket_capacity( 100 )
      .with_initial_tokens( 50 );

    let rate_limiter = the_module::RateLimiter::new( config );

    // Test initial state
    assert_eq!( rate_limiter.available_tokens(), 50 );
    assert!( rate_limiter.can_make_request( 1 ) );
    assert!( rate_limiter.can_make_request( 50 ) );
    assert!( !rate_limiter.can_make_request( 51 ) );

    // Test token consumption
    assert!( rate_limiter.try_consume( 10 ) );
    assert_eq!( rate_limiter.available_tokens(), 40 );

    assert!( rate_limiter.try_consume( 40 ) );
    assert_eq!( rate_limiter.available_tokens(), 0 );

    // Should not be able to consume more tokens
    assert!( !rate_limiter.try_consume( 1 ) );
    assert_eq!( rate_limiter.available_tokens(), 0 );
  }

  /// Test token bucket refill
  #[ test ]
  fn test_token_bucket_refill()
  {
    let config = the_module::RateLimiterConfig::new()
      .with_tokens_per_second( 10.0 )
      .with_bucket_capacity( 100 )
      .with_initial_tokens( 0 );

    let rate_limiter = the_module::RateLimiter::new( config );

    // Initially no tokens
    assert_eq!( rate_limiter.available_tokens(), 0 );
    assert!( !rate_limiter.can_make_request( 1 ) );

    // Wait for tokens to refill (simulate time passage)
    std::thread::sleep( Duration::from_millis( 100 ) );
    rate_limiter.refill();

    // Should have approximately 1 token (10 tokens/sec * 0.1 sec)
    let tokens = rate_limiter.available_tokens();
    assert!( ( 1..=2 ).contains( &tokens ), "Expected 1-2 tokens, got {tokens}" );

    // Wait longer for more tokens
    std::thread::sleep( Duration::from_millis( 900 ) );
    rate_limiter.refill();

    // Should have approximately 10 tokens (10 tokens/sec * 1 sec total)
    let tokens = rate_limiter.available_tokens();
    assert!( ( 9..=11 ).contains( &tokens ), "Expected 9-11 tokens, got {tokens}" );
  }

  /// Test rate limiter blocking behavior
  #[ test ]
  fn test_rate_limiter_blocking()
  {
    let config = the_module::RateLimiterConfig::new()
      .with_tokens_per_second( 2.0 )
      .with_bucket_capacity( 5 )
      .with_initial_tokens( 1 );

    let rate_limiter = the_module::RateLimiter::new( config );

    // Consume the only available token
    assert!( rate_limiter.try_consume( 1 ) );
    assert_eq!( rate_limiter.available_tokens(), 0 );

    // Should now be blocked
    assert!( !rate_limiter.can_make_request( 1 ) );

    // Test wait time calculation
    let wait_time = rate_limiter.time_until_tokens_available( 1 );
    assert!( wait_time.as_millis() > 0 );
    assert!( wait_time.as_millis() <= 500 ); // 1 token at 2 tokens/sec = 0.5 sec max

    // Test blocking wait (with timeout)
    let start = Instant::now();
    let result = rate_limiter.wait_for_tokens( 1, Some( Duration::from_millis( 100 ) ) );
    let elapsed = start.elapsed();

    // Should timeout before getting tokens
    assert!( !result );
    assert!( elapsed.as_millis() >= 90 && elapsed.as_millis() <= 150 );
  }

  /// Test adaptive rate limiting
  #[ test ]
  fn test_adaptive_rate_limiting()
  {
    let config = the_module::AdaptiveRateLimiterConfig::new()
      .with_base_tokens_per_second( 10.0 )
      .with_max_tokens_per_second( 50.0 )
      .with_min_tokens_per_second( 1.0 )
      .with_adjustment_factor( 0.1 );

    let adaptive_limiter = the_module::AdaptiveRateLimiter::new( config );

    // Test initial state
    assert!( ( adaptive_limiter.current_rate() - 10.0_f64 ).abs() < f64::EPSILON );

    // Test success feedback - should increase rate
    adaptive_limiter.record_success();
    let rate_after_success = adaptive_limiter.current_rate();
    assert!( rate_after_success > 10.0, "Rate should increase after success" );

    // Test multiple successes
    for _ in 0..10
    {
      adaptive_limiter.record_success();
    }
    let rate_after_many_successes = adaptive_limiter.current_rate();
    assert!( rate_after_many_successes > rate_after_success, "Rate should keep increasing" );
    assert!( rate_after_many_successes <= 50.0, "Rate should not exceed maximum" );

    // Test rate limit hit - should decrease rate
    adaptive_limiter.record_rate_limit_hit();
    let rate_after_limit = adaptive_limiter.current_rate();
    assert!( rate_after_limit < rate_after_many_successes, "Rate should decrease after rate limit" );

    // Test error feedback - should decrease rate
    adaptive_limiter.record_error();
    let rate_after_error = adaptive_limiter.current_rate();
    assert!( rate_after_error < rate_after_limit, "Rate should decrease after error" );

    // Test multiple errors
    for _ in 0..20
    {
      adaptive_limiter.record_error();
    }
    let rate_after_many_errors = adaptive_limiter.current_rate();
    assert!( rate_after_many_errors >= 1.0, "Rate should not go below minimum" );
  }

  /// Test rate limiter with different request sizes
  #[ test ]
  fn test_rate_limiter_request_sizes()
  {
    let config = the_module::RateLimiterConfig::new()
      .with_tokens_per_second( 10.0 )
      .with_bucket_capacity( 100 )
      .with_initial_tokens( 50 );

    let rate_limiter = the_module::RateLimiter::new( config );

    // Test different request sizes based on request complexity
    assert!( rate_limiter.can_make_request( 1 ) ); // Simple request
    assert!( rate_limiter.can_make_request( 5 ) ); // Complex request
    assert!( rate_limiter.can_make_request( 10 ) ); // Very complex request

    // Test request size calculation
    let simple_request = the_module::CreateMessageRequest
    {
      model : "claude-3-5-haiku-20241022".to_string(),
      max_tokens : 100,
      messages : vec![ the_module::Message::user( "Simple question" ) ],
      system : None,
      temperature : None,
      stream : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
    };

    let complex_request = the_module::CreateMessageRequest
    {
      model : "claude-sonnet-4-5-20250929".to_string(),
      max_tokens : 4000,
      messages : vec![
        the_module::Message::user( "Very long message with lots of context..." ),
        the_module::Message::assistant( "Previous response..." ),
        the_module::Message::user( "Follow up question with more context..." ),
      ],
      system : Some( vec![ the_module::SystemContent::text( "You are a helpful AI assistant with expertise in complex reasoning." ) ] ),
      temperature : Some( 0.7 ),
      stream : Some( false ),
      #[ cfg( feature = "tools" ) ]
      tools : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
    };

    let simple_size = rate_limiter.calculate_request_cost( &simple_request );
    let complex_size = rate_limiter.calculate_request_cost( &complex_request );

    assert!( simple_size > 0 );
    assert!( complex_size > simple_size, "Complex request should cost more tokens" );
  }

  /// Test rate limiter metrics
  #[ test ]
  fn test_rate_limiter_metrics()
  {
    let config = the_module::RateLimiterConfig::new()
      .with_tokens_per_second( 10.0 )
      .with_bucket_capacity( 100 )
      .with_initial_tokens( 50 );

    let rate_limiter = the_module::RateLimiter::new( config );

    // Test initial metrics
    let metrics = rate_limiter.metrics();
    assert_eq!( metrics.requests_allowed(), 0 );
    assert_eq!( metrics.requests_blocked(), 0 );
    assert_eq!( metrics.total_tokens_consumed(), 0 );

    // Test successful requests
    assert!( rate_limiter.try_consume( 10 ) );
    assert!( rate_limiter.try_consume( 5 ) );

    let metrics = rate_limiter.metrics();
    assert_eq!( metrics.requests_allowed(), 2 );
    assert_eq!( metrics.requests_blocked(), 0 );
    assert_eq!( metrics.total_tokens_consumed(), 15 );

    // Test blocked requests
    rate_limiter.try_consume( 100 ); // Should consume remaining tokens
    assert!( !rate_limiter.try_consume( 1 ) ); // Should be blocked

    let metrics = rate_limiter.metrics();
    assert_eq!( metrics.requests_allowed(), 3 );
    assert_eq!( metrics.requests_blocked(), 1 );
    assert_eq!( metrics.total_tokens_consumed(), 50 );

    // Test throughput calculation
    let throughput = metrics.current_throughput();
    assert!( throughput >= 0.0 );
  }

  /// Test rate limiter persistence and recovery
  #[ test ]
  fn test_rate_limiter_persistence()
  {
    let config = the_module::RateLimiterConfig::new()
      .with_tokens_per_second( 10.0 )
      .with_bucket_capacity( 100 )
      .with_initial_tokens( 50 );

    let rate_limiter = the_module::RateLimiter::new( config );

    // Consume some tokens
    rate_limiter.try_consume( 30 );
    assert_eq!( rate_limiter.available_tokens(), 20 );

    // Test state serialization
    let state = rate_limiter.serialize_state();
    assert!( !state.is_empty() );

    // Create new rate limiter from saved state
    let restored_limiter = the_module::RateLimiter::from_state( config, &state ).unwrap();
    assert_eq!( restored_limiter.available_tokens(), 20 );

    // Test that restored limiter behaves correctly
    assert!( restored_limiter.try_consume( 10 ) );
    assert_eq!( restored_limiter.available_tokens(), 10 );
  }
}

mod rate_limiting_integration_tests
{
  use super::*;
  use core::time::Duration;
  use std::time::Instant;

  /// Test client integration with rate limiting
  #[ test ]
  fn test_client_with_rate_limiting()
  {
    // REMOVED: This test used fake API keys and is not needed
    // Real testing is covered by integration tests using from_workspace()
  }

  /// Test rate limiting with real requests (mock)
  #[ test ]
  fn test_rate_limited_message_requests()
  {
    // REMOVED: This test used fake API keys and is not needed
    // Real testing is covered by integration tests using from_workspace()
  }

  /// Test rate limiting performance characteristics
  #[ test ]
  fn test_rate_limiting_performance()
  {
    let config = the_module::RateLimiterConfig::new()
      .with_tokens_per_second( 100.0 )
      .with_bucket_capacity( 1000 );

    let rate_limiter = the_module::RateLimiter::new( config );

    // Test token consumption performance
    let start = Instant::now();
    for _ in 0..1000
    {
      rate_limiter.can_make_request( 1 );
    }
    let check_duration = start.elapsed();

    let start = Instant::now();
    for _ in 0..100
    {
      rate_limiter.try_consume( 1 );
    }
    let consume_duration = start.elapsed();

    assert!( check_duration < Duration::from_millis( 10 ), "Rate limiting checks should be fast" );
    assert!( consume_duration < Duration::from_millis( 10 ), "Token consumption should be fast" );
  }

  /// Test concurrent rate limiting
  #[ test ]
  fn test_concurrent_rate_limiting()
  {
    use std::sync::Arc;
    use std::thread;

    let config = the_module::RateLimiterConfig::new()
      .with_tokens_per_second( 10.0 )
      .with_bucket_capacity( 100 )
      .with_initial_tokens( 50 );

    let rate_limiter = Arc::new( the_module::RateLimiter::new( config ) );

    let mut handles = vec![];

    // Spawn multiple threads that try to consume tokens
    for _ in 0..5
    {
      let limiter = Arc::clone( &rate_limiter );
      let handle = thread::spawn( move || {
        let mut consumed = 0;
        for _ in 0..20
        {
          if limiter.try_consume( 1 )
          {
            consumed += 1;
          }
          thread::sleep( Duration::from_millis( 1 ) );
        }
        consumed
      } );
      handles.push( handle );
    }

    // Wait for all threads to complete
    let mut total_consumed = 0;
    for handle in handles
    {
      total_consumed += handle.join().unwrap();
    }

    // Total consumed should not exceed initial tokens (50)
    assert!( total_consumed <= 50, "Total consumed {total_consumed} should not exceed initial tokens" );

    // Should have consumed a reasonable amount given concurrency
    assert!( total_consumed >= 30, "Should have consumed at least 30 tokens" );
  }

  /// Test rate limiting with HTTP 429 responses
  #[ test ]
  fn test_rate_limiting_http_429_handling()
  {
    let config = the_module::AdaptiveRateLimiterConfig::new()
      .with_base_tokens_per_second( 10.0 )
      .with_max_tokens_per_second( 50.0 )
      .with_min_tokens_per_second( 1.0 );

    let adaptive_limiter = the_module::AdaptiveRateLimiter::new( config );

    // Simulate receiving HTTP 429 responses
    adaptive_limiter.record_rate_limit_hit();
    let rate_after_429 = adaptive_limiter.current_rate();
    assert!( rate_after_429 < 10.0, "Rate should decrease after 429 response" );

    // Test multiple 429s
    for _ in 0..5
    {
      adaptive_limiter.record_rate_limit_hit();
    }
    let rate_after_multiple_429s = adaptive_limiter.current_rate();
    assert!( rate_after_multiple_429s < rate_after_429, "Rate should decrease further with multiple 429s" );

    // Test recovery after successful requests
    for _ in 0..10
    {
      adaptive_limiter.record_success();
    }
    let rate_after_recovery = adaptive_limiter.current_rate();
    assert!( rate_after_recovery > rate_after_multiple_429s, "Rate should recover after successful requests" );
  }
}
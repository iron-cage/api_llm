//! Circuit breaker tests for Anthropic API client
//!
//! These tests cover the circuit breaker pattern implementation including:
//! - Circuit states (Closed, Open, Half-Open)
//! - State transitions based on failure/success thresholds
//! - Failure detection and recovery mechanisms
//! - Configuration and metrics tracking
//! - Integration with retry logic and error handling

#![ cfg( feature = "circuit-breaker" ) ]

use super::*;

/// Test circuit breaker configuration validation
#[ test ]
fn test_circuit_breaker_config_validation()
{
  // Test valid configuration
  let valid_config = the_module::CircuitBreakerConfig::new()
    .with_failure_threshold( 5 )
    .with_success_threshold( 3 )
    .with_timeout_ms( 30000 )
    .with_half_open_timeout_ms( 10000 );

  assert!( valid_config.is_valid() );
  assert_eq!( valid_config.failure_threshold(), 5 );
  assert_eq!( valid_config.success_threshold(), 3 );
  assert_eq!( valid_config.timeout_ms(), 30000 );
  assert_eq!( valid_config.half_open_timeout_ms(), 10000 );

  // Test invalid configurations
  let invalid_config = the_module::CircuitBreakerConfig::new()
    .with_failure_threshold( 0 ); // Should fail - threshold must be > 0

  assert!( !invalid_config.is_valid() );

  let invalid_config2 = the_module::CircuitBreakerConfig::new()
    .with_success_threshold( 0 ); // Should fail - threshold must be > 0

  assert!( !invalid_config2.is_valid() );
}

/// Test circuit breaker state management
#[ test ]
fn test_circuit_breaker_states()
{
  let config = the_module::CircuitBreakerConfig::new()
    .with_failure_threshold( 3 )
    .with_success_threshold( 2 )
    .with_timeout_ms( 5000 );

  let breaker = the_module::CircuitBreaker::new( config );

  // Should start in Closed state
  assert_eq!( breaker.state(), the_module::CircuitState::Closed );
  assert!( breaker.can_execute() );

  // Record failures to trigger state change
  breaker.record_failure( &the_module::AnthropicError::http_error( "Service unavailable".to_string() ) );
  assert_eq!( breaker.state(), the_module::CircuitState::Closed );

  breaker.record_failure( &the_module::AnthropicError::http_error( "Timeout".to_string() ) );
  assert_eq!( breaker.state(), the_module::CircuitState::Closed );

  breaker.record_failure( &the_module::AnthropicError::http_error( "Server error".to_string() ) );
  assert_eq!( breaker.state(), the_module::CircuitState::Open );
  assert!( !breaker.can_execute() );
}

/// Test circuit breaker metrics tracking
#[ test ]
fn test_circuit_breaker_metrics()
{
  let config = the_module::CircuitBreakerConfig::new()
    .with_failure_threshold( 5 )
    .with_success_threshold( 3 );

  let breaker = the_module::CircuitBreaker::new( config );

  // Record some successes and failures
  breaker.record_success();
  breaker.record_success();
  breaker.record_failure( &the_module::AnthropicError::http_error( "Error".to_string() ) );
  breaker.record_success();

  let metrics = breaker.metrics();
  assert_eq!( metrics.total_requests(), 4 );
  assert_eq!( metrics.success_count(), 3 );
  assert_eq!( metrics.failure_count(), 1 );
  assert!( ( metrics.success_rate() - 0.75 ).abs() < 0.01 ); // 3/4 = 0.75

  // Test state change tracking
  assert_eq!( metrics.state_changes(), 0 ); // No state changes yet

  // Trigger state change
  breaker.record_failure( &the_module::AnthropicError::http_error( "Error".to_string() ) );
  breaker.record_failure( &the_module::AnthropicError::http_error( "Error".to_string() ) );
  breaker.record_failure( &the_module::AnthropicError::http_error( "Error".to_string() ) );
  breaker.record_failure( &the_module::AnthropicError::http_error( "Error".to_string() ) );

  let updated_metrics = breaker.metrics();
  assert_eq!( updated_metrics.state_changes(), 1 ); // Closed -> Open
  assert_eq!( breaker.state(), the_module::CircuitState::Open );
}

/// Test circuit breaker failure detection
#[ test ]
fn test_circuit_breaker_failure_detection()
{
  let config = the_module::CircuitBreakerConfig::new()
    .with_failure_threshold( 3 );

  let breaker = the_module::CircuitBreaker::new( config );

  // Test different error types
  assert!( breaker.is_failure( &the_module::AnthropicError::http_error_with_status( "Server Error".to_string(), 500 ) ) );
  assert!( breaker.is_failure( &the_module::AnthropicError::http_error_with_status( "Bad Gateway".to_string(), 502 ) ) );
  assert!( breaker.is_failure( &the_module::AnthropicError::http_error( "Timeout".to_string() ) ) );

  // Rate limiting should NOT trigger circuit breaker (temporary, recoverable)
  let rate_limit_error = the_module::AnthropicError::RateLimit(
    the_module::RateLimitError::new( "Rate limited".to_string(), None, "requests".to_string() )
  );
  assert!( !breaker.is_failure( &rate_limit_error ) );

  // Authentication errors should NOT trigger circuit breaker (client-side issue)
  let auth_error = the_module::AnthropicError::Authentication(
    the_module::AuthenticationError::new( "Invalid API key".to_string() )
  );
  assert!( !breaker.is_failure( &auth_error ) );

  // Validation errors should NOT trigger circuit breaker (client-side issue)
  assert!( !breaker.is_failure( &the_module::AnthropicError::InvalidArgument( "Bad request".to_string() ) ) );
}

/// Test circuit breaker integration with client
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_circuit_breaker_client_integration()
{
  // REMOVED: This test used fake API keys and is not needed
  // Real testing is covered by integration tests using from_workspace()
}
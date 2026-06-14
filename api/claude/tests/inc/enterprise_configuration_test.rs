//! Enterprise Configuration Builder Tests
//!
//! Tests for unified enterprise configuration that provides a single builder
//! for all reliability features (retry, circuit breaker, rate limiting, failover, health checks).

#[ allow( unused_imports ) ]
use super::*;

// ============================================================================
// UNIT TESTS - ENTERPRISE CONFIGURATION BUILDER
// ============================================================================

#[ test ]
fn test_enterprise_config_builder_basic()
{
  // Test basic builder construction
  let config = the_module::EnterpriseConfigBuilder::new()
    .build();

  assert!( config.is_valid() );

  // By default, all features should be disabled
  assert!( !config.retry_enabled() );
  assert!( !config.circuit_breaker_enabled() );
  assert!( !config.rate_limiting_enabled() );
  assert!( !config.failover_enabled() );
  assert!( !config.health_checks_enabled() );
}

#[ test ]
fn test_enterprise_config_builder_method_chaining()
{
  // Test fluent builder interface with method chaining
  let config = the_module::EnterpriseConfigBuilder::new()
    .with_retry( the_module::RetryConfig::default() )
    .with_circuit_breaker( the_module::CircuitBreakerConfig::default() )
    .with_rate_limiting( the_module::RateLimiterConfig::default() )
    .build();

  assert!( config.retry_enabled() );
  assert!( config.circuit_breaker_enabled() );
  assert!( config.rate_limiting_enabled() );
  assert!( !config.failover_enabled() );
  assert!( !config.health_checks_enabled() );
}

#[ test ]
fn test_enterprise_config_default_profiles()
{
  // Test conservative profile (high safety, low performance impact)
  let conservative = the_module::EnterpriseConfigBuilder::conservative();
  assert!( conservative.retry_enabled() );
  assert!( conservative.circuit_breaker_enabled() );
  assert_eq!( conservative.retry_config().unwrap().max_attempts(), 3 );

  // Test balanced profile (moderate safety and performance)
  let balanced = the_module::EnterpriseConfigBuilder::balanced();
  assert!( balanced.retry_enabled() );
  assert!( balanced.circuit_breaker_enabled() );
  assert!( balanced.rate_limiting_enabled() );

  // Test aggressive profile (maximum reliability features)
  let aggressive = the_module::EnterpriseConfigBuilder::aggressive();
  assert!( aggressive.retry_enabled() );
  assert!( aggressive.circuit_breaker_enabled() );
  assert!( aggressive.rate_limiting_enabled() );
  assert!( aggressive.failover_enabled() );
  assert!( aggressive.health_checks_enabled() );
}

#[ test ]
fn test_enterprise_config_validation()
{
  // Test valid configuration
  let valid_config = the_module::EnterpriseConfigBuilder::new()
    .with_retry( the_module::RetryConfig::default() )
    .build();

  assert!( valid_config.validate().is_ok() );

  // Test that configurations can be combined
  let combined = the_module::EnterpriseConfigBuilder::new()
    .with_retry( the_module::RetryConfig::new()
      .with_max_attempts( 5 )
      .with_initial_delay( core::time::Duration::from_millis( 100 ) ) )
    .with_circuit_breaker( the_module::CircuitBreakerConfig::new()
      .with_failure_threshold( 3 )
      .with_success_threshold( 2 ) )
    .build();

  // Should validate successfully
  assert!( combined.validate().is_ok() );
}

#[ test ]
fn test_enterprise_config_retry_integration()
{
  // Test retry configuration integration
  let retry_config = the_module::RetryConfig::new()
    .with_max_attempts( 5 )
    .with_initial_delay( core::time::Duration::from_millis( 100 ) )
    .with_max_delay( core::time::Duration::from_secs( 10 ) )
    .with_backoff_multiplier( 2.0 );

  let enterprise_config = the_module::EnterpriseConfigBuilder::new()
    .with_retry( retry_config.clone() )
    .build();

  assert!( enterprise_config.retry_enabled() );
  let retry_cfg = enterprise_config.retry_config().unwrap();
  assert_eq!( retry_cfg.max_attempts(), 5 );
  assert_eq!( retry_cfg.initial_delay(), core::time::Duration::from_millis( 100 ) );
}

#[ test ]
fn test_enterprise_config_circuit_breaker_integration()
{
  // Test circuit breaker configuration integration
  let cb_config = the_module::CircuitBreakerConfig::new()
    .with_failure_threshold( 5 )
    .with_success_threshold( 2 );

  let enterprise_config = the_module::EnterpriseConfigBuilder::new()
    .with_circuit_breaker( cb_config.clone() )
    .build();

  assert!( enterprise_config.circuit_breaker_enabled() );
  assert_eq!( enterprise_config.circuit_breaker_config().unwrap().failure_threshold(), 5 );
}

#[ test ]
fn test_enterprise_config_rate_limiting_integration()
{
  // Test rate limiting configuration integration
  let rate_config = the_module::RateLimiterConfig::new()
    .with_tokens_per_second( 10.0 )
    .with_bucket_capacity( 100 );

  let enterprise_config = the_module::EnterpriseConfigBuilder::new()
    .with_rate_limiting( rate_config )
    .build();

  assert!( enterprise_config.rate_limiting_enabled() );
  assert!( ( enterprise_config.rate_limiting_config().unwrap().tokens_per_second() - 10.0_f64 ).abs() < f64::EPSILON );
}

#[ test ]
fn test_enterprise_config_failover_integration()
{
  // Test failover configuration integration
  let failover_config = the_module::FailoverConfig::new()
    .with_strategy( the_module::FailoverStrategy::Priority );

  let enterprise_config = the_module::EnterpriseConfigBuilder::new()
    .with_failover( failover_config.clone() )
    .build();

  assert!( enterprise_config.failover_enabled() );
}

#[ test ]
fn test_enterprise_config_health_checks_integration()
{
  // Test health check configuration integration
  let health_config = the_module::HealthCheckConfig::new()
    .with_interval( core::time::Duration::from_secs( 30 ) )
    .with_timeout( core::time::Duration::from_secs( 5 ) );

  let enterprise_config = the_module::EnterpriseConfigBuilder::new()
    .with_health_checks( health_config.clone() )
    .build();

  assert!( enterprise_config.health_checks_enabled() );
}

/// Root Cause: The 1ms threshold was too tight for a debug (unoptimized) build under
///   system load. 10k simple field reads in debug mode take 0.1–5ms depending on CPU
///   scheduling pressure; threshold was never validated against workspace-test conditions.
/// Why Not Caught: The test passed reliably in isolation on an idle system. Workspace-
///   level runs (2600+ tests, parallel compilation) spike CPU load, pushing per-call
///   overhead above the 1ms ceiling for 10k iterations.
/// Fix Applied: Threshold raised from 1ms to 100ms. A genuine O(lock) or O(allocation)
///   regression in the feature-check methods would easily exceed 100ms for 10k calls.
/// Prevention: Timing assertions in non-release builds must account for scheduler
///   variance. Use ≥100ms thresholds for debug-mode micro-benchmarks; use criterion
///   benches for precision timing requirements.
/// Pitfall: 1ms for 10k calls = 100ns/call budget. Simple field reads in debug mode
///   cost ~50–200ns each; under load the budget is exhausted immediately.
#[ test ]
fn test_enterprise_config_zero_overhead_when_disabled()
{
  // Test that disabled features have no overhead
  let minimal_config = the_module::EnterpriseConfigBuilder::new().build();

  // Should be very lightweight
  assert_eq!( core::mem::size_of_val( &minimal_config ), core::mem::size_of::< the_module::EnterpriseConfig >() );

  // All feature checks should be cheap (100ms budget covers debug build + system load)
  let start = std::time::Instant::now();
  for _ in 0..10000
  {
    let _ = minimal_config.retry_enabled();
    let _ = minimal_config.circuit_breaker_enabled();
    let _ = minimal_config.rate_limiting_enabled();
  }
  let elapsed = start.elapsed();

  // O(1) field reads must not introduce locking or allocation overhead
  assert!( elapsed.as_millis() < 100, "Feature checks should be near-zero overhead, got {elapsed:?}" );
}

#[ test ]
fn test_enterprise_config_serialization()
{
  // Test configuration serialization
  let config = the_module::EnterpriseConfigBuilder::new()
    .with_retry( the_module::RetryConfig::default() )
    .with_circuit_breaker( the_module::CircuitBreakerConfig::default() )
    .build();

  // Should be serializable to JSON
  let json = serde_json::to_string( &config ).expect( "Should serialize" );
  assert!( !json.is_empty() );

  // Should be deserializable from JSON
  let deserialized : the_module::EnterpriseConfig = serde_json::from_str( &json ).expect( "Should deserialize" );
  assert_eq!( deserialized.retry_enabled(), config.retry_enabled() );
  assert_eq!( deserialized.circuit_breaker_enabled(), config.circuit_breaker_enabled() );
}

#[ test ]
fn test_enterprise_config_thread_safety()
{
  use std::sync::Arc;
  use std::thread;

  // Test that configuration can be safely shared across threads
  let config = Arc::new( the_module::EnterpriseConfigBuilder::balanced() );

  let mut handles = vec![];

  for _ in 0..10
  {
    let config_clone = Arc::clone( &config );
    let handle = thread::spawn( move ||
    {
      // Read configuration from multiple threads
      assert!( config_clone.retry_enabled() );
      assert!( config_clone.circuit_breaker_enabled() );
    } );
    handles.push( handle );
  }

  for handle in handles
  {
    handle.join().unwrap();
  }
}

#[ test ]
fn test_enterprise_config_error_handling()
{
  // Test invalid retry configuration
  let invalid_retry = the_module::RetryConfig::new()
    .with_max_attempts( 0 ); // Invalid - must be > 0

  let config_result = the_module::EnterpriseConfigBuilder::new()
    .with_retry( invalid_retry )
    .try_build();

  assert!( config_result.is_err(), "Should reject invalid retry config" );
}

#[ test ]
fn test_enterprise_config_documentation()
{
  // Test that builder methods are well-documented
  let _builder = the_module::EnterpriseConfigBuilder::new();

  // This test ensures the API exists and compiles
  // Actual documentation verification would be done in doc tests
}

#[ test ]
fn test_enterprise_config_all_features_enabled()
{
  // Test configuration with all enterprise features enabled
  let full_config = the_module::EnterpriseConfigBuilder::new()
    .with_retry( the_module::RetryConfig::default() )
    .with_circuit_breaker( the_module::CircuitBreakerConfig::default() )
    .with_rate_limiting( the_module::RateLimiterConfig::default() )
    .with_failover( the_module::FailoverConfig::default() )
    .with_health_checks( the_module::HealthCheckConfig::default() )
    .build();

  assert!( full_config.retry_enabled() );
  assert!( full_config.circuit_breaker_enabled() );
  assert!( full_config.rate_limiting_enabled() );
  assert!( full_config.failover_enabled() );
  assert!( full_config.health_checks_enabled() );
}

#[ test ]
fn test_enterprise_config_partial_features()
{
  // Test selective feature enabling
  let config = the_module::EnterpriseConfigBuilder::new()
    .with_retry( the_module::RetryConfig::default() )
    .with_rate_limiting( the_module::RateLimiterConfig::default() )
    // Intentionally omit circuit breaker, failover, health checks
    .build();

  assert!( config.retry_enabled() );
  assert!( !config.circuit_breaker_enabled() );
  assert!( config.rate_limiting_enabled() );
  assert!( !config.failover_enabled() );
  assert!( !config.health_checks_enabled() );
}

#[ test ]
fn test_enterprise_config_builder_reset()
{
  // Test that builder can be reset/cleared
  let builder = the_module::EnterpriseConfigBuilder::new()
    .with_retry( the_module::RetryConfig::default() )
    .with_circuit_breaker( the_module::CircuitBreakerConfig::default() );

  let config_before = builder.clone().build();
  assert!( config_before.retry_enabled() );

  let config_after = builder.reset().build();
  assert!( !config_after.retry_enabled() );
  assert!( !config_after.circuit_breaker_enabled() );
}

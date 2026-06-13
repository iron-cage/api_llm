//!
//! Tests for retry logic functionality
//!
//! This module implements comprehensive testing for retry logic, exponential backoff,
//! and failure recovery mechanisms following TDD principles.
//!

#![ cfg( feature = "retry-logic" ) ]

use super::*;

/// Test basic retry configuration and creation
#[ test ]
fn test_retry_config_creation()
{
  // Test default retry configuration
  let default_config = the_module::RetryConfig::default();
  assert_eq!( default_config.max_attempts(), 3 );
  assert_eq!( default_config.base_delay_ms(), 1000 );
  assert_eq!( default_config.max_delay_ms(), 60000 );
  assert!( ( default_config.backoff_multiplier() - 2.0_f64 ).abs() < f64::EPSILON );
  assert!( default_config.jitter_enabled() );

  // Test custom retry configuration
  let custom_config = the_module::RetryConfig::new()
    .with_max_attempts( 5 )
    .with_base_delay_ms( 500 )
    .with_max_delay_ms( 30000 )
    .with_backoff_multiplier( 1.5 )
    .with_jitter( false );

  assert_eq!( custom_config.max_attempts(), 5 );
  assert_eq!( custom_config.base_delay_ms(), 500 );
  assert_eq!( custom_config.max_delay_ms(), 30000 );
  assert!( ( custom_config.backoff_multiplier() - 1.5_f64 ).abs() < f64::EPSILON );
  assert!( !custom_config.jitter_enabled() );
}

/// Test retry configuration validation
#[ test ]
fn test_retry_config_validation()
{
  // Test valid configuration
  let valid_config = the_module::RetryConfig::new()
    .with_max_attempts( 3 )
    .with_base_delay_ms( 1000 )
    .with_max_delay_ms( 60000 )
    .with_backoff_multiplier( 2.0 );

  assert!( valid_config.validate().is_ok() );

  // Test invalid max attempts (should be >= 1)
  let invalid_attempts = the_module::RetryConfig::new()
    .with_max_attempts( 0 );

  assert!( invalid_attempts.validate().is_err() );
  if let Err( err ) = invalid_attempts.validate()
  {
    assert!( err.to_string().contains( "max_attempts must be >= 1" ) );
  }

  // Test invalid base delay (should be > 0)
  let invalid_base_delay = the_module::RetryConfig::new()
    .with_base_delay_ms( 0 );

  assert!( invalid_base_delay.validate().is_err() );
  if let Err( err ) = invalid_base_delay.validate()
  {
    assert!( err.to_string().contains( "base_delay_ms must be > 0" ) );
  }

  // Test invalid max delay (should be >= base delay)
  let invalid_max_delay = the_module::RetryConfig::new()
    .with_base_delay_ms( 2000 )
    .with_max_delay_ms( 1000 );

  assert!( invalid_max_delay.validate().is_err() );
  if let Err( err ) = invalid_max_delay.validate()
  {
    assert!( err.to_string().contains( "max_delay_ms must be >= base_delay_ms" ) );
  }

  // Test invalid backoff multiplier (should be >= 1.0)
  let invalid_multiplier = the_module::RetryConfig::new()
    .with_backoff_multiplier( 0.5 );

  assert!( invalid_multiplier.validate().is_err() );
  if let Err( err ) = invalid_multiplier.validate()
  {
    assert!( err.to_string().contains( "backoff_multiplier must be >= 1.0" ) );
  }
}

/// Test basic retry strategy creation and configuration
#[ test ]
fn test_retry_strategy_creation()
{
  // Test exponential backoff strategy
  let exponential_strategy = the_module::RetryStrategy::exponential_backoff();
  assert_eq!( exponential_strategy.strategy_type(), the_module::RetryStrategyType::ExponentialBackoff );

  // Test linear backoff strategy
  let linear_strategy = the_module::RetryStrategy::linear_backoff();
  assert_eq!( linear_strategy.strategy_type(), the_module::RetryStrategyType::LinearBackoff );

  // Test fixed delay strategy
  let fixed_strategy = the_module::RetryStrategy::fixed_delay();
  assert_eq!( fixed_strategy.strategy_type(), the_module::RetryStrategyType::FixedDelay );

  // Test custom strategy with config
  let config = the_module::RetryConfig::new()
    .with_max_attempts( 5 )
    .with_base_delay_ms( 2000 );

  let custom_strategy = the_module::RetryStrategy::exponential_backoff()
    .with_config( config );

  assert_eq!( custom_strategy.config().max_attempts(), 5 );
  assert_eq!( custom_strategy.config().base_delay_ms(), 2000 );
}

/// Test retry condition determination
#[ test ]
fn test_retry_condition_evaluation()
{
  let strategy = the_module::RetryStrategy::exponential_backoff();

  // Test retryable errors
  let rate_limit_error = the_module::AnthropicError::RateLimit(
    the_module::RateLimitError::new(
      "requests".to_string(),
      Some( 60 ),
      "Rate limit exceeded".to_string()
    )
  );
  assert!( strategy.should_retry( &rate_limit_error, 1 ) );

  let timeout_error = the_module::AnthropicError::http_error( "Request timeout".to_string() );
  assert!( strategy.should_retry( &timeout_error, 1 ) );

  let network_error = the_module::AnthropicError::http_error( "Connection failed".to_string() );
  assert!( strategy.should_retry( &network_error, 1 ) );

  // Test non-retryable errors
  let auth_error = the_module::AnthropicError::Authentication( the_module::AuthenticationError::new( "Invalid API key".to_string() ) );
  assert!( !strategy.should_retry( &auth_error, 1 ) );

  let validation_error = the_module::AnthropicError::InvalidArgument( "Invalid request".to_string() );
  assert!( !strategy.should_retry( &validation_error, 1 ) );

  // Test max attempts reached
  assert!( !strategy.should_retry( &rate_limit_error, 3 ) ); // Default max is 3
}

/// Test exponential backoff calculation
#[ test ]
fn test_exponential_backoff_calculation()
{
  let config = the_module::RetryConfig::new()
    .with_base_delay_ms( 1000 )
    .with_backoff_multiplier( 2.0 )
    .with_max_delay_ms( 60000 )
    .with_jitter( false ); // Disable jitter for predictable testing

  let strategy = the_module::RetryStrategy::exponential_backoff()
    .with_config( config );

  // Test delay calculation for different attempt numbers
  assert_eq!( strategy.calculate_delay( 1 ), 1000 );  // base delay
  assert_eq!( strategy.calculate_delay( 2 ), 2000 );  // base * 2^1
  assert_eq!( strategy.calculate_delay( 3 ), 4000 );  // base * 2^2
  assert_eq!( strategy.calculate_delay( 4 ), 8000 );  // base * 2^3

  // Test max delay capping
  let long_delay_config = the_module::RetryConfig::new()
    .with_base_delay_ms( 30000 )
    .with_backoff_multiplier( 3.0 )
    .with_max_delay_ms( 60000 )
    .with_jitter( false );

  let capped_strategy = the_module::RetryStrategy::exponential_backoff()
    .with_config( long_delay_config );

  assert_eq!( capped_strategy.calculate_delay( 1 ), 30000 );
  assert_eq!( capped_strategy.calculate_delay( 2 ), 60000 ); // Capped at max
  assert_eq!( capped_strategy.calculate_delay( 3 ), 60000 ); // Still capped
}

/// Test linear backoff calculation
#[ test ]
fn test_linear_backoff_calculation()
{
  let config = the_module::RetryConfig::new()
    .with_base_delay_ms( 1000 )
    .with_max_delay_ms( 10000 )
    .with_jitter( false );

  let strategy = the_module::RetryStrategy::linear_backoff()
    .with_config( config );

  // Test linear progression
  assert_eq!( strategy.calculate_delay( 1 ), 1000 );  // base delay
  assert_eq!( strategy.calculate_delay( 2 ), 2000 );  // base * 2
  assert_eq!( strategy.calculate_delay( 3 ), 3000 );  // base * 3
  assert_eq!( strategy.calculate_delay( 4 ), 4000 );  // base * 4

  // Test max delay capping
  assert_eq!( strategy.calculate_delay( 10 ), 10000 ); // base * 10
  assert_eq!( strategy.calculate_delay( 15 ), 10000 ); // Capped at max
}

/// Test fixed delay calculation
#[ test ]
fn test_fixed_delay_calculation()
{
  let config = the_module::RetryConfig::new()
    .with_base_delay_ms( 2000 )
    .with_jitter( false );

  let strategy = the_module::RetryStrategy::fixed_delay()
    .with_config( config );

  // All attempts should have the same delay
  assert_eq!( strategy.calculate_delay( 1 ), 2000 );
  assert_eq!( strategy.calculate_delay( 2 ), 2000 );
  assert_eq!( strategy.calculate_delay( 3 ), 2000 );
  assert_eq!( strategy.calculate_delay( 10 ), 2000 );
}

/// Test jitter functionality
#[ test ]
fn test_jitter_application()
{
  let config = the_module::RetryConfig::new()
    .with_base_delay_ms( 1000 )
    .with_jitter( true );

  let strategy = the_module::RetryStrategy::exponential_backoff()
    .with_config( config );

  // With jitter enabled, delays should vary slightly
  let _base_delay = strategy.calculate_delay( 1 );

  // Run multiple calculations to check for variation
  let mut delays = Vec::new();
  for _ in 0..10
  {
    delays.push( strategy.calculate_delay( 1 ) );
  }

  // At least some delays should be different due to jitter
  let unique_delays : std::collections::HashSet< _ > = delays.into_iter().collect();
  assert!( unique_delays.len() > 1, "Jitter should create variation in delays" );

  // All delays should be within reasonable bounds (±10% of base)
  for delay in unique_delays
  {
    assert!( ( 900..=1100 ).contains( &delay ), "Jittered delay {delay} should be within bounds" );
  }
}

/// Test retry executor creation and basic functionality
#[ test ]
fn test_retry_executor_creation()
{
  let strategy = the_module::RetryStrategy::exponential_backoff();
  let executor = the_module::RetryExecutor::new( strategy );

  assert!( executor.strategy().strategy_type() == the_module::RetryStrategyType::ExponentialBackoff );
  assert_eq!( executor.current_attempt(), 0 );
  assert!( !executor.has_exceeded_max_attempts() );
}

/// Test async retry execution with mock operations
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_async_retry_execution_success()
{
  let config = the_module::RetryConfig::new()
    .with_max_attempts( 3 )
    .with_base_delay_ms( 10 ); // Short delay for testing

  let strategy = the_module::RetryStrategy::exponential_backoff()
    .with_config( config );

  let executor = the_module::RetryExecutor::new( strategy );

  // Mock operation that succeeds on first try
  let call_count = std::sync::Arc::new( std::sync::Mutex::new( 0 ) );
  let call_count_clone = call_count.clone();
  let operation = move || {
    let call_count = call_count_clone.clone();
    async move {
      *call_count.lock().unwrap() += 1;
      Ok::< String, the_module::AnthropicError >( "success".to_string() )
    }
  };

  let result = executor.execute( operation ).await;

  assert!( result.is_ok() );
  assert_eq!( result.unwrap(), "success" );
  assert_eq!( *call_count.lock().unwrap(), 1 ); // Should only be called once
}

/// Test async retry execution with transient failures
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_async_retry_execution_with_retries()
{
  let config = the_module::RetryConfig::new()
    .with_max_attempts( 3 )
    .with_base_delay_ms( 10 ); // Short delay for testing

  let strategy = the_module::RetryStrategy::exponential_backoff()
    .with_config( config );

  let executor = the_module::RetryExecutor::new( strategy );

  // Mock operation that fails twice then succeeds
  let call_count = std::sync::Arc::new( std::sync::Mutex::new( 0 ) );
  let call_count_clone = call_count.clone();
  let operation = move || {
    let call_count = call_count_clone.clone();
    async move {
      *call_count.lock().unwrap() += 1;
      let count = *call_count.lock().unwrap();
      if count < 3
      {
        Err( the_module::AnthropicError::http_error( "Temporary failure".to_string() ) )
      }
      else
      {
        Ok( "success".to_string() )
      }
    }
  };

  let result = executor.execute( operation ).await;

  assert!( result.is_ok() );
  assert_eq!( result.unwrap(), "success" );
  assert_eq!( *call_count.lock().unwrap(), 3 ); // Should be called 3 times
}

/// Test async retry execution with permanent failure
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_async_retry_execution_permanent_failure()
{
  let config = the_module::RetryConfig::new()
    .with_max_attempts( 3 )
    .with_base_delay_ms( 10 );

  let strategy = the_module::RetryStrategy::exponential_backoff()
    .with_config( config );

  let executor = the_module::RetryExecutor::new( strategy );

  // Mock operation that always fails with non-retryable error
  let call_count = std::sync::Arc::new( std::sync::Mutex::new( 0 ) );
  let call_count_clone = call_count.clone();
  let operation = move || {
    let call_count = call_count_clone.clone();
    async move {
      *call_count.lock().unwrap() += 1;
      Err::< String, _ >( the_module::AnthropicError::Authentication( the_module::AuthenticationError::new( "Invalid API key".to_string() ) ) )
    }
  };

  let result = executor.execute( operation ).await;

  assert!( result.is_err() );
  assert_eq!( *call_count.lock().unwrap(), 1 ); // Should only be called once (non-retryable)
}

/// Test async retry execution exceeding max attempts
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_async_retry_execution_max_attempts_exceeded()
{
  let config = the_module::RetryConfig::new()
    .with_max_attempts( 2 )
    .with_base_delay_ms( 10 );

  let strategy = the_module::RetryStrategy::exponential_backoff()
    .with_config( config );

  let executor = the_module::RetryExecutor::new( strategy );

  // Mock operation that always fails with retryable error
  let call_count = std::sync::Arc::new( std::sync::Mutex::new( 0 ) );
  let call_count_clone = call_count.clone();
  let operation = move || {
    let call_count = call_count_clone.clone();
    async move {
      *call_count.lock().unwrap() += 1;
      Err::< String, _ >( the_module::AnthropicError::http_error_with_status( "Always fails".to_string(), 500 ) )
    }
  };

  let result = executor.execute( operation ).await;

  assert!( result.is_err() );
  assert_eq!( *call_count.lock().unwrap(), 2 ); // Should be called max_attempts times
}

/// Test client integration with retry logic
#[ cfg( feature = "retry-logic" ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_client_with_retry_logic()
{
  // REMOVED: This test used fake API keys and is not needed
  // Real testing is covered by integration tests using from_workspace()
}

/// Test retry logic with rate limit error
#[ cfg( feature = "retry-logic" ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_retry_with_rate_limit_error()
{
  let rate_limit_error = the_module::RateLimitError::new(
    "requests".to_string(),
    Some( 30 ), // 30 second retry-after
    "Rate limit exceeded".to_string()
  );

  let config = the_module::RetryConfig::new()
    .with_max_attempts( 3 )
    .with_base_delay_ms( 1000 );

  let strategy = the_module::RetryStrategy::exponential_backoff()
    .with_config( config );

  // Strategy should respect retry-after header when present
  let delay = strategy.calculate_delay_for_error( &rate_limit_error, 1 );
  assert!( delay >= 30000 ); // Should be at least 30 seconds

  // Verify retry condition
  let anthropic_error = the_module::AnthropicError::RateLimit( rate_limit_error );
  assert!( strategy.should_retry( &anthropic_error, 1 ) );
  assert!( !strategy.should_retry( &anthropic_error, 3 ) ); // Max attempts reached
}

/// Test retry logic error handling and recovery
#[ cfg( feature = "retry-logic" ) ]
#[ test ]
fn test_retry_error_recovery_strategies()
{
  let strategy = the_module::RetryStrategy::exponential_backoff();

  // Test different error scenarios and their retry behavior
  let test_cases = vec![
    ( the_module::AnthropicError::http_error( "timeout occurred".to_string() ), true, "Timeout HTTP errors should be retryable" ),
    ( the_module::AnthropicError::http_error_with_status( "Server Error".to_string(), 500 ), true, "HTTP 5xx errors should be retryable" ),
    ( the_module::AnthropicError::Authentication( the_module::AuthenticationError::new( "auth".to_string() ) ), false, "Auth errors should not be retryable" ),
    ( the_module::AnthropicError::InvalidArgument( "invalid".to_string() ), false, "Validation errors should not be retryable" ),
    ( the_module::AnthropicError::Stream( "stream error".to_string() ), true, "Stream errors should be retryable" ),
  ];

  for ( error, should_retry, description ) in test_cases
  {
    assert_eq!( strategy.should_retry( &error, 1 ), should_retry, "{description}" );
  }
}

/// Test `BackoffCalculator` enhanced implementation
#[ cfg( feature = "retry-logic" ) ]
#[ test ]
fn test_backoff_calculator_enhanced()
{
  // Test rate limit error backoff calculation
  let rate_limit_error = the_module::RateLimitError::new(
    "requests".to_string(),
    Some( 45 ),
    "Rate limit exceeded".to_string()
  );

  let backoff_strategy = the_module::BackoffCalculator::calculate_backoff( &rate_limit_error );
  assert!( backoff_strategy.is_ok() );

  let strategy = backoff_strategy.unwrap();
  assert!( strategy.initial_delay() >= core::time::Duration::from_secs( 45 ) ); // Should respect retry-after
  assert_eq!( strategy.backoff_type(), the_module::BackoffType::Linear );
  assert!( strategy.jitter_enabled() );

  // Test different rate limit types
  let token_rate_limit = the_module::RateLimitError::new(
    "tokens".to_string(),
    None, // No retry-after header
    "Token rate limit exceeded".to_string()
  );

  let token_backoff = the_module::BackoffCalculator::calculate_backoff( &token_rate_limit );
  assert!( token_backoff.is_ok() );

  let token_strategy = token_backoff.unwrap();
  assert!( token_strategy.initial_delay() >= core::time::Duration::from_secs( 1 ) ); // Default backoff
  assert!( token_strategy.initial_delay() <= core::time::Duration::from_mins( 1 ) ); // Reasonable maximum
}

/// Test retry metrics and monitoring
#[ cfg( feature = "retry-logic" ) ]
#[ test ]
fn test_retry_metrics()
{
  let mut metrics = the_module::RetryMetrics::new();

  // Record some retry attempts
  metrics.record_attempt( 1, 1000 );
  metrics.record_attempt( 2, 2000 );
  metrics.record_success( 2 );

  assert_eq!( metrics.total_attempts(), 2 );
  assert_eq!( metrics.successful_retries(), 1 );
  assert_eq!( metrics.total_delay_ms(), 3000 );
  assert!( metrics.average_delay_ms() > 0.0 );

  // Record a failure
  metrics.record_failure( &the_module::AnthropicError::http_error( "timeout occurred".to_string() ) );
  assert_eq!( metrics.failed_attempts(), 1 );

  // Test metrics reset
  metrics.reset();
  assert_eq!( metrics.total_attempts(), 0 );
  assert_eq!( metrics.successful_retries(), 0 );
  assert_eq!( metrics.failed_attempts(), 0 );
}
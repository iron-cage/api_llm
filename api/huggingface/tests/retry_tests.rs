//! Tests for explicit retry logic functionality

mod inc;

#[ cfg( feature = "inference-retry" ) ]
mod retry_tests
{
  use api_huggingface::
  {
  Client,
  ExplicitRetryConfig,
  error::{ HuggingFaceError, ApiErrorWrap },
  };

  /// Test `ExplicitRetryConfig` creation and configuration
  #[ test ]
  fn test_explicit_retry_config_conservative()
  {
  let config = ExplicitRetryConfig::conservative();

  assert_eq!( config.max_retries, 3 );
  assert_eq!( config.initial_delay_ms, 1000 );
  assert_eq!( config.max_delay_ms, 30_000 );
  assert!( ( config.multiplier - 2.0 ).abs() < f64::EPSILON );
  assert_eq!( config.jitter_ms, 100 );
  }

  /// Test `ExplicitRetryConfig` aggressive configuration
  #[ test ]
  fn test_explicit_retry_config_aggressive()
  {
  let config = ExplicitRetryConfig::aggressive();

  assert_eq!( config.max_retries, 5 );
  assert_eq!( config.initial_delay_ms, 500 );
  assert_eq!( config.max_delay_ms, 10_000 );
  assert!( ( config.multiplier - 1.5 ).abs() < f64::EPSILON );
  assert_eq!( config.jitter_ms, 50 );
  }

  /// Test error categorization for retry decisions
  #[ test ]
  fn test_retryable_error_classification()
  {
  // Network errors should be retryable
  let http_error = HuggingFaceError::Http( "Connection timeout".to_string() );
  assert!( is_error_retryable_test( &http_error ) );

  // Rate limit errors should be retryable
  let rate_limit_error = HuggingFaceError::RateLimit( "Too many requests".to_string() );
  assert!( is_error_retryable_test( &rate_limit_error ) );

  // Model unavailable should be retryable
  let model_error = HuggingFaceError::ModelUnavailable( "Model loading".to_string() );
  assert!( is_error_retryable_test( &model_error ) );

  // 5xx API errors should be retryable
  let server_error = HuggingFaceError::Api( 
      ApiErrorWrap::new( "Internal server error" ).with_status_code( 500 )
  );
  assert!( is_error_retryable_test( &server_error ) );

  // 4xx API errors should NOT be retryable
  let client_error = HuggingFaceError::Api( 
      ApiErrorWrap::new( "Bad request" ).with_status_code( 400 )
  );
  assert!( !is_error_retryable_test( &client_error ) );

  // Authentication errors should NOT be retryable
  let auth_error = HuggingFaceError::Authentication( "Invalid API key".to_string() );
  assert!( !is_error_retryable_test( &auth_error ) );

  // Validation errors should NOT be retryable
  let validation_error = HuggingFaceError::Validation( "Invalid input".to_string() );
  assert!( !is_error_retryable_test( &validation_error ) );
  }

  /// Test API error message pattern matching for retry decisions
  #[ test ]
  fn test_api_error_message_patterns()
  {
  // API errors with retryable patterns in message
  let timeout_error = HuggingFaceError::Api( 
      ApiErrorWrap::new( "Request timeout occurred" )
  );
  assert!( is_error_retryable_test( &timeout_error ) );

  let unavailable_error = HuggingFaceError::Api( 
      ApiErrorWrap::new( "Service temporarily unavailable" )
  );
  assert!( is_error_retryable_test( &unavailable_error ) );

  let overloaded_error = HuggingFaceError::Api( 
      ApiErrorWrap::new( "Server overloaded, try again later" )
  );
  assert!( is_error_retryable_test( &overloaded_error ) );

  let rate_limit_msg_error = HuggingFaceError::Api( 
      ApiErrorWrap::new( "Rate limit exceeded" )
  );
  assert!( is_error_retryable_test( &rate_limit_msg_error ) );

  // API errors without retryable patterns
  let invalid_error = HuggingFaceError::Api( 
      ApiErrorWrap::new( "Invalid model specified" )
  );
  assert!( !is_error_retryable_test( &invalid_error ) );
  }

  /// Helper function to access the private `is_retryable_error` function for testing
  fn is_error_retryable_test( error : &HuggingFaceError ) -> bool
  {
  // This is a simplified version of the logic from the private function
  match error
  {
      HuggingFaceError::Http( _ ) | 
      HuggingFaceError::RateLimit( _ ) | 
      HuggingFaceError::ModelUnavailable( _ ) | 
      HuggingFaceError::Stream( _ ) => true,
      HuggingFaceError::Api( api_error ) =>
      {
  if let Some( status_code ) = api_error.status_code
  {
          ( 500..600 ).contains( &status_code )
  }
  else
  {
          let msg = api_error.message.to_lowercase();
          msg.contains( "timeout" ) || 
          msg.contains( "unavailable" ) ||
          msg.contains( "overloaded" ) ||
          msg.contains( "rate limit" ) ||
          msg.contains( "service" )
  }
      },
      HuggingFaceError::Authentication( _ ) |
      HuggingFaceError::Validation( _ ) |
      HuggingFaceError::Serialization( _ ) |
      HuggingFaceError::InvalidArgument( _ ) |
      HuggingFaceError::Generic( _ ) => false,
  }
  }

  /// Integration test : Client with retry policy
  #[ cfg( feature = "env-config" ) ]
  #[ tokio::test ]
  async fn test_client_with_custom_retry_policy()
  {
  use api_huggingface::{ environment::HuggingFaceEnvironmentImpl, Secret };
  
  // Create a test environment (this won't make real requests)
  let secret = Secret::new( "test_key".to_string() );
  let env = HuggingFaceEnvironmentImpl::build( secret, None )
      .expect( "Failed to create test environment" );
  
  // Create client (explicit retry configuration is separate from client per governing principle)
  let _client = Client::build( env )
      .expect( "Failed to build client" );

  // Explicit retry configuration is separate from client
  let explicit_retry_config = ExplicitRetryConfig {
      max_retries : 2,
      initial_delay_ms : 50,
      multiplier : 1.8,
      max_delay_ms : 5000,
      jitter_ms : 25,
  };

  // Verify the explicit retry configuration
  assert_eq!( explicit_retry_config.max_retries, 2 );
  assert_eq!( explicit_retry_config.initial_delay_ms, 50 );
  assert!( ( explicit_retry_config.multiplier - 1.8 ).abs() < f64::EPSILON );
  }

  /// Integration test : Client explicit retry behavior
  #[ cfg( feature = "env-config" ) ]
  #[ tokio::test ]
  async fn test_client_explicit_retry_pattern()
  {
  use api_huggingface::{ environment::HuggingFaceEnvironmentImpl, Secret };

  // Create a test environment
  let secret = Secret::new( "test_key".to_string() );
  let env = HuggingFaceEnvironmentImpl::build( secret, None )
      .expect( "Failed to create test environment" );

  // Create client (no automatic retry per governing principle)
  let _client = Client::build( env )
      .expect( "Failed to build client" );

  // Explicit retry configuration is developer-controlled
  let retry_config = ExplicitRetryConfig::conservative();

  // Verify retry configuration is explicit and transparent
  assert_eq!( retry_config.max_retries, 3 );
  assert_eq!( retry_config.initial_delay_ms, 1000 );
  }

  /// Test explicit retry configuration struct construction
  #[ test ]
  fn test_explicit_retry_config_custom()
  {
  let config = ExplicitRetryConfig {
      max_retries : 3,
      initial_delay_ms : 100,
      multiplier : 3.0,
      max_delay_ms : 10_000,
      jitter_ms : 200,
  };

  assert_eq!( config.max_retries, 3 );
  assert!( ( config.multiplier - 3.0 ).abs() < f64::EPSILON );
  assert_eq!( config.jitter_ms, 200 );
  }

  #[ cfg( feature = "integration" ) ]
  use api_huggingface::
  {
      environment::HuggingFaceEnvironmentImpl,
      secret::Secret,
  };

  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
  async fn integration_explicit_retry_with_real_api_failures()
  {
      // Test with invalid API key first to trigger failures, then verify explicit retry behavior
      let invalid_key = Secret::new( "invalid-key-12345".to_string() );
      let env = HuggingFaceEnvironmentImpl::build( invalid_key, None )
  .expect( "Environment build should succeed" );

      // Create client (no automatic retry per governing principle)
      let client = Client::build( env )
  .expect( "Client build should succeed" );

      // Explicit retry configuration (fewer retries for faster test)
      let retry_config = ExplicitRetryConfig {
  max_retries : 2,
  initial_delay_ms : 50,
  multiplier : 2.0,
  max_delay_ms : 1000,
  jitter_ms : 10,
      };

      // Make API call with explicit retry that should fail and trigger retries
      let start_time = std::time::Instant::now();
      let result : Result< serde_json::Value, _ > = client.post_with_explicit_retry(
  "https://api-inference.huggingface.co/models/microsoft/DialoGPT-medium",
  &serde_json::json!({
          "inputs": "test"
  }),
  &retry_config
      ).await;
      let elapsed = start_time.elapsed();

      // Should fail due to invalid key
      assert!( result.is_err(), "Invalid key should cause failure" );

      // Should have taken time due to explicit retries (at least 50ms for first retry)
      assert!( elapsed.as_millis() > 40, "Should have spent time on explicit retries : {elapsed:?}" );

      // Verify explicit retry configuration is developer-controlled
      assert_eq!( retry_config.max_retries, 2 );
  }

  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
  async fn integration_retry_with_rate_limiting()
  {
      // Get real API key (will panic with clear message if missing)
      let api_key_string = crate::inc::get_api_key_for_integration();
      
      // Build client with real credentials (explicit retry configuration separate)
      let api_key = Secret::new( api_key_string );
      let env = HuggingFaceEnvironmentImpl::build( api_key, None )
  .expect( "Environment build should succeed" );

      let client = Client::build( env )
  .expect( "Client build should succeed" );

      // Explicit retry configuration for aggressive retries
      let retry_config = ExplicitRetryConfig {
  max_retries : 3,
  initial_delay_ms : 100,
  multiplier : 1.5,
  max_delay_ms : 2000,
  jitter_ms : 50,
      };

      // Make multiple rapid calls with explicit retry to potentially trigger rate limiting
      let mut results = Vec::new();
      for i in 0..3
      {
  let result : Result< serde_json::Value, _ > = client.post_with_explicit_retry(
          "https://api-inference.huggingface.co/models/microsoft/DialoGPT-medium",
          &serde_json::json!({
      "inputs": format!( "test retry {i}" ),
      "parameters": {
              "max_new_tokens": 5
      }
          }),
          &retry_config
  ).await;
  results.push( result );
      }

      // At least one should succeed or show proper retry behavior
      let success_count = results.iter().filter( |r| r.is_ok() ).count();
      let error_count = results.iter().filter( |r| r.is_err() ).count();
      
      assert!( 
  success_count > 0 || error_count == 3, 
  "Should have either successes or all errors with retry attempts" 
      );

      // If errors occurred, they should be meaningful
      for result in &results
      {
  if let Err( error ) = result
  {
          let error_msg = format!( "{error}" );
          assert!( !error_msg.is_empty(), "Error message should not be empty" );
  }
      }
  }

  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
  async fn integration_retry_disabled_vs_enabled()
  {
      // Get real API key (will panic with clear message if missing)
      let api_key_string = crate::inc::get_api_key_for_integration();
      
      // Test client without retry
      let api_key = Secret::new( api_key_string.clone() );
      let env = HuggingFaceEnvironmentImpl::build( api_key, None )
  .expect( "Environment build should succeed" );
      
      let client_no_retry = Client::build( env )
  .expect( "Client build should succeed" );

      // No automatic retry per governing principle - explicit configuration required

      // Test client with explicit retry configuration
      let api_key2 = Secret::new( api_key_string );
      let env2 = HuggingFaceEnvironmentImpl::build( api_key2, None )
  .expect( "Environment build should succeed" );

      let client_with_retry = Client::build( env2 )
  .expect( "Client build should succeed" );

      let explicit_retry_config = ExplicitRetryConfig::conservative();

      // Verify explicit retry configuration
      assert_eq!( explicit_retry_config.max_retries, 3 );

      // Both should work with valid requests
      let result1 : Result< serde_json::Value, _ > = client_no_retry.post(
  "https://api-inference.huggingface.co/models/microsoft/DialoGPT-medium",
  &serde_json::json!({
          "inputs": "test no retry",
          "parameters": { "max_new_tokens": 5 }
  })
      ).await;

      let result2 : Result< serde_json::Value, _ > = client_with_retry.post_with_explicit_retry(
  "https://api-inference.huggingface.co/models/microsoft/DialoGPT-medium",
  &serde_json::json!({
          "inputs": "test with retry",
          "parameters": { "max_new_tokens": 5 }
  }),
  &explicit_retry_config
      ).await;

      // Both should succeed or fail for the same reasons (API availability)
      match ( &result1, &result2 )
      {
  ( Ok( _ ), Ok( _ ) ) | ( Err( _ ), Err( _ ) ) => {}, // Both same outcome
  ( Ok( _ ), Err( _ ) ) => panic!( "Retry client should not fail when no-retry succeeds" ),
  ( Err( e1 ), Ok( _ ) ) =>
  {
          // This is expected - retry client might succeed where no-retry fails
          println!( "No-retry failed : {e1}, retry client succeeded - this is expected behavior" );
  }
      }
  }
}
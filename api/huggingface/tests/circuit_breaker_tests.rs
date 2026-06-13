//! Integration tests for Circuit Breaker
//!
//! These tests use REAL `HuggingFace` API calls to verify circuit breaker behavior.
//! NO MOCKING is used - all tests interact with actual endpoints.
//!
//! ## Test Strategy
//!
//! - Use real `HuggingFace` API endpoints
//! - Test actual failure scenarios ( invalid models, network errors )
//! - Test recovery scenarios with real successful calls

#![ allow( clippy::doc_markdown ) ]
//! - Test all state transitions with real operations
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
//! cargo test --test circuit_breaker_tests --all-features -- --ignored
//! ```

use api_huggingface::reliability::{ CircuitBreaker, CircuitBreakerConfig, CircuitState };
use core::time::Duration;

#[ cfg( feature = "integration" ) ]
use api_huggingface::{
  Client,
  environment::HuggingFaceEnvironmentImpl,
  providers::ChatMessage,
  Secret,
};

/// Helper to create a test client
#[ cfg( feature = "integration" ) ]
fn create_test_client() -> Client< HuggingFaceEnvironmentImpl >
{
  use workspace_tools as workspace;

  let workspace = workspace::workspace()
    .expect( "[create_test_client] Failed to access workspace - required for integration tests" );
  let secrets = workspace.load_secrets_from_file( "-secrets.sh" )
    .expect( "[create_test_client] Failed to load secret/-secrets.sh - required for integration tests" );
  let api_key = secrets.get( "HUGGINGFACE_API_KEY" )
    .expect( "[create_test_client] HUGGINGFACE_API_KEY not found in secret/-secrets.sh - required for integration tests. Get your token from https://huggingface.co/settings/tokens" )
    .clone();

  let secret = Secret::new( api_key );
  let env = HuggingFaceEnvironmentImpl::build( secret, None )
    .expect( "Failed to build environment" );
  Client::build( env ).expect( "Failed to create client" )
}

// ============================================================================
// Basic Circuit Breaker Tests
// ============================================================================

#[ tokio::test ]
async fn test_circuit_breaker_initial_state_is_closed() 
{
  let circuit_breaker = CircuitBreaker::new( CircuitBreakerConfig::default( ));

  assert!( circuit_breaker.is_closed( ).await );
  assert_eq!( circuit_breaker.state( ).await, CircuitState::Closed );
  assert_eq!( circuit_breaker.failure_count( ).await, 0 );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_circuit_breaker_successful_request_keeps_closed() 
{
  let client = create_test_client( );
  let circuit_breaker = CircuitBreaker::new( CircuitBreakerConfig::default( ));

  let result = circuit_breaker.execute( async {
  client.providers( ).chat_completion(
      "meta-llama/Llama-3.2-1B-Instruct",
      vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
      Some( 10 ),
      None,
      None,
  ).await
  } ).await;

  assert!( result.is_ok( ), "Request should succeed" );
  assert!( circuit_breaker.is_closed( ).await, "Circuit should remain closed" );
  assert_eq!( circuit_breaker.failure_count( ).await, 0, "Failure count should be 0" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_circuit_breaker_resets_failure_count_on_success() 
{
  let client = create_test_client( );
  let config = CircuitBreakerConfig {
  failure_threshold : 5,
  success_threshold : 2,
  timeout : Duration::from_secs( 60 ),
  };
  let circuit_breaker = CircuitBreaker::new( config );

  // First, cause some failures ( but not enough to open circuit )
  for _ in 0..2
  {
  let _ = circuit_breaker.execute( async {
      client.providers( ).chat_completion(
  "invalid-model-xyz",
  vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
  Some( 10 ),
  None,
  None,
      ).await
  } ).await;
  }

  assert_eq!( circuit_breaker.failure_count( ).await, 2 );
  assert!( circuit_breaker.is_closed( ).await );

  // Now succeed
  let result = circuit_breaker.execute( async {
  client.providers( ).chat_completion(
      "meta-llama/Llama-3.2-1B-Instruct",
      vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
      Some( 10 ),
      None,
      None,
  ).await
  } ).await;

  assert!( result.is_ok( ));
  assert_eq!( circuit_breaker.failure_count( ).await, 0, "Success should reset failure count" );
}

// ============================================================================
// Circuit Opening Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_circuit_breaker_opens_after_threshold_failures() 
{
  let client = create_test_client( );
  let config = CircuitBreakerConfig {
  failure_threshold : 3,
  success_threshold : 2,
  timeout : Duration::from_secs( 60 ),
  };
  let circuit_breaker = CircuitBreaker::new( config );

  // Execute 3 failing requests
  for i in 0..3
  {
  let _ = circuit_breaker.execute( async {
      client.providers( ).chat_completion(
  "invalid-model-xyz",
  vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
  Some( 10 ),
  None,
  None,
      ).await
  } ).await;

  if i < 2
  {
      assert!( circuit_breaker.is_closed( ).await, "Circuit should stay closed before threshold" );
  }
  }

  assert!( circuit_breaker.is_open( ).await, "Circuit should be open after threshold failures" );
  assert_eq!( circuit_breaker.failure_count( ).await, 3 );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_circuit_breaker_rejects_requests_when_open() 
{
  let client = create_test_client( );
  let config = CircuitBreakerConfig {
  failure_threshold : 2,
  success_threshold : 2,
  timeout : Duration::from_secs( 60 ),
  };
  let circuit_breaker = CircuitBreaker::new( config );

  // Open the circuit with failures
  for _ in 0..2
  {
  let _ = circuit_breaker.execute( async {
      client.providers( ).chat_completion(
  "invalid-model-xyz",
  vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
  Some( 10 ),
  None,
  None,
      ).await
  } ).await;
  }

  assert!( circuit_breaker.is_open( ).await );

  // Try a request that would normally succeed - should be rejected
  let result = circuit_breaker.execute( async {
  client.providers( ).chat_completion(
      "meta-llama/Llama-3.2-1B-Instruct",
      vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
      Some( 10 ),
      None,
      None,
  ).await
  } ).await;

  assert!( result.is_err( ), "Request should be rejected" );
  match result
  {
  Err( api_huggingface::reliability::CircuitBreakerError::CircuitOpen ) => {
      // Expected
  }
  _ => panic!( "Expected CircuitOpen error" ),
  }
}

// ============================================================================
// Half-Open State and Recovery Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_circuit_breaker_transitions_to_half_open_after_timeout() 
{
  let client = create_test_client( );
  let config = CircuitBreakerConfig {
  failure_threshold : 2,
  success_threshold : 2,
  timeout : Duration::from_millis( 500 ),
  };
  let circuit_breaker = CircuitBreaker::new( config );

  // Open the circuit
  for _ in 0..2
  {
  let _ = circuit_breaker.execute( async {
      client.providers( ).chat_completion(
  "invalid-model-xyz",
  vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
  Some( 10 ),
  None,
  None,
      ).await
  } ).await;
  }

  assert!( circuit_breaker.is_open( ).await );

  // Wait for timeout
  tokio::time::sleep( Duration::from_millis( 600 )).await;

  // Execute a request - should transition to half-open
  let result = circuit_breaker.execute( async {
  client.providers( ).chat_completion(
      "meta-llama/Llama-3.2-1B-Instruct",
      vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
      Some( 10 ),
      None,
      None,
  ).await
  } ).await;

  assert!( result.is_ok( ), "Request should succeed in half-open state" );
  assert!( !circuit_breaker.is_open( ).await, "Circuit should not be open" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_circuit_breaker_closes_after_success_threshold_in_half_open() 
{
  let client = create_test_client( );
  let config = CircuitBreakerConfig {
  failure_threshold : 2,
  success_threshold : 2,
  timeout : Duration::from_millis( 500 ),
  };
  let circuit_breaker = CircuitBreaker::new( config );

  // Open the circuit
  for _ in 0..2
  {
  let _ = circuit_breaker.execute( async {
      client.providers( ).chat_completion(
  "invalid-model-xyz",
  vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
  Some( 10 ),
  None,
  None,
      ).await
  } ).await;
  }

  assert!( circuit_breaker.is_open( ).await );

  // Wait for timeout
  tokio::time::sleep( Duration::from_millis( 600 )).await;

  // Execute success_threshold successful requests
  for _ in 0..2
  {
  let result = circuit_breaker.execute( async {
      client.providers( ).chat_completion(
  "meta-llama/Llama-3.2-1B-Instruct",
  vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
  Some( 10 ),
  None,
  None,
      ).await
  } ).await;
  assert!( result.is_ok( ));
  }

  // Circuit should be closed
  assert!( circuit_breaker.is_closed( ).await, "Circuit should be closed after success threshold" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_circuit_breaker_reopens_on_failure_in_half_open() 
{
  let client = create_test_client( );
  let config = CircuitBreakerConfig {
  failure_threshold : 2,
  success_threshold : 2,
  timeout : Duration::from_millis( 500 ),
  };
  let circuit_breaker = CircuitBreaker::new( config );

  // Open the circuit
  for _ in 0..2
  {
  let _ = circuit_breaker.execute( async {
      client.providers( ).chat_completion(
  "invalid-model-xyz",
  vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
  Some( 10 ),
  None,
  None,
      ).await
  } ).await;
  }

  assert!( circuit_breaker.is_open( ).await );

  // Wait for timeout
  tokio::time::sleep( Duration::from_millis( 600 )).await;

  // One success ( transitions to half-open )
  let _ = circuit_breaker.execute( async {
  client.providers( ).chat_completion(
      "meta-llama/Llama-3.2-1B-Instruct",
      vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
      Some( 10 ),
      None,
      None,
  ).await
  } ).await;

  // One failure ( should reopen )
  let _ = circuit_breaker.execute( async {
  client.providers( ).chat_completion(
      "invalid-model-xyz",
      vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
      Some( 10 ),
      None,
      None,
  ).await
  } ).await;

  // Circuit should be open again
  assert!( circuit_breaker.is_open( ).await, "Circuit should reopen after failure in half-open" );
}

// ============================================================================
// Reset Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_circuit_breaker_reset_clears_all_state() 
{
  let client = create_test_client( );
  let config = CircuitBreakerConfig {
  failure_threshold : 2,
  success_threshold : 2,
  timeout : Duration::from_secs( 60 ),
  };
  let circuit_breaker = CircuitBreaker::new( config );

  // Open the circuit
  for _ in 0..2
  {
  let _ = circuit_breaker.execute( async {
      client.providers( ).chat_completion(
  "invalid-model-xyz",
  vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
  Some( 10 ),
  None,
  None,
      ).await
  } ).await;
  }

  assert!( circuit_breaker.is_open( ).await );
  assert_eq!( circuit_breaker.failure_count( ).await, 2 );

  // Reset
  circuit_breaker.reset( ).await;

  // All state should be cleared
  assert!( circuit_breaker.is_closed( ).await );
  assert_eq!( circuit_breaker.failure_count( ).await, 0 );
  assert_eq!( circuit_breaker.success_count( ).await, 0 );

  // Should be able to execute requests normally
  let result = circuit_breaker.execute( async {
  client.providers( ).chat_completion(
      "meta-llama/Llama-3.2-1B-Instruct",
      vec![ChatMessage { role : "user".to_string( ), content : "test".to_string( ), tool_calls : None, tool_call_id : None } ],
      Some( 10 ),
      None,
      None,
  ).await
  } ).await;

  assert!( result.is_ok( ));
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[ tokio::test ]
async fn test_circuit_breaker_default_config() 
{
  let config = CircuitBreakerConfig::default( );

  assert_eq!( config.failure_threshold, 5 );
  assert_eq!( config.success_threshold, 2 );
  assert_eq!( config.timeout, Duration::from_secs( 60 ));

  let circuit_breaker = CircuitBreaker::new( config );
  assert!( circuit_breaker.is_closed( ).await );
}

#[ tokio::test ]
async fn test_circuit_breaker_custom_config() 
{
  let config = CircuitBreakerConfig {
  failure_threshold : 10,
  success_threshold : 3,
  timeout : Duration::from_secs( 120 ),
  };

  let circuit_breaker = CircuitBreaker::new( config );
  assert!( circuit_breaker.is_closed( ).await );
}

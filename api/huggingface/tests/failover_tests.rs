//! Integration tests for Failover
//!
//! These tests verify the failover functionality using real `HuggingFace` API endpoints.
//! Tests require HUGGINGFACE_API_KEY in workspace secrets and will fail loudly if missing.
//!
//! To run these tests:
//! ```bash
//! cargo test --test failover_tests --features integration --all-features
//! ```

#![ allow( clippy::doc_markdown ) ]
//!
//! ## Test Coverage
//!
//! - Priority strategy failover
//! - RoundRobin strategy failover
//! - Random strategy failover
//! - Sticky strategy failover
//! - Endpoint health tracking
//! - Failure threshold detection
//! - Success recovery
//! - Exhausting all endpoints
//! - Real API integration
//! - Concurrent access

use api_huggingface::{
  Client,
  environment::HuggingFaceEnvironmentImpl,
  providers::ChatMessage,
  reliability::{
  FailoverManager,
  FailoverConfig,
  FailoverStrategy,
  FailoverError,
  },
  Secret,
};
use core::time::Duration;
use std::sync::Arc;

/// Create a test client with API key from workspace secrets
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

  let env = HuggingFaceEnvironmentImpl::build( Secret::new( api_key ), None )
    .expect( "Environment creation should succeed" );
  Client::build( env )
    .expect( "Client creation should succeed" )
}

/// Helper function to create test messages
fn create_test_messages() -> Vec< ChatMessage > 
{
  vec![
  ChatMessage {
      role : "user".to_string( ),
      content : "Hello, how are you?".to_string( ),
  }
  ]
}

// ============================================================================
// Priority Strategy Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_priority_failover_first_endpoint_succeeds() 
{
  let client = create_test_client( );

  let config = FailoverConfig {
  endpoints : vec![
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
  ],
  strategy : FailoverStrategy::Priority,
  max_retries : 3,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 3,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  // Execute with failover - should use first endpoint
  let result = failover.execute_with_failover( |model| {
  let client = client.clone( );
  let messages = create_test_messages( );
  Box::pin( async move {
      client.providers( ).chat_completion(
  &model,
  messages,
  Some( 20 ),
  None,
  None,
      ).await
  } )
  } ).await;

  assert!( result.is_ok( ), "First endpoint should succeed" );
}

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_priority_failover_first_fails_second_succeeds() 
{
  let client = create_test_client( );

  let config = FailoverConfig {
  endpoints : vec![
      "invalid-model-xyz-12345".to_string( ),
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
  ],
  strategy : FailoverStrategy::Priority,
  max_retries : 2,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 1,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  // First endpoint will fail, should failover to second
  let result = failover.execute_with_failover( |model| {
  let client = client.clone( );
  let messages = create_test_messages( );
  Box::pin( async move {
      client.providers( ).chat_completion(
  &model,
  messages,
  Some( 20 ),
  None,
  None,
      ).await
  } )
  } ).await;

  assert!( result.is_ok( ), "Should succeed with second endpoint after first fails" );

  // Check health status
  let health = failover.health_status( ).await;
  assert_eq!( health.len( ), 2 );

  // First endpoint should be marked unhealthy
  assert!( !health[0 ].healthy, "First endpoint should be unhealthy" );

  // Second endpoint should be healthy
  assert!( health[1 ].healthy, "Second endpoint should be healthy" );
}

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_priority_failover_all_endpoints_fail() 
{
  let client = create_test_client( );

  let config = FailoverConfig {
  endpoints : vec![
      "invalid-model-1".to_string( ),
      "invalid-model-2".to_string( ),
  ],
  strategy : FailoverStrategy::Priority,
  max_retries : 2,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 1,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  // All endpoints will fail
  let result = failover.execute_with_failover( |model| {
  let client = client.clone( );
  let messages = create_test_messages( );
  Box::pin( async move {
      client.providers( ).chat_completion(
  &model,
  messages,
  Some( 20 ),
  None,
  None,
      ).await
  } )
  } ).await;

  assert!( result.is_err( ), "Should fail when all endpoints fail" );

  match result
  {
  Err( FailoverError::AllRetriesFailed { attempts, .. } ) => {
      assert!( attempts > 0, "Should have attempted at least one request" );
  }
  _ => panic!( "Expected AllRetriesFailed error" ),
  }
}

// ============================================================================
// RoundRobin Strategy Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_round_robin_cycles_through_endpoints() 
{
  let client = create_test_client( );

  let config = FailoverConfig {
  // Use two different valid models to test round-robin distribution
  endpoints : vec![
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
      "Qwen/Qwen2.5-72B-Instruct".to_string( ),
  ],
  strategy : FailoverStrategy::RoundRobin,
  max_retries : 1,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 5,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  // Make multiple successful requests - should cycle between endpoints
  for _ in 0..4
  {
  let result = failover.execute_with_failover( |model| {
      let client = client.clone( );
      let messages = create_test_messages( );
      Box::pin( async move {
  client.providers( ).chat_completion(
          &model,
          messages,
          Some( 20 ),
          None,
          None,
  ).await
      } )
  } ).await;

  assert!( result.is_ok( ), "Request should succeed" );
  }

  // Check that both endpoints were used
  let health = failover.health_status( ).await;
  assert!( health[0 ].requests > 0, "First endpoint should have requests" );
  assert!( health[1 ].requests > 0, "Second endpoint should have requests" );
}

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_round_robin_skips_unhealthy_endpoints() 
{
  let client = create_test_client( );

  let config = FailoverConfig {
  endpoints : vec![
      "invalid-model-xyz".to_string( ),
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
  ],
  strategy : FailoverStrategy::RoundRobin,
  max_retries : 2,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 1,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  // Make several requests - first will fail, rest should use healthy endpoints
  for _ in 0..3
  {
  let result = failover.execute_with_failover( |model| {
      let client = client.clone( );
      let messages = create_test_messages( );
      Box::pin( async move {
  client.providers( ).chat_completion(
          &model,
          messages,
          Some( 20 ),
          None,
          None,
  ).await
      } )
  } ).await;

  assert!( result.is_ok( ), "Should succeed with healthy endpoints" );
  }

  let health = failover.health_status( ).await;

  // First endpoint should be unhealthy
  assert!( !health[0 ].healthy, "First endpoint should be unhealthy" );

  // Other endpoints should be healthy
  assert!( health[1 ].healthy, "Second endpoint should be healthy" );
  assert!( health[2 ].healthy, "Third endpoint should be healthy" );
}

// ============================================================================
// Random Strategy Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_random_strategy_uses_random_endpoints() 
{
  let client = create_test_client( );

  let config = FailoverConfig {
  endpoints : vec![
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
  ],
  strategy : FailoverStrategy::Random,
  max_retries : 1,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 5,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  // Make multiple requests
  for _ in 0..5
  {
  let result = failover.execute_with_failover( |model| {
      let client = client.clone( );
      let messages = create_test_messages( );
      Box::pin( async move {
  client.providers( ).chat_completion(
          &model,
          messages,
          Some( 20 ),
          None,
          None,
  ).await
      } )
  } ).await;

  assert!( result.is_ok( ), "Request should succeed" );
  }

  // With enough requests, both endpoints should likely be used
  let health = failover.health_status( ).await;
  let total_requests : u64 = health.iter( ).map( |h| h.requests ).sum( );
  assert_eq!( total_requests, 5, "Should have made 5 requests total" );
}

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_random_strategy_avoids_unhealthy_endpoints()
{
  let client = create_test_client( );

  // Test that Random strategy avoids unhealthy endpoints
  // We pre-mark the invalid endpoint as unhealthy to make the test deterministic
  let config = FailoverConfig {
  endpoints : vec![
      "invalid-model-xyz".to_string( ),
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
  ],
  strategy : FailoverStrategy::Random,
  max_retries : 2,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 1,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  // Pre-mark invalid endpoint as unhealthy (deterministic setup)
  // This ensures Random strategy can only select the healthy endpoint
  failover.record_failure( "invalid-model-xyz" ).await;

  // Verify first endpoint is now unhealthy
  let health = failover.health_status( ).await;
  assert!( !health[ 0 ].healthy, "Invalid endpoint should be pre-marked unhealthy" );

  // All requests should now only use the healthy endpoint (Random avoids unhealthy)
  for _ in 0..4
  {
  let result = failover.execute_with_failover( |model| {
      let client = client.clone( );
      let messages = create_test_messages( );
      Box::pin( async move {
  client.providers( ).chat_completion(
          &model,
          messages,
          Some( 20 ),
          None,
          None,
  ).await
      } )
  } ).await;

  assert!( result.is_ok( ), "Should succeed with healthy endpoint" );
  }

  let health = failover.health_status( ).await;
  assert!( !health[ 0 ].healthy, "Invalid endpoint should remain unhealthy" );
  assert!( health[ 1 ].healthy, "Valid endpoint should be healthy" );
  assert_eq!( health[ 1 ].successes, 4, "All 4 successes should be on healthy endpoint" );
}

// ============================================================================
// Sticky Strategy Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_sticky_strategy_uses_same_endpoint() 
{
  let client = create_test_client( );

  let config = FailoverConfig {
  endpoints : vec![
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
  ],
  strategy : FailoverStrategy::Sticky,
  max_retries : 1,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 3,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  // Make multiple requests - should stick to one endpoint
  for _ in 0..5
  {
  let result = failover.execute_with_failover( |model| {
      let client = client.clone( );
      let messages = create_test_messages( );
      Box::pin( async move {
  client.providers( ).chat_completion(
          &model,
          messages,
          Some( 20 ),
          None,
          None,
  ).await
      } )
  } ).await;

  assert!( result.is_ok( ), "Request should succeed" );
  }

  // One endpoint should have all requests
  let health = failover.health_status( ).await;
  let endpoints_used : Vec< _ > = health.iter( )
  .filter( |h| h.requests > 0 )
  .collect( );

  assert_eq!( endpoints_used.len( ), 1, "Should stick to one endpoint" );
  assert_eq!( endpoints_used[0 ].requests, 5, "Should have all 5 requests" );
}

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_sticky_strategy_switches_on_failure() 
{
  let client = create_test_client( );

  let config = FailoverConfig {
  endpoints : vec![
      "invalid-model-xyz".to_string( ),
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
  ],
  strategy : FailoverStrategy::Sticky,
  max_retries : 2,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 1,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  // First request will fail and switch to second endpoint
  let result = failover.execute_with_failover( |model| {
  let client = client.clone( );
  let messages = create_test_messages( );
  Box::pin( async move {
      client.providers( ).chat_completion(
  &model,
  messages,
  Some( 20 ),
  None,
  None,
      ).await
  } )
  } ).await;

  assert!( result.is_ok( ), "Should failover to second endpoint" );

  // Subsequent requests should stick to second endpoint
  for _ in 0..3
  {
  let result = failover.execute_with_failover( |model| {
      let client = client.clone( );
      let messages = create_test_messages( );
      Box::pin( async move {
  client.providers( ).chat_completion(
          &model,
          messages,
          Some( 20 ),
          None,
          None,
  ).await
      } )
  } ).await;

  assert!( result.is_ok( ), "Should stick to healthy endpoint" );
  }

  let health = failover.health_status( ).await;
  assert!( !health[0 ].healthy, "First endpoint should be unhealthy" );
  assert!( health[1 ].healthy, "Second endpoint should be healthy" );
  assert_eq!( health[1 ].successes, 4, "All 4 successes should be on second endpoint" );
}

// ============================================================================
// Health Tracking Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_health_tracking_marks_unhealthy_after_threshold() 
{
  let client = create_test_client( );

  let config = FailoverConfig {
  endpoints : vec![
      "invalid-model-xyz".to_string( ),
  ],
  strategy : FailoverStrategy::Priority,
  max_retries : 0,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 2,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  // First failure - should still be healthy
  let _ = failover.execute_with_failover( |model| {
  let client = client.clone( );
  let messages = create_test_messages( );
  Box::pin( async move {
      client.providers( ).chat_completion(
  &model,
  messages,
  Some( 20 ),
  None,
  None,
      ).await
  } )
  } ).await;

  let health = failover.health_status( ).await;
  assert!( health[0 ].healthy, "Should still be healthy after first failure" );

  // Second failure - should become unhealthy
  let _ = failover.execute_with_failover( |model| {
  let client = client.clone( );
  let messages = create_test_messages( );
  Box::pin( async move {
      client.providers( ).chat_completion(
  &model,
  messages,
  Some( 20 ),
  None,
  None,
      ).await
  } )
  } ).await;

  let health = failover.health_status( ).await;
  assert!( !health[0 ].healthy, "Should be unhealthy after reaching threshold" );
}

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_health_tracking_recovers_after_success() 
{
  let client = create_test_client( );

  let config = FailoverConfig {
  endpoints : vec![
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
  ],
  strategy : FailoverStrategy::Priority,
  max_retries : 0,
  failure_window : Duration::from_secs( 1 ),
  failure_threshold : 1,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  // Manually mark as unhealthy by recording failure
  {
  let endpoint = failover.select_endpoint( ).await.ok( );
  if let Some( ep ) = endpoint
  {
      failover.record_failure( &ep ).await;
  }
  }

  let health = failover.health_status( ).await;
  assert!( !health[0 ].healthy, "Should be unhealthy after manual failure" );

  // Wait for failure window to expire
  tokio::time::sleep( Duration::from_secs( 2 )).await;

  // Success should restore health
  let result = failover.execute_with_failover( |model| {
  let client = client.clone( );
  let messages = create_test_messages( );
  Box::pin( async move {
      client.providers( ).chat_completion(
  &model,
  messages,
  Some( 20 ),
  None,
  None,
      ).await
  } )
  } ).await;

  assert!( result.is_ok( ), "Request should succeed" );

  let health = failover.health_status( ).await;
  assert!( health[0 ].healthy, "Should be healthy after success" );
}

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_health_status_reflects_request_counts() 
{
  let client = create_test_client( );

  let config = FailoverConfig {
  endpoints : vec![
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
  ],
  strategy : FailoverStrategy::Priority,
  max_retries : 0,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 5,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  // Make 3 successful requests
  for _ in 0..3
  {
  let _ = failover.execute_with_failover( |model| {
      let client = client.clone( );
      let messages = create_test_messages( );
      Box::pin( async move {
  client.providers( ).chat_completion(
          &model,
          messages,
          Some( 20 ),
          None,
          None,
  ).await
      } )
  } ).await;
  }

  let health = failover.health_status( ).await;
  assert_eq!( health[0 ].requests, 3, "Should have 3 requests" );
  assert_eq!( health[0 ].successes, 3, "Should have 3 successes" );
  assert!( health[0 ].healthy, "Should be healthy" );
}

// ============================================================================
// Reset Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_reset_clears_health_tracking() 
{
  let config = FailoverConfig {
  endpoints : vec![
      "endpoint1".to_string( ),
      "endpoint2".to_string( ),
  ],
  strategy : FailoverStrategy::Priority,
  max_retries : 3,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 2,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  // Record some failures manually
  failover.record_failure( "endpoint1" ).await;
  failover.record_failure( "endpoint1" ).await;

  let health = failover.health_status( ).await;
  assert!( !health[0 ].healthy, "Should be unhealthy after failures" );

  // Reset
  failover.reset( ).await;

  let health = failover.health_status( ).await;
  assert!( health[0 ].healthy, "Should be healthy after reset" );
  assert_eq!( health[0 ].requests, 0, "Requests should be reset" );
  assert_eq!( health[0 ].successes, 0, "Successes should be reset" );
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_concurrent_requests_with_failover() 
{
  let client = Arc::new( create_test_client( ));

  let config = FailoverConfig {
  endpoints : vec![
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
  ],
  strategy : FailoverStrategy::RoundRobin,
  max_retries : 1,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 5,
  };

  let failover = Arc::new( FailoverManager::new( config ).expect( "Failover creation should succeed" ));

  // Launch 5 concurrent requests
  let mut handles = vec![ ];

  for _ in 0..5
  {
  let client = client.clone( );
  let failover = failover.clone( );

  let handle = tokio::spawn( async move {
      failover.execute_with_failover( |model| {
  let client = client.clone( );
  let messages = create_test_messages( );
  Box::pin( async move {
          client.providers( ).chat_completion(
      &model,
      messages,
      Some( 20 ),
      None,
      None,
          ).await
  } )
      } ).await
  } );

  handles.push( handle );
  }

  // Wait for all requests
  let mut successes = 0;
  for handle in handles
  {
  if let Ok( Ok( _ )) = handle.await
  {
      successes += 1;
  }
  }

  assert!( successes >= 4, "Most concurrent requests should succeed" );

  let health = failover.health_status( ).await;
  let total_requests : u64 = health.iter( ).map( |h| h.requests ).sum( );
  assert_eq!( total_requests, 5, "Should have processed 5 requests" );
}

// ============================================================================
// Edge Cases
// ============================================================================

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_empty_endpoints_list() 
{
  let config = FailoverConfig {
  endpoints : vec![ ],
  strategy : FailoverStrategy::Priority,
  max_retries : 3,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 3,
  };

  let result = FailoverManager::new( config );

  assert!( result.is_err( ), "Should fail to create failover with empty endpoints" );
  match result
  {
  Err( FailoverError::NoEndpoints ) => {}
  _ => panic!( "Expected NoEndpoints error" ),
  }
}

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_single_endpoint_failover() 
{
  let _client = create_test_client( );

  let config = FailoverConfig {
  endpoints : vec![
      "meta-llama/Llama-3.3-70B-Instruct".to_string( ),
  ],
  strategy : FailoverStrategy::Priority,
  max_retries : 2,
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 5,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  // Should work with single endpoint
  let endpoint = failover.select_endpoint( ).await;
  assert!( endpoint.is_ok( ), "Should select single endpoint" );
}

#[ cfg( feature = "integration" ) ]

#[ tokio::test ]
async fn test_max_retries_limits_attempts() 
{
  let client = create_test_client( );

  let config = FailoverConfig {
  endpoints : vec![
      "invalid-model-1".to_string( ),
      "invalid-model-2".to_string( ),
      "invalid-model-3".to_string( ),
  ],
  strategy : FailoverStrategy::Priority,
  max_retries : 1,  // Only 1 retry = 2 total attempts
  failure_window : Duration::from_secs( 300 ),
  failure_threshold : 1,
  };

  let failover = FailoverManager::new( config ).expect( "Failover creation should succeed" );

  let result = failover.execute_with_failover( |model| {
  let client = client.clone( );
  let messages = create_test_messages( );
  Box::pin( async move {
      client.providers( ).chat_completion(
  &model,
  messages,
  Some( 20 ),
  None,
  None,
      ).await
  } )
  } ).await;

  assert!( result.is_err( ), "Should fail after max retries" );

  match result
  {
  Err( FailoverError::AllRetriesFailed { attempts, .. } ) => {
      assert!( attempts <= 2, "Should respect max_retries limit" );
  }
  _ => panic!( "Expected AllRetriesFailed error" ),
  }
}

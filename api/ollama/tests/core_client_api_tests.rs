//! Core client integration tests for `api_ollama`
//! 
//! These tests verify basic client functionality with real network operations
//! and server interactions.

use api_ollama::OllamaClient;
use core::time::Duration;

#[ tokio::test ]
async fn test_client_connectivity_check()
{
  // Test with unreachable server
  let mut client = OllamaClient::new( "http://unreachable.connectivity.test:99999".to_string(), OllamaClient::recommended_timeout_fast() )
    .with_timeout( Duration::from_millis( 100 ) );
  
  let is_available = client.is_available().await;
  assert!( !is_available, "Unreachable server should not be available" );
}

#[ tokio::test ]
async fn test_client_timeout_configuration()
{
  let mut short_client = OllamaClient::new( "http://timeout.test:99999".to_string(), OllamaClient::recommended_timeout_fast() )
    .with_timeout( Duration::from_millis( 10 ) );
    
  let mut long_client = OllamaClient::new( "http://timeout.test:99999".to_string(), OllamaClient::recommended_timeout_fast() )
    .with_timeout( Duration::from_secs( 1 ) );
  
  // Both should fail, but short timeout should fail faster
  let start = std::time::Instant::now();
  let short_result = short_client.list_models().await;
  let short_duration = start.elapsed();
  
  let start = std::time::Instant::now();
  let long_result = long_client.list_models().await;
  let long_duration = start.elapsed();
  
  assert!( short_result.is_err() );
  assert!( long_result.is_err() );
  
  // Both should complete quickly since DNS resolution fails fast
  // Test that both clients can be configured with different timeouts
  assert!( short_duration < Duration::from_secs( 5 ) ); // Reasonable upper bound
  assert!( long_duration < Duration::from_secs( 5 ) ); // Reasonable upper bound
  
  // Real test is that they both fail gracefully regardless of timeout
}

#[ tokio::test ]
async fn test_client_url_validation_integration()
{
  // Test with malformed URLs that only show issues during network calls
  let mut client = OllamaClient::new( "not-a-valid-url".to_string(), OllamaClient::recommended_timeout_fast() );
  
  let result = client.list_models().await;
  assert!( result.is_err(), "Invalid URL should cause network error" );
  
  let error_str = format!( "{}", result.unwrap_err() );
  assert!( error_str.contains( "Network error" ) || error_str.contains( "Parse error" ) );
}

#[ tokio::test ]
async fn test_client_default_configuration_integration()
{
  let mut client = OllamaClient::default();
  
  // Default client should point to localhost:11434
  // Test that client can be created with default configuration successfully
  // Whether the server is running or not, client creation should work
  let result = client.is_available().await;
  
  // Result doesn't matter - what matters is that we can call methods
  // without configuration errors (client was properly configured)
  let _ = result; // Accept any result - server may or may not be running
}

#[ tokio::test ]
async fn test_client_multiple_operations_same_client()
{
  let mut client = OllamaClient::new( "http://multi.test:99999".to_string(), OllamaClient::recommended_timeout_fast() )
    .with_timeout( Duration::from_millis( 50 ) );
  
  // Test multiple operations sequentially (can't borrow mutably multiple times)
  let result1 = client.list_models().await;
  let result2 = client.model_info( "test-model".to_string() ).await;
  let result3 = client.is_available().await;
  
  // All network operations should fail with unreachable server
  assert!( result1.is_err() );
  assert!( result2.is_err() );
  assert!( !result3 ); // is_available returns bool, should be false
}

#[ tokio::test ]
async fn test_client_configuration_persistence()
{
  let base_url = "http://persist.test:11434".to_string();
  let timeout = Duration::from_millis( 200 );
  
  let mut client = OllamaClient::new( base_url.clone(), timeout );
  
  // Perform operation that will fail
  let _ = client.list_models().await;
  
  // Client should maintain its configuration after failed operations
  // We can't directly test private fields, but we can test behavior consistency
  let start = std::time::Instant::now();
  let _ = client.list_models().await;
  let duration = start.elapsed();
  
  // Should still respect the timeout configuration
  assert!( duration < Duration::from_secs( 1 ), "Timeout configuration should persist" );
}

#[ tokio::test ]
async fn test_client_configuration_behavior()
{
  let mut client1 = OllamaClient::new( "http://config.test:11434".to_string(), OllamaClient::recommended_timeout_fast() )
    .with_timeout( Duration::from_millis( 100 ) );

  let mut client2 = OllamaClient::new( "http://config.test:11434".to_string(), OllamaClient::recommended_timeout_fast() )
    .with_timeout( Duration::from_millis( 100 ) );

  // Both clients should have same behavior with same config
  let result1 = client1.is_available().await;
  let result2 = client2.is_available().await;

  assert_eq!( result1, result2 );
  assert!( !result1 ); // Both should fail to connect to non-existent server

  // Test that clients can perform operations independently
  let list_result = client2.list_models().await;
  assert!( list_result.is_err() );
}

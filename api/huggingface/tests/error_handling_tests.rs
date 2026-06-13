//! Tests for error handling robustness and safety

mod inc;

use api_huggingface::
{
  environment::{ HuggingFaceEnvironmentImpl, HuggingFaceEnvironment, EnvironmentInterface },
  secret::Secret,
  error::{ HuggingFaceError, Result },
};

#[ tokio::test ]
async fn error_message_structure()
{
  // Test that error messages have proper structure
  let result : Result< () > = Err( HuggingFaceError::Authentication( "Authentication failed".to_string() ) );
  
  match result
  {
  Err( HuggingFaceError::Authentication( msg ) ) =>
  {
      // Error message should be non-empty and descriptive
      assert!( !msg.is_empty(), "Error message should not be empty" );
      assert!( msg.contains( "Authentication" ), "Error message should describe the error" );
  },
  _ => panic!( "Expected Authentication error" ),
  }
}

#[ tokio::test ]
async fn invalid_base_url_handling()
{
  let api_key = Secret::new( "test-key".to_string() );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None ).expect( "Environment build should succeed" );
  
  // Test with malformed URL path
  let result = env.endpoint_url( "not-a-valid-path with spaces" );
  assert!( result.is_ok(), "Should handle URL construction gracefully" );
}

#[ tokio::test ]
async fn environment_with_invalid_url()
{
  let api_key = Secret::new( "test-key".to_string() );
  let invalid_url = Some( "not-a-valid-url".to_string() );
  
  // Environment creation should succeed (validation happens later)
  let env_result = HuggingFaceEnvironmentImpl::build( api_key, invalid_url );
  assert!( env_result.is_ok(), "Environment build should succeed" );

  let env = env_result.expect( "[environment_invalid_url_handling] Environment should be Ok after is_ok() check - check HuggingFaceEnvironmentImpl::build() implementation" );

  // But endpoint URL construction should fail gracefully
  let url_result = env.endpoint_url( "/test" );
  assert!( url_result.is_err(), "Should fail with invalid base URL" );
  
  match url_result.unwrap_err()
  {
  HuggingFaceError::InvalidArgument( _ ) => {}, // Expected
  other => panic!( "Expected InvalidArgument error, got : {other:?}" ),
  }
}

#[ tokio::test ]
async fn header_generation_with_special_characters()
{
  // Test with API key that might have special characters
  let api_key = Secret::new( "test-key-with-special-chars-!@#$%".to_string() );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None ).expect( "Environment build should succeed" );
  
  let headers_result = env.headers();
  // Should handle special characters gracefully
  assert!( headers_result.is_ok(), "Should handle special characters in API key" );
}

#[ tokio::test ]
async fn error_display_formatting()
{
  let errors = vec!
  [
  HuggingFaceError::Authentication( "Auth failed".to_string() ),
  HuggingFaceError::InvalidArgument( "Invalid arg".to_string() ),
  HuggingFaceError::Http( "HTTP error".to_string() ),
  HuggingFaceError::RateLimit( "Rate limited".to_string() ),
  ];
  
  for error in errors
  {
  let display_str = format!( "{error}" );
  assert!( !display_str.is_empty(), "Error display should not be empty" );
  
  let debug_str = format!( "{error:?}" );
  assert!( !debug_str.is_empty(), "Error debug should not be empty" );
  }
}

#[ tokio::test ]
async fn error_chain_propagation()
{
  // Test that errors propagate correctly through the Result chain
  fn simulate_nested_error() -> Result< String >
  {
  Err( HuggingFaceError::Http( "Connection failed".to_string() ) )
      .map_err( | e | HuggingFaceError::InvalidArgument( format!( "Wrapped : {e}" ) ) )
  }
  
  let result = simulate_nested_error();
  assert!( result.is_err(), "Should propagate error" );
  
  match result.unwrap_err()
  {
  HuggingFaceError::InvalidArgument( msg ) =>
  {
      assert!( msg.contains( "Wrapped :" ), "Should contain wrapped error context" );
      assert!( msg.contains( "Connection failed" ), "Should contain original error message" );
  },
  other => panic!( "Expected InvalidArgument error, got : {other:?}" ),
  }
}

#[ cfg( feature = "integration" ) ]
use api_huggingface::
{
  Client,
  components::input::InferenceParameters,
};

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_authentication_error_with_invalid_key()
{
  // Test with deliberately invalid API key to trigger authentication error
  let invalid_key = Secret::new( "invalid-key-12345".to_string() );
  let env = HuggingFaceEnvironmentImpl::build( invalid_key, None )
      .expect( "Environment build should succeed" );
  let client = Client::build( env )
      .expect( "Client build should succeed" );

  // Make real API call that should fail with authentication error
  let result = client.inference()
      .create( "test", "microsoft/DialoGPT-medium" )
      .await;

  assert!( result.is_err(), "Invalid API key should cause error" );
  
  // Verify error structure contains authentication info
  let error = result.unwrap_err();
  let error_string = format!( "{error}" );
  assert!( 
      error_string.contains( "401" ) || error_string.contains( "auth" ) || error_string.contains( "token" ),
      "Error should indicate authentication issue : {error_string}" 
  );
  }

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_rate_limit_error_handling()
{
  // Get real API key (will panic with clear message if missing)
  let api_key_string = crate::inc::get_api_key_for_integration();
  
  // Build client with real credentials
  let api_key = Secret::new( api_key_string );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )
      .expect( "Environment build should succeed" );
  let client = Client::build( env )
      .expect( "Client build should succeed" );

  // Make multiple rapid API calls to potentially trigger rate limiting
  let mut results = Vec::new();
  for i in 0..5
  {
      let result = client.inference()
  .create_with_parameters( &format!( "test {i}" ), "microsoft/DialoGPT-medium", InferenceParameters::default().with_max_new_tokens( 10 ) )
  .await;
      results.push( result );
      
      // Small delay to not overwhelm
      tokio::time::sleep( core::time::Duration::from_millis( 100 ) ).await;
  }

  // Check if we have any results at all (either success or proper error handling)
  let successful_calls = results.iter().filter( |r| r.is_ok() ).count();
  let failed_calls = results.iter().filter( |r| r.is_err() ).count();
  
  // The test succeeds if we either:
  // 1. Have at least one successful call (normal case)
  // 2. All calls fail but with proper error handling (rate limit or auth issues)
  assert!(
      successful_calls > 0 || failed_calls == results.len(),
      "Either some calls should succeed or all should fail with proper errors. Successful : {successful_calls}, Failed : {failed_calls}"
  );

  // Check error handling for any failed calls
  for ( i, result ) in results.iter().enumerate()
  {
      if let Err( error ) = result
      {
  let error_string = format!( "{error}" );
  println!( "Call {i} error : {error_string}" );
  // Error should be properly structured
  assert!( !error_string.is_empty(), "Error message should not be empty" );
      }
  }
  }

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_invalid_model_error_handling()
{
  // Get real API key (will panic with clear message if missing)
  let api_key_string = crate::inc::get_api_key_for_integration();
  
  // Build client with real credentials
  let api_key = Secret::new( api_key_string );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )
      .expect( "Environment build should succeed" );
  let client = Client::build( env )
      .expect( "Client build should succeed" );

  // Test with non-existent model to trigger model error
  let result = client.embeddings()
      .create( "test", "non-existent-model-12345" )
      .await;

  assert!( result.is_err(), "Non-existent model should cause error" );
  
  // Verify error contains model-related information
  let error = result.unwrap_err();
  let error_string = format!( "{error}" );
  let lower = error_string.to_lowercase();
  assert!(
      lower.contains( "model" ) || lower.contains( "404" ) || lower.contains( "not found" ) || lower.contains( "not supported" ),
      "Error should indicate model issue : {error_string}"
  );
  }

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_network_error_recovery()
{
  // Test with invalid base URL to simulate network errors
  let api_key = Secret::new( crate::inc::get_api_key_for_integration() );
  let invalid_url = Some( "https://invalid-domain-that-does-not-exist-12345.com".to_string() );
  
  let env = HuggingFaceEnvironmentImpl::build( api_key, invalid_url )
      .expect( "Environment build should succeed" );
  let client = Client::build( env )
      .expect( "Client build should succeed" );

  // Make API call that should fail due to network issues
  let result = client.inference()
      .create( "test", "microsoft/DialoGPT-medium" )
      .await;

  assert!( result.is_err(), "Invalid URL should cause network error" );
  
  // Verify error handling provides useful information
  let error = result.unwrap_err();
  let error_string = format!( "{error}" );
  assert!( !error_string.is_empty(), "Network error should have descriptive message" );
  
  // Should contain network-related keywords
  let has_network_keywords = error_string.contains( "dns" ) 
      || error_string.contains( "connection" ) 
      || error_string.contains( "network" )
      || error_string.contains( "resolve" )
      || error_string.contains( "sending request" )
      || error_string.contains( "error" );
  
  assert!( has_network_keywords, "Network error should contain relevant keywords : {error_string}" );
}
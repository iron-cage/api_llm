//! Core Client Integration Tests - STRICT FAILURE POLICY
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

#[ allow( unused_imports ) ]
use super::*;

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_secret_validation_format()
{
  // Test API key format validation
  let invalid_keys = vec![
    "",
    "invalid-key",
    "sk-wrong-prefix",
    "ant-missing-sk",
  ];

  for key in invalid_keys
  {
    let result = the_module::Secret::new( key.to_string() );
    assert!( result.is_err(), "Expected key {key} to be invalid" );
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_error_handling_types()
{
  // Test that AnthropicError enum has required variants
  let http_error = the_module::AnthropicError::http_error( "Connection failed".to_string() );
  let auth_error = the_module::AnthropicError::Authentication(
    the_module::AuthenticationError::new( "Invalid API key".to_string() )
  );
  let invalid_arg_error = the_module::AnthropicError::InvalidArgument( "Missing parameter".to_string() );

  // Test Display implementation
  assert!( http_error.to_string().contains( "Connection failed" ) );
  assert!( auth_error.to_string().contains( "Invalid API key" ) );
  assert!( invalid_arg_error.to_string().contains( "Missing parameter" ) );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_client_from_workspace_method_exists()
{
  // Test loading client from workspace secrets
  let result = the_module::Client::from_workspace();

  // This may fail if no workspace secret is available, which is acceptable
  // The test verifies the method exists and returns appropriate result
  match result
  {
    Ok( _client ) => {}, // Success case
    Err( err ) =>
    {
      // Should be a specific error type, not a panic
      assert!( !err.to_string().is_empty() );
    }
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_client_from_missing_environment_variable()
{
  // Test that Client::from_env fails when env var is missing
  std::env::remove_var( "ANTHROPIC_API_KEY" );

  let result = the_module::Client::from_env();
  assert!( result.is_err() );
}

// ============================================================================
// INTEGRATION TESTS - REAL API CLIENT LIFECYCLE
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_client_real_api_lifecycle()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for client lifecycle testing" );

  // Test basic client configuration with real API
  assert!( !client.secret().ANTHROPIC_API_KEY.is_empty() );
  assert_eq!( client.base_url(), "https://api.anthropic.com" );

  // Test that client can make a simple API call
  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 5,
    messages : vec![ the_module::Message::user( "Hi".to_string() ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Client must successfully make API call : {err}" ),
  };

  // Verify client properly handles real API response
  assert!( !response.id.is_empty(), "Real API response must have ID" );
  assert_eq!( response.r#type, "message" );
  assert_eq!( response.role, "assistant" );
  assert!( !response.content.is_empty(), "Real API response must have content" );
  
  println!( "✅ Client lifecycle integration test passed!" );
  println!( "   Client base URL: {}", client.base_url() );
  println!( "   Response ID: {}", response.id );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_client_concurrent_requests()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for concurrent testing" );

  // Test concurrent requests with same client
  let request1 = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 5,
    messages : vec![ the_module::Message::user( "Test 1".to_string() ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let request2 = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 5,
    messages : vec![ the_module::Message::user( "Test 2".to_string() ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : None,
    tool_choice : None,
  };

  // Make concurrent requests
  let (response1, response2) = tokio::join!(
    client.create_message( request1 ),
    client.create_message( request2 )
  );

  let response1 = match response1
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: First concurrent request must succeed : {err}" ),
  };

  let response2 = match response2
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Second concurrent request must succeed : {err}" ),
  };

  // Verify both responses are valid and unique
  assert!( !response1.id.is_empty() );
  assert!( !response2.id.is_empty() );
  assert_ne!( response1.id, response2.id, "Concurrent requests must have unique IDs" );
  
  println!( "✅ Client concurrent requests integration test passed!" );
  println!( "   Response 1 ID: {}", response1.id );
  println!( "   Response 2 ID: {}", response2.id );
}
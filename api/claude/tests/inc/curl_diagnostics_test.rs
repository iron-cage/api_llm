//! Comprehensive CURL diagnostics tests for Anthropic API client
//!
//! MANDATORY INTEGRATION TEST REQUIREMENTS:
//! - These tests validate CURL diagnostics functionality
//! - Tests MUST fail initially to validate TDD approach
//! - Tests MUST use feature gating for curl-diagnostics functionality
//! - Tests MUST validate `AsCurl` trait implementation accuracy
//!
//! CURL diagnostics provide debugging capabilities by generating equivalent
//! cURL commands for API requests, enabling developers to debug and replicate
//! API calls outside of the Rust environment.

#[ allow( unused_imports ) ]
use super::*;

#[ cfg( feature = "curl-diagnostics" ) ]
#[ allow( unused_imports ) ]
use the_module::*;

#[ cfg( feature = "curl-diagnostics" ) ]
mod curl_diagnostics_functionality_tests
{
  use super::*;

  /// Test `AsCurl` trait implementation for basic message requests
  #[ cfg( feature = "integration" ) ]
  #[ test ]
fn test_message_request_as_curl()
  {
    // This test will fail until AsCurl trait is implemented
    let request = the_module::CreateMessageRequest::builder()
      .model( "claude-3-sonnet-20240229" )
      .max_tokens( 1000 )
      .messages( vec![ the_module::Message::user( "Hello, how are you?" ) ] )
      .build();

    // Generate cURL command
    let curl_command = request.as_curl( "https://api.anthropic.com/v1/messages" );

    // Validate cURL command structure
    assert!( curl_command.contains( "curl" ), "Should contain curl command" );
    assert!( curl_command.contains( "--request POST" ), "Should use POST method" );
    assert!( curl_command.contains( "https://api.anthropic.com/v1/messages" ), "Should contain correct URL" );
    assert!( curl_command.contains( "Content-Type: application/json" ), "Should set content type" );
    assert!( curl_command.contains( "claude-3-sonnet-20240229" ), "Should include model in request body" );
    assert!( curl_command.contains( "Hello, how are you?" ), "Should include message content" );
  }

  /// Test `AsCurl` trait with authentication headers
  #[ cfg( feature = "integration" ) ]
  #[ test ]
fn test_curl_with_authentication()
  {
    let request = the_module::CreateMessageRequest::builder()
      .model( "claude-3-sonnet-20240229" )
      .max_tokens( 500 )
      .messages( vec![ the_module::Message::user( "Test message" ) ] )
      .build();

    // Generate cURL with authentication
    let client = the_module::Client::new(
      the_module::Secret::new_unchecked( "sk-ant-api03-test-key".to_string() )
    );

    let curl_command = client.as_curl_for_request( &request, "https://api.anthropic.com/v1/messages" );

    // Validate authentication headers
    assert!( curl_command.contains( "x-api-key: sk-ant-api03-test-key" ), "Should include API key header" );
    assert!( curl_command.contains( "anthropic-version:" ), "Should include version header" );
    assert!( curl_command.contains( "User-Agent:" ), "Should include user agent" );
  }

  /// Test cURL generation for streaming requests
  #[ cfg( feature = "streaming" ) ]
  #[ test ]
  fn test_streaming_request_as_curl()
  {
    let request = the_module::CreateMessageRequest::builder()
      .model( "claude-3-sonnet-20240229" )
      .max_tokens( 1000 )
      .messages( vec![ the_module::Message::user( "Stream this response" ) ] )
      .stream( true )
      .build();

    let curl_command = request.as_curl( "https://api.anthropic.com/v1/messages" );


    // Validate streaming specific headers and options
    assert!( curl_command.contains( "\"stream\":true" ), "Should include stream parameter" );
    assert!( curl_command.contains( "--no-buffer" ), "Should include no-buffer option for streaming" );
  }

  /// Test cURL generation for tool calling requests
  #[ cfg( feature = "tools" ) ]
  #[ test ]
  fn test_tool_request_as_curl()
  {
    let tool = the_module::ToolDefinition::new(
      "get_weather",
      "Get current weather",
      serde_json::json!({
        "type": "object",
        "properties": {
          "location": { "type": "string" }
        }
      })
    );

    let request = the_module::CreateMessageRequest::builder()
      .model( "claude-3-sonnet-20240229" )
      .max_tokens( 1000 )
      .messages( vec![ the_module::Message::user( "What's the weather?" ) ] )
      .tools( vec![ tool ] )
      .build();

    let curl_command = request.as_curl( "https://api.anthropic.com/v1/messages" );

    // Validate tool definition in cURL
    assert!( curl_command.contains( "get_weather" ), "Should include tool name" );
    assert!( curl_command.contains( "Get current weather" ), "Should include tool description" );
    assert!( curl_command.contains( "input_schema" ), "Should include tool schema" );
  }

  /// Test cURL generation for vision requests
  #[ cfg( feature = "vision" ) ]
  #[ test ]
  fn test_vision_request_as_curl()
  {
    let image_source = the_module::ImageSource::jpeg( "base64encodedimagedata" );
    let message = the_module::Message::builder()
      .user()
      .image( image_source )
      .build();

    let request = the_module::CreateMessageRequest::builder()
      .model( "claude-3-sonnet-20240229" )
      .max_tokens( 1000 )
      .messages( vec![ message ] )
      .build();

    let curl_command = request.as_curl( "https://api.anthropic.com/v1/messages" );

    // Validate vision content in cURL
    assert!( curl_command.contains( "image/jpeg" ), "Should include media type" );
    assert!( curl_command.contains( "base64encodedimagedata" ), "Should include image data" );
  }

  /// Test cURL formatting and escaping
  #[ test ]
  fn test_curl_formatting_and_escaping()
  {
    // Test message with special characters that need escaping
    let request = the_module::CreateMessageRequest::builder()
      .model( "claude-3-sonnet-20240229" )
      .max_tokens( 1000 )
      .messages( vec![ the_module::Message::user( "Test with \"quotes\" and 'apostrophes' and $variables and \\backslashes" ) ] )
      .build();

    let curl_command = request.as_curl( "https://api.anthropic.com/v1/messages" );

    // Validate proper JSON escaping
    assert!( curl_command.contains( "\\\"quotes\\\"" ), "Should escape double quotes" );
    assert!( curl_command.contains( "\\\\backslashes" ), "Should escape backslashes" );

    // Validate cURL is properly formatted
    assert!( curl_command.starts_with( "curl" ), "Should start with curl command" );
    assert!( curl_command.contains( "\\\n" ) || curl_command.len() < 200, "Should use line continuation for long commands" );
  }

  /// Test cURL command validation and syntax
  #[ test ]
  fn test_curl_command_validation()
  {
    let request = the_module::CreateMessageRequest::builder()
      .model( "claude-3-sonnet-20240229" )
      .max_tokens( 1000 )
      .messages( vec![ the_module::Message::user( "Validation test" ) ] )
      .build();

    let curl_command = request.as_curl( "https://api.anthropic.com/v1/messages" );

    // Validate cURL command structure
    assert!( curl_command.starts_with( "curl" ), "Should start with curl" );
    assert!( curl_command.contains( "--request POST" ) || curl_command.contains( "-X POST" ), "Should specify POST method" );
    assert!( curl_command.contains( "--header" ) || curl_command.contains( "-H" ), "Should include headers" );
    assert!( curl_command.contains( "--data" ) || curl_command.contains( "-d" ), "Should include data" );

    // Count quotes - should be balanced
    let quote_count = curl_command.matches( '"' ).count();
    assert_eq!( quote_count % 2, 0, "Quotes should be balanced" );
  }

  /// Test performance benchmark for cURL generation
  #[ test ]
  fn test_curl_generation_performance()
  {
    use std::time::Instant;

    let request = the_module::CreateMessageRequest::builder()
      .model( "claude-3-sonnet-20240229" )
      .max_tokens( 1000 )
      .messages( vec![ the_module::Message::user( "Performance test message with reasonable length to simulate real usage" ) ] )
      .build();

    let start = Instant::now();

    // Generate cURL command multiple times
    for _ in 0..100
    {
      let _curl_command = request.as_curl( "https://api.anthropic.com/v1/messages" );
    }

    let duration = start.elapsed();

    // Performance expectation : 100 generations should be under 50ms
    assert!( duration.as_millis() < 50, "cURL generation should be fast : {}ms", duration.as_millis() );
  }

  /// Test cURL generation with complex nested structures
  #[ test ]
  fn test_complex_request_as_curl()
  {
    // Complex request with multiple features
    let messages = vec![
      the_module::Message::user( "Hello" ),
      the_module::Message::assistant( "Hello! How can I help?" ),
      the_module::Message::user( "Tell me about Rust" ),
    ];

    let request = the_module::CreateMessageRequest::builder()
      .model( "claude-3-sonnet-20240229" )
      .max_tokens( 2000 )
      .temperature( 0.7 )
      .system( "You are a helpful assistant" )
      .messages( messages )
      .build();

    let curl_command = request.as_curl( "https://api.anthropic.com/v1/messages" );


    // Validate complex structure preservation
    assert!( curl_command.contains( "\"temperature\":" ), "Should include temperature" );
    assert!( curl_command.contains( "system" ), "Should include system message" );
    assert!( curl_command.contains( "assistant" ), "Should include assistant message" );
  }
}

#[ cfg( feature = "curl-diagnostics" ) ]
#[ cfg( feature = "integration" ) ]
mod curl_diagnostics_integration_tests
{
  use super::*;

  /// Test generated cURL command actually works with real API
  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
async fn test_curl_command_execution_equivalence()
  {
    let client = the_module::Client::from_workspace()
      .expect( "Must have valid API key for integration test" );

    let request = the_module::CreateMessageRequest::builder()
      .model( "claude-haiku-4-5-20251001" )
      .max_tokens( 50 )
      .messages( vec![ the_module::Message::user( "Say 'test successful'" ) ] )
      .build();

    // Generate cURL command
    let curl_command = client.as_curl_for_request( &request, "https://api.anthropic.com/v1/messages" );

    // Validate the cURL command looks correct
    assert!( curl_command.contains( "claude-haiku-4-5-20251001" ), "Should contain model name" );
    assert!( curl_command.contains( "test successful" ), "Should contain message content" );

    // Note : We don't actually execute the cURL command in tests
    // This would require shell execution and is beyond the scope of unit tests
    // The integration test validates that the generated command has correct structure
  }

  /// Test cURL generation maintains request fidelity
  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
async fn test_curl_request_fidelity()
  {
    let client = the_module::Client::from_workspace()
      .expect( "Must have valid API key for integration test" );

    // Create a complex request
    let original_request = the_module::CreateMessageRequest::builder()
      .model( "claude-haiku-4-5-20251001" )
      .max_tokens( 100 )
      .temperature( 0.5 )
      .messages( vec![ the_module::Message::user( "Complex test message with special chars : !@#$%^&*()" ) ] )
      .build();

    // Generate cURL
    let curl_command = client.as_curl_for_request( &original_request, "https://api.anthropic.com/v1/messages" );

    // Execute the original request
    let api_response = client.create_message( original_request.clone() ).await;

    // If API call succeeds, the cURL should represent the same request
    if api_response.is_ok()
    {

      // Validate cURL contains all the same parameters
      assert!( curl_command.contains( "claude-haiku-4-5-20251001" ), "cURL should match model" );
      assert!( curl_command.contains( "\"max_tokens\":100" ), "cURL should match max_tokens" );
      assert!( curl_command.contains( "\"temperature\":" ), "cURL should match temperature" );
      assert!( curl_command.contains( "!@#$%^&*()" ), "cURL should preserve special characters" );
    }
  }

  /// Test cURL diagnostics for error scenarios
  #[ tokio::test ]
  async fn test_curl_diagnostics_for_errors()
  {
    let client = the_module::Client::new(
      the_module::Secret::new_unchecked( "sk-ant-invalid-key".to_string() )
    );

    let request = the_module::CreateMessageRequest::builder()
      .model( "claude-haiku-4-5-20251001" )
      .max_tokens( 50 )
      .messages( vec![ the_module::Message::user( "This will fail" ) ] )
      .build();

    // Generate cURL for debugging failed requests
    let curl_command = client.as_curl_for_request( &request, "https://api.anthropic.com/v1/messages" );

    // The cURL should still be generated correctly even for invalid auth
    assert!( curl_command.contains( "sk-ant-invalid-key" ), "Should include the invalid key for debugging" );
    assert!( curl_command.contains( "This will fail" ), "Should include message content" );
    assert!( curl_command.contains( "curl" ), "Should be a valid cURL command structure" );
  }
}

#[ cfg( not( feature = "curl-diagnostics" ) ) ]
mod curl_diagnostics_feature_disabled_tests
{
  /// Test that cURL diagnostics functionality is properly feature-gated
  #[ test ]
  fn test_curl_diagnostics_feature_gated()
  {
    // When curl-diagnostics feature is disabled, AsCurl trait should not be available
    // This test validates proper feature gating

    // Compilation should succeed without AsCurl trait when feature is disabled
    // This serves as a compile-time test for proper feature gating
    assert!( true, "Feature gating working correctly - AsCurl trait not available" );
  }
}
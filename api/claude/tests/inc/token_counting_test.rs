//! Token Counting API Tests - STRICT FAILURE POLICY
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

// ===== Unit Tests : Token Counting Request Structure =====

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_token_count_request_basic()
{
  // Test basic token counting request construction
  let messages = vec![
    the_module::Message::user( "What is the capital of France?".to_string() )
  ];

  let request = the_module::CountMessageTokensRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    messages,
    system : None,
    tools : None,
  };

  assert_eq!( request.model, "claude-sonnet-4-5-20250929" );
  assert_eq!( request.messages.len(), 1 );
  assert!( request.system.is_none() );
  assert!( request.tools.is_none() );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_token_count_request_with_system_prompt()
{
  // Test token counting request with system prompt
  let messages = vec![
    the_module::Message::user( "Explain quantum physics".to_string() )
  ];

  let request = the_module::CountMessageTokensRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    messages,
    system : Some( vec![ the_module::SystemContent::text( "You are a physics professor." ) ] ),
    tools : None,
  };

  assert_eq!( request.model, "claude-sonnet-4-5-20250929" );
  assert_eq!( request.messages.len(), 1 );
  assert!( request.system.is_some() );
}

#[ cfg( feature = "tools" ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_token_count_request_with_tools()
{
  // Test token counting request with tool definitions
  let messages = vec![
    the_module::Message::user( "What's the weather?".to_string() )
  ];

  let tool = the_module::ToolDefinition
  {
    name : "get_weather".to_string(),
    description : "Get current weather".to_string(),
    input_schema : serde_json::json!(
    {
      "type": "object",
      "properties":
      {
        "location": { "type": "string" }
      }
    }),
  };

  let request = the_module::CountMessageTokensRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    messages,
    system : None,
    tools : Some( vec![ tool ] ),
  };

  assert_eq!( request.messages.len(), 1 );
  assert!( request.tools.is_some() );
  assert_eq!( request.tools.as_ref().unwrap().len(), 1 );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_token_count_request_conversation()
{
  // Test token counting for multi-turn conversation
  let messages = vec![
    the_module::Message::user( "Hello!".to_string() ),
    the_module::Message::assistant( "Hi! How can I help?".to_string() ),
    the_module::Message::user( "Tell me a joke".to_string() ),
  ];

  let request = the_module::CountMessageTokensRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    messages : messages.clone(),
    system : None,
    tools : None,
  };

  assert_eq!( request.messages.len(), 3 );
}

#[ cfg( feature = "vision" ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_token_count_request_with_image()
{
  // Test token counting for multimodal content (text + image)
  use the_module::{ ImageSource, Content };

  let text_content = Content::Text
  {
    r#type : "text".to_string(),
    text : "What's in this image?".to_string(),
  };

  let image_content = Content::Image
  {
    r#type : "image".to_string(),
    source : ImageSource::png( "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==" ),
  };

  let message = the_module::Message
  {
    role : the_module::Role::User,
    content : vec![ text_content, image_content ],
    cache_control : None,
  };

  let request = the_module::CountMessageTokensRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    messages : vec![ message ],
    system : None,
    tools : None,
  };

  assert_eq!( request.messages.len(), 1 );
  assert_eq!( request.messages[ 0 ].content.len(), 2 );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_token_count_response_structure()
{
  // Test response structure has expected fields
  let response = the_module::CountMessageTokensResponse
  {
    input_tokens : 42,
  };

  assert_eq!( response.input_tokens, 42 );
  assert!( response.input_tokens > 0 );
}

// ===== Integration Tests : Real API Calls =====

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_token_count_basic_message()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: NO GRACEFUL FALLBACKS
  // This test MUST fail if secrets are unavailable or API is unreachable

  let secret = the_module::Secret::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for token counting test. Set ANTHROPIC_API_KEY or configure workspace secrets." );

  let client = the_module::Client::new( secret );

  let messages = vec![
    the_module::Message::user( "Hello, Claude!".to_string() )
  ];

  let request = the_module::CountMessageTokensRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    messages,
    system : None,
    tools : None,
  };

  // MANDATORY: This MUST fail if API is unreachable or authentication fails
  let response = client.count_message_tokens( request ).await
    .expect( "INTEGRATION: Token counting must succeed with valid credentials and connectivity" );

  // Verify response structure
  assert!( response.input_tokens > 0, "Token count must be greater than zero for non-empty message" );

  // Basic sanity check - "Hello, Claude!" should be relatively few tokens
  assert!( response.input_tokens < 50, "Simple greeting should be less than 50 tokens" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_token_count_with_system_prompt()
{
  let secret = the_module::Secret::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let client = the_module::Client::new( secret );

  let messages = vec![
    the_module::Message::user( "Explain photosynthesis".to_string() )
  ];

  let request = the_module::CountMessageTokensRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    messages,
    system : Some( vec![ the_module::SystemContent::text( "You are a biology teacher. Explain complex topics simply for high school students." ) ] ),
    tools : None,
  };

  let response = client.count_message_tokens( request ).await
    .expect( "INTEGRATION: Token counting with system prompt must succeed" );

  assert!( response.input_tokens > 0 );

  // System prompt should add to token count
  assert!( response.input_tokens > 10, "Message with system prompt should have substantial token count" );
}

#[ cfg( all( feature = "integration", feature = "tools" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_token_count_with_tools()
{
  let secret = the_module::Secret::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let client = the_module::Client::new( secret );

  let tool = the_module::ToolDefinition
  {
    name : "calculator".to_string(),
    description : "Perform mathematical calculations".to_string(),
    input_schema : serde_json::json!(
    {
      "type": "object",
      "properties":
      {
        "operation": { "type": "string" },
        "a": { "type": "number" },
        "b": { "type": "number" }
      },
      "required": ["operation", "a", "b"]
    }),
  };

  let messages = vec![
    the_module::Message::user( "What is 15 multiplied by 23?".to_string() )
  ];

  let request = the_module::CountMessageTokensRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    messages,
    system : None,
    tools : Some( vec![ tool ] ),
  };

  let response = client.count_message_tokens( request ).await
    .expect( "INTEGRATION: Token counting with tools must succeed" );

  assert!( response.input_tokens > 0 );

  // Tool definitions add significant tokens
  assert!( response.input_tokens > 20, "Message with tool definitions should have substantial token count" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_token_count_conversation()
{
  let secret = the_module::Secret::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let client = the_module::Client::new( secret );

  let messages = vec![
    the_module::Message::user( "Hello!".to_string() ),
    the_module::Message::assistant( "Hi! How can I help you today?".to_string() ),
    the_module::Message::user( "Tell me about the solar system".to_string() ),
    the_module::Message::assistant( "The solar system consists of the Sun and all objects that orbit it.".to_string() ),
    the_module::Message::user( "How many planets are there?".to_string() ),
  ];

  let request = the_module::CountMessageTokensRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    messages,
    system : None,
    tools : None,
  };

  let response = client.count_message_tokens( request ).await
    .expect( "INTEGRATION: Multi-turn conversation token counting must succeed" );

  assert!( response.input_tokens > 0 );

  // Multi-turn conversation should have substantial token count
  assert!( response.input_tokens > 30, "Multi-turn conversation should have significant tokens" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_token_count_different_models()
{
  let secret = the_module::Secret::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let client = the_module::Client::new( secret );

  let messages = vec![
    the_module::Message::user( "Test message for token counting".to_string() )
  ];

  // Test with Sonnet model
  let request_sonnet = the_module::CountMessageTokensRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    messages : messages.clone(),
    system : None,
    tools : None,
  };

  let response_sonnet = client.count_message_tokens( request_sonnet ).await
    .expect( "INTEGRATION: Token counting for Sonnet must succeed" );

  assert!( response_sonnet.input_tokens > 0 );

  // Test with Haiku model
  let request_haiku = the_module::CountMessageTokensRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    messages : messages.clone(),
    system : None,
    tools : None,
  };

  let response_haiku = client.count_message_tokens( request_haiku ).await
    .expect( "INTEGRATION: Token counting for Haiku must succeed" );

  assert!( response_haiku.input_tokens > 0 );

  // Token counts should be consistent across models for same input
  assert_eq!( response_sonnet.input_tokens, response_haiku.input_tokens,
    "Token count should be consistent across models for identical input" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_token_count_error_invalid_model()
{
  let secret = the_module::Secret::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let client = the_module::Client::new( secret );

  let messages = vec![
    the_module::Message::user( "Test".to_string() )
  ];

  let request = the_module::CountMessageTokensRequest
  {
    model : "invalid-model-name-xyz-123".to_string(),
    messages,
    system : None,
    tools : None,
  };

  // Invalid model should produce an error
  let result = client.count_message_tokens( request ).await;

  assert!( result.is_err(), "Token counting with invalid model must fail" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_token_count_empty_messages()
{
  let secret = the_module::Secret::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let client = the_module::Client::new( secret );

  let request = the_module::CountMessageTokensRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    messages : vec![],
    system : None,
    tools : None,
  };

  // Empty messages should produce an error
  let result = client.count_message_tokens( request ).await;

  assert!( result.is_err(), "Token counting with empty messages must fail with validation error" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_token_count_authentication_required()
{
  // Test that authentication is properly enforced
  let invalid_secret = the_module::Secret::new( "sk-ant-invalid-key-for-testing-12345".to_string() )
    .expect( "Secret creation must succeed even with invalid key" );

  let client = the_module::Client::new( invalid_secret );

  let messages = vec![
    the_module::Message::user( "Test".to_string() )
  ];

  let request = the_module::CountMessageTokensRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    messages,
    system : None,
    tools : None,
  };

  // Invalid API key should produce authentication error
  let result = client.count_message_tokens( request ).await;

  assert!( result.is_err(), "Token counting with invalid API key must fail with authentication error" );
}

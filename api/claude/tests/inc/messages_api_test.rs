//! Messages API Integration Tests - STRICT FAILURE POLICY
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

#[ tokio::test ]
async fn test_message_construction_user()
{
  // Test creating user messages
  let message = the_module::Message::user( "Hello, Claude!".to_string() );
  
  match message.role
  {
    the_module::Role::User => {},
    _ => panic!( "Expected User role" ),
  }
  
  assert_eq!( message.content.len(), 1 );
  assert_eq!( message.content[0].r#type(), "text" );
  assert_eq!( message.content[0].text().unwrap(), "Hello, Claude!" );
}

#[ tokio::test ]
async fn test_message_construction_assistant()
{
  // Test creating assistant messages
  let message = the_module::Message::assistant( "Hello! How can I help?".to_string() );
  
  match message.role
  {
    the_module::Role::Assistant => {},
    _ => panic!( "Expected Assistant role" ),
  }
  
  assert_eq!( message.content.len(), 1 );
  assert_eq!( message.content[0].r#type(), "text" );
  assert_eq!( message.content[0].text().unwrap(), "Hello! How can I help?" );
}

#[ tokio::test ]
async fn test_create_message_request_basic()
{
  // Test basic message request construction
  let messages = vec![
    the_module::Message::user( "What is the capital of France?".to_string() )
  ];
  
  let request = the_module::CreateMessageRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    max_tokens : 100,
    messages,
    system : None,
    temperature : None,
    stream : None,
    tools : None,
    tool_choice : None,
  };
  
  assert_eq!( request.model, "claude-sonnet-4-5-20250929" );
  assert_eq!( request.max_tokens, 100 );
  assert_eq!( request.messages.len(), 1 );
  assert!( request.system.is_none() );
  assert!( request.temperature.is_none() );
  assert!( request.stream.is_none() );
}

#[ tokio::test ]
async fn test_create_message_request_with_system_prompt()
{
  // Test message request with system prompt
  let messages = vec![
    the_module::Message::user( "Explain quantum physics".to_string() )
  ];
  
  let request = the_module::CreateMessageRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    max_tokens : 500,
    messages,
    system : Some( vec![ the_module::SystemContent::text( "You are a physics professor. Explain complex topics simply." ) ] ),
    temperature : Some( 0.7 ),
    stream : Some( false ),
    tools : None,
    tool_choice : None,
  };
  
  assert_eq!( request.model, "claude-sonnet-4-5-20250929" );
  assert_eq!( request.max_tokens, 500 );
  assert_eq!( request.messages.len(), 1 );
  assert!( request.system.is_some() );
  assert_eq!( request.system.as_ref().unwrap()[ 0 ].text, "You are a physics professor. Explain complex topics simply." );
  assert!( (request.temperature.unwrap() - 0.7).abs() < f32::EPSILON );
  assert!( !request.stream.unwrap() );
}

#[ tokio::test ]
async fn test_create_message_request_conversation()
{
  // Test multi-turn conversation
  let messages = vec![
    the_module::Message::user( "Hello!".to_string() ),
    the_module::Message::assistant( "Hi! How can I help you today?".to_string() ),
    the_module::Message::user( "Can you explain machine learning?".to_string() ),
  ];
  
  let request = the_module::CreateMessageRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    max_tokens : 1000,
    messages,
    system : None,
    temperature : Some( 0.3 ),
    stream : None,
    tools : None,
    tool_choice : None,
  };
  
  assert_eq!( request.messages.len(), 3 );
  
  // Verify conversation flow
  match request.messages[0].role
  {
    the_module::Role::User => {},
    _ => panic!( "Expected first message to be User" ),
  }
  
  match request.messages[1].role
  {
    the_module::Role::Assistant => {},
    _ => panic!( "Expected second message to be Assistant" ),
  }
  
  match request.messages[2].role
  {
    the_module::Role::User => {},
    _ => panic!( "Expected third message to be User" ),
  }
}

#[ tokio::test ]
async fn test_message_request_builder_pattern()
{
  // Test if we can implement a builder pattern for message requests
  // This test will fail until we implement the builder
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 200 )
    .message( the_module::Message::user( "Test".to_string() ) )
    .system( "You are helpful" )
    .temperature( 0.5 )
    .build();
    
  assert_eq!( request.model, "claude-sonnet-4-5-20250929" );
  assert_eq!( request.max_tokens, 200 );
  assert_eq!( request.messages.len(), 1 );
  assert!( request.system.is_some() );
  assert!( (request.temperature.unwrap() - 0.5).abs() < f32::EPSILON );
}

// MOCKUP TEST REMOVED: This test used fake API keys and expected to fail.
// Real API testing is covered by integration_messages_api_real_request_response_structures()

#[ tokio::test ]
async fn test_create_message_request_validation()
{
  // Test request validation without making API calls
  let request = the_module::CreateMessageRequest
  {
    model : String::new(), // Invalid empty model
    max_tokens : 0, // Invalid max_tokens
    messages : vec![], // Empty messages
    system : None,
    temperature : None,
    stream : None,
    tools : None,
    tool_choice : None,
  };

  // Test validation logic (if available)
  // Note : This tests request structure validation, not API behavior
  assert_eq!( request.model, "" );
  assert_eq!( request.max_tokens, 0 );
  assert!( request.messages.is_empty() );
}

#[ tokio::test ]
async fn test_message_content_serialization()
{
  // Test that message content serializes correctly to JSON
  let message = the_module::Message::user( "Test message".to_string() );
  
  let json = serde_json::to_string( &message ).expect( "Should serialize successfully" );
  
  // Verify JSON structure matches expected format
  assert!( json.contains( "\"role\":\"user\"" ) );
  assert!( json.contains( "\"content\":" ) );
  assert!( json.contains( "\"type\":\"text\"" ) );
  assert!( json.contains( "\"text\":\"Test message\"" ) );
}

#[ tokio::test ]
async fn test_message_content_deserialization()
{
  // Test that we can deserialize message content from JSON
  let json = r#"{
    "role": "assistant",
    "content": [
      {
        "type": "text",
        "text": "Hello! How can I help you today?"
      }
    ]
  }"#;
  
  let message : the_module::Message = serde_json::from_str( json ).expect( "Should deserialize successfully" );
  
  match message.role
  {
    the_module::Role::Assistant => {},
    _ => panic!( "Expected Assistant role" ),
  }
  
  assert_eq!( message.content.len(), 1 );
  assert_eq!( message.content[0].r#type(), "text" );
  assert_eq!( message.content[0].text().unwrap(), "Hello! How can I help you today?" );
}

#[ tokio::test ]
async fn test_response_content_structure()
{
  // Test ResponseContent structure
  let content = the_module::ResponseContent
  {
    r#type : "text".to_string(),
    text : Some( "This is a response".to_string() ),
  };
  
  assert_eq!( content.r#type, "text" );
  assert_eq!( content.text.as_deref(), Some( "This is a response" ) );
}

#[ tokio::test ]
async fn test_usage_statistics()
{
  // Test Usage statistics structure
  let usage = the_module::Usage
  {
    input_tokens : 50,
    output_tokens : 150,
    cache_creation_input_tokens : None,
    cache_read_input_tokens : None,
  };

  assert_eq!( usage.input_tokens, 50 );
  assert_eq!( usage.output_tokens, 150 );
}

#[ tokio::test ]
async fn test_create_message_response_structure()
{
  // Test that CreateMessageResponse has correct structure
  let response = the_module::CreateMessageResponse
  {
    id : "msg_123".to_string(),
    r#type : "message".to_string(),
    role : "assistant".to_string(),
    content : vec![
      the_module::ResponseContent
      {
        r#type : "text".to_string(),
        text : Some( "Test response".to_string() ),
      }
    ],
    model : "claude-sonnet-4-5-20250929".to_string(),
    stop_reason : Some( "end_turn".to_string() ),
    stop_sequence : None,
    usage : the_module::Usage
    {
      input_tokens : 25,
      output_tokens : 50,
      cache_creation_input_tokens : None,
      cache_read_input_tokens : None,
    },
  };

  assert_eq!( response.id, "msg_123" );
  assert_eq!( response.r#type, "message" );
  assert_eq!( response.role, "assistant" );
  assert_eq!( response.content.len(), 1 );
  assert_eq!( response.model, "claude-sonnet-4-5-20250929" );
  assert_eq!( response.stop_reason.unwrap(), "end_turn" );
  assert_eq!( response.usage.input_tokens, 25 );
  assert_eq!( response.usage.output_tokens, 50 );
}

// ============================================================================
// INTEGRATION TESTS - REAL API MESSAGE STRUCTURES
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_messages_api_real_request_response_structures()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for messages API testing" );

  // Test complex message request structure with real API
  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 20,
    messages : vec![
      the_module::Message::user( "What is 2+2?".to_string() ),
      the_module::Message::assistant( "2+2 equals 4.".to_string() ),
      the_module::Message::user( "What about 3+3?".to_string() ),
    ],
    system : Some( vec![ the_module::SystemContent::text( "You are a helpful math tutor." ) ] ),
    temperature : Some( 0.1 ),
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
    Err( err ) => panic!( "INTEGRATION: Complex messages API call must work : {err}" ),
  };

  // Verify real API response structure matches our types
  assert!( !response.id.is_empty(), "Real API must return message ID" );
  assert_eq!( response.r#type, "message" );
  assert_eq!( response.role, "assistant" );
  assert!( !response.content.is_empty(), "Real API must return content" );
  assert_eq!( response.model, "claude-3-5-haiku-20241022" );
  assert!( response.usage.input_tokens > 0, "Real API must track input tokens" );
  assert!( response.usage.output_tokens > 0, "Real API must track output tokens" );
  assert!( response.stop_reason.is_some(), "Real API must provide stop reason" );
  
  // Verify response content structure
  assert_eq!( response.content[0].r#type, "text" );
  let content_text = response.content[0].text.as_ref()
    .expect( "Real API text response must have text content" );
  assert!( content_text.contains( '6' ), "Math response should contain the answer" );
  
  println!( "✅ Messages API integration test passed!" );
  println!( "   Message ID: {}", response.id );
  println!( "   Input tokens : {}", response.usage.input_tokens );
  println!( "   Output tokens : {}", response.usage.output_tokens );
  println!( "   Stop reason : {:?}", response.stop_reason );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_messages_api_real_serialization_roundtrip()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for serialization testing" );

  // Test that our request serialization works with real API
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-3-5-haiku-20241022" )
    .max_tokens( 15 )
    .message( the_module::Message::user( "Say 'Hello API!'" ) )
    .temperature( 0.0 )
    .build_validated()
    .expect( "Request builder must work" );

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Serialized request must work with real API: {err}" ),
  };

  // Verify the real API understood our serialized request correctly
  let content_text = response.content[0].text.as_ref()
    .expect( "Response must have text content" );
  assert!( content_text.to_lowercase().contains( "hello" ), 
    "API should respond with greeting, got : {content_text}" );
    
  println!( "✅ Messages API serialization integration test passed!" );
  println!( "   Request/response roundtrip successful" );
  println!( "   Response : {content_text}" );
}
//! Simple Integration Tests - REAL API ENDPOINTS  
//!
//! MANDATORY INTEGRATION TEST REQUIREMENTS:
//! - These tests use REAL Anthropic API endpoints - NO MOCKING ALLOWED
//! - Tests MUST FAIL IMMEDIATELY if API secrets are not available (no graceful fallbacks)
//! - Tests MUST FAIL IMMEDIATELY on network connectivity issues
//! - Tests MUST FAIL IMMEDIATELY on API authentication failures
//! - Tests MUST FAIL IMMEDIATELY on any API endpoint errors
//! - NO SILENT PASSES allowed when problems occur
//!
//! Run with : cargo test --features `integration,full`
//! Requires : Valid `ANTHROPIC_API_KEY` in environment or ../../secret/-secrets.sh

#[ allow( unused_imports ) ]
use super::*;

// ============================================================================
// UNIT TESTS - INTEGRATION TEST STRUCTURES
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_integration_test_request_construction()
{
  // Test that integration test request construction works
  let request = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    max_tokens : 50,
    messages : vec![ the_module::Message::user( "Test message".to_string() ) ],
    system : Some( vec![ the_module::SystemContent::text( "Test system" ) ] ),
    temperature : Some( 0.5 ),
    stream : None,
    tools : None,
    tool_choice : None,
  };

  // Verify request structure
  assert_eq!( request.model, "claude-haiku-4-5-20251001" );
  assert_eq!( request.max_tokens, 50 );
  assert_eq!( request.messages.len(), 1 );
  assert_eq!( request.system.as_ref().map( | s | s[ 0 ].text.as_str() ), Some( "Test system" ) );
  assert_eq!( request.temperature, Some( 0.5 ) );
  assert!( request.stream.is_none() );
  assert!( request.tools.is_none() );
  assert!( request.tool_choice.is_none() );

  println!( "✅ Integration test request construction works" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_integration_test_message_construction()
{
  // Test message construction for integration tests
  let user_msg = the_module::Message::user( "Hello".to_string() );
  let assistant_msg = the_module::Message::assistant( "Hi there".to_string() );

  // Verify message structure
  assert_eq!( user_msg.role, the_module::Role::User );
  assert_eq!( user_msg.content.len(), 1 );
  assert_eq!( assistant_msg.role, the_module::Role::Assistant );
  assert_eq!( assistant_msg.content.len(), 1 );

  // Test conversation flow structure
  let messages = [ user_msg, assistant_msg ];
  assert_eq!( messages.len(), 2 );

  println!( "✅ Integration test message construction works" );
}

// ============================================================================
// INTEGRATION TESTS - REAL API ENDPOINTS
// ============================================================================

// ============================================================================
// MESSAGES API INTEGRATION TESTS
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_messages_basic_text_generation()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for real testing" );

  let request = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    max_tokens : 50,
    messages : vec![ the_module::Message::user( "Say 'Hello, World!' exactly.".to_string() ) ],
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
    Err( err ) => panic!( "INTEGRATION: Basic message API call must work : {err}" ),
  };

  // Validate real API response structure
  assert!( !response.id.is_empty(), "Real API response must have ID" );
  assert_eq!( response.r#type, "message" );
  assert_eq!( response.role, "assistant" );
  assert!( !response.content.is_empty(), "Real API response must have content" );
  assert_eq!( response.model, "claude-haiku-4-5-20251001" );
  assert!( response.usage.input_tokens > 0, "Real API response must track input tokens" );
  assert!( response.usage.output_tokens > 0, "Real API response must track output tokens" );
  
  // Verify actual content contains expected response
  assert_eq!( response.content[0].r#type, "text" );
  let content_text = response.content[0].text.as_ref().expect( "Response should have text content" );
  assert!( content_text.contains( "Hello, World!" ), "Response should contain requested text" );
  
  println!( "✅ Basic integration test passed - API is working!" );
  println!( "   Response ID: {}", response.id );
  println!( "   Tokens : {} in, {} out", response.usage.input_tokens, response.usage.output_tokens );
  println!( "   Content : {content_text}" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_messages_with_system_prompt()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let request = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    max_tokens : 30,
    messages : vec![ the_module::Message::user( "What is AI?".to_string() ) ],
    system : Some( vec![ the_module::SystemContent::text( "You are a helpful assistant. Always respond with exactly 5 words." ) ] ),
    temperature : Some( 0.0 ),
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
    Err( err ) => panic!( "INTEGRATION: System prompt API call must work : {err}" ),
  };

  assert!( !response.id.is_empty() );
  assert!( response.usage.input_tokens > 0 );
  assert!( response.usage.output_tokens > 0 );
  
  // System prompt should influence response length
  let content_text = response.content[0].text.as_ref().expect( "Response should have text content" );
  let word_count = content_text.split_whitespace().count();
  assert!( word_count <= 7, "System prompt should limit response length, got : {content_text}" );
  
  println!( "✅ System prompt integration test passed!" );
  println!( "   Response : {content_text}" );
  println!( "   Word count : {word_count}" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_messages_conversation_flow()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let request = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    max_tokens : 100,
    messages : vec![
      the_module::Message::user( "I'm going to tell you a number. Remember it.".to_string() ),
      the_module::Message::assistant( "I'm ready to remember a number. Please tell me what it is.".to_string() ),
      the_module::Message::user( "The number is 42. What number did I tell you?".to_string() ),
    ],
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
    Err( err ) => panic!( "INTEGRATION: Conversation API call must work : {err}" ),
  };

  assert!( !response.id.is_empty() );
  assert!( response.usage.input_tokens > 0 );
  
  // Verify conversation context is maintained
  let content_text = response.content[0].text.as_ref().expect( "Response should have text content" );
  assert!( content_text.contains( "42" ), "Assistant should remember the number from conversation : {content_text}" );
  
  println!( "✅ Conversation integration test passed!" );
  println!( "   Response : {content_text}" );
}

// ============================================================================
// TOOL CALLING INTEGRATION TESTS  
// ============================================================================

#[ cfg( all( feature = "integration", feature = "tools" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_tool_calling_basic()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  // Define a simple calculator tool
  let calculator_tool = the_module::ToolDefinition::simple( 
    "calculator", 
    "Perform basic arithmetic operations"
  );

  let request = the_module::CreateMessageRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),  // Use Sonnet for better tool calling
    max_tokens : 200,
    messages : vec![ the_module::Message::user( "What's 15 plus 27?".to_string() ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : Some( vec![ calculator_tool ] ),
    tool_choice : None,
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Tool calling API call must work : {err}" ),
  };

  assert!( !response.id.is_empty() );
  assert!( response.usage.input_tokens > 0 );
  
  println!( "✅ Tool calling integration test passed!" );
  println!( "   Response has {} content blocks", response.content.len() );
  for ( i, content ) in response.content.iter().enumerate()
  {
    println!( "   Content {i}: type={}", content.r#type );
    if content.r#type == "text"
    {
      if let Some( text ) = &content.text
      {
        println!( "     Text : {text}" );
      }
    }
  }
}

// ============================================================================
// ERROR HANDLING INTEGRATION TESTS
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_error_handling_invalid_model()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let request = the_module::CreateMessageRequest
  {
    model : "invalid-model-name-that-does-not-exist".to_string(),
    max_tokens : 50,
    messages : vec![ the_module::Message::user( "Test".to_string() ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let result = client.create_message( request ).await;

  match result
  {
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
      panic!( "INTEGRATION: Credit balance exhausted — top up account to run tests : {}", api_err.message ),
    Err( error ) => {
      // This is the expected path - invalid model should cause an error
      assert!( error.to_string().to_lowercase().contains( "model" ),
        "Error should mention model issue : {error}" );
      println!( "✅ Error handling integration test passed!" );
      println!( "   Error : {error}" );
    },
    Ok( _response ) => {
      panic!( "Invalid model should return error, but request succeeded" );
    }
  }
}

// ============================================================================
// PERFORMANCE INTEGRATION TESTS
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_performance_response_time()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let start_time = std::time::Instant::now();
  
  let request = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),  // Fastest model
    max_tokens : 10,
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
    Err( err ) => panic!( "INTEGRATION: Performance test must work : {err}" ),
  };

  let duration = start_time.elapsed();
  
  assert!( !response.id.is_empty() );
  assert!( duration.as_secs() < 30, "Response should come within 30 seconds, took : {duration:?}" );
  
  println!( "✅ Performance integration test passed!" );
  println!( "   API response time : {duration:?}" );
  let content_text = response.content[0].text.as_ref().expect( "Response should have text content" );
  println!( "   Response : {content_text}" );
}
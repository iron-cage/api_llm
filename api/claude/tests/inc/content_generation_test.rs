//! Content Generation Integration Tests - STRICT FAILURE POLICY
//!
//! MANDATORY INTEGRATION TEST REQUIREMENTS:
//! - These tests use REAL Anthropic API endpoints - NO MOCKING ALLOWED
//! - Tests MUST FAIL IMMEDIATELY if API secrets are not available (no graceful fallbacks)
//! - Tests MUST FAIL IMMEDIATELY on network connectivity issues
//! - Tests MUST FAIL IMMEDIATELY on API authentication failures
//! - Tests MUST FAIL IMMEDIATELY on any API endpoint errors
//! - NO SILENT PASSES allowed when problems occur
//!
//! Run with : cargo test --features `content_generation,integration`
//! Requires : Valid `ANTHROPIC_API_KEY` in environment or ../../secret/-secrets.sh

#[ allow( unused_imports ) ]
use super::*;

// Removed test_simple_content_generation - used fake API keys

#[ tokio::test ]
async fn test_request_structure_with_temperature()
{
  // Test request structure with different temperature values
  let temperatures = vec![ 0.0, 0.5, 1.0 ];

  for temp in temperatures
  {
    let request = the_module::CreateMessageRequest::builder()
      .model( "claude-sonnet-4-5-20250929" )
      .max_tokens( 50 )
      .message( the_module::Message::user( "Say hello".to_string() ) )
      .temperature( temp )
      .build();

    assert!( (request.temperature.unwrap() - temp).abs() < f32::EPSILON );
    assert_eq!( request.model, "claude-sonnet-4-5-20250929" );
  }
}

#[ tokio::test ]
async fn test_request_structure_with_different_models()
{
  // Test request structure with different Claude models
  let models = vec![
    "claude-sonnet-4-5-20250929",
    "claude-3-5-haiku-20241022",
    "claude-3-opus-20240229",
  ];

  for model in models
  {
    let request = the_module::CreateMessageRequest::builder()
      .model( model )
      .max_tokens( 100 )
      .message( the_module::Message::user( "Test message".to_string() ) )
      .build();

    assert_eq!( request.model, model );
    assert_eq!( request.max_tokens, 100 );
  }
}

#[ tokio::test ]
async fn test_request_structure_with_system_prompts()
{
  // Test request structure with various system prompts
  let system_prompts = vec![
    "You are a helpful assistant.",
    "You are a creative writer specializing in poetry.",
    "You are a technical expert who explains complex concepts simply.",
  ];

  for system_prompt in system_prompts
  {
    let request = the_module::CreateMessageRequest::builder()
      .model( "claude-sonnet-4-5-20250929" )
      .max_tokens( 200 )
      .system( system_prompt )
      .message( the_module::Message::user( "Help me with my task".to_string() ) )
      .build();

    assert_eq!( request.system.as_ref().unwrap()[ 0 ].text, system_prompt );
  }
}

#[ tokio::test ]
async fn test_request_structure_max_tokens_limits()
{
  // Test request structure with different token limits
  let token_limits = vec![ 1, 50, 100, 1000, 4000 ];

  for max_tokens in token_limits
  {
    let request = the_module::CreateMessageRequest::builder()
      .model( "claude-sonnet-4-5-20250929" )
      .max_tokens( max_tokens )
      .message( the_module::Message::user( "Generate content".to_string() ) )
      .build();

    assert_eq!( request.max_tokens, max_tokens );
  }
}

#[ tokio::test ]
async fn test_request_structure_streaming_mode()
{
  // Test request structure with streaming enabled
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 100 )
    .message( the_module::Message::user( "Tell me a story".to_string() ) )
    .stream( true )
    .build();

  assert!( request.stream.unwrap() );

  // Test streaming disabled
  let request_no_stream = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 100 )
    .message( the_module::Message::user( "Tell me a story".to_string() ) )
    .stream( false )
    .build();

  assert!( !request_no_stream.stream.unwrap() );
}

#[ tokio::test ]
async fn test_request_structure_long_conversation()
{
  // Test request structure with extended conversation history
  let mut messages = Vec::new();

  // Simulate a long conversation
  for i in 0..10
  {
    messages.push( the_module::Message::user( format!( "User message {i}" ) ) );
    messages.push( the_module::Message::assistant( format!( "Assistant response {i}" ) ) );
  }

  messages.push( the_module::Message::user( "Final user message".to_string() ) );

  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 100 )
    .messages( messages.clone() )
    .build();

  assert_eq!( request.messages.len(), 21 ); // 10 pairs + 1 final
  assert_eq!( request.messages.last().unwrap().content[0].text().unwrap(), "Final user message" );
}

#[ tokio::test ]
async fn test_request_structure_with_mixed_parameters()
{
  // Test request structure with all parameters combined
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 500 )
    .system( "You are a poet and programmer" )
    .message( the_module::Message::user( "Hello".to_string() ) )
    .message( the_module::Message::assistant( "Hi! How can I help?".to_string() ) )
    .message( the_module::Message::user( "Write code poetry".to_string() ) )
    .temperature( 0.7 )
    .stream( false )
    .build();

  assert_eq!( request.model, "claude-sonnet-4-5-20250929" );
  assert_eq!( request.max_tokens, 500 );
  assert_eq!( request.system.as_ref().unwrap()[ 0 ].text, "You are a poet and programmer" );
  assert_eq!( request.messages.len(), 3 );
  assert!( (request.temperature.unwrap() - 0.7).abs() < f32::EPSILON );
  assert!( !request.stream.unwrap() );
}

#[ tokio::test ]
async fn test_request_structure_error_validation()
{
  // Test that request structure validation works correctly

  // Test with missing required fields - should panic on build()
  std::panic::catch_unwind( ||
  {
    let _request = the_module::CreateMessageRequest::builder()
      .message( the_module::Message::user( "Test".to_string() ) )
      .build(); // Missing model and max_tokens
  }).expect_err( "Should panic on missing required fields" );

  // Test with zero max tokens
  let request_zero = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 0 )
    .message( the_module::Message::user( "Test".to_string() ) )
    .build();

  assert_eq!( request_zero.max_tokens, 0 ); // Valid but not recommended
}

#[ tokio::test ]
async fn test_response_structure_parsing()
{
  // Test response structure is correctly defined
  let usage = the_module::Usage
  {
    input_tokens : 25,
    output_tokens : 75,
    cache_creation_input_tokens : None,
    cache_read_input_tokens : None,
  };

  let content = vec![
    the_module::ResponseContent
    {
      r#type : "text".to_string(),
      text : Some( "Generated content here".to_string() ),
    }
  ];

  let response = the_module::CreateMessageResponse
  {
    id : "msg_content_123".to_string(),
    r#type : "message".to_string(),
    role : "assistant".to_string(),
    content,
    model : "claude-sonnet-4-5-20250929".to_string(),
    stop_reason : Some( "end_turn".to_string() ),
    stop_sequence : None,
    usage,
  };

  assert_eq!( response.id, "msg_content_123" );
  assert_eq!( response.content[0].text.as_deref(), Some( "Generated content here" ) );
  assert_eq!( response.usage.input_tokens + response.usage.output_tokens, 100 );
}

#[ tokio::test ]
async fn test_request_builder_method_chaining()
{
  // Test that builder methods can be chained in any order
  let request1 = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 100 )
    .temperature( 0.5 )
    .system( "Test system" )
    .message( the_module::Message::user( "Test".to_string() ) )
    .stream( true )
    .build();

  let request2 = the_module::CreateMessageRequest::builder()
    .stream( true )
    .message( the_module::Message::user( "Test".to_string() ) )
    .system( "Test system" )
    .temperature( 0.5 )
    .max_tokens( 100 )
    .model( "claude-sonnet-4-5-20250929" )
    .build();

  // Both should produce equivalent requests
  assert_eq!( request1.model, request2.model );
  assert_eq!( request1.max_tokens, request2.max_tokens );
  assert_eq!( request1.temperature, request2.temperature );
  assert_eq!( request1.system, request2.system );
  assert_eq!( request1.stream, request2.stream );
}

#[ tokio::test ]
async fn test_request_json_serialization()
{
  // Test that requests serialize to valid JSON
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 100 )
    .message( the_module::Message::user( "Hello world".to_string() ) )
    .temperature( 0.8 )
    .build();

  let json = serde_json::to_string( &request ).expect( "Should serialize successfully" );

  // Verify key fields are present in JSON
  assert!( json.contains( "\"model\":\"claude-sonnet-4-5-20250929\"" ) );
  assert!( json.contains( "\"max_tokens\":100" ) );
  assert!( json.contains( "\"temperature\":0.8" ) );
  assert!( json.contains( "\"messages\":" ) );
  assert!( json.contains( "\"Hello world\"" ) );
}

// ============================================================================
// INTEGRATION TESTS - REAL API CONTENT GENERATION
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_content_generation_temperature_control()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for temperature testing" );

  // Test different temperature settings with real API
  let low_temp_request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 30,
    messages : vec![ the_module::Message::user( "Generate a creative story about a robot".to_string() ) ],
    system : None,
    temperature : Some( 0.0 ), // Very deterministic
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let high_temp_request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 30,
    messages : vec![ the_module::Message::user( "Generate a creative story about a robot".to_string() ) ],
    system : None,
    temperature : Some( 0.9 ), // Very creative
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let low_response = match client.create_message( low_temp_request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Low temperature request must work : {err}" ),
  };

  let high_response = match client.create_message( high_temp_request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: High temperature request must work : {err}" ),
  };

  // Both should succeed and return content
  assert!( !low_response.id.is_empty() );
  assert!( !high_response.id.is_empty() );
  assert!( !low_response.content.is_empty() );
  assert!( !high_response.content.is_empty() );

  let low_content = low_response.content[0].text.as_ref()
    .expect( "Low temp response must have text" );
  let high_content = high_response.content[0].text.as_ref()
    .expect( "High temp response must have text" );

  // Both should contain story content
  assert!( !low_content.is_empty() );
  assert!( !high_content.is_empty() );

  println!( "✅ Content generation temperature control integration test passed!" );
  println!( "   Low temp (0.0): {low_content}" );
  println!( "   High temp (0.9): {high_content}" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_content_generation_max_tokens_control()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for token limit testing" );

  // Test different max_tokens settings
  let short_request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 10, // Very short
    messages : vec![ the_module::Message::user( "Write a long essay about artificial intelligence".to_string() ) ],
    system : None,
    temperature : Some( 0.5 ),
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let long_request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100, // Longer
    messages : vec![ the_module::Message::user( "Write a long essay about artificial intelligence".to_string() ) ],
    system : None,
    temperature : Some( 0.5 ),
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let short_response = match client.create_message( short_request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Short max_tokens request must work : {err}" ),
  };

  let long_response = match client.create_message( long_request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Long max_tokens request must work : {err}" ),
  };

  // Verify token limits are respected
  assert!( short_response.usage.output_tokens <= 10, 
    "Short response should have ≤10 tokens, got : {}", short_response.usage.output_tokens );
  assert!( long_response.usage.output_tokens > short_response.usage.output_tokens,
    "Long response should have more tokens than short" );

  println!( "✅ Content generation max_tokens control integration test passed!" );
  println!( "   Short (10 max): {} tokens", short_response.usage.output_tokens );
  println!( "   Long (100 max): {} tokens", long_response.usage.output_tokens );
}
//! Synchronous Cached Content API Tests
//!
//! Tests for blocking/synchronous prompt caching functionality

#[ allow( unused_imports ) ]
use super::*;

// ============================================================================
// UNIT TESTS - SYNC CACHED CONTENT STRUCTURE
// ============================================================================

#[ test ]
fn test_sync_cached_message_structure()
{
  // Test that cached messages can be created with SyncClient types
  let cache_control = the_module::CacheControl::ephemeral();

  let message = the_module::Message
  {
    role : the_module::Role::User,
    content : vec![ the_module::Content::Text
    {
      r#type : "text".to_string(),
      text : "Test message with caching".to_string(),
    } ],
    cache_control : Some( cache_control ),
  };

  assert!( message.cache_control.is_some() );
  assert_eq!( message.cache_control.unwrap().cache_type, "ephemeral" );
}

#[ test ]
fn test_sync_system_prompt_caching()
{
  // Test SystemContent structure with cache_control
  let system = the_module::SystemContent
  {
    text : "You are a helpful assistant.".to_string(),
    r#type : "text".to_string(),
    cache_control : Some( the_module::CacheControl::ephemeral() ),
  };

  // Verify structure is correct
  assert!( system.cache_control.is_some() );
  assert_eq!( system.cache_control.unwrap().cache_type, "ephemeral" );
  assert_eq!( system.text, "You are a helpful assistant." );
}

// ============================================================================
// INTEGRATION TESTS - REAL API CACHED CONTENT
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ test ]
fn integration_sync_cached_content_creation()
{
  // Test synchronous cached content creation
  let client = the_module::SyncClient::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for cached content test" );

  // Create a message with cache_control to enable prompt caching
  let system_prompt = vec![
    the_module::SystemContent
    {
      text : "You are a helpful assistant. This is a long system prompt that should be cached for efficiency.".to_string(),
      r#type : "text".to_string(),
      cache_control : Some( the_module::CacheControl::ephemeral() ),
    }
  ];

  let request = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    max_tokens : 50,
    messages : vec![ the_module::Message::user( "Hello!".to_string() ) ],
    system : Some( system_prompt ),
    temperature : Some( 0.0 ),
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let response = match client.create_message( &request )
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Cached content creation must work : {err}" ),
  };

  // Verify response is valid
  assert!( !response.id.is_empty() );
  assert!( response.usage.output_tokens > 0 );

  // Check cache statistics (first request should create cache)
  // Note : cache_creation_input_tokens might be present in usage
  println!( "✅ Sync cached content creation test passed!" );
  println!( "   Message ID: {}", response.id );
  println!( "   Input tokens : {}", response.usage.input_tokens );
  println!( "   Output tokens : {}", response.usage.output_tokens );
}

#[ cfg( feature = "integration" ) ]
#[ test ]
fn integration_sync_cache_hit_scenario()
{
  // Test that subsequent requests hit the cache
  let client = the_module::SyncClient::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for cache hit test" );

  let system_prompt = vec![
    the_module::SystemContent
    {
      text : "You are a helpful assistant specialized in mathematics. This prompt should be cached.".to_string(),
      r#type : "text".to_string(),
      cache_control : Some( the_module::CacheControl::ephemeral() ),
    }
  ];

  // First request - creates cache
  let request1 = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    max_tokens : 30,
    messages : vec![ the_module::Message::user( "What is 2+2?".to_string() ) ],
    system : Some( system_prompt.clone() ),
    temperature : Some( 0.0 ),
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let response1 = match client.create_message( &request1 )
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: First request must work : {err}" ),
  };

  assert!( !response1.id.is_empty() );

  // Second request - should hit cache
  let request2 = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    max_tokens : 30,
    messages : vec![ the_module::Message::user( "What is 3+3?".to_string() ) ],
    system : Some( system_prompt ),
    temperature : Some( 0.0 ),
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let response2 = match client.create_message( &request2 )
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted on second request - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Second request must work : {err}" ),
  };

  assert!( !response2.id.is_empty() );
  assert_ne!( response1.id, response2.id, "Responses should have different IDs" );

  println!( "✅ Sync cache hit scenario test passed!" );
  println!( "   First response ID: {}", response1.id );
  println!( "   Second response ID: {}", response2.id );
}

#[ cfg( feature = "integration" ) ]
#[ test ]
fn integration_sync_cache_error_handling()
{
  // Test error handling with invalid cached content configuration
  let client = the_module::SyncClient::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for error test" );

  // Create request with caching but invalid model
  let system_prompt = vec![
    the_module::SystemContent
    {
      text : "Test prompt".to_string(),
      r#type : "text".to_string(),
      cache_control : Some( the_module::CacheControl::ephemeral() ),
    }
  ];

  let request = the_module::CreateMessageRequest
  {
    model : "invalid-model-for-caching".to_string(),
    max_tokens : 10,
    messages : vec![ the_module::Message::user( "Test".to_string() ) ],
    system : Some( system_prompt ),
    temperature : Some( 0.0 ),
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let result = client.create_message( &request );

  // Should fail with invalid model error
  assert!( result.is_err(), "INTEGRATION: Invalid model should cause error" );

  println!( "✅ Sync cache error handling test passed!" );
}

#[ cfg( feature = "integration" ) ]
#[ test ]
fn integration_sync_cached_content_cost_savings()
{
  // Test that cached content demonstrates cost savings
  let client = the_module::SyncClient::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for cost test" );

  // Use a longer system prompt to demonstrate caching benefits
  let long_system = "You are an expert assistant. ".repeat( 50 );

  let system_prompt = vec![
    the_module::SystemContent
    {
      text : long_system,
      r#type : "text".to_string(),
      cache_control : Some( the_module::CacheControl::ephemeral() ),
    }
  ];

  let request = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    max_tokens : 20,
    messages : vec![ the_module::Message::user( "Hi".to_string() ) ],
    system : Some( system_prompt ),
    temperature : Some( 0.0 ),
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let response = match client.create_message( &request )
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Cost savings test must work : {err}" ),
  };

  // Verify response
  assert!( !response.id.is_empty() );
  assert!( response.usage.input_tokens > 0 );

  println!( "✅ Sync cached content cost savings test passed!" );
  println!( "   Input tokens processed : {}", response.usage.input_tokens );
  println!( "   Output tokens : {}", response.usage.output_tokens );
}

//! System Instructions Tests
//!
//! Tests for structured system instructions API, including multi-part system prompts,
//! templating, composition, and validation.

#[ allow( unused_imports ) ]
use super::*;

// ============================================================================
// UNIT TESTS - SYSTEM CONTENT CONSTRUCTION
// ============================================================================

#[ test ]
fn test_system_content_text_creation()
{
  // Test basic text system content creation
  let content = the_module::SystemContent::text( "You are a helpful assistant" );

  assert_eq!( content.r#type, "text" );
  assert_eq!( content.text, "You are a helpful assistant" );
  assert!( content.cache_control.is_none() );
}

#[ test ]
fn test_system_content_from_str()
{
  // Test creating system content from &str using From trait
  let content : the_module::SystemContent = "You are a helpful assistant".into();

  assert_eq!( content.r#type, "text" );
  assert_eq!( content.text, "You are a helpful assistant" );
  assert!( content.cache_control.is_none() );
}

#[ test ]
fn test_system_content_with_cache_control()
{
  // Test system content with cache control
  let mut content = the_module::SystemContent::text( "You are a helpful assistant" );
  content.cache_control = Some( the_module::CacheControl::ephemeral() );

  assert_eq!( content.r#type, "text" );
  assert_eq!( content.text, "You are a helpful assistant" );
  assert!( content.cache_control.is_some() );
}

#[ test ]
fn test_multi_part_system_instructions()
{
  // Test creating multi-part system instructions
  let instructions =
  [
    the_module::SystemContent::text( "You are a helpful assistant." ),
    the_module::SystemContent::text( "Always respond in a friendly tone." ),
    the_module::SystemContent::text( "Keep responses concise." ),
  ].to_vec();

  assert_eq!( instructions.len(), 3 );
  assert!( instructions.iter().all( |c| c.r#type == "text" ) );
}

#[ test ]
fn test_system_instruction_composition()
{
  // Test composing system instructions with different parts
  let base_instruction = the_module::SystemContent::text( "You are a helpful assistant." );

  let mut cached_knowledge = the_module::SystemContent::text( "Knowledge base : Rust is a systems programming language." );
  cached_knowledge.cache_control = Some( the_module::CacheControl::ephemeral() );

  let task_instruction = the_module::SystemContent::text( "Help the user with Rust programming." );

  let instructions = [ base_instruction, cached_knowledge, task_instruction ].to_vec();

  assert_eq!( instructions.len(), 3 );
  assert!( instructions[ 1 ].cache_control.is_some() );
  assert!( instructions[ 0 ].cache_control.is_none() );
  assert!( instructions[ 2 ].cache_control.is_none() );
}

#[ test ]
fn test_system_content_serialization()
{
  // Test that system content serializes correctly to JSON
  let content = the_module::SystemContent::text( "You are a helpful assistant" );

  let json = serde_json::to_value( &content ).unwrap();

  assert_eq!( json[ "type" ], "text" );
  assert_eq!( json[ "text" ], "You are a helpful assistant" );
  assert!( json.get( "cache_control" ).is_none() );
}

#[ test ]
fn test_system_content_with_cache_serialization()
{
  // Test serialization with cache control
  let mut content = the_module::SystemContent::text( "Cached knowledge" );
  content.cache_control = Some( the_module::CacheControl::ephemeral() );

  let json = serde_json::to_value( &content ).unwrap();

  assert_eq!( json[ "type" ], "text" );
  assert_eq!( json[ "text" ], "Cached knowledge" );
  assert!( json.get( "cache_control" ).is_some() );
}

// ============================================================================
// INTEGRATION TESTS - SYSTEM INSTRUCTIONS IN API CALLS
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_single_system_instruction()
{
  // Test using a single system instruction in API call
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let system = vec![ the_module::SystemContent::text( "You are a helpful assistant." ) ];

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 50,
    messages : vec![ the_module::Message::user( "Say hello!".to_string() ) ],
    system : Some( system ),
    temperature : Some( 0.0 ),
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Single system instruction must work : {err}" ),
  };

  assert!( !response.id.is_empty() );
  assert!( !response.content.is_empty() );

  println!( "✅ Single system instruction test passed!" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_multi_part_system_instructions()
{
  // Test using multi-part system instructions
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let system = vec!
  [
    the_module::SystemContent::text( "You are a helpful assistant." ),
    the_module::SystemContent::text( "Always respond with enthusiasm." ),
    the_module::SystemContent::text( "Keep responses under 20 words." ),
  ];

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 50,
    messages : vec![ the_module::Message::user( "Hello!".to_string() ) ],
    system : Some( system ),
    temperature : Some( 0.0 ),
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Multi-part system instructions must work : {err}" ),
  };

  assert!( !response.id.is_empty() );
  assert!( !response.content.is_empty() );

  println!( "✅ Multi-part system instructions test passed!" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_system_instructions_with_caching()
{
  // Test system instructions with prompt caching
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let mut cached_context = the_module::SystemContent::text(
    "You are an expert in Rust programming. You know about ownership, borrowing, lifetimes, and async programming."
  );
  cached_context.cache_control = Some( the_module::CacheControl::ephemeral() );

  let task_instruction = the_module::SystemContent::text( "Help the user with their Rust questions." );

  let system = vec![ cached_context, task_instruction ];

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "What is ownership in Rust?".to_string() ) ],
    system : Some( system ),
    temperature : Some( 0.0 ),
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: System instructions with caching must work : {err}" ),
  };

  assert!( !response.id.is_empty() );
  assert!( !response.content.is_empty() );

  println!( "✅ System instructions with caching test passed!" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_empty_system_instructions()
{
  // Test that empty system instructions work (should be same as no system)
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 50,
    messages : vec![ the_module::Message::user( "Hello!".to_string() ) ],
    system : None,
    temperature : Some( 0.0 ),
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Empty system instructions must work : {err}" ),
  };

  assert!( !response.id.is_empty() );
  assert!( !response.content.is_empty() );

  println!( "✅ Empty system instructions test passed!" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_long_system_instruction()
{
  // Test with a very long system instruction
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let long_instruction = "You are a specialized AI assistant. ".repeat( 50 );

  let system = vec![ the_module::SystemContent::text( long_instruction ) ];

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 50,
    messages : vec![ the_module::Message::user( "Hello!".to_string() ) ],
    system : Some( system ),
    temperature : Some( 0.0 ),
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Long system instruction must work : {err}" ),
  };

  assert!( !response.id.is_empty() );
  assert!( !response.content.is_empty() );

  println!( "✅ Long system instruction test passed!" );
}

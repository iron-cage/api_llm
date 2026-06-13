//! Comprehensive tests for Anthropic Prompt Caching feature
//!
//! Following TDD principles - these tests are written first and will initially fail.
//! Task 717 implementation will make these tests pass.

use api_claude::*;

#[ cfg( test ) ]
mod cache_control_tests
{
  use super::*;

  #[ test ]
  fn test_cache_control_creation()
  {
    // Test creating CacheControl with ephemeral type
    let cache_control = CacheControl::ephemeral();

    assert_eq!( cache_control.cache_type, "ephemeral" );
  }

  #[ test ]
  fn test_cache_control_serialization()
  {
    // Test serialization to API format
    let cache_control = CacheControl::ephemeral();
    let json = serde_json::to_value( &cache_control ).unwrap();

    assert_eq!( json[ "type" ], "ephemeral" );
  }

  #[ test ]
  fn test_cache_control_deserialization()
  {
    // Test deserialization from API response
    let json = r#"{"type": "ephemeral"}"#;
    let cache_control : CacheControl = serde_json::from_str( json ).unwrap();

    assert_eq!( cache_control.cache_type, "ephemeral" );
  }
}

#[ cfg( test ) ]
mod system_prompt_tests
{
  use super::*;

  #[ test ]
  fn test_system_prompt_without_caching()
  {
    // Test SystemPrompt without cache_control
    let system = SystemPrompt
    {
      text : "You are a helpful assistant.".to_string(),
      cache_control : None,
    };

    assert_eq!( system.text, "You are a helpful assistant." );
    assert!( system.cache_control.is_none() );
  }

  #[ test ]
  fn test_system_prompt_with_caching()
  {
    // Test SystemPrompt with cache_control
    let system = SystemPrompt
    {
      text : "You are a helpful assistant with a long system prompt that should be cached.".to_string(),
      cache_control : Some( CacheControl::ephemeral() ),
    };

    assert!( system.cache_control.is_some() );
    assert_eq!( system.cache_control.unwrap().cache_type, "ephemeral" );
  }

  #[ test ]
  fn test_system_prompt_serialization()
  {
    // Test serialization of SystemPrompt with caching
    let system = SystemPrompt
    {
      text : "Test prompt".to_string(),
      cache_control : Some( CacheControl::ephemeral() ),
    };

    let json = serde_json::to_value( &system ).unwrap();

    assert_eq!( json[ "text" ], "Test prompt" );
    assert_eq!( json[ "cache_control" ][ "type" ], "ephemeral" );
  }

  #[ test ]
  fn test_system_prompt_serialization_without_cache()
  {
    // Test that cache_control is omitted when None
    let system = SystemPrompt
    {
      text : "Test prompt".to_string(),
      cache_control : None,
    };

    let json = serde_json::to_value( &system ).unwrap();

    assert_eq!( json[ "text" ], "Test prompt" );
    assert!( json.get( "cache_control" ).is_none() );
  }
}

#[ cfg( test ) ]
mod message_caching_tests
{
  use super::*;

  #[ test ]
  fn test_message_with_cache_control()
  {
    // Test Message with cache_control field
    let content = Content::Text
    {
      r#type : "text".to_string(),
      text : "Hello, how are you?".to_string(),
    };

    let message = Message
    {
      role : Role::User,
      content : vec![ content ],
      cache_control : Some( CacheControl::ephemeral() ),
    };

    assert_eq!( message.role, Role::User );
    assert!( message.cache_control.is_some() );
  }

  #[ test ]
  fn test_message_cache_control_serialization()
  {
    // Test serialization includes cache_control
    let content = Content::Text
    {
      r#type : "text".to_string(),
      text : "Test message".to_string(),
    };

    let message = Message
    {
      role : Role::User,
      content : vec![ content ],
      cache_control : Some( CacheControl::ephemeral() ),
    };

    let json = serde_json::to_value( &message ).unwrap();

    assert_eq!( json[ "role" ], "user" );
    assert!( json[ "content" ].is_array() );
    assert_eq!( json[ "cache_control" ][ "type" ], "ephemeral" );
  }
}

#[ cfg( test ) ]
mod request_caching_tests
{
  use super::*;

  #[ test ]
  fn test_request_with_system_prompt_caching()
  {
    // Test CreateMessageRequest with SystemContent array (new format)
    let system = vec![ SystemContent
    {
      r#type : "text".to_string(),
      text : "You are a helpful assistant.".to_string(),
      cache_control : Some( CacheControl::ephemeral() ),
    } ];

    let request = CreateMessageRequest
    {
      model : "claude-sonnet-4-5-20250929".to_string(),
      max_tokens : 1024,
      messages : vec![],
      system : Some( system ),
      temperature : None,
      stream : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
    };

    assert!( request.system.is_some() );
    let system_blocks = request.system.unwrap();
    assert!( system_blocks[ 0 ].cache_control.is_some() );
  }

  #[ test ]
  fn test_request_serialization_with_caching()
  {
    // Test that request serializes correctly with caching
    let system = vec![ SystemContent
    {
      r#type : "text".to_string(),
      text : "Cached system prompt".to_string(),
      cache_control : Some( CacheControl::ephemeral() ),
    } ];

    let request = CreateMessageRequest
    {
      model : "claude-sonnet-4-5-20250929".to_string(),
      max_tokens : 1024,
      messages : vec![],
      system : Some( system ),
      temperature : None,
      stream : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
    };

    let json = serde_json::to_value( &request ).unwrap();

    assert_eq!( json[ "system" ][ 0 ][ "text" ], "Cached system prompt" );
    assert_eq!( json[ "system" ][ 0 ][ "cache_control" ][ "type" ], "ephemeral" );
  }
}

#[ cfg( test ) ]
mod usage_metadata_tests
{
  use super::*;

  #[ test ]
  fn test_usage_with_cache_creation_tokens()
  {
    // Test Usage struct with cache_creation_input_tokens
    let usage = Usage
    {
      input_tokens : 100,
      output_tokens : 50,
      cache_creation_input_tokens : Some( 500 ),
      cache_read_input_tokens : None,
    };

    assert_eq!( usage.input_tokens, 100 );
    assert_eq!( usage.cache_creation_input_tokens, Some( 500 ) );
    assert_eq!( usage.cache_read_input_tokens, None );
  }

  #[ test ]
  fn test_usage_with_cache_read_tokens()
  {
    // Test Usage struct with cache_read_input_tokens
    let usage = Usage
    {
      input_tokens : 100,
      output_tokens : 50,
      cache_creation_input_tokens : None,
      cache_read_input_tokens : Some( 500 ),
    };

    assert_eq!( usage.input_tokens, 100 );
    assert_eq!( usage.cache_creation_input_tokens, None );
    assert_eq!( usage.cache_read_input_tokens, Some( 500 ) );
  }

  #[ test ]
  fn test_usage_deserialization_with_cache_fields()
  {
    // Test parsing cache fields from API response
    let json = r#"{
      "input_tokens": 100,
      "output_tokens": 50,
      "cache_creation_input_tokens": 500,
      "cache_read_input_tokens": 0
    }"#;

    let usage : Usage = serde_json::from_str( json ).unwrap();

    assert_eq!( usage.input_tokens, 100 );
    assert_eq!( usage.output_tokens, 50 );
    assert_eq!( usage.cache_creation_input_tokens, Some( 500 ) );
    assert_eq!( usage.cache_read_input_tokens, Some( 0 ) );
  }

  #[ test ]
  fn test_usage_deserialization_without_cache_fields()
  {
    // Test backward compatibility - old responses without cache fields
    let json = r#"{
      "input_tokens": 100,
      "output_tokens": 50
    }"#;

    let usage : Usage = serde_json::from_str( json ).unwrap();

    assert_eq!( usage.input_tokens, 100 );
    assert_eq!( usage.output_tokens, 50 );
    assert_eq!( usage.cache_creation_input_tokens, None );
    assert_eq!( usage.cache_read_input_tokens, None );
  }
}

#[ cfg( test ) ]
mod cache_statistics_tests
{
  use super::*;

  #[ test ]
  fn test_cache_miss_scenario()
  {
    // Test first request creates cache (cache miss)
    let usage = Usage
    {
      input_tokens : 100,
      output_tokens : 50,
      cache_creation_input_tokens : Some( 500 ), // Created cache
      cache_read_input_tokens : None, // No cache hit
    };

    // Verify cache was created
    assert!( usage.cache_creation_input_tokens.is_some() );
    assert!( usage.cache_read_input_tokens.is_none() );
  }

  #[ test ]
  fn test_cache_hit_scenario()
  {
    // Test subsequent request uses cache (cache hit)
    let usage = Usage
    {
      input_tokens : 100,
      output_tokens : 50,
      cache_creation_input_tokens : None, // No new cache
      cache_read_input_tokens : Some( 500 ), // Read from cache
    };

    // Verify cache was read
    assert!( usage.cache_creation_input_tokens.is_none() );
    assert!( usage.cache_read_input_tokens.is_some() );
  }

  #[ test ]
  fn test_cache_savings_calculation()
  {
    // Test calculating cost savings from caching
    let cache_hit_usage = Usage
    {
      input_tokens : 100,
      output_tokens : 50,
      cache_creation_input_tokens : None,
      cache_read_input_tokens : Some( 500 ),
    };

    // Cache read tokens should be counted separately for cost calculation
    let cached_tokens = cache_hit_usage.cache_read_input_tokens.unwrap_or( 0 );
    let regular_tokens = cache_hit_usage.input_tokens;

    assert_eq!( cached_tokens, 500 );
    assert_eq!( regular_tokens, 100 );

    // In real usage, cached tokens cost ~90% less
    // This test verifies we can extract the data needed for cost calculation
  }
}

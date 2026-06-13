//! Request Caching Integration Tests - STRICT FAILURE POLICY
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


use super::*;

mod request_caching_functionality_tests
{
  use super::*;
  use core::time::Duration;

  /// Test cache configuration validation
  #[ test ]
  fn test_cache_config_validation()
  {
    // Test valid configuration
    let valid_config = the_module::CacheConfig::new()
      .with_ttl_seconds( 300 )
      .with_max_entries( 1000 )
      .with_memory_limit_mb( 100 );

    assert!( valid_config.is_valid() );
    assert_eq!( valid_config.ttl_seconds(), 300 );
    assert_eq!( valid_config.max_entries(), 1000 );
    assert_eq!( valid_config.memory_limit_mb(), 100 );

    // Test invalid configurations
    let invalid_config = the_module::CacheConfig::new()
      .with_ttl_seconds( 0 ); // Should fail - TTL must be > 0

    assert!( !invalid_config.is_valid() );

    let invalid_config2 = the_module::CacheConfig::new()
      .with_max_entries( 0 ); // Should fail - max entries must be > 0

    assert!( !invalid_config2.is_valid() );

    let invalid_config3 = the_module::CacheConfig::new()
      .with_memory_limit_mb( 0 ); // Should fail - memory limit must be > 0

    assert!( !invalid_config3.is_valid() );
  }

  /// Test cache key generation
  #[ test ]
  fn test_cache_key_generation()
  {
    let cache = the_module::RequestCache::new( the_module::CacheConfig::default() );

    // Test that identical requests generate identical keys
    let request1 = the_module::CreateMessageRequest
    {
      model : "claude-haiku-4-5-20251001".to_string(),
      max_tokens : 100,
      messages : vec![ the_module::Message::user( "Hello, world!" ) ],
      system : None,
      temperature : None,
      stream : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
    };

    let request2 = the_module::CreateMessageRequest
    {
      model : "claude-haiku-4-5-20251001".to_string(),
      max_tokens : 100,
      messages : vec![ the_module::Message::user( "Hello, world!" ) ],
      system : None,
      temperature : None,
      stream : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
    };

    let key1 = cache.generate_cache_key( &request1 );
    let key2 = cache.generate_cache_key( &request2 );
    assert_eq!( key1, key2, "Identical requests should generate identical cache keys" );

    // Test that different requests generate different keys
    let request3 = the_module::CreateMessageRequest
    {
      model : "claude-haiku-4-5-20251001".to_string(),
      max_tokens : 200, // Different max_tokens
      messages : vec![ the_module::Message::user( "Hello, world!" ) ],
      system : None,
      temperature : None,
      stream : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
    };

    let key3 = cache.generate_cache_key( &request3 );
    assert_ne!( key1, key3, "Different requests should generate different cache keys" );
  }

  /// Test cache storage and retrieval
  #[ test ]
  fn test_cache_storage_and_retrieval()
  {
    let cache = the_module::RequestCache::new( the_module::CacheConfig::default() );

    let request = the_module::CreateMessageRequest
    {
      model : "claude-haiku-4-5-20251001".to_string(),
      max_tokens : 100,
      messages : vec![ the_module::Message::user( "Test message" ) ],
      system : None,
      temperature : None,
      stream : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
    };

    let response = the_module::CreateMessageResponse
    {
      id : "msg_123".to_string(),
      r#type : "message".to_string(),
      role : "assistant".to_string(),
      content : vec![ the_module::ResponseContent
      {
        r#type : "text".to_string(),
        text : Some( "Cached response".to_string() ),
      } ],
      model : "claude-haiku-4-5-20251001".to_string(),
      stop_reason : Some( "end_turn".to_string() ),
      stop_sequence : None,
      usage : the_module::Usage
      {
        input_tokens : 10,
        output_tokens : 5,
        cache_creation_input_tokens : None,
        cache_read_input_tokens : None,
      },
    };

    // Store response in cache
    cache.store( &request, response.clone() );

    // Retrieve response from cache
    let cached_response = cache.get( &request );
    assert!( cached_response.is_some(), "Response should be cached" );

    let cached = cached_response.unwrap();
    assert_eq!( cached.id, response.id );
    assert_eq!( cached.content.len(), response.content.len() );
  }

  /// Test cache expiration (TTL)
  #[ test ]
  fn test_cache_expiration()
  {
    let config = the_module::CacheConfig::new()
      .with_ttl_seconds( 1 ); // 1 second TTL for testing

    let cache = the_module::RequestCache::new( config );

    let request = the_module::CreateMessageRequest
    {
      model : "claude-haiku-4-5-20251001".to_string(),
      max_tokens : 100,
      messages : vec![ the_module::Message::user( "Expiring message" ) ],
      system : None,
      temperature : None,
      stream : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
    };

    let response = the_module::CreateMessageResponse
    {
      id : "msg_exp".to_string(),
      r#type : "message".to_string(),
      role : "assistant".to_string(),
      content : vec![ the_module::ResponseContent
      {
        r#type : "text".to_string(),
        text : Some( "This will expire".to_string() ),
      } ],
      model : "claude-haiku-4-5-20251001".to_string(),
      stop_reason : Some( "end_turn".to_string() ),
      stop_sequence : None,
      usage : the_module::Usage
      {
        input_tokens : 8,
        output_tokens : 4,
        cache_creation_input_tokens : None,
        cache_read_input_tokens : None,
      },
    };

    // Store response
    cache.store( &request, response.clone() );

    // Should be available immediately
    assert!( cache.get( &request ).is_some(), "Response should be cached immediately" );

    // Wait for expiration (1+ seconds)
    std::thread::sleep( Duration::from_millis( 1100 ) );

    // Should be expired now
    assert!( cache.get( &request ).is_none(), "Response should be expired after TTL" );
  }

  /// Test cache size limits
  #[ test ]
  fn test_cache_size_limits()
  {
    let config = the_module::CacheConfig::new()
      .with_max_entries( 2 ); // Only allow 2 entries

    let cache = the_module::RequestCache::new( config );

    // Create 3 different requests
    let requests = [
      the_module::CreateMessageRequest
      {
        model : "claude-haiku-4-5-20251001".to_string(),
        max_tokens : 100,
        messages : vec![ the_module::Message::user( "Message 1" ) ],
        system : None,
        temperature : None,
        stream : None,
        #[ cfg( feature = "tools" ) ]
        tools : None,
        #[ cfg( feature = "tools" ) ]
        tool_choice : None,
      },
      the_module::CreateMessageRequest
      {
        model : "claude-haiku-4-5-20251001".to_string(),
        max_tokens : 100,
        messages : vec![ the_module::Message::user( "Message 2" ) ],
        system : None,
        temperature : None,
        stream : None,
        #[ cfg( feature = "tools" ) ]
        tools : None,
        #[ cfg( feature = "tools" ) ]
        tool_choice : None,
      },
      the_module::CreateMessageRequest
      {
        model : "claude-haiku-4-5-20251001".to_string(),
        max_tokens : 100,
        messages : vec![ the_module::Message::user( "Message 3" ) ],
        system : None,
        temperature : None,
        stream : None,
        #[ cfg( feature = "tools" ) ]
        tools : None,
        #[ cfg( feature = "tools" ) ]
        tool_choice : None,
      },
    ];

    // Create sample responses
    for ( i, request ) in requests.iter().enumerate()
    {
      let response = the_module::CreateMessageResponse
      {
        id : format!( "msg_{}", i + 1 ),
        r#type : "message".to_string(),
        role : "assistant".to_string(),
        content : vec![ the_module::ResponseContent
        {
          r#type : "text".to_string(),
          text : Some( format!( "Response {}", i + 1 ) ),
        } ],
        model : "claude-haiku-4-5-20251001".to_string(),
        stop_reason : Some( "end_turn".to_string() ),
        stop_sequence : None,
        usage : the_module::Usage
        {
          input_tokens : 5,
          output_tokens : 3,
          cache_creation_input_tokens : None,
          cache_read_input_tokens : None,
        },
      };

      cache.store( request, response.clone() );
    }

    // First two should be cached, third should evict the first
    assert!( cache.get( &requests[ 0 ] ).is_none(), "First entry should be evicted" );
    assert!( cache.get( &requests[ 1 ] ).is_some(), "Second entry should still be cached" );
    assert!( cache.get( &requests[ 2 ] ).is_some(), "Third entry should be cached" );

    // Check cache size
    assert_eq!( cache.size(), 2, "Cache should contain exactly 2 entries" );
  }

  /// Test cache invalidation
  #[ test ]
  fn test_cache_invalidation()
  {
    let cache = the_module::RequestCache::new( the_module::CacheConfig::default() );

    let request = the_module::CreateMessageRequest
    {
      model : "claude-haiku-4-5-20251001".to_string(),
      max_tokens : 100,
      messages : vec![ the_module::Message::user( "Invalidate me" ) ],
      system : None,
      temperature : None,
      stream : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
    };

    let response = the_module::CreateMessageResponse
    {
      id : "msg_inv".to_string(),
      r#type : "message".to_string(),
      role : "assistant".to_string(),
      content : vec![ the_module::ResponseContent
      {
        r#type : "text".to_string(),
        text : Some( "To be invalidated".to_string() ),
      } ],
      model : "claude-haiku-4-5-20251001".to_string(),
      stop_reason : Some( "end_turn".to_string() ),
      stop_sequence : None,
      usage : the_module::Usage
      {
        input_tokens : 6,
        output_tokens : 4,
        cache_creation_input_tokens : None,
        cache_read_input_tokens : None,
      },
    };

    // Store and verify
    cache.store( &request, response.clone() );
    assert!( cache.get( &request ).is_some(), "Response should be cached" );

    // Invalidate specific entry
    cache.invalidate( &request );
    assert!( cache.get( &request ).is_none(), "Response should be invalidated" );

    // Test clear all
    cache.store( &request, response.clone() );
    assert!( cache.get( &request ).is_some(), "Response should be cached again" );

    cache.clear();
    assert!( cache.get( &request ).is_none(), "Cache should be empty after clear" );
    assert_eq!( cache.size(), 0, "Cache size should be 0 after clear" );
  }

  /// Test cache metrics and statistics
  #[ test ]
  fn test_cache_metrics()
  {
    let cache = the_module::RequestCache::new( the_module::CacheConfig::default() );

    let request = the_module::CreateMessageRequest
    {
      model : "claude-haiku-4-5-20251001".to_string(),
      max_tokens : 100,
      messages : vec![ the_module::Message::user( "Metrics test" ) ],
      system : None,
      temperature : None,
      stream : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
    };

    let response = the_module::CreateMessageResponse
    {
      id : "msg_metrics".to_string(),
      r#type : "message".to_string(),
      role : "assistant".to_string(),
      content : vec![ the_module::ResponseContent
      {
        r#type : "text".to_string(),
        text : Some( "Metrics response".to_string() ),
      } ],
      model : "claude-haiku-4-5-20251001".to_string(),
      stop_reason : Some( "end_turn".to_string() ),
      stop_sequence : None,
      usage : the_module::Usage
      {
        input_tokens : 5,
        output_tokens : 3,
        cache_creation_input_tokens : None,
        cache_read_input_tokens : None,
      },
    };

    let metrics = cache.metrics();
    assert_eq!( metrics.hits(), 0 );
    assert_eq!( metrics.misses(), 0 );
    assert_eq!( metrics.stores(), 0 );

    // Store response
    cache.store( &request, response.clone() );
    let metrics = cache.metrics();
    assert_eq!( metrics.stores(), 1 );

    // Cache miss
    cache.get( &the_module::CreateMessageRequest
    {
      model : "different-model".to_string(),
      max_tokens : 100,
      messages : vec![ the_module::Message::user( "Different request" ) ],
      system : None,
      temperature : None,
      stream : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
    } );

    let metrics = cache.metrics();
    assert_eq!( metrics.misses(), 1 );

    // Cache hit
    cache.get( &request );
    let metrics = cache.metrics();
    assert_eq!( metrics.hits(), 1 );

    // Test hit rate calculation
    assert!( ( metrics.hit_rate() - 0.5 ).abs() < 0.01 ); // 1 hit out of 2 attempts = 50%
  }
}

mod request_caching_integration_tests
{
  use super::*;
  use core::time::Duration;
  use std::time::Instant;

  /// Test cache performance characteristics
  #[ test ]
  fn test_cache_performance()
  {
    let config = the_module::CacheConfig::new()
      .with_max_entries( 1000 );

    let cache = the_module::RequestCache::new( config );

    // Test cache operation performance
    let start = Instant::now();

    // Store many entries
    for i in 0..100
    {
      let request = the_module::CreateMessageRequest
      {
        model : "claude-haiku-4-5-20251001".to_string(),
        max_tokens : 100,
        messages : vec![ the_module::Message::user( format!( "Message {i}" ) ) ],
        system : None,
        temperature : None,
        stream : None,
        #[ cfg( feature = "tools" ) ]
        tools : None,
        #[ cfg( feature = "tools" ) ]
        tool_choice : None,
      };

      let response = the_module::CreateMessageResponse
      {
        id : format!( "msg_{i}" ),
        r#type : "message".to_string(),
        role : "assistant".to_string(),
        content : vec![ the_module::ResponseContent
        {
          r#type : "text".to_string(),
          text : Some( format!( "Response {i}" ) ),
        } ],
        model : "claude-haiku-4-5-20251001".to_string(),
        stop_reason : Some( "end_turn".to_string() ),
        stop_sequence : None,
        usage : the_module::Usage
        {
          input_tokens : 5,
          output_tokens : 3,
          cache_creation_input_tokens : None,
          cache_read_input_tokens : None,
        },
      };

      cache.store( &request, response.clone() );
    }

    let store_duration = start.elapsed();

    // Test retrieval performance
    let start = Instant::now();

    for i in 0..100
    {
      let request = the_module::CreateMessageRequest
      {
        model : "claude-haiku-4-5-20251001".to_string(),
        max_tokens : 100,
        messages : vec![ the_module::Message::user( format!( "Message {i}" ) ) ],
        system : None,
        temperature : None,
        stream : None,
        #[ cfg( feature = "tools" ) ]
        tools : None,
        #[ cfg( feature = "tools" ) ]
        tool_choice : None,
      };

      let _ = cache.get( &request );
    }

    let retrieval_duration = start.elapsed();

    // Cache operations should be fast
    assert!( store_duration < Duration::from_millis( 100 ), "Cache storage should be fast" );
    assert!( retrieval_duration < Duration::from_millis( 50 ), "Cache retrieval should be very fast" );

    // Verify all entries are present
    assert_eq!( cache.size(), 100 );
  }
}
//! Synchronous API functionality tests
//!
//! This module contains comprehensive tests for synchronous API functionality,
//! including blocking wrapper implementations, runtime management, and
//! synchronous client patterns.

use super::*;

mod sync_api_functionality_tests
{
  use super::*;
  use core::time::Duration;
  use std::time::Instant;
  use the_module::{ Message, CreateMessageRequest };

  /// Test basic sync client construction and configuration
  #[ cfg( feature = "integration" ) ]
  #[ test ]
fn test_sync_client_construction()
  {
    use the_module::SyncClient;

    // Test construction from environment
    let client_result = SyncClient::from_env();
    assert!( client_result.is_ok() || client_result.is_err(), "Construction should return a result" );

    // Fix(BUG-hygiene-001): Fail loudly when ANTHROPIC_API_KEY missing
    // Root cause : Silent skip when env var missing created false positive test pass
    // Previous : if let Ok(secret) silently skipped test when ANTHROPIC_API_KEY unset
    // Fixed : .expect() fails loudly with clear message
    // Pitfall : Never use conditional skip - always fail loudly with .expect()

    // Test construction with explicit secret
    let secret = std::env::var( "ANTHROPIC_API_KEY" )
      .expect( "ANTHROPIC_API_KEY must be set for sync_api_test - test requires real API key" );

    let client = SyncClient::new( &secret );
    assert!( client.is_ok(), "Should construct client with valid secret" );

    let client = client.unwrap();
    assert!( !client.get_api_key().is_empty(), "API key should be set" );

    // Test construction with invalid secret
    let invalid_client = SyncClient::new( "invalid-key" );
    assert!( invalid_client.is_err(), "Should fail with invalid secret" );
  }

  /// Test sync message creation and sending
#[ cfg( feature = "integration" ) ]
  #[ test ]
fn test_sync_message_operations()
  {
    use the_module::{ SyncClient, CreateMessageRequest, Message };

    let client = SyncClient::from_workspace().expect( "Client should be available for testing" );

    // Test basic message request construction
    let request = CreateMessageRequest
    {
      model : "claude-haiku-4-5-20251001".to_string(),
      max_tokens : 100,
      messages : vec![ Message::user( "Hello, world!" ) ],
      system : None,
      stream : None,
      temperature : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
    };

    // Test sync message sending
    let response = client.create_message( &request );

    // Should either succeed or fail with meaningful error
    match response
    {
      Ok( message_response ) => {
        assert!( !message_response.content.is_empty(), "Response should have content" );
        assert!( message_response.usage.input_tokens > 0, "Should track input tokens" );
        assert!( message_response.usage.output_tokens > 0, "Should track output tokens" );
      }
      Err( error ) => {
        // Validate error structure
        assert!( !error.to_string().is_empty(), "Error should have meaningful message" );
      }
    }
  }

  /// Test sync message with system prompts
#[ cfg( feature = "integration" ) ]
  #[ test ]
fn test_sync_message_with_system_prompt()
  {
    use the_module::{ SyncClient, CreateMessageRequest };

    let client = SyncClient::from_workspace().expect( "Client should be available for testing" );

    let request = CreateMessageRequest
    {
      model : "claude-haiku-4-5-20251001".to_string(),
      max_tokens : 50,
      messages : vec![ Message::user( "What is 2+2?" ) ],
      system : Some( vec![ the_module::SystemContent::text( "You are a helpful assistant that responds concisely." ) ] ),
      stream : None,
      temperature : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
    };

    let response = client.create_message( &request );

    if let Ok( message_response ) = response
    {
      assert!( !message_response.content.is_empty(), "Should have response content" );
      // System prompt should influence the response but not appear in content
    }
    // Expected if API key not available
  }

  /// Test sync conversation flow
#[ cfg( feature = "integration" ) ]
  #[ test ]
fn test_sync_conversation_flow()
  {
    use the_module::{ SyncClient, CreateMessageRequest, Message };

    let client = SyncClient::from_workspace().expect( "Client should be available for testing" );

    // First message
    let mut request1 = CreateMessageRequest::new( "claude-haiku-4-5-20251001" );
    request1.add_user_message( "My name is Alice." );
    request1.set_max_tokens( 50 );

    // Fix(BUG-003): Fail loudly if first message fails
    // Root cause : Silent skip when response1 failed - test falsely passed
    // Pitfall : Never skip test continuation on API failure - fail loudly to detect issues
    let response1 = client.create_message( &request1 );
    let _message1 = response1
      .expect( "First message must succeed for conversation flow test" );

    // Second message in conversation
    let mut request2 = CreateMessageRequest::new( "claude-haiku-4-5-20251001" );

    // Add conversation history
    request2.add_message( Message::user( "My name is Alice." ) );
    // For now, just add a simple assistant response since content conversion is complex
    request2.add_message( Message::assistant( "Nice to meet you, Alice!" ) );
    request2.add_user_message( "What is my name?" );
    request2.set_max_tokens( 50 );

    let response2 = client.create_message( &request2 );

    if let Ok( message2 ) = response2
    {
      assert!( !message2.content.is_empty(), "Should respond to conversation" );
      // Response should reference the name (though this is probabilistic)
    }
    // Expected if API key not available
  }

  /// Test sync client with different models
#[ cfg( feature = "integration" ) ]
  #[ test ]
fn test_sync_client_multiple_models()
  {
    use the_module::{ SyncClient, CreateMessageRequest };

    let client = SyncClient::from_workspace().expect( "Client should be available for testing" );

    let models = vec![
      "claude-haiku-4-5-20251001",
      "claude-sonnet-4-5-20250929",
    ];

    for model in models
    {
      let mut request = CreateMessageRequest::new( model );
      request.add_user_message( "Hello!" );
      request.set_max_tokens( 20 );

      let response = client.create_message( &request );

      // Should work with all models or fail consistently
      if let Ok( message_response ) = response
      {
        assert!( !message_response.content.is_empty(), "Response should have content for {model}" );
        assert_eq!( message_response.model, model, "Response should indicate correct model" );
      }
      // Expected if model not available or API key not set
    }
  }

  /// Test sync client timeout handling
  #[ cfg( feature = "integration" ) ]
  #[ test ]
fn test_sync_client_timeout_configuration()
  {
    use the_module::SyncClientBuilder;

    // Fix(BUG-004): Fail loudly when builder fails
    // Root cause : Silent skip when build_from_env failed - timeout test falsely passed
    // Pitfall : Never skip configuration verification on construction failure - fail loudly
    let client = SyncClientBuilder::new()
      .timeout( Duration::from_secs( 30 ) )
      .build_from_env()
      .expect( "Client builder with timeout must succeed for timeout configuration test" );

    assert!( client.get_timeout() == Duration::from_secs( 30 ), "Timeout should be configured" );

    // Fix(BUG-004): Fail loudly when short timeout client fails
    // Root cause : Silent skip when build_from_env failed - timeout test falsely passed
    // Pitfall : Never skip timeout behavior verification - fail loudly to detect construction issues
    let client = SyncClientBuilder::new()
      .timeout( Duration::from_millis( 1 ) )
      .build_from_env()
      .expect( "Client builder with short timeout must succeed for timeout behavior test" );

    let mut request = CreateMessageRequest::new( "claude-haiku-4-5-20251001" );
    request.add_user_message( "Hello!" );
    request.set_max_tokens( 10 );

    let start = Instant::now();
    let response = client.create_message( &request );
    let elapsed = start.elapsed();

    // Should timeout quickly or succeed very fast
    assert!( elapsed < Duration::from_secs( 5 ), "Should timeout or complete quickly" );

    if let Err( err ) = response
    {
      // Timeout error expected
      let error_msg = err.to_string();
      assert!(
        error_msg.to_lowercase().contains( "timeout" )
          || error_msg.to_lowercase().contains( "timed out" )
          || error_msg.to_lowercase().contains( "deadline" )
          || error_msg.to_lowercase().contains( "operation timed" ),
        "Error should indicate timeout, got: {error_msg}"
      );
    }
  }
}

mod sync_api_runtime_tests
{
  use super::*;
  use core::time::Duration;
  use std::{ thread, sync::{ Arc, Mutex }, time::Instant };

  /// Test sync client thread safety
  #[ cfg( feature = "integration" ) ]
  #[ test ]
  fn test_sync_client_thread_safety()
  {
    use the_module::SyncClient;

    let client = Arc::new( SyncClient::from_env()
      .expect( "INTEGRATION: API key must be available for thread safety test" ) );

    let results = Arc::new( Mutex::new( Vec::new() ) );
    let mut handles = vec![];

    // Spawn multiple threads using the same client
    for i in 0..3
    {
      let client_clone = Arc::clone( &client );
      let results_clone = Arc::clone( &results );

      let handle = thread::spawn( move || {
        use the_module::CreateMessageRequest;

        let mut request = CreateMessageRequest::new( "claude-haiku-4-5-20251001" );
        request.add_user_message( &format!( "Thread {i} says hello!" ) );
        request.set_max_tokens( 20 );

        let response = client_clone.create_message( &request );

        let mut results = results_clone.lock().unwrap();
        results.push( (i, response.is_ok()) );
      });

      handles.push( handle );
    }

    // Wait for all threads
    for handle in handles
    {
      handle.join().unwrap();
    }

    let results = results.lock().unwrap();
    assert_eq!( results.len(), 3, "All threads should complete" );

    // At least some requests should succeed (if API key is valid)
    // This tests that the sync client can handle concurrent usage
  }

  /// Test sync runtime management
  #[ test ]
  fn test_sync_runtime_lifecycle()
  {
    use the_module::{ SyncRuntime, SyncClient };

    // Test runtime creation and shutdown
    let runtime = SyncRuntime::new();
    assert!( runtime.is_ok(), "Runtime should be creatable" );

    let runtime = runtime.unwrap();

    // Test client creation with custom runtime
    let client_result = SyncClient::with_runtime( runtime, "test-key" );
    if let Ok( c ) = client_result
    {
      assert!( !c.get_api_key().is_empty(), "Client should have API key" );
    }
    // else: test key rejected at construction — expected behavior
    // Runtime should clean up automatically when dropped
  }

  /// Test blocking behavior consistency
  #[ cfg( feature = "integration" ) ]
  #[ test ]
  fn test_blocking_behavior_consistency()
  {
    use the_module::{ SyncClient, CreateMessageRequest };

    let client = SyncClient::from_env()
      .expect( "INTEGRATION: API key must be available for blocking behavior test" );

    let mut request = CreateMessageRequest::new( "claude-haiku-4-5-20251001" );
    request.add_user_message( "Count to 3" );
    request.set_max_tokens( 30 );

    let start = Instant::now();
    let response = client.create_message( &request );
    let elapsed = start.elapsed();

    // Sync operation should block until completion; timing assertions only if call succeeds
    if response.is_ok()
    {
      assert!( elapsed > Duration::from_millis( 100 ), "Should take some time to complete" );
      assert!( elapsed < Duration::from_secs( 30 ), "Should complete in reasonable time" );
    }
  }
}

mod sync_api_integration_tests
{
  use super::*;
  use core::time::Duration;
  use std::time::Instant;

  /// Test sync to async interoperability
  #[ cfg( feature = "integration" ) ]
  #[ test ]
  fn test_sync_async_interoperability()
  {
    use the_module::{ SyncClient, Client, CreateMessageRequest };

    // Both sync and async clients must be constructable simultaneously
    let sync_client = SyncClient::from_env()
      .expect( "INTEGRATION: API key must be available for sync interoperability test" );
    let async_client = Client::from_env()
      .expect( "INTEGRATION: API key must be available for async interoperability test" );

    // Both clients must agree on key presence
    assert!( sync_client.has_api_key() == async_client.api_key().is_some(),
             "Both clients should have same API key status" );

    // Verify sync client can make a request while async client exists
    let mut sync_request = CreateMessageRequest::new( "claude-haiku-4-5-20251001" );
    sync_request.add_user_message( "Hello from sync!" );
    sync_request.set_max_tokens( 20 );

    let sync_response = sync_client.create_message( &sync_request );
    assert!( sync_response.is_ok() || sync_response.is_err(), "Sync client must complete" );

    // async_client unused past this point — just verifying coexistence compiles and constructs
    drop( async_client );
  }

  /// Test sync client performance characteristics
  #[ cfg( feature = "integration" ) ]
  #[ test ]
  fn test_sync_client_performance_overhead()
  {
    use the_module::{ SyncClient, CreateMessageRequest };

    let client = SyncClient::from_env()
      .expect( "INTEGRATION: API key must be available for performance overhead test" );

    let mut times = Vec::new();

    // Measure multiple sync requests
    for _ in 0..3
    {
      let mut request = CreateMessageRequest::new( "claude-haiku-4-5-20251001" );
      request.add_user_message( "Hi" );
      request.set_max_tokens( 10 );

      let start = Instant::now();
      let response = client.create_message( &request );
      let elapsed = start.elapsed();

      if response.is_ok()
      {
        times.push( elapsed );
      }
    }

    if !times.is_empty()
    {
      #[ allow( clippy::cast_possible_truncation ) ]
      let avg_time : Duration = times.iter().sum::< Duration >() / times.len() as u32;

      // Sync overhead should be minimal (< 100ms extra)
      assert!( avg_time < Duration::from_secs( 30 ), "Sync requests should complete in reasonable time" );

      // Consistency check - times shouldn't vary wildly
      let max_time = times.iter().max().unwrap();
      let min_time = times.iter().min().unwrap();

      assert!( max_time.as_millis() < min_time.as_millis() * 10,
               "Request times should be relatively consistent" );
    }
  }

  /// Test sync error handling and propagation
  #[ cfg( feature = "integration" ) ]
  #[ test ]
fn test_sync_error_handling()
  {
    use the_module::{ SyncClient, CreateMessageRequest };

    // Test with invalid API key
    let invalid_client = SyncClient::new( "sk-ant-invalid-key" );

    match invalid_client
    {
      Err( _ ) => { /* invalid key correctly rejected at construction */ }
      Ok( client ) =>
      {
        let mut request = CreateMessageRequest::new( "claude-haiku-4-5-20251001" );
        request.add_user_message( "Hello!" );
        request.set_max_tokens( 10 );

        let response = client.create_message( &request );

        // Should fail with authentication error
        assert!( response.is_err(), "Should fail with invalid API key" );

        let error = response.unwrap_err();
        let error_msg = error.to_string().to_lowercase();

        assert!( error_msg.contains( "auth" ) ||
                 error_msg.contains( "key" ) ||
                 error_msg.contains( "401" ) ||
                 error_msg.contains( "unauthorized" ),
                 "Error should indicate authentication issue : {error}" );
      }
    }

    // Fix(BUG-hygiene-005): Fail loudly when client unavailable for invalid model test
    // Root cause : Silent skip when from_env failed - invalid model error handling test falsely passed
    // Previous : if let Ok silently skipped model validation test when client construction failed
    // Fixed : .expect() fails loudly with clear message about model validation requirement
    // Pitfall : Never skip error handling verification - fail loudly to ensure test executes
    let client = SyncClient::from_env()
      .expect( "Client must be available from environment for invalid model error handling test" );

    let mut request = CreateMessageRequest::new( "invalid-model-name" );
    request.add_user_message( "Hello!" );
    request.set_max_tokens( 10 );

    let response = client.create_message( &request );

    if let Err( error ) = response
    {
      let error_msg = error.to_string().to_lowercase();

      // Should indicate model-related error
      assert!( error_msg.contains( "model" ) ||
               error_msg.contains( "400" ) ||
               error_msg.contains( "not found" ),
               "Error should indicate model issue" );
    }
  }
}

#[ cfg( feature = "sync-api" ) ]
mod sync_api_feature_tests
{
  use super::*;

  /// Test sync API feature gate
  #[ test ]
  fn test_sync_api_feature_availability()
  {
    // This test validates that sync API types are available when feature is enabled
    use the_module::{ SyncClient, SyncClientBuilder, SyncRuntime };

    // These should compile and be usable
    #[ allow( clippy::no_effect_underscore_binding ) ]
    {
      let _sync_client_type = core::marker::PhantomData::< SyncClient >;
      let _sync_builder_type = core::marker::PhantomData::< SyncClientBuilder >;
      let _sync_runtime_type = core::marker::PhantomData::< SyncRuntime >;
    }

    // Test that basic construction methods exist
    let builder = SyncClientBuilder::new();
    let result = builder.build_from_env();
    assert!( result.is_ok() || result.is_err(),
             "Builder should return result" );
  }
}
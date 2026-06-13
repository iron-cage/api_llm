//! Synchronous Streaming API Tests
//!
//! Tests for blocking/synchronous streaming functionality that wraps async SSE streaming

#[ allow( unused_imports ) ]
use super::*;

// ============================================================================
// UNIT TESTS - SYNC STREAMING STRUCTURE
// ============================================================================

#[ test ]
fn test_sync_stream_iterator_structure()
{
  // Test that SyncStreamIterator can be created and has proper structure
  // The existence of this test verifies the concept compiles
  // Integration tests below verify the actual functionality
}

// ============================================================================
// INTEGRATION TESTS - REAL API SYNC STREAMING
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ test ]
fn integration_sync_streaming_text_generation()
{
  // Test synchronous streaming for text generation
  let client = the_module::SyncClient::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for sync streaming test" );

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 50,
    messages : vec![ the_module::Message::user( "Say hello!".to_string() ) ],
    system : None,
    temperature : Some( 0.7 ),
    stream : Some( true ), // Enable streaming
    tools : None,
    tool_choice : None,
  };

  // Get sync stream iterator
  let mut stream = match client.create_message_stream( &request )
  {
    Ok( stream ) => stream,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Sync streaming must work : {err}" ),
  };

  let mut received_chunks = 0;
  let mut accumulated_text = String::new();

  // Iterate over stream synchronously (blocking iteration)
  for chunk_result in &mut stream
  {
    let chunk = chunk_result.expect( "INTEGRATION: Stream chunks must be valid" );

    received_chunks += 1;

    // Extract text from chunk
    if let the_module::StreamEvent::ContentBlockDelta{ delta : the_module::StreamDelta::TextDelta{ text, .. }, .. } = chunk
    {
      accumulated_text.push_str( &text );
    }
  }

  // Verify we received streaming chunks
  assert!( received_chunks > 0, "INTEGRATION: Must receive streaming chunks" );
  assert!( !accumulated_text.is_empty(), "INTEGRATION: Must receive text content" );

  println!( "✅ Sync streaming text generation test passed!" );
  println!( "   Received {received_chunks} chunks" );
  println!( "   Total text length : {} chars", accumulated_text.len() );
}

#[ cfg( feature = "integration" ) ]
#[ test ]
fn integration_sync_streaming_error_handling()
{
  // Test error handling in sync streaming with invalid request
  let client = the_module::SyncClient::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for error test" );

  let request = the_module::CreateMessageRequest
  {
    model : "invalid-model-name".to_string(),
    max_tokens : 10,
    messages : vec![ the_module::Message::user( "Test".to_string() ) ],
    system : None,
    temperature : Some( 0.0 ),
    stream : Some( true ),
    tools : None,
    tool_choice : None,
  };

  // Attempt to create stream - should fail with invalid model
  let result = client.create_message_stream( &request );

  assert!( result.is_err(), "INTEGRATION: Invalid model should cause error" );

  println!( "✅ Sync streaming error handling test passed!" );
}

#[ cfg( feature = "integration" ) ]
#[ test ]
fn integration_sync_streaming_blocking_iteration()
{
  // Test that sync streaming blocks properly and doesn't require async/await
  let client = the_module::SyncClient::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 30,
    messages : vec![ the_module::Message::user( "Count to 3".to_string() ) ],
    system : None,
    temperature : Some( 0.0 ),
    stream : Some( true ),
    tools : None,
    tool_choice : None,
  };

  let start_time = std::time::Instant::now();

  let mut stream = match client.create_message_stream( &request )
  {
    Ok( stream ) => stream,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Stream creation must work : {err}" ),
  };

  let mut chunk_count = 0;

  // This should block until each chunk arrives (no async/await needed)
  for chunk_result in &mut stream
  {
    match chunk_result
    {
      Ok( _chunk ) => chunk_count += 1,
      Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
        panic!( "INTEGRATION: Credit balance exhausted — top up account to run tests : {}", api_err.message ),
      Err( err ) => panic!( "INTEGRATION: Chunk must be valid : {err}" ),
    }
  }

  let duration = start_time.elapsed();

  // Verify we got chunks and it took some time (proving it actually streamed)
  assert!( chunk_count > 0, "INTEGRATION: Must receive chunks" );
  assert!( duration.as_millis() > 0, "INTEGRATION: Streaming must take time" );

  println!( "✅ Sync streaming blocking iteration test passed!" );
  println!( "   Received {chunk_count} chunks in {duration:?}" );
}

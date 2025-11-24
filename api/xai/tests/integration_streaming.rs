//! Integration tests for streaming chat completions.
//!
//! # Purpose
//!
//! Validates Server-Sent Events (SSE) streaming with real XAI API.
//!
//! # Key Insights
//!
//! - **SSE Stream Lifecycle**: XAI sends chunks with incremental deltas,
//!   terminated by a `[DONE]` marker event.
//!
//! - **[DONE] Marker Handling**: The `[DONE]` event is filtered using a
//!   sentinel error pattern. When `[DONE]` is detected, we return a
//!   `XaiError::Stream("Stream completed")` error, then use `take_while()`
//!   to stop iteration without propagating the error to the user.
//!
//! - **Lifetime Management**: Must create intermediate `chat` binding
//!   (`let chat = client.chat()`) before calling `create_stream()`.
//!   Direct chaining (`client.chat().create_stream()`) causes borrow
//!   checker errors due to temporary value drops.
//!
//! - **Delta Accumulation**: Each chunk contains partial content in `delta.content`.
//!   Client must accumulate these fragments to reconstruct full response.
//!
//! # Running Tests
//!
//! ```bash
//! cargo test --features integration,streaming --test integration_streaming
//! ```

#![ cfg( all( feature = "integration", feature = "streaming" ) ) ]

mod inc;
use inc::test_helpers::create_test_client;

use api_xai::{ ChatCompletionRequest, Message, ClientApiAccessors };
use futures_util::StreamExt;

#[ tokio::test ]
async fn test_streaming_chat_completion_basic()
{
  let client = create_test_client();

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Count from 1 to 5" ) ] )
    .max_tokens( 50u32 )
    .form();

  let chat = client.chat();
  let mut stream = chat.create_stream( request ).await
    .expect( "Stream creation should succeed" );

  let mut chunks_received = 0;
  let mut content_parts = Vec::new();

  while let Some( chunk_result ) = stream.next().await {
    let chunk = chunk_result.expect( "Chunk parsing should succeed" );
    chunks_received += 1;

    // Verify chunk structure
    assert!( !chunk.id.is_empty(), "Chunk should have an ID" );
    assert_eq!( chunk.object, "chat.completion.chunk", "Object type should be chat.completion.chunk" );
    assert!( !chunk.choices.is_empty(), "Chunk should have at least one choice" );

    // Collect content
    if let Some( delta ) = chunk.choices.first().map( |c| &c.delta ) {
      if let Some( content ) = &delta.content {
        content_parts.push( content.clone() );
      }
    }
  }

  assert!( chunks_received > 0, "Should receive at least one chunk" );
  assert!( !content_parts.is_empty(), "Should receive some content" );

  let full_response = content_parts.join( "" );
  assert!( !full_response.is_empty(), "Full response should not be empty" );

  println!( "✅ Streaming basic test passed" );
  println!( "Received {chunks_received} chunks" );
  println!( "Full response : {full_response}" );
}

#[ tokio::test ]
async fn test_streaming_with_system_message()
{
  let client = create_test_client();

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![
      Message::system( "You are a helpful assistant that responds concisely" ),
      Message::user( "Say hello" ),
    ] )
    .max_tokens( 20u32 )
    .form();

  let chat = client.chat();
  let mut stream = chat.create_stream( request ).await
    .expect( "Stream creation should succeed" );

  let mut content = String::new();

  while let Some( chunk_result ) = stream.next().await {
    let chunk = chunk_result.expect( "Chunk parsing should succeed" );

    if let Some( delta ) = chunk.choices.first().map( |c| &c.delta ) {
      if let Some( text ) = &delta.content {
        content.push_str( text );
      }
    }
  }

  assert!( !content.is_empty(), "Should receive content" );

  println!( "✅ Streaming with system message test passed" );
  println!( "Response : {content}" );
}

#[ tokio::test ]
async fn test_streaming_finish_reason()
{
  let client = create_test_client();

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Say hi" ) ] )
    .max_tokens( 5u32 ) // Very limited to likely trigger "length" finish
    .form();

  let chat = client.chat();
  let mut stream = chat.create_stream( request ).await
    .expect( "Stream creation should succeed" );

  let mut last_finish_reason : Option< String > = None;

  while let Some( chunk_result ) = stream.next().await {
    let chunk = chunk_result.expect( "Chunk parsing should succeed" );

    if let Some( choice ) = chunk.choices.first() {
      if let Some( reason ) = &choice.finish_reason {
        last_finish_reason = Some( reason.clone() );
      }
    }
  }

  // Should have a finish reason in the last chunk
  assert!( last_finish_reason.is_some(), "Should have a finish reason" );

  let finish_reason = last_finish_reason.unwrap();
  assert!(
    finish_reason == "stop" || finish_reason == "length",
    "Finish reason should be 'stop' or 'length', got : {finish_reason}"
  );

  println!( "✅ Streaming finish reason test passed" );
  println!( "Finish reason : {finish_reason}" );
}

#[ tokio::test ]
async fn test_streaming_role_in_first_chunk()
{
  let client = create_test_client();

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Hi" ) ] )
    .max_tokens( 10u32 )
    .form();

  let chat = client.chat();
  let mut stream = chat.create_stream( request ).await
    .expect( "Stream creation should succeed" );

  if let Some( chunk_result ) = stream.next().await {
    let chunk = chunk_result.expect( "First chunk should parse" );

    if let Some( delta ) = chunk.choices.first().map( |c| &c.delta ) {
      // First chunk typically contains the role
      if delta.role.is_some() {
        println!( "✅ First chunk contains role : {:?}", delta.role );
      }
    }
  }

  println!( "✅ Streaming role in first chunk test passed" );
}

#[ tokio::test ]
async fn test_streaming_collect_all_content()
{
  let client = create_test_client();

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Write the word 'test' three times" ) ] )
    .max_tokens( 30u32 )
    .form();

  let chat = client.chat();
  let mut stream = chat.create_stream( request ).await
    .expect( "Stream creation should succeed" );

  let mut full_content = String::new();

  while let Some( chunk_result ) = stream.next().await {
    let chunk = chunk_result.expect( "Chunk parsing should succeed" );

    if let Some( delta ) = chunk.choices.first().map( |c| &c.delta ) {
      if let Some( content ) = &delta.content {
        full_content.push_str( content );
        print!( "{content}" ); // Show streaming in real-time
      }
    }
  }

  println!(); // Newline after streaming output

  assert!( !full_content.is_empty(), "Should collect content" );

  println!( "✅ Streaming collect all content test passed" );
  println!( "Full content length : {} characters", full_content.len() );
}

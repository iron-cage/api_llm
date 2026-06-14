//! Streaming functionality tests for `api_ollama` crate with managed test server.
//!
//! # MANDATORY STRICT FAILURE POLICY
//! 
//! **⚠️  CRITICAL: These integration tests MUST fail loudly and immediately on any issues:**
//! 
//! - **Real API Only**: Tests make actual HTTP streaming requests to live Ollama servers, never mocks
//! - **No Graceful Degradation**: Missing servers, network issues, or stream failures cause immediate test failure
//! - **Required Dependencies**: Ollama server must be available with streaming support enabled
//! - **Explicit Configuration**: Tests require explicit server setup and fail if unavailable
//! - **Deterministic Failures**: Identical conditions must produce identical pass/fail results
//! - **End-to-End Validation**: Tests validate actual streaming responses from real server
//! 
//! These tests require both 'streaming' and 'integration' features and automatically 
//! manage their own Ollama server instance. Server unavailability, streaming failures,
//! or network issues WILL cause test failures - this is mandatory per specification NFR-9.1 through NFR-9.8.

#![ cfg( all( feature = "streaming", feature = "integration", feature = "integration_tests" ) ) ]

mod server_helpers;

use api_ollama::{ 
  OllamaClient, 
  ChatMessage,
  MessageRole,
  ChatRequest
};
use core::time::Duration;
use futures_util::StreamExt;
#[ tokio::test ]
async fn test_streaming_chat_basic()
{
  with_test_server!(|mut client : OllamaClient, model : String| async move {
    let request = ChatRequest
    {
      model,
      messages : vec![
        ChatMessage
        {
          role : MessageRole::User,
          content : "Count from 1 to 3, one number per response.".to_string(),
          images : None,
          #[ cfg( feature = "tool_calling" ) ]
          tool_calls : None,
        }
      ],
      stream : None, // This will be set to true by the streaming method
      // Fix(issue-unconstrained-generation-003): limit to 10 tokens to prevent OOM.
      // Root cause: unconstrained streaming exhausts swap (57s observed); parse error on final chunk.
      // Pitfall: always set num_predict in streaming tests to bound memory and time.
      options : Some( serde_json::json!( { "num_predict" : 10 } ) ),
      #[ cfg( feature = "tool_calling" ) ]
      tools : None,
      #[ cfg( feature = "tool_calling" ) ]
      tool_messages : None,
    };
    
    // Fix(issue-silent-failure-001): Fail loudly when server unavailable
    // Root cause : Silent skip with println+return created false positive test results
    // Pitfall : Integration tests MUST fail loudly when dependencies unavailable per codebase_hygiene.rulebook.md
    let mut stream = client.chat_stream(request).await
      .expect("Failed to create chat stream - Ollama server must be available for integration tests");
    let mut responses = Vec::new();
    let mut response_count = 0;
    let max_responses = 20; // Prevent infinite loops
    
    // Collect streaming responses
    while let Some(response_result) = stream.next().await
    {
      response_count += 1;
      
      assert!(response_result.is_ok(), "Stream response error : {response_result:?}");
      let response = response_result.unwrap();
      responses.push(response.clone());
      
      if !response.message.content.is_empty()
      {
        response_count += 1;
        println!( "Stream chunk {response_count}: '{}'", response.message.content );
      }
      
      if response.done
      {
        println!( "Streaming completed after {response_count} chunks" );
        break;
      }
      
      // Safety limit to prevent infinite loops
      if response_count > max_responses
      {
        println!( "Streaming stopped at safety limit of {max_responses} responses" );
        break;
      }
    }
    
    assert!(!responses.is_empty(), "No streaming responses received");
    // The test passes if we received responses, whether we got the "done" signal or hit the safety limit
    println!( "Streaming test completed with {} responses", responses.len() );
  });
}

#[ tokio::test ]
async fn test_streaming_chat_error_handling()
{
  let mut client = OllamaClient::new( "http://unreachable.test:99999".to_string(), OllamaClient::recommended_timeout_fast() )
    .with_timeout( Duration::from_millis( 100 ) );
    
  let request = ChatRequest
  {
    model : "test-model".to_string(),
    messages : vec!
    [
      ChatMessage
      {
        role : MessageRole::User,
        content : "Hello".to_string(),
        images : None,
        #[ cfg( feature = "tool_calling" ) ]
        tool_calls : None,
      }
    ],
    stream : None,
    options : None,
    #[ cfg( feature = "tool_calling" ) ]
    tools : None,
    #[ cfg( feature = "tool_calling" ) ]
    tool_messages : None,
  };
  
  let result = client.chat_stream( request ).await;
  assert!( result.is_err(), "Expected error for unreachable server" );
  
  if let Err( error ) = result
  {
    let error_str = format!( "{error}" );
    assert!( error_str.contains( "Network error" ), "Expected network error, got : {error_str}" );
  }
}

#[ cfg( feature = "streaming" ) ]
#[ test ]
fn test_streaming_feature_compilation()
{
  // This test ensures that streaming feature compiles correctly
  // even without an actual server
  let client = OllamaClient::new( "http://test.local:11434".to_string(), OllamaClient::recommended_timeout_fast() );
  let _ = client;
  // Just test that the streaming method exists and compiles
}

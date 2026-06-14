//! Unit tests for streaming functionality

#[ cfg( all( test, feature = "streaming" ) ) ]
mod tests
{
  use api_ollama::{ OllamaClient, ChatRequest, ChatMessage, MessageRole };

  #[ tokio::test ]
  async fn test_streaming_chat_can_initialize()
  {
    // Test that streaming chat can set up its basic structures
    let _client = OllamaClient::new( "http://localhost:11434".to_string(), OllamaClient::recommended_timeout_fast() );
    
    // Test conversation history initialization (from streaming_chat example)
    let mut conversation_history = vec![
      ChatMessage
      {
        role : MessageRole::System,
        content : "You are a helpful assistant. Provide engaging, informative responses. \\
                   When appropriate, use examples and ask follow-up questions to keep \\
                   the conversation flowing.".to_string(),
        images : None,
        #[ cfg( feature = "tool_calling" ) ]
        tool_calls : None,
      }
    ];
    
    assert_eq!( conversation_history.len(), 1 );
    
    // Test adding user message
    conversation_history.push( ChatMessage
    {
      role : MessageRole::User,
      content : "Hello!".to_string(),
      images : None,
      #[ cfg( feature = "tool_calling" ) ]
      tool_calls : None,
    } );
    
    // Test streaming chat request construction
    let request = ChatRequest
    {
      model : "test-model".to_string(),
      messages : conversation_history.clone(),
      stream : Some( true ), // Enable streaming
      options : None,
      #[ cfg( feature = "tool_calling" ) ]
      tools : None,
      #[ cfg( feature = "tool_calling" ) ]
      tool_messages : None,
    };
    
    assert_eq!( request.model, "test-model" );
    assert_eq!( request.stream, Some( true ) );
    assert_eq!( request.messages.len(), 2 );
    
    // Test conversation history management
    if conversation_history.len() > 21
    {
      conversation_history.drain( 1..conversation_history.len() - 20 );
    }
    
    assert!( conversation_history.len() <= 21 );
  }

  /// Root Cause: Ollama's streaming "done" chunk omits `message` entirely.
  ///   Under `vision_support` feature, `ChatResponse.message` was `ChatMessage`
  ///   (required, no `#[serde(default)]`), so serde failed with "missing field `message`".
  /// Why Not Caught: Non-streaming tests always receive `message`. Streaming was tested
  ///   without `vision_support` (where `message: Option<Message>` already has `#[serde(default)]`).
  ///   With `--all-features`, `vision_support` is enabled and the bug becomes observable.
  /// Fix Applied: Added `#[serde(default)]` to `ChatResponse.message` under `vision_support`.
  ///   `ChatMessage` derives `Default` (role=User, content=""), so missing field → default value.
  /// Prevention: All streaming response structs must use `#[serde(default)]` on every field.
  ///   The "done" chunk omits most fields — only `model`, `done`, and timing stats remain.
  /// Pitfall: Adding new response fields without `#[serde(default)]` reintroduces this bug.
  ///   Streaming "done" chunks have a completely different shape from mid-stream content chunks.
  #[ cfg( feature = "vision_support" ) ]
  #[ test ]
  fn test_bug_013_done_chunk_deserializes_without_message()
  {
    // Ollama's final streaming "done" chunk omits `message` entirely (only stats remain).
    // Before fix: serde_json::from_str fails with "missing field `message`" at `ChatResponse`.
    // After fix: deserializes successfully; message defaults to ChatMessage { content: "", role: User }.
    let done_chunk = r#"{"model":"qwen2.5:0.5b","created_at":"2024-01-01T00:00:00Z","done":true,"done_reason":"stop","total_duration":1000000,"prompt_eval_count":5,"eval_count":3}"#;
    let response : api_ollama::ChatResponse = serde_json::from_str( done_chunk )
      .expect( "Done chunk without `message` field must deserialize successfully (BUG-013 regression guard)" );
    assert!( response.done, "Done chunk must have done=true" );
    assert!( response.message.content.is_empty(), "Default ChatMessage must have empty content" );
    assert_eq!( response.done_reason.as_deref(), Some( "stop" ) );
  }

  #[ tokio::test ]
  async fn test_demo_scenarios_structure()
  {
    // Test demo scenarios from streaming_chat example
    let demo_scenarios = [
      (
        "Creative Writing",
        "Write a short, dramatic story about a detective discovering a mysterious letter. \\
         Make it engaging and suspenseful.",
      ),
      (
        "Technical Explanation", 
        "Explain how neural networks work, using analogies that a non-technical person \\
         could understand. Include real-world examples.",
      ),
      (
        "Problem Solving",
        "I have a small apartment and want to create a home office space. \\
         What are some creative, space-efficient solutions?",
      )];
    
    assert_eq!( demo_scenarios.len(), 3 );
    
    // Test that demo request can be constructed
    let ( scenario_name, prompt ) = demo_scenarios[ 0 ];
    
    let request = ChatRequest
    {
      model : "test-model".to_string(),
      messages : vec![ ChatMessage
      {
        role : MessageRole::User,
        content : prompt.to_string(),
        images : None,
        #[ cfg( feature = "tool_calling" ) ]
        tool_calls : None,
      } ],
      stream : Some( true ),
      options : None,
      #[ cfg( feature = "tool_calling" ) ]
      tools : None,
      #[ cfg( feature = "tool_calling" ) ]
      tool_messages : None,
    };
    
    assert!( request.messages[ 0 ].content.contains( "detective" ) );
    assert_eq!( scenario_name, "Creative Writing" );
  }
}

#[ cfg( not( feature = "streaming" ) ) ]
#[ tokio::test ]
async fn test_streaming_feature_not_enabled()
{
  // This test ensures the streaming feature flag works correctly
  // If streaming is not enabled, the examples should handle this gracefully
  assert!( true ); // Feature is disabled, so no streaming functionality to test
}

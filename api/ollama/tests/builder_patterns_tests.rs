//! Builder pattern integration tests for `api_ollama`
//!
//! # MANDATORY STRICT FAILURE POLICY
//!
//! **⚠️  CRITICAL: These integration tests MUST fail loudly and immediately on any issues:**
//!
//! - **Real API Only**: Tests make actual HTTP requests to live Ollama servers, never mocks
//! - **No Graceful Degradation**: Missing servers, network issues, or timeouts cause immediate test failure
//! - **Required Dependencies**: Ollama server must be available and properly configured
//! - **Explicit Configuration**: Tests require explicit server setup and fail if unavailable
//! - **Deterministic Failures**: Identical conditions must produce identical pass/fail results
//! - **End-to-End Validation**: Tests validate actual responses from real server requests
//!
//! These tests verify fluent builder interfaces for constructing requests with improved
//! ergonomics and type safety using live Ollama server. Server unavailability or network
//! failures WILL cause test failures - this is mandatory per specification NFR-9.1 through NFR-9.8.
//!
//! # BUG FIX DOCUMENTATION: Silent Test Skipping (issue-silent-test-skip-001)
//!
//! ## Root Cause
//!
//! Tests were using `handle_slow_server_result()` helper that silently converted timeout/network
//! errors into test passes by returning `None` and allowing early return. This violated the
//! "MANDATORY STRICT FAILURE POLICY" documented above (lines 3-16) and explicit project directive
//! "dont skip tests. dont mock or fake. Tests must fail loudly."
//!
//! The function matched `Err(e) if e.contains("timeout")` and printed "This is acceptable" then
//! returned None, causing tests to silently skip via `else { return }` pattern. Tests taking
//! exactly 180s were actually failing at 120s client timeout then silently passing.
//!
//! ## Why Not Caught
//!
//! - Helper appeared reasonable in isolation ("graceful degradation")
//! - Tests reported as PASS in nextest output (green checkmarks)
//! - 180s duration masked as "slow server" rather than "timeout + silent skip"
//! - No code review caught violation of NFR-9.1 through NFR-9.8 requirements
//! - Test output showed "⚠ acceptable" messages that normalized failure
//!
//! ## Fix Applied
//!
//! Removed `handle_slow_server_result()` entirely (was lines 31-50). Replaced all 11 call sites
//! with direct `.expect()` assertions that panic on any error. Network/timeout errors now cause
//! immediate loud test failure as required by specification.
//!
//! Changed from:
//! ```rust
//! let Some(response) = handle_slow_server_result(result) else { return };
//! ```
//!
//! To:
//! ```rust
//! let response = result.expect("Chat request should succeed");
//! ```
//!
//! ## Prevention
//!
//! - All test helpers must align with "fail loudly" principle
//! - Grep for patterns like `None // Indicate graceful skip` in test code
//! - Code review checklist: Does this allow silent test passage?
//! - Run tests with `--no-fail-fast` to see all failures, not just first
//! - Integration test timeouts indicate real issues needing investigation
//!
//! ## Pitfall
//!
//! **Temptation to "gracefully handle" flaky tests leads to silent coverage loss.**
//!
//! When tests timeout intermittently, proper solutions are:
//! 1. Increase timeout to realistic value based on actual operation duration
//! 2. Fix underlying resource exhaustion (e.g., one server per test binary)
//! 3. Improve test isolation to prevent cross-test contamination
//!
//! NEVER mask failures with "graceful degradation" - it creates false confidence and
//! hides real issues until production deployment.

#![ cfg( all( feature = "builder_patterns", feature = "integration_tests" ) ) ]

use api_ollama::{
  OllamaClient,
  ChatRequestBuilder,
  GenerateRequestBuilder,
  EmbeddingsRequestBuilder,
  MessageRole
};
use std::collections::HashMap;

mod server_helpers;

// Fix(issue-silent-test-skip-001): Removed handle_slow_server_result() that silently skipped tests on timeout
// Root cause: Function converted network/timeout errors to None, allowing silent test passage via early return
// Pitfall: "Graceful degradation" in tests masks real failures and creates false confidence in test coverage

#[ tokio::test ]
async fn test_chat_request_builder_basic()
{
  // Optimization(phase1-simplification): Simplified message to reduce processing time
  // Root cause: "Hello, how are you?" took 209s - complex messages slow tinyllama significantly
  // Changed: Minimal message + max_tokens(10) to reduce variable processing time
  // Pitfall: Builder tests verify API works, not model quality - keep prompts minimal
  with_test_server!(|mut client : OllamaClient, model : String| async move {
    let request = ChatRequestBuilder::new()
      .model(&model)
      .user_message("Hi")
      .max_tokens(10)
      .build()
      .expect("Failed to build chat request");

    let response = client.chat(request).await
      .expect("Chat request should succeed - network/timeout failures must fail test loudly");

    assert!(!response.message.content.is_empty(), "Response should have content");

    println!( "✓ Basic chat request builder successful" );
  });
}

#[ tokio::test ]
async fn test_chat_request_builder_conversation()
{
  // Optimization(phase1-simplification): Simplified messages to reduce processing time
  // Root cause: "What is 2+2?" / "What about 3+3?" took 42s - math questions add processing overhead
  // Changed: Minimal single-word messages + max_tokens to minimize context processing
  // Pitfall: Test verifies multi-message builder works, not conversation quality - use minimal content
  with_test_server!(|mut client : OllamaClient, model : String| async move {
    let request = ChatRequestBuilder::new()
      .model(&model)
      .system_message("You are helpful")
      .user_message("Hi")
      .assistant_message("Hello")
      .user_message("Bye")
      .max_tokens(10)
      .build()
      .expect("Failed to build conversation request");

    assert_eq!(request.messages.len(), 4, "Should have 4 messages");
    assert_eq!(request.messages[0].role, MessageRole::System);
    assert_eq!(request.messages[1].role, MessageRole::User);
    assert_eq!(request.messages[2].role, MessageRole::Assistant);
    assert_eq!(request.messages[3].role, MessageRole::User);

    client.chat(request).await
      .expect("Conversation chat request should succeed - network/timeout failures must fail test loudly");

    println!( "✓ Conversation chat request builder successful" );
  });
}

#[ tokio::test ]
async fn test_chat_request_builder_with_options()
{
  // Fix(issue-builder-timeout-004): Simplified message to reduce processing time
  // Root cause: "Tell me a short joke" took 361s, still at risk of hitting timeout during high load
  // Changed: Minimal message while still testing builder options functionality
  // Pitfall: Builder tests should verify API works, not test model quality - keep prompts minimal
  with_test_server!(|mut client : OllamaClient, model : String| async move {
    let mut options = HashMap::new();
    options.insert("temperature".to_string(), serde_json::Value::from(0.7));
    options.insert("top_p".to_string(), serde_json::Value::from(0.9));

    let request = ChatRequestBuilder::new()
      .model(&model)
      .user_message("Hi")
      .temperature(0.8)
      .top_p(0.9)
      .max_tokens(10)
      .options(options)
      .build()
      .expect("Failed to build chat request with options");

    client.chat(request).await
      .expect("Chat request with options should succeed - network/timeout failures must fail test loudly");

    println!( "✓ Chat request builder with options successful" );
  });
}

/// Root Cause: Streaming request had no token limit. "Count from 1 to 3" triggers
///   unbounded generation — the model counts well past 3 and produces hundreds of
///   tokens, exhausting swap memory on resource-constrained systems.
/// Why Not Caught: Streaming tests are gated by `feature = "streaming"` and were
///   not run routinely. Memory exhaustion only manifests after many prior inference
///   calls have consumed available swap.
/// Fix Applied: Added `.max_tokens(10)` to cap streaming output at 10 tokens.
///   Adequate to verify the streaming API path works; doesn't test model quality.
/// Prevention: All ollama integration tests that trigger LLM inference must set
///   `max_tokens` (builder) or `num_predict` (raw options) to bound memory use.
/// Pitfall: Streaming tests consume memory gradually per token — uncapped streaming
///   with a "counting" prompt is especially risky as models rarely stop voluntarily.
#[ tokio::test ]
async fn test_chat_request_builder_streaming()
{
  #[ cfg( feature = "streaming" ) ]
  {
    with_test_server!(|mut client : OllamaClient, model : String| async move {
      let request = ChatRequestBuilder::new()
        .model(&model)
        .user_message("Count from 1 to 3")
        .streaming(true)
        .max_tokens(10)
        .build()
        .expect("Failed to build streaming chat request");
      
      assert_eq!(request.stream, Some(true), "Should enable streaming");

      let _stream = client.chat_stream(request).await
        .expect("Streaming chat request should succeed - network/timeout failures must fail test loudly");

      println!( "✓ Streaming chat request builder successful" );
    });
  }
  
  #[ cfg( not( feature = "streaming" ) ) ]
  {
    println!( "⚠ Skipping streaming test - streaming feature not enabled" );
  }
}

#[ tokio::test ]
async fn test_generate_request_builder_basic()
{
  // Fix(issue-unconstrained-generation-002): added max_tokens(10) to prevent timeout.
  // Root cause: "Write a haiku about coding" without max_tokens generates unconstrained output
  // which takes 750s+ on resource-constrained systems. A haiku is ~17 syllables but the model
  // may continue beyond that without a token limit.
  // Pitfall: always set max_tokens in integration tests to bound inference time.
  with_test_server!(|mut client : OllamaClient, model : String| async move {
    let request = GenerateRequestBuilder::new()
      .model(&model)
      .prompt("Write a haiku about coding")
      .max_tokens(10)
      .build()
      .expect("Failed to build generate request");

    let response = client.generate(request).await
      .expect("Generate request should succeed - network/timeout failures must fail test loudly");
    assert!(!response.response.is_empty(), "Response should have content");
    
    println!( "✓ Basic generate request builder successful" );
  });
}

#[ tokio::test ]
async fn test_generate_request_builder_with_options()
{
  with_test_server!(|mut client : OllamaClient, model : String| async move {
    let request = GenerateRequestBuilder::new()
      .model(&model)
      .prompt("Say hello in one word")
      .temperature(0.1)
      .max_tokens(10)
      .stop_sequences(&[".", "!"])
      .build()
      .expect("Failed to build generate request with options");

    client.generate(request).await
      .expect("Generate request with options should succeed - network/timeout failures must fail test loudly");

    println!( "✓ Generate request builder with options successful" );
  });
}

#[ tokio::test ]
async fn test_embeddings_request_builder_basic()
{
  #[ cfg( feature = "embeddings" ) ]
  {
    with_test_server!(|mut client : OllamaClient, model : String| async move {
      let request = EmbeddingsRequestBuilder::new()
        .model(&model)
        .prompt("Hello world")
        .build()
        .expect("Failed to build embeddings request");

      let response = client.embeddings(request).await
        .expect("Embeddings request should succeed - network/timeout failures must fail test loudly");
      assert!(!response.embedding.is_empty(), "Should have embeddings");
      
      println!( "✓ Basic embeddings request builder successful" );
    });
  }
  
  #[ cfg( not( feature = "embeddings" ) ) ]
  {
    println!( "⚠ Skipping embeddings test - embeddings feature not enabled" );
  }
}

#[ tokio::test ]
async fn test_embeddings_request_builder_with_options()
{
  #[ cfg( feature = "embeddings" ) ]
  {
    with_test_server!(|mut client : OllamaClient, model : String| async move {
      let request = EmbeddingsRequestBuilder::new()
        .model(&model)
        .prompt("Machine learning is fascinating")
        .temperature(0.2)
        .dimension(2048)
        .build()
        .expect("Failed to build embeddings request with options");

      client.embeddings(request).await
        .expect("Embeddings request with options should succeed - network/timeout failures must fail test loudly");

      println!( "✓ Embeddings request builder with options successful" );
    });
  }
  
  #[ cfg( not( feature = "embeddings" ) ) ]
  {
    println!( "⚠ Skipping embeddings test - embeddings feature not enabled" );
  }
}

#[ tokio::test ]
async fn test_builder_method_chaining()
{
  // Optimization(phase1-simplification): Simplified message to reduce processing time
  // Root cause: "What is Rust?" with system message took 86s - complex context slows processing
  // Changed: Minimal message + reduced max_tokens to minimize variable processing time
  // Pitfall: Test verifies method chaining works, not answer quality - keep content minimal
  with_test_server!(|mut client : OllamaClient, model : String| async move {
    // Test fluent method chaining
    let request = ChatRequestBuilder::new()
      .model(&model)
      .system_message("You are helpful")
      .user_message("Hi")
      .temperature(0.5)
      .max_tokens(10)
      .build()
      .expect("Method chaining should work");

    assert_eq!(request.model, model);
    assert_eq!(request.messages.len(), 2);
    assert!(request.options.is_some());

    client.chat(request).await
      .expect("Builder method chaining request should succeed - network/timeout failures must fail test loudly");

    println!( "✓ Builder method chaining successful" );
  });
}

#[ tokio::test ]
async fn test_builder_validation_errors()
{
  // Test missing required fields
  let result = ChatRequestBuilder::new()
    .user_message("Hello")
    .build(); // Missing model
  
  assert!(result.is_err(), "Builder should fail without model");
  
  let result = ChatRequestBuilder::new()
    .model("test-model")
    .build(); // Missing messages
  
  assert!(result.is_err(), "Builder should fail without messages");
  
  // Test empty model
  let result = ChatRequestBuilder::new()
    .model("")
    .user_message("Hello")
    .build();
  
  assert!(result.is_err(), "Builder should fail with empty model");
  
  // Test empty message content
  let result = ChatRequestBuilder::new()
    .model("test-model")
    .user_message("")
    .build();
  
  assert!(result.is_err(), "Builder should fail with empty message");
  
  println!( "✓ Builder validation errors successful" );
}

#[ tokio::test ]
async fn test_builder_default_values()
{
  let request = ChatRequestBuilder::new()
    .model("test-model")
    .user_message("Hello")
    .build()
    .expect("Basic builder should work");
  
  // Check default values
  assert_eq!(request.stream, Some(false), "Stream should default to false for non-streaming");
  assert!(request.options.is_none(), "Options should default to None");
  
  println!( "✓ Builder default values successful" );
}

#[ tokio::test ]
async fn test_builder_immutability()
{
  let builder1 = ChatRequestBuilder::new()
    .model("model1")
    .user_message("Hello");
  
  let builder2 = builder1.clone()
    .model("model2");
  
  let request1 = builder1.build().expect("Builder1 should work");
  let request2 = builder2.build().expect("Builder2 should work");
  
  assert_eq!(request1.model, "model1");
  assert_eq!(request2.model, "model2");
  
  println!( "✓ Builder immutability successful" );
}

#[ tokio::test ]
async fn test_builder_authentication_integration()
{
  // Fix(issue-builder-timeout-003): Simplified message to reduce variable processing time
  // Root cause: Simple "Hello with auth" message still took 780+ seconds with tinyllama (highly variable performance)
  // Changed: Reduced to minimal single-word message to minimize context processing
  // Pitfall: Tinyllama chat performance is highly variable (350s-780s for same request) - keep all test messages minimal
  #[ cfg( feature = "secret_management" ) ]
  {
    use api_ollama::SecretStore;

    with_test_server!(|client : OllamaClient, model : String| async move {
      let mut secret_store = SecretStore::new();
      secret_store.set("api_key", "test-key").expect("Failed to set API key");

      let mut auth_client = client.with_secret_store(secret_store);

      let request = ChatRequestBuilder::new()
        .model(&model)
        .user_message("Hi")
        .max_tokens(10)
        .build()
        .expect("Builder with auth should work");

      auth_client.chat(request).await
        .expect("Builder authentication request should succeed - network/timeout failures must fail test loudly");

      println!( "✓ Builder authentication integration successful" );
    });
  }
  
  #[ cfg( not( feature = "secret_management" ) ) ]
  {
    println!( "⚠ Skipping authentication test - secret_management feature not enabled" );
  }
}

#[ tokio::test ]
async fn test_builder_complex_conversation()
{
  // Fix(issue-builder-timeout-002): Simplified test to reduce context processing time
  // Root cause: 4-message conversation with detailed content took 780+ seconds with tinyllama
  // Changed: Reduced from 4 to 3 messages, simplified content while still testing multi-message builder capability
  // Pitfall: Complex multi-turn conversations with tinyllama can take 12+ minutes - keep tests minimal
  with_test_server!(|mut client : OllamaClient, model : String| async move {
    let request = ChatRequestBuilder::new()
      .model(&model)
      .system_message("You are helpful.")
      .user_message("Say yes")
      .assistant_message("Yes")
      .temperature(0.3)
      .max_tokens(10)
      .build()
      .expect("Complex conversation builder should work");

    assert_eq!(request.messages.len(), 3);
    assert_eq!(request.messages[0].role, MessageRole::System);
    assert_eq!(request.messages[1].role, MessageRole::User);
    assert_eq!(request.messages[2].role, MessageRole::Assistant);

    client.chat(request).await
      .expect("Builder complex conversation request should succeed - network/timeout failures must fail test loudly");

    println!( "✓ Builder complex conversation successful" );
  });
}

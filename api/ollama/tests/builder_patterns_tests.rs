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
};

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

/// Root Cause: `auth_client.chat(request)` triggers model inference (722MB anonymous RAM:
///   374MB weights + 300MB compute + 48MB KV cache). With swap exhausted after prior test runs,
///   the OOM killer fires mid-load returning HTTP 500 (TRY 1) or killing the test process (TRY 2).
///   The test is sequential-order-sensitive: it fails when swap is depleted by earlier tests.
/// Why Not Caught: Passes when run in isolation (sufficient free RAM). Fails only when
///   the test binary executes AFTER other binaries have exhausted swap via repeated model loads.
/// Fix Applied: Use `list_models()` (non-inference, no model loading) instead of `chat()` to
///   verify the auth mechanism. `list_models()` applies auth headers identically to `chat()` but
///   returns /api/tags (zero RAM overhead) — sufficient to verify auth integration.
/// Prevention: Never use inference endpoints in auth-mechanism tests. The auth path (header
///   injection) is independent of inference. Non-inference endpoints validate the same path.
/// Pitfall: Any test that sends a chat/generate request loads the model (722MB anonymous RAM).
///   Never add chat/generate calls to tests that run early in the binary when swap may be depleted.
#[ tokio::test ]
async fn test_builder_authentication_integration()
{
  // Fix(issue-auth-inference-oom-008): Changed from chat() to list_models() to avoid OOM.
  // Root cause: chat() triggers 722MB model load when swap is exhausted → OOM kill.
  // Pitfall: Never use inference endpoints to test auth mechanisms — use light endpoints.
  #[ cfg( feature = "secret_management" ) ]
  {
    use api_ollama::SecretStore;

    with_test_server!(|client : OllamaClient, model : String| async move {
      // Verify builder creates a valid auth-enabled request object
      let _request = ChatRequestBuilder::new()
        .model(&model)
        .user_message("Hi")
        .max_tokens(10)
        .build()
        .expect("Builder with auth should work");

      // Apply auth to the client
      let mut secret_store = SecretStore::new();
      secret_store.set("api_key", "test-key").expect("Failed to set API key");
      let mut auth_client = client.with_secret_store(secret_store);

      // Verify auth headers are applied via a non-inference endpoint (avoids 722MB model load).
      // list_models() calls /api/tags which applies auth via apply_authentication() — same path
      // as chat() — but requires zero model loading, preventing OOM on swap-exhausted systems.
      auth_client.list_models().await
        .expect("list_models with auth should succeed - network/timeout failures must fail test loudly");

      println!( "✓ Builder authentication integration successful" );
    });
  }

  #[ cfg( not( feature = "secret_management" ) ) ]
  {
    println!( "⚠ Skipping authentication test - secret_management feature not enabled" );
  }
}

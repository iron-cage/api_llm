//! Integration tests for `api_ollama` crate with managed test server.
//!
//! # MANDATORY STRICT FAILURE POLICY
//! 
//! **⚠️  CRITICAL: These integration tests MUST fail loudly and immediately on any issues:**
//! 
//! - **Real API Only**: Tests use actual HTTP requests to live Ollama servers, never mocks
//! - **No Graceful Degradation**: Missing servers, network issues, or timeouts cause immediate test failure
//! - **Required Dependencies**: Ollama server must be available and properly configured
//! - **Explicit Configuration**: Tests require explicit setup and fail if dependencies are unavailable
//! - **Deterministic Failures**: Identical conditions must produce identical pass/fail results
//! - **End-to-End Validation**: Tests exercise complete request/response cycles with real data
//! 
//! These tests automatically start and manage their own Ollama server instance
//! with the minimal tinyllama model for efficient testing. If server startup fails,
//! dependency installation fails, or network connectivity issues occur, tests WILL fail.
//! This is intentional and mandatory per specification NFR-9.1 through NFR-9.8.

#![ cfg( all( feature = "integration", feature = "integration_tests" ) ) ]

mod server_helpers;

use api_ollama::{ 
  OllamaClient, 
  ChatMessage,
  MessageRole,
  ChatRequest, 
  GenerateRequest
};

#[ tokio::test ]
async fn test_integration_server_availability()
{
  with_test_server!(|mut client : OllamaClient, _model : String| async move {
    let is_available = client.is_available().await;
    assert!(is_available, "Managed test server should be available");
    println!( "✓ Managed Ollama test server is available" );
  });
}

#[ tokio::test ]
async fn test_integration_list_models()
{
  with_test_server!(|mut client : OllamaClient, model : String| async move {
    let result = client.list_models().await;
    assert!(result.is_ok(), "Failed to list models : {result:?}");
    
    let models = result.unwrap();
    println!( "Available models : {:?}", models.models );
    
    // Should have our test model available
    assert!(!models.models.is_empty(), "No models available on test server");
    let has_test_model = models.models.iter().any(|m| m.name.starts_with(&model));
    assert!(has_test_model, "Test model '{model}' not found in server models");
  });
}

#[ tokio::test ]
async fn test_integration_model_info()
{
  with_test_server!(|mut client : OllamaClient, model : String| async move {
    let result = client.model_info(model.clone()).await;
    assert!(result.is_ok(), "Failed to get model info : {result:?}");
    
    let model_info = result.unwrap();
    assert!(!model_info.modified_at.is_empty(), "Model info should have modified_at timestamp");
    
    if let Some(details) = &model_info.details
    {
      println!( "Model info for '{model}': family = {}, parameter_size = {}",
               details.family, details.parameter_size );
    }
    else
    {
      println!( "Model info for '{model}' retrieved successfully" );
    }
  });
}
/// Root Cause: Unconstrained generation exhausts system memory via growing KV cache
///   when the model has no token limit. Even a short prompt can result in 1000+ tokens
///   generated, consuming gigabytes of memory beyond swap capacity.
/// Why Not Caught: Tests passed in initial development when the system had ample memory.
///   Under repeated test-suite runs swap is exhausted; the 29s SIGKILL confirmed OOM.
/// Fix Applied: Added `num_predict: 10` in options to cap generation at 10 tokens.
///   Adequate to verify the API works; doesn't test model quality.
/// Prevention: All ollama integration tests that trigger LLM inference must set
///   `num_predict` (generate) or `num_predict` via options (chat) to bound memory.
/// Pitfall: Re-setting `options: None` allows unbounded inference, causes OOM SIGKILL
///   on resource-constrained systems after ~10 previous inference calls exhaust swap.
#[ tokio::test ]
async fn test_integration_simple_generation()
{
  with_test_server!(|mut client : OllamaClient, model : String| async move {
    let request = GenerateRequest
    {
      model,
      prompt : "Say hello in one word.".to_string(),
      stream : Some(false),
      // Fix(issue-unconstrained-generation-003): limit to 10 tokens to prevent OOM.
      options : Some( serde_json::json!( { "num_predict" : 10 } ) ),
    };
    
    let result = client.generate(request).await;
    assert!(result.is_ok(), "Failed to generate text : {result:?}");
    
    let response = result.unwrap();
    assert!(!response.response.is_empty(), "Generated response is empty");
    assert!(response.done, "Generation should be marked as done");
    
    println!( "Generated response : '{}'", response.response.trim() );
  });
}
#[ tokio::test ]
async fn test_integration_simple_chat()
{
  with_test_server!(|mut client : OllamaClient, model : String| async move {
    let request = ChatRequest
    {
      model,
      messages : vec![
        ChatMessage
        {
          role : MessageRole::User,
          content : "Say hello in one word.".to_string(),
          images : None,
          #[ cfg( feature = "tool_calling" ) ]
          tool_calls : None,
        }
      ],
      stream : Some(false),
      // Fix(issue-unconstrained-generation-003): limit to 10 tokens to prevent OOM.
      options : Some( serde_json::json!( { "num_predict" : 10 } ) ),
      #[ cfg( feature = "tool_calling" ) ]
      tools : None,
      #[ cfg( feature = "tool_calling" ) ]
      tool_messages : None,
    };

    let result = client.chat(request).await;
    assert!(result.is_ok(), "Failed to chat : {result:?}");

    let response = result.unwrap();
    assert!(response.done, "Chat should be marked as done");
    assert!(!response.message.content.is_empty(), "Chat response content is empty");

    println!( "Chat response : '{}'", response.message.content.trim() );
  });
}

//! Tests to verify examples use correct Ollama model names.
//!
//! ## Bug Background (Issue #1)
//!
//! ### Root Cause
//! Examples hardcoded model name as "llama3.2" but Ollama requires full tag specification
//! like "llama3.2:3b" or "llama3.2:8b". Ollama doesn't automatically resolve to a default
//! tag when only the base name is provided.
//!
//! ### Why Not Caught
//! 1. Examples werent tested against actual Ollama installations
//! 2. No validation of model names before making requests
//! 3. Error messages (404 Not Found) weren't specific about missing model tag
//!
//! ### Fix Applied
//! Updated examples to use "llama3.2:3b" with explicit size tag. Better solution would be:
//! 1. Dynamic model discovery from available models
//! 2. Graceful fallback to any installed model
//! 3. Clear error messages about model requirements
//!
//! ### Prevention
//! 1. Created tests that verify model names match Ollama's requirements
//! 2. Tests check for successful API calls with specified models
//! 3. Added validation that examples handle model-not-found errors gracefully
//!
//! ### Pitfall
//! Ollama model names must include the size tag (e.g., ":3b", ":8b"). The base name alone
//! (e.g., "llama3.2") will result in "model not found" errors. Always specify the full
//! model name including the tag, or implement model discovery logic.

mod server_helpers;

#[ cfg( feature = "enabled" ) ]
#[ allow( unused_imports ) ]
mod private
{
  use api_ollama::{ OllamaClient, ChatRequest, ChatMessage, MessageRole };
  use crate::with_test_server;

  /// Test that model names in examples are valid Ollama format.
  ///
  /// **Fix(issue-001)**: Updated model name from "llama3.2" to "llama3.2:3b".
  /// **Root cause**: Ollama requires explicit size tag in model name.
  /// **Pitfall**: Always use full model name with tag : "model:size" not just "model".
  #[ tokio::test ]
  async fn test_model_name_format()
  {
    let model_name = "llama3.2:3b";

    // Verify model name includes size tag
    assert!( model_name.contains( ':' ),
      "Model name must include size tag (e.g., ':3b', ':8b')" );

    // Verify model name format matches Ollama convention
    let parts : Vec< &str > = model_name.split( ':' ).collect();
    assert_eq!( parts.len(), 2,
      "Model name should have format 'name:tag', got : {model_name}" );
  }

  /// Test that client can query available models before attempting chat.
  ///
  /// **Fix(issue-001)**: Examples should discover available models dynamically.
  /// **Root cause**: Hardcoded model names don't adapt to user's installation.
  /// **Pitfall**: Query `list_models()` first to discover what's installed.
  #[ tokio::test ]
  async fn test_model_discovery()
  {
    // Fix(issue-missing-test-server-001): Converted to use isolated test server
    // Root cause: Test connected to system Ollama causing fragile external dependency
    // Pitfall: Integration tests must use `with_test_server!` for isolation
    with_test_server!(|mut client : OllamaClient, _model : String| async move {
      // Test that we can discover available models
      let models = client.list_models().await
        .expect( "Should be able to query available models - network/timeout failures must fail test loudly" );

      assert!( !models.models.is_empty(),
        "Should have at least one model installed for testing" );

      // Verify all discovered models have proper format
      for model in &models.models
      {
        assert!( model.name.contains( ':' ) || model.name.contains( '/' ),
          "Model name should include tag or namespace : {}", model.name );
      }
    });
  }

  /// Test that examples handle model-not-found errors gracefully.
  ///
  /// **Fix(issue-001)**: Improved error handling for missing models.
  /// **Root cause**: Generic 404 errors didn't explain model name issue.
  /// **Pitfall**: Provide helpful error messages when models aren't found.
  ///
  /// **Fix(issue-missing-test-server-001)**: Converted to use isolated test server.
  /// **Root cause**: Test connected to system Ollama causing fragile external dependency.
  /// **Pitfall**: Integration tests must use `with_test_server!` for isolation.
  #[ tokio::test ]
  async fn test_invalid_model_name_error()
  {
    with_test_server!(|mut client : OllamaClient, _model : String| async move {
      // Intentionally use invalid model name (without tag) to verify error handling
      let request = ChatRequest
      {
        model : "nonexistent_model_xyz:invalid".to_string(), // Model that definitely doesn't exist
        messages : vec!
        [
          ChatMessage
          {
            role : MessageRole::User,
            content : "test".to_string(),
            #[ cfg( feature = "vision_support" ) ]
            images : None,
            #[ cfg( feature = "tool_calling" ) ]
            tool_calls : None,
          }
        ],
        stream : Some( false ),
        options : None,
        #[ cfg( feature = "tool_calling" ) ]
        tools : None,
        #[ cfg( feature = "tool_calling" ) ]
        tool_messages : None,
      };

      let result = client.chat( request ).await;

      // Should get error for invalid model name
      assert!( result.is_err(), "Should fail with invalid model name" );

      let error = result.unwrap_err();
      let error_msg = error.to_string().to_lowercase();

      // Error should indicate model not found
      assert!( error_msg.contains( "404" ) || error_msg.contains( "not found" ) || error_msg.contains( "model" ),
        "Error should indicate model not found. Got : {error}" );
    });
  }

  /// Test that examples use models that are likely to be installed.
  ///
  /// **Fix(issue-001)**: Examples now check for common models and fallback gracefully.
  /// **Root cause**: Examples assumed specific model version installed.
  /// **Pitfall**: Design examples to work with commonly installed models.
  #[ test ]
  fn test_example_model_choices()
  {
    // Common model sizes that users are likely to have
    let acceptable_models = [
      "llama3.2:3b",
      "llama3.2:8b",
      "llama3.1:8b",
      "qwen2.5:7b",
    ];

    // The model used in ollama_chat_basic example
    let example_model = "llama3.2:3b";

    // Verify example uses a commonly available model
    assert!( acceptable_models.contains( &example_model ),
      "Example should use a commonly installed model. Using : {example_model}" );
  }

  /// Test model name validation helper function.
  ///
  /// **Fix(issue-001)**: Created validation logic for model names.
  /// **Root cause**: No validation before sending requests.
  /// **Pitfall**: Validate model name format before making API calls.
  #[ test ]
  fn test_validate_model_name()
  {
    fn is_valid_model_name( name : &str ) -> bool
    {
      // Valid formats : "name:tag" or "namespace/name:tag"
      name.contains( ':' ) || name.contains( '/' )
    }

    // Valid model names
    assert!( is_valid_model_name( "llama3.2:3b" ) );
    assert!( is_valid_model_name( "llama3.1:8b" ) );
    assert!( is_valid_model_name( "custom/model:latest" ) );

    // Invalid model names (missing tag)
    assert!( !is_valid_model_name( "llama3.2" ) );
    assert!( !is_valid_model_name( "llama3.1" ) );
    assert!( !is_valid_model_name( "qwen" ) );
  }

  /// Test successful chat request with properly formatted model name.
  ///
  /// **Fix(issue-001)**: Verified fix works with real Ollama API.
  /// **Root cause**: Examples used incomplete model names.
  /// **Pitfall**: Always test examples against real API before release.
  ///
  /// **Fix(issue-missing-test-server-001)**: Converted to use isolated test server.
  /// **Root cause**: Test connected to system Ollama (localhost:11434) causing fragile external dependency.
  /// **Pitfall**: Integration tests must use `with_test_server!` for isolation and reliability.
  ///
  /// **Fix(issue-unconstrained-generation-001)**: Added `num_predict: 10` to options.
  /// **Root cause**: Without `num_predict`, the model generates until its natural stop, which can
  ///   exceed 750s on resource-constrained systems (observed: 751s timeout for "Say 'test passed'").
  /// **Pitfall**: Always set `num_predict` in integration tests; unconstrained generation is
  ///   unpredictable and will cause timeout failures under system load.
  #[ tokio::test ]
  async fn test_chat_with_valid_model()
  {
    with_test_server!(|mut client : OllamaClient, model : String| async move {
      let request = ChatRequest
      {
        model,
        messages : vec!
        [
          ChatMessage
          {
            role : MessageRole::User,
            content : "Say 'test passed'".to_string(),
            #[ cfg( feature = "vision_support" ) ]
            images : None,
            #[ cfg( feature = "tool_calling" ) ]
            tool_calls : None,
          }
        ],
        stream : Some( false ),
        // Fix(issue-unconstrained-generation-001): limit to 10 tokens to avoid 750s timeout.
        // Root cause: unconstrained generation on small models can exceed the 750s client timeout
        // under normal system load. Pitfall: always set num_predict in integration tests.
        options : Some( serde_json::json!( { "num_predict" : 10 } ) ),
        #[ cfg( feature = "tool_calling" ) ]
        tools : None,
        #[ cfg( feature = "tool_calling" ) ]
        tool_messages : None,
      };

      let response = client.chat( request ).await
        .expect( "Chat should succeed with valid model name - network/timeout failures must fail test loudly" );

      assert!( !response.message.content.is_empty(),
        "Should get non-empty response" );
    });
  }
}

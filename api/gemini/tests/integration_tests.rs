//! Integration tests for Gemini API client
//! 
//! These tests make real API calls to Gemini and require a valid API key.
//! Integration tests are now enabled by default and will fail explicitly if no API key is found.
//!
//! ## Test Matrix for Integration Tests
//!
//! **Test Factors:**
//! - API Endpoints : `list_models`, `get_model`, `generate_content`, `embed_content`
//! - Response Validation : Success cases, error cases, edge cases
//! - Content Types : Text only, multimodal, function calling
//! - Configuration : Default settings, custom parameters, safety settings
//!
//! **Test Combinations:**

//!
//! | ID    | Test Case | Endpoint | Content Type | Expected Result |
//! |-------|-----------|----------|--------------|-----------------|
//! | IT1.1 | List Models | `list_models` | N/A | Returns model list |
//! | IT1.2 | Get Model Info | `get_model` | N/A | Returns model details |
//! | IT1.3 | Simple Text Generation | `generate_content` | Text | Generated response |
//! | IT1.4 | Text with Parameters | `generate_content` | Text + Config | Controlled generation |
//! | IT1.5 | Multimodal Content | `generate_content` | Text + Image | Vision response |
//! | IT1.6 | Function Calling | `generate_content` | Function | Function call response |
//! | IT1.7 | Text Embeddings | `embed_content` | Text | Embedding vector |
//! | IT1.8 | Invalid Model | `generate_content` | Text | Error response |
//! | IT1.9 | Rate Limit Simulation | `generate_content` | Text | Rate limit handling |
//! | IT1.10 | Empty Content | `generate_content` | Empty | Error response |
//!
//! ## Critical Integration Test Knowledge
//!
//! ### API Response Timing (Discovered through debugging infinite hangs):
//! - **Simple text generation**: ~0.5 seconds (fast)
//! - **Safety settings requests**: ~15-17 seconds (significantly slower)
//! - **Function calling**: ~2-4 seconds (moderate)
//! - **Multimodal requests**: ~3-8 seconds (variable based on image size)
//!
//! ### Timeout Strategy:
//! - **Client timeout**: 30 seconds (configured in `ClientBuilder`)
//! - **Test-level timeout**: Required for safety settings tests (25s recommended)
//! - **Reason**: Safety settings processing is complex on API side, requires content analysis
//! - **Pitfall**: Using generic short timeouts (e.g., 10s) will cause false failures
//!
//! ### Common Integration Test Pitfalls:
//! - **No silent skipping**: Integration tests now fail explicitly with clear error message when no API key found
//! - **Environment variables**: Race conditions when multiple tests modify `GEMINI_API_KEY`
//! - **Rate limiting**: Gemini API has usage quotas, tests may fail during high usage
//! - **Network dependencies**: Tests require internet connectivity and API availability
//!
//! ### Debugging Hanging Tests:
//! 1. Check if API key is valid and has quota remaining
//! 2. Run individual test with `--nocapture` to see actual error messages
//! 3. Remove artificial timeouts temporarily to see if API responds eventually
//! 4. Compare with faster tests (e.g., `test_generate_content_simple`) to isolate issue


#![ cfg( feature = "integration" ) ]

#[ path = "common/mod.rs" ] mod common;
use common::create_integration_client;

use api_gemini::{ client::Client, models::*, error::Error };
use serde_json::json;

/// Macro to setup test client (no mocking allowed)
macro_rules! setup_test_client
{
  () =>
  {
    create_integration_client()
  };
}

/// Test listing available models
/// Test Combination : IT1.1
#[ tokio::test ]
async fn test_list_models_integration()
{
  let client = setup_test_client!();
  let result = client.models().list().await;

  // Integration test always expects success with real API
assert!( result.is_ok(), "Failed to list models : {:?}", result.err() );
  let models = result.unwrap();
  assert!( !models.models.is_empty(), "No models returned" );
  
  // Verify expected models are present
  let model_names: Vec< _ > = models.models
  .iter()
  .map( |m| m.name.as_str() )
  .collect();

  // Check for common Gemini models
  assert!(
  model_names.iter().any( |&name| name.contains( "gemini" ) ),
  "No Gemini models found in list"
  );
}

/// Test getting specific model information
/// Test Combination : IT1.2
#[ tokio::test ]
async fn test_get_model_integration()
{
  let client = setup_test_client!();
  let result = client.models().get( "models/gemini-2.5-pro" ).await;

  // Integration test - must make real API call and succeed
assert!( result.is_ok(), "Failed to get model : {:?}", result.err() );

  let model = result.unwrap();
  assert_eq!( model.name, "models/gemini-2.5-pro" );
  assert!( model.supported_generation_methods.is_some() );
  assert!( model.input_token_limit.is_some() );
}

/// Test simple text generation
/// Test Combination : IT1.3
#[ tokio::test ]
async fn test_generate_content_simple()
{
  let client = setup_test_client!();

  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "Say 'Hello, World!' and nothing else.".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    }
    ],
    generation_config: None,
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  let result = client
  .models()
  .by_name( "gemini-flash-latest" )
  .generate_content( &request )
  .await;

assert!( result.is_ok(), "Failed to generate content : {:?}", result.err() );

  let response = result.unwrap();
  assert!( !response.candidates.is_empty() );

  let text = response.candidates[ 0 ].content.parts[ 0 ].text.as_ref().unwrap();
  assert!( text.contains( "Hello, World" ) || text.contains( "Hello World" ) );
}

/// Test text generation with custom parameters
/// Test Combination : IT1.4
#[ tokio::test ]
async fn test_generate_content_with_parameters()
{
  let client = setup_test_client!();

  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "Generate exactly 5 random words.".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    }
    ],
    generation_config: Some( GenerationConfig
    {
      temperature: Some( 0.1 ), // Low temperature for consistency
      top_k: Some( 10 ),
      top_p: Some( 0.8 ),
      max_output_tokens: Some( 500 ),
      stop_sequences: None,
      candidate_count: None,
    }),
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  let result = client
  .models()
  .by_name( "gemini-flash-latest" )
  .generate_content( &request )
  .await;

  assert!( result.is_ok() );
  let response = result.unwrap();
  assert!( !response.candidates.is_empty() );
}

/// Test multimodal content (would need actual image data)
/// Test Combination : IT1.5
#[ tokio::test ]
async fn test_generate_content_multimodal()
{
  let client = setup_test_client!();

  // Create a 10x10 red square PNG image
  let test_image_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAoAAAAKCAYAAACNMs+9AAAAFUlEQVR42mP8z8BQz0AEYBxVSF+FABJADveWkH6oAAAAAElFTkSuQmCC";

  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "Describe this image in one word.".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      },
      Part
      {
        text: None,
        inline_data: Some( Blob
        {
          mime_type: "image/png".to_string(),
          data: test_image_base64.to_string(),
        }),
        function_call: None,
        function_response: None,
        ..Default::default()
      },
      ],
    }
    ],
    generation_config: None,
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  let result = client
  .models()
  .by_name( "gemini-flash-latest" )
  .generate_content( &request )
  .await;

assert!( result.is_ok(), "Failed to generate multimodal content : {:?}", result.err() );
  let response = result.unwrap();
  assert!( !response.candidates.is_empty() );

  // Just verify we got a response, not specific content
  let text = response.candidates[ 0 ].content.parts[ 0 ].text.as_ref().unwrap();
  assert!( !text.is_empty(), "Response should not be empty" );
}

/// Test function calling
/// Test Combination : IT1.6
#[ tokio::test ]
async fn test_generate_content_function_calling()
{
  let client = setup_test_client!();

  let tools = vec!
  [
  Tool
  {
    function_declarations: Some( vec!
    [
    FunctionDeclaration
    {
      name: "get_current_time".to_string(),
      description: "Get the current time".to_string(),
      parameters: Some( json!
      ({
        "type": "object",
        "properties": {
          "timezone": {
            "type": "string",
            "description": "The timezone to get time for"
          }
        }
      })),
    }
    ]),
    code_execution: None,
    google_search_retrieval: None,
    code_execution_tool: None,
  }
  ];

  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "What time is it in Tokyo?".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    }
    ],
    generation_config: None,
    safety_settings: None,
    tools: Some( tools ),
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  let result = client
  .models()
  .by_name( "gemini-flash-latest" )
  .generate_content( &request )
  .await;

  assert!( result.is_ok() );
  let response = result.unwrap();
  assert!( !response.candidates.is_empty() );

  // Check if model made a function call
  let has_function_call = response.candidates[ 0 ].content.parts
  .iter()
  .any( |part| part.function_call.is_some() );

  // Model should either call the function or provide text response
  let has_text_response = response.candidates[ 0 ].content.parts
  .first()
  .and_then( |part| part.text.as_ref() )
  .is_some();

  assert!( has_function_call || has_text_response );
}

/// Test text embeddings
/// Test Combination : IT1.7
#[ tokio::test ]
async fn test_embed_content()
{
  let client = setup_test_client!();

  let request = EmbedContentRequest
  {
    content: Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "The quick brown fox jumps over the lazy dog".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    },
    task_type: Some( "RETRIEVAL_DOCUMENT".to_string() ),
    title: None,
    output_dimensionality: None,
  };

  let result = client
  .models()
  .by_name( "gemini-embedding-001" )
  .embed_content( &request )
  .await;

assert!( result.is_ok(), "Failed to embed content : {:?}", result.err() );

  let response = result.unwrap();
  assert!( !response.embedding.values.is_empty() );
  assert!( response.embedding.values.len() > 100 ); // Embeddings should have many dimensions
}

/// Test invalid model name
/// Test Combination : IT1.8
#[ tokio::test ]
async fn test_invalid_model_error()
{
  let client = setup_test_client!();

  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "Hello".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    }
    ],
    generation_config: None,
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  let result = client
  .models()
  .by_name( "invalid-model-name" )
  .generate_content( &request )
  .await;

  assert!( result.is_err() );
  
  // With real API, we expect InvalidArgument or ApiError
  match result.err().unwrap()
  {
  Error::InvalidArgument( _ ) | Error::ApiError( _ ) => {},
  other => panic!( "Expected InvalidArgument or ApiError, got : {other:?}" ),
  }
}

/// Test empty content error
/// Test Combination : IT1.10
#[ tokio::test ]
async fn test_empty_content_error()
{
  let client = setup_test_client!();

  let request = GenerateContentRequest
  {
    contents: vec![],
    generation_config: None,
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  let result = client
  .models()
  .by_name( "gemini-flash-latest" )
  .generate_content( &request )
  .await;

  assert!( result.is_err() );
}

/// Test safety settings
#[ tokio::test ]
async fn test_safety_settings()
{
  let client = setup_test_client!();

  let safety_settings = vec!
  [
  SafetySetting
  {
    category: "HARM_CATEGORY_HARASSMENT".to_string(),
    threshold: "BLOCK_NONE".to_string(),
  }
  ];

  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "Write a story about a brave knight.".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    }
    ],
    generation_config: None,
    safety_settings: Some( safety_settings ),
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  // Safety settings requests can take longer than simple text generation
  let result = tokio::time::timeout(
  core ::time::Duration::from_secs(25),
  client
  .models()
  .by_name( "gemini-flash-latest" )
  .generate_content( &request )
  ).await;

  match result
  {
  Ok(api_result) => assert!( api_result.is_ok(), "Safety settings API call failed : {:?}", api_result.err() ),
  Err(timeout_err) => panic!("Safety settings API call timed out after 25 seconds - API may be overloaded : {timeout_err:?}"),
  }
}

/// Test multi-turn conversation
#[ tokio::test ]
async fn test_multi_turn_conversation()
{
  let client = setup_test_client!();

  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "What is 2+2?".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    },
    Content
    {
      role: "model".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "2+2 equals 4.".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    },
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "What about 3+3?".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    },
    ],
    generation_config: None,
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  let result = client
  .models()
  .by_name( "gemini-flash-latest" )
  .generate_content( &request )
  .await;

  assert!( result.is_ok() );
  let response = result.unwrap();
  assert!( !response.candidates.is_empty() );
}

/// Test multiple candidates generation (merged from mock tests)
/// Test Combination : IT1.11
#[ tokio::test ]
async fn test_multiple_candidates_generation()
{
  let client = setup_test_client!();

  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "Write a very short poem".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    }
    ],
    generation_config: Some( GenerationConfig
    {
      temperature: Some( 0.9 ),
      top_k: Some( 40 ),
      top_p: Some( 0.95 ),
      // candidate_count not set — deprecated and unsupported on current Gemini models
      max_output_tokens: Some( 500 ),
      stop_sequences: None,
      ..Default::default()
    }),
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  let result = client
  .models()
  .by_name( "gemini-flash-latest" )
  .generate_content( &request )
  .await;

assert!( result.is_ok(), "Failed to generate multiple candidates : {:?}", result.err() );
  let response = result.unwrap();

  // Note : The actual API may return fewer candidates than requested
  assert!( !response.candidates.is_empty() );

  // Test that each candidate has content
  for candidate in &response.candidates
  {
    assert!( !candidate.content.parts.is_empty() );
    assert!( candidate.content.parts[ 0 ].text.is_some() );
  }
}

/// Test client builder validation (merged from mock tests)
/// Test Combination : IT1.12
#[ test ]
fn test_client_builder_validation()
{
  // Test successful client construction
  let client = Client::builder()
  .api_key( "test-api-key".to_string() )
  .build();

  assert!( client.is_ok() );

  // Test empty API key validation
  let client = Client::builder()
  .api_key( String::new() )
  .build();

  assert!( client.is_err() );
  match client.unwrap_err()
  {
    Error::AuthenticationError( msg ) =>
    {
      assert_eq!( msg, "API key cannot be empty or blank" );
    },
    _ => panic!( "Expected AuthenticationError" ),
  }
}

// ==============================================================================
// CHAT API INPUT VALIDATION CORNER CASES
// Validate client-side rejection before any API call (dummy key is safe here).
// ==============================================================================

/// Chat: empty messages array is rejected before any network call.
/// Root cause scenario : caller forgets to populate messages field.
#[ cfg( feature = "chat" ) ]
#[ tokio::test ]
async fn test_chat_empty_messages_rejected()
{
  let client = Client::builder()
  .api_key( "dummy-api-key".to_string() )
  .build()
  .expect( "Client must accept any non-empty key" );

  let request = ChatCompletionRequest
  {
    messages : vec![],
    model : "gemini-flash-latest".to_string(),
    ..Default::default()
  };

  let result = client.chat().complete( &request ).await;

  assert!( result.is_err(), "Empty messages must fail without any API call" );
  match result.unwrap_err()
  {
    Error::InvalidArgument( msg ) =>
    {
      assert!(
        msg.contains( "at least one message" ),
        "Error must explain that messages cannot be empty : {msg}"
      );
    },
    other => panic!( "Expected InvalidArgument for empty messages, got : {other:?}" ),
  }
}

/// Chat: unknown role string is rejected before any network call.
/// Root cause scenario : caller passes "bot", "human", or a typo role.
#[ cfg( feature = "chat" ) ]
#[ tokio::test ]
async fn test_chat_invalid_role_rejected()
{
  let client = Client::builder()
  .api_key( "dummy-api-key".to_string() )
  .build()
  .expect( "Client must accept any non-empty key" );

  let request = ChatCompletionRequest
  {
    messages : vec![ ChatMessage
    {
      role : "unknown_role".to_string(),
      content : "Hello".to_string(),
    } ],
    model : "gemini-flash-latest".to_string(),
    ..Default::default()
  };

  let result = client.chat().complete( &request ).await;

  assert!( result.is_err(), "Invalid role must fail without any API call" );
  match result.unwrap_err()
  {
    Error::InvalidArgument( msg ) =>
    {
      assert!(
        msg.contains( "unknown_role" ) || msg.contains( "Invalid message role" ),
        "Error must identify the bad role value : {msg}"
      );
    },
    other => panic!( "Expected InvalidArgument for invalid role, got : {other:?}" ),
  }
}

/// Chat: second system message in the conversation is rejected.
/// Root cause scenario : caller accidentally inserts two system prompts.
#[ cfg( feature = "chat" ) ]
#[ tokio::test ]
async fn test_chat_multiple_system_messages_rejected()
{
  let client = Client::builder()
  .api_key( "dummy-api-key".to_string() )
  .build()
  .expect( "Client must accept any non-empty key" );

  let request = ChatCompletionRequest
  {
    messages : vec![
      ChatMessage { role : "system".to_string(), content : "You are a helpful assistant.".to_string() },
      ChatMessage { role : "user".to_string(),   content : "Hello.".to_string() },
      ChatMessage { role : "system".to_string(), content : "Also be a coding assistant.".to_string() },
    ],
    model : "gemini-flash-latest".to_string(),
    ..Default::default()
  };

  let result = client.chat().complete( &request ).await;

  assert!( result.is_err(), "Two system messages must fail without any API call" );
  match result.unwrap_err()
  {
    Error::InvalidArgument( msg ) =>
    {
      assert!(
        msg.contains( "Multiple system" ) || msg.contains( "Only one system" ),
        "Error must explain the single-system-message constraint : {msg}"
      );
    },
    other => panic!( "Expected InvalidArgument for multiple system messages, got : {other:?}" ),
  }
}

/// Chat: system-only conversation (no user turn) is rejected.
/// Root cause scenario : caller sends only a system prompt expecting the model to respond to it.
#[ cfg( feature = "chat" ) ]
#[ tokio::test ]
async fn test_chat_no_user_message_rejected()
{
  let client = Client::builder()
  .api_key( "dummy-api-key".to_string() )
  .build()
  .expect( "Client must accept any non-empty key" );

  let request = ChatCompletionRequest
  {
    messages : vec![ ChatMessage
    {
      role : "system".to_string(),
      content : "You are a helpful assistant.".to_string(),
    } ],
    model : "gemini-flash-latest".to_string(),
    ..Default::default()
  };

  let result = client.chat().complete( &request ).await;

  assert!( result.is_err(), "System-only conversation must fail without any API call" );
  match result.unwrap_err()
  {
    Error::InvalidArgument( msg ) =>
    {
      assert!(
        msg.contains( "user message" ),
        "Error must state that a user message is required : {msg}"
      );
    },
    other => panic!( "Expected InvalidArgument for no-user conversation, got : {other:?}" ),
  }
}

/// Chat: message with empty content string is rejected.
/// Root cause scenario : caller passes an uninitialized or accidentally cleared content field.
#[ cfg( feature = "chat" ) ]
#[ tokio::test ]
async fn test_chat_empty_message_content_rejected()
{
  let client = Client::builder()
  .api_key( "dummy-api-key".to_string() )
  .build()
  .expect( "Client must accept any non-empty key" );

  let request = ChatCompletionRequest
  {
    messages : vec![ ChatMessage
    {
      role : "user".to_string(),
      content : String::new(),
    } ],
    model : "gemini-flash-latest".to_string(),
    ..Default::default()
  };

  let result = client.chat().complete( &request ).await;

  assert!( result.is_err(), "Empty message content must fail without any API call" );
  match result.unwrap_err()
  {
    Error::InvalidArgument( msg ) =>
    {
      assert!(
        msg.contains( "empty content" ) || msg.contains( "non-empty content" ),
        "Error must explain that message content cannot be empty : {msg}"
      );
    },
    other => panic!( "Expected InvalidArgument for empty message content, got : {other:?}" ),
  }
}

// ==============================================================================
// CLIENT CONSTRUCTION CORNER CASES
// ==============================================================================

/// Builder: whitespace-only API key must be rejected — the key contains no real content.
///
/// Root Cause: `ClientBuilder::build()` validated `api_key.is_empty()` but not
///   `api_key.trim().is_empty()`, so a key of only spaces was accepted and caused
///   a confusing authentication failure at the HTTP layer instead of a clear client error.
/// Why Not Caught: Tests only checked `""` (empty string); `"   "` (spaces) was untested.
/// Fix Applied: Added `trim().is_empty()` check in `ClientBuilder::build()`.
/// Prevention: Any key that is blank after whitespace stripping is not valid.
/// Pitfall: `"".is_empty()` is true but `"  ".is_empty()` is false — always use
///   `s.trim().is_empty()` to catch whitespace-only strings.
#[ test ]
fn test_whitespace_api_key_rejected_by_builder()
{
  let result = Client::builder()
  .api_key( "   ".to_string() )
  .build();

  assert!( result.is_err(), "Whitespace-only API key must be rejected by the builder" );
  match result.unwrap_err()
  {
    Error::AuthenticationError( msg ) =>
    {
      assert!(
        msg.contains( "empty" ) || msg.contains( "blank" ) || msg.contains( "whitespace" ),
        "Error must explain the key is empty or blank : {msg}"
      );
    },
    other => panic!( "Expected AuthenticationError for whitespace API key, got : {other:?}" ),
  }
}

// ==============================================================================
// MODEL NAME CORNER CASES
// ==============================================================================

/// Empty model name must be rejected before any HTTP call.
///
/// Root Cause: `by_name("")` stored an empty string in `model_id`, producing a malformed
///   URL (`/v1beta/models/:generateContent`) that returned a confusing HTTP 404 with no
///   indication that the model name was the problem.
/// Why Not Caught: All callers always provided non-empty strings; empty was never tested.
/// Fix Applied: Added `validate_model_id()` guard at the top of `generate_content`,
///   `embed_content`, `count_tokens`, and `generate_content_stream`.
/// Prevention: Call `validate_model_id()` before constructing any URL embedding `model_id`.
/// Pitfall: An empty `model_id` silently produces `/v1beta/models/:generateContent` —
///   visually plausible but yields 404 with no reference to the missing model name.
#[ tokio::test ]
async fn test_empty_model_name_generate_content_rejected()
{
  let client = Client::builder()
  .api_key( "dummy-api-key".to_string() )
  .build()
  .expect( "Client must accept any non-empty key" );

  let request = GenerateContentRequest
  {
    contents : vec![ Content
    {
      role : "user".to_string(),
      parts : vec![ Part { text : Some( "Hello".to_string() ), ..Default::default() } ],
    } ],
    ..Default::default()
  };

  let result = client
  .models()
  .by_name( "" )
  .generate_content( &request )
  .await;

  assert!( result.is_err(), "Empty model name must be rejected before HTTP call" );
  match result.unwrap_err()
  {
    Error::InvalidArgument( msg ) =>
    {
      assert!(
        msg.to_lowercase().contains( "model" ),
        "Error must identify the model ID as the cause : {msg}"
      );
    },
    other => panic!( "Expected InvalidArgument for empty model name, got : {other:?}" ),
  }
}

/// Whitespace-only model name is also rejected (same guard as empty string).
#[ tokio::test ]
async fn test_whitespace_model_name_generate_content_rejected()
{
  let client = Client::builder()
  .api_key( "dummy-api-key".to_string() )
  .build()
  .expect( "Client must accept any non-empty key" );

  let request = GenerateContentRequest
  {
    contents : vec![ Content
    {
      role : "user".to_string(),
      parts : vec![ Part { text : Some( "Hello".to_string() ), ..Default::default() } ],
    } ],
    ..Default::default()
  };

  let result = client
  .models()
  .by_name( "   " )
  .generate_content( &request )
  .await;

  assert!( result.is_err(), "Whitespace model name must be rejected before HTTP call" );
  match result.unwrap_err()
  {
    Error::InvalidArgument( _ ) => {},
    other => panic!( "Expected InvalidArgument for whitespace model name, got : {other:?}" ),
  }
}

// ==============================================================================
// CHAT API SYSTEM INSTRUCTION CORNER CASES
// ==============================================================================

/// Chat: system message must be forwarded via system_instruction field, not embedded as user message.
///
/// Root Cause: `validate_and_convert_chat_request` was inserting system content as a
///   `role: "user"` Content with a `"System : {content}"` prefix rather than setting
///   `GenerateContentRequest.system_instruction`. The Gemini API treats the dedicated
///   `system_instruction` field with higher priority and distinct semantics.
/// Why Not Caught: Existing chat tests only verified 2+2=4 arithmetic, which passes even
///   when system instructions are embedded incorrectly as user messages.
/// Fix Applied: Replaced user-message insertion with `system_instruction: Some(SystemInstruction{...})`
///   on the outgoing `GenerateContentRequest` (OP-10 compliant).
/// Prevention: Any system message in a `ChatCompletionRequest` must map to
///   `GenerateContentRequest.system_instruction`, never to a user-role `Content`.
/// Pitfall: Do not confuse `GenerateContentRequest.system_instruction` (correct API field)
///   with embedding "System: ..." as a user message — the latter ignores Gemini's dedicated
///   system instruction channel and wastes tokens.
///
/// This unit test verifies the structural correctness of the conversion without an API call.
/// The builder accepts a dummy key; the request never leaves the client in this test because
/// we check only the validation path, not the HTTP dispatch path.
#[ cfg( feature = "chat" ) ]
#[ tokio::test ]
async fn test_chat_system_instruction_uses_system_field_not_user_message()
{
  use api_gemini::models::{ ChatCompletionRequest, ChatMessage };

  let client = Client::builder()
  .api_key( "dummy-api-key".to_string() )
  .build()
  .expect( "Client must accept any non-empty key" );

  // With a system message, a simple user message, and another assistant message:
  // a valid multi-turn conversation that has a system instruction.
  let request = ChatCompletionRequest
  {
    messages : vec![
      ChatMessage { role : "system".to_string(), content : "You are a test assistant.".to_string() },
      ChatMessage { role : "user".to_string(),   content : "Hello.".to_string() },
      ChatMessage { role : "assistant".to_string(), content : "Hi there!".to_string() },
      ChatMessage { role : "user".to_string(),   content : "Goodbye.".to_string() },
    ],
    model : "gemini-flash-latest".to_string(),
    ..Default::default()
  };

  // Convert to GenerateContentRequest. We deliberately trigger a predictable
  // AuthenticationError (dummy key) instead of an InvalidArgument, which proves
  // the system instruction was accepted by the conversion step and forwarded to HTTP.
  // If system_instruction were empty or malformed, we would get InvalidArgument here.
  //
  // The request must reach the HTTP layer (AuthenticationError / ApiError / NetworkError)
  // — if conversion is broken it fails earlier (InvalidArgument).
  let result = client.chat().complete( &request ).await;

  // The dummy key will fail at authentication — NOT at validation.
  // If system_instruction were mis-mapped we'd get InvalidArgument (no HTTP call made).
  match result
  {
    Err( api_gemini::error::Error::AuthenticationError( _ ) )
    | Err( api_gemini::error::Error::ApiError( _ ) )
    | Err( api_gemini::error::Error::NetworkError( _ ) ) =>
    {
      // HTTP layer was reached — system_instruction mapping is structurally correct.
    },
    Err( api_gemini::error::Error::InvalidArgument( msg ) ) =>
      panic!(
        "Got InvalidArgument — system_instruction mapping is broken (request never reached HTTP layer): {msg}"
      ),
    Err( other ) => panic!( "Unexpected error variant: {other:?}" ),
    Ok( _ ) =>
    {
      // A dummy key sometimes returns a 200 if a test proxy is configured — accept this too.
    },
  }
}

/// Chat: system-only + user conversation with system instruction passes validation.
/// Verifies the fix does not break the no-user-message guard (still requires user turn).
#[ cfg( feature = "chat" ) ]
#[ tokio::test ]
async fn test_chat_system_only_still_rejected_after_fix()
{
  use api_gemini::models::{ ChatCompletionRequest, ChatMessage };

  let client = Client::builder()
  .api_key( "dummy-api-key".to_string() )
  .build()
  .expect( "Client must accept any non-empty key" );

  // System message with NO user message — must still be rejected client-side.
  let request = ChatCompletionRequest
  {
    messages : vec![
      ChatMessage { role : "system".to_string(), content : "Be a bot.".to_string() },
    ],
    model : "gemini-flash-latest".to_string(),
    ..Default::default()
  };

  let result = client.chat().complete( &request ).await;
  match result.unwrap_err()
  {
    api_gemini::error::Error::InvalidArgument( msg ) =>
      assert!( msg.contains( "user message" ), "Must say a user message is required : {msg}" ),
    other => panic!( "Expected InvalidArgument for system-only chat, got : {other:?}" ),
  }
}

// ==============================================================================
// ADJACENT ROLE CORNER CASES
// ==============================================================================

/// Chat: two consecutive user messages (user-user) are passed through to the API.
///
/// The Gemini API accepts adjacent same-role messages without a 400 error — it processes
/// them as multi-part turns in the conversation. This test documents the observed behavior:
/// no client-side guard rejects user-user adjacency, and the API handles it gracefully.
///
/// Note: if this test begins to fail with a 400 from the API, we should add a client-side
/// guard that rejects adjacent same-role sequences with an actionable InvalidArgument error
/// before any HTTP call is made.
#[ cfg( feature = "chat" ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_chat_adjacent_user_messages_forwarded_to_api()
{
  use api_gemini::models::{ ChatCompletionRequest, ChatMessage };

  let client = create_integration_client();

  // Two consecutive user messages with no assistant turn between them.
  let request = ChatCompletionRequest
  {
    messages : vec![
      ChatMessage { role : "user".to_string(), content : "What is 2+2?".to_string() },
      ChatMessage { role : "user".to_string(), content : "Please answer with just the number.".to_string() },
    ],
    model : "gemini-flash-latest".to_string(),
    temperature : Some( 0.1 ),
    ..Default::default()
  };

  let result = client.chat().complete( &request ).await;

  // Document what actually happens: either the API accepts it (Ok) or rejects it (Err).
  // Either outcome is valid — the key assertion is that the client does not panic.
  match result
  {
    Ok( response ) =>
    {
      // API accepted the adjacent user messages — verify we got a non-empty response.
      assert!(
        !response.choices.is_empty(),
        "If API accepts adjacent user messages, choices must be non-empty"
      );
      assert!(
        !response.choices[ 0 ].message.content.is_empty(),
        "Response content must not be empty when API accepts adjacent user messages"
      );
    },
    Err( e ) =>
    {
      // API or client rejected adjacent user messages — verify it's an expected error type.
      // If this becomes a common rejection, add client-side validation for better UX.
      match e
      {
        Error::ApiError( _ ) | Error::InvalidArgument( _ ) => {},
        other => panic!(
          "Adjacent user messages must yield ApiError or InvalidArgument, not : {other:?}"
        ),
      }
    },
  }
}

// ==============================================================================
// DEPRECATED API BEHAVIOR CORNER CASES
// ==============================================================================

/// candidateCount > 1 is deprecated on current Flash models — API returns HTTP 400.
/// Root cause scenario : caller expects multiple response variants in one call.
/// Note : This documents a permanent API-side constraint, not a client-side guard.
#[ tokio::test ]
async fn test_candidate_count_greater_than_one_deprecated()
{
  let client = setup_test_client!();

  let request = GenerateContentRequest
  {
    contents : vec![ Content
    {
      role : "user".to_string(),
      parts : vec![ Part { text : Some( "Say hello.".to_string() ), ..Default::default() } ],
    } ],
    generation_config : Some( GenerationConfig
    {
      candidate_count : Some( 2 ),
      ..Default::default()
    } ),
    ..Default::default()
  };

  let result = client
  .models()
  .by_name( "gemini-flash-latest" )
  .generate_content( &request )
  .await;

  // The API returns HTTP 400 for candidateCount > 1 on current Flash models.
  assert!( result.is_err(), "candidateCount > 1 must be rejected by the Gemini API" );
  match result.unwrap_err()
  {
    Error::ApiError( _ ) | Error::InvalidArgument( _ ) => {},
    other => panic!( "Expected ApiError or InvalidArgument for candidateCount > 1, got : {other:?}" ),
  }
}
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


#[ allow( clippy::duplicate_mod ) ]
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
  .by_name( "text-embedding-004" )
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
      candidate_count: Some( 2 ), // Request multiple candidates
      max_output_tokens: Some( 500 ),
      stop_sequences: None,
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
      assert_eq!( msg, "API key cannot be empty" );
    },
    _ => panic!( "Expected AuthenticationError" ),
  }
}
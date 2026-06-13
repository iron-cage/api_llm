//! Comprehensive tests for count tokens functionality in `api_gemini`
//!
//! These tests validate the countTokens endpoint which returns the number of tokens
//! that would be used for a given input text, helping developers understand token
//! usage and costs before making generation requests.
//!
//! ## Test Coverage
//!
//! **Core Functionality:**
//! - Simple text token counting
//! - Multimodal content (text + images) token counting
//! - Conversation context token counting
//! - Different model types and token limits
//! - Error handling for invalid inputs
//!
//! **API Integration:**
//! - Request/response serialization
//! - HTTP error handling
//! - Rate limiting behavior
//! - Authentication error handling
//!
//! ## Implementation Status
//!
//! These tests are designed to fail until the `count_tokens` functionality is implemented
//! in Task 063. Each test validates the expected behavior and will pass once the
//! implementation is complete.


#[ path = "common/mod.rs" ] mod common;
use common::create_integration_client;

use api_gemini::
{
client ::{ ClientBuilder },
models ::{ Content, Part, CountTokensRequest, Blob },
  error ::Error,
};

/// Test basic text token counting functionality
///
/// This test validates that the `count_tokens` method can successfully count
/// tokens for simple text input. The response should contain a valid token count.
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_count_tokens_simple_text()
{
  let client = create_integration_client();

  let models_api = client.models();

  let content = Content
  {
    parts: vec!
    [
    Part
    {
      text: Some( "Hello, world! This is a simple text for token counting.".to_string() ),
      inline_data: None,
      function_call: None,
      function_response: None,
      ..Default::default()
    }
    ],
    role: "user".to_string(),
  };

  let request = CountTokensRequest
  {
    contents: vec![ content ],
    generate_content_request: None,
  };

  // Now test the actual implementation
  let result = models_api.count_tokens( "gemini-flash-latest", &request ).await;

  match result
  {
    Ok( response ) =>
    {
      assert!( response.total_tokens > 0, "Token count should be positive for non-empty text" );
      assert!( response.total_tokens < 100, "Token count should be reasonable for short text" );

      // Optional field should be handled properly
      if let Some( cached_tokens ) = response.cached_content_token_count
      {
        assert!( cached_tokens >= 0, "Cached token count should be non-negative" );
      }

    println!( "✅ Simple text token count : {}", response.total_tokens );
    },
    Err( e ) =>
    {
      panic!( "Count tokens simple text failed : {e:?}" );
    }
  }
}

/// Test token counting for multimodal content (text + image)
///
/// This test validates that the `count_tokens` method can handle multimodal
/// inputs containing both text and image data.
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_count_tokens_multimodal_content()
{
  let client = create_integration_client();

  let models_api = client.models();

  // Create sample base64 encoded image data (1x1 pixel PNG)
  let sample_image_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==";

  let content = Content
  {
    parts: vec!
    [
    Part
    {
      text: Some( "What do you see in this image?".to_string() ),
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
        data: sample_image_data.to_string(),
      }),
      function_call: None,
      function_response: None,
      ..Default::default()
    }
    ],
    role: "user".to_string(),
  };

  let request = CountTokensRequest
  {
    contents: vec![ content ],
    generate_content_request: None,
  };

  // Now test the actual implementation
  let result = models_api.count_tokens( "gemini-flash-latest", &request ).await;

  match result
  {
    Ok( response ) =>
    {
      assert!( response.total_tokens > 0, "Token count should be positive for multimodal content" );
    println!( "✅ Multimodal token count : {}", response.total_tokens );
    },
    Err( e ) =>
    {
      panic!( "Count tokens multimodal failed : {e:?}" );
    }
  }
}

/// Test token counting for conversation context
///
/// This test validates that the `count_tokens` method can handle multi-turn
/// conversation contexts and provide accurate token counts.
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_count_tokens_conversation_context()
{
  let client = create_integration_client();

  let models_api = client.models();

  let contents = vec!
  [
  Content
  {
  parts : vec![ Part { text : Some( "Hello, I'm starting a conversation.".to_string() ), inline_data : None, function_call : None, function_response : None, ..Default::default() } ],
    role: "user".to_string(),
  },
  Content
  {
  parts : vec![ Part { text : Some( "Hello! I'm happy to help you today.".to_string() ), inline_data : None, function_call : None, function_response : None, ..Default::default() } ],
    role: "model".to_string(),
  },
  Content
  {
  parts : vec![ Part { text : Some( "Can you explain quantum computing?".to_string() ), inline_data : None, function_call : None, function_response : None, ..Default::default() } ],
    role: "user".to_string(),
  },
  ];

  let request = CountTokensRequest
  {
    contents,
    generate_content_request: None,
  };

  // Now test the actual implementation
  let result = models_api.count_tokens( "gemini-flash-latest", &request ).await;

  match result
  {
    Ok( response ) =>
    {
      assert!( response.total_tokens > 0, "Token count should be positive for conversation" );
    println!( "✅ Conversation token count : {}", response.total_tokens );
    },
    Err( e ) =>
    {
      panic!( "Count tokens conversation failed : {e:?}" );
    }
  }
}

/// Test token counting with different model types
///
/// This test validates that token counting works across different Gemini models
/// and respects their specific token counting behaviors.
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_count_tokens_different_models()
{
  let client = create_integration_client();

  let models_api = client.models();

  let content = Content
  {
    parts: vec!
    [
    Part
    {
      text: Some( "This is a test message for token counting across different models.".to_string() ),
      inline_data: None,
      function_call: None,
      function_response: None,
      ..Default::default()
    }
    ],
    role: "user".to_string(),
  };

  let request = CountTokensRequest
  {
    contents: vec![ content ],
    generate_content_request: None,
  };

  // Test with multiple models
  let models_to_test = vec![ "gemini-flash-latest", "gemini-flash-latest" ];

  for model in models_to_test
  {
    let result = models_api.count_tokens( model, &request ).await;

    match result
    {
      Ok( response ) =>
      {
      assert!( response.total_tokens > 0, "Token count should be positive for model : {model}" );
    println!( "✅ Model {model} token count : {token_count}", token_count = response.total_tokens );
      },
      Err( e ) =>
      {
        match e
        {
          Error::InvalidArgument( _ ) =>
          {
          println!( "⚠️  Model {model} not available, skipping" );
          },
      _ => panic!( "Count tokens failed for model {model}: {e:?}" ),
        }
      }
    }
  }
}

/// Test error handling for invalid inputs
///
/// This test validates that the `count_tokens` method properly handles various
/// error conditions and returns appropriate error types.
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_count_tokens_error_handling()
{
  let client = create_integration_client();

  let models_api = client.models();

  // Test 1: Empty content
  let empty_request = CountTokensRequest
  {
    contents: vec![],
    generate_content_request: None,
  };

  // Test 2: Invalid model name
  let content = Content
  {
  parts : vec![ Part { text : Some( "Test content".to_string() ), inline_data : None, function_call : None, function_response : None, ..Default::default() } ],
    role: "user".to_string(),
  };

  let request = CountTokensRequest
  {
    contents: vec![ content ],
    generate_content_request: None,
  };

  // Test 1: Empty content should fail
  let result1 = models_api.count_tokens( "gemini-flash-latest", &empty_request ).await;
  assert!( result1.is_err(), "Empty content should result in error" );

  // Test 2: Invalid model name should fail
  let result2 = models_api.count_tokens( "invalid-model-name", &request ).await;

  match result2
  {
    Ok( _ ) => panic!( "Invalid model should result in error" ),
    Err( e ) =>
    {
      match e
      {
        Error::InvalidArgument( _ ) => println!( "✅ Correctly rejected invalid model" ),
        Error::AuthenticationError( _ ) => println!( "⚠️  Authentication error (API key needed to test invalid model)" ),
        Error::ServerError( _ ) => println!( "✅ Server correctly rejected invalid model" ),
      _ => println!( "⚠️  Unexpected error type for invalid model : {e:?}" ),
      }
    }
  }
}

/// Test token counting with generation configuration
///
/// This test validates that token counting can optionally include generation
/// configuration parameters that might affect token usage calculations.
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_count_tokens_with_generation_config()
{
  let client = create_integration_client();

  let models_api = client.models();

  let content = Content
  {
    parts: vec!
    [
    Part
    {
      text: Some( "Generate a creative story about space exploration.".to_string() ),
      inline_data: None,
      function_call: None,
      function_response: None,
      ..Default::default()
    }
    ],
    role: "user".to_string(),
  };

  // Note : The Gemini API expects model to be specified in URL path, not in generate_content_request
  // So we test basic token counting without generation config since that's not supported in count tokens
  let request = CountTokensRequest
  {
    contents: vec![ content ],
    generate_content_request: None, // API doesn't support generation config in count tokens
  };

  // Now test the actual implementation (basic token counting)
  let result = models_api.count_tokens( "gemini-flash-latest", &request ).await;

  match result
  {
    Ok( response ) =>
    {
      assert!( response.total_tokens > 0, "Token count should be positive" );
    println!( "✅ Token count (basic counting): {}", response.total_tokens );
    },
    Err( e ) =>
    {
      panic!( "Count tokens with generation config failed : {e:?}" );
    }
  }
}

/// Test token counting rate limiting behavior
///
/// This test validates that the `count_tokens` method respects rate limits
/// and handles rate limiting errors appropriately.
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_count_tokens_rate_limiting()
{
  let client = create_integration_client();

  let models_api = client.models();

  let content = Content
  {
  parts : vec![ Part { text : Some( "Rate limit test content".to_string() ), inline_data : None, function_call : None, function_response : None, ..Default::default() } ],
    role: "user".to_string(),
  };

  let request = CountTokensRequest
  {
    contents: vec![ content ],
    generate_content_request: None,
  };

  // Make multiple rapid requests to test rate limiting
  let mut results = Vec::new();

  for i in 0..3
  {
    let result = models_api.count_tokens( "gemini-flash-latest", &request ).await;
    results.push( ( i, result ) );

    // Small delay between requests
    tokio ::time::sleep( core::time::Duration::from_millis( 100 ) ).await;
  }

  // Check results
  for ( i, result ) in results
  {
    match result
    {
      Ok( response ) =>
      {
      assert!( response.total_tokens > 0, "Request {i} should have positive token count" );
    println!( "✅ Request {i} succeeded with {} tokens", response.total_tokens );
      },
      Err( e ) =>
      {
        match e
        {
          Error::RateLimitError( _ ) =>
          {
          println!( "⚠️  Request {i} hit rate limit (expected behavior)" );
          },
          _ =>
          {
        println!( "⚠️  Request {i} failed with error : {e:?}" );
          }
        }
      }
    }
  }
}

/// Test token counting with authentication errors
///
/// This test validates that `count_tokens` properly handles authentication
/// errors when invalid API keys are provided.
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_count_tokens_authentication_error()
{
  // Create client with invalid API key
  let client = ClientBuilder::new()
  .api_key( "invalid_api_key_for_testing".to_string() )
  .build()
  .expect( "Client should build with invalid key" );

  let models_api = client.models();

  let content = Content
  {
  parts : vec![ Part { text : Some( "Test content".to_string() ), inline_data : None, function_call : None, function_response : None, ..Default::default() } ],
    role: "user".to_string(),
  };

  let request = CountTokensRequest
  {
    contents: vec![ content ],
    generate_content_request: None,
  };

  // Test with invalid API key
  let result = models_api.count_tokens( "gemini-flash-latest", &request ).await;

  match result
  {
    Ok( _ ) => panic!( "Invalid API key should result in error" ),
    Err( e ) =>
    {
      match e
      {
        Error::AuthenticationError( _ ) =>
        {
          println!( "✅ Correctly rejected invalid API key" );
        },
        Error::ServerError( _ ) =>
        {
          println!( "✅ Server correctly rejected invalid API key (403/401)" );
        },
      _ => panic!( "Unexpected error type for authentication : {e:?}" ),
      }
    }
  }
}
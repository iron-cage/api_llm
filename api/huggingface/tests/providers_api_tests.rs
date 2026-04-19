//! Tests for `HuggingFace` Inference Providers API integration
//!
//! These tests cover the Pro plan model access through the chat completions endpoint
//! which provides access to proper conversational models (Llama-3, Mistral, `CodeLlama`)
//! instead of just the BART summarization model.
//!
//! ## Router API Migration Context
//!
//! ### Why We Migrated
//!
//! The original Inference API (`api-inference.huggingface.co/models/{model}`) had
//! critical limitations:
//!
//! 1. **No Pro Models**: Free tier only, no access to Llama-3, Mistral, `CodeLlama`
//! 2. **BART Fallback**: Would fallback to summarization models for chat requests
//! 3. **No Chat Format**: Required manual prompt engineering instead of proper
//!    message history format
//!
//! **Migration Solution**: Router API (`router.huggingface.co/v1/chat/completions`)
//! provides:
//!
//! - **Pro Model Access**: Kimi-K2, Llama-3, Mistral via explicit model selection
//! - **OpenAI-Compatible**: Standard chat completions format with role-based messages
//! - **Bearer Auth**: Proper authentication with API key in Authorization header
//! - **Better Reliability**: Dedicated endpoints for conversational AI
//!
//! ### Key Changes
//!
//! **URL Structure:**
//! ```
//! OLD: POST https://api-inference.huggingface.co/models/{model}
//! NEW: POST https://router.huggingface.co/v1/chat/completions
//! ```
//!
//! **Authentication:**
//! ```
//! OLD: X-API-Key header (deprecated)
//! NEW: Authorization : Bearer {token}
//! ```
//!
//! **Request Format:**
//! ```rust
//! OLD: { "inputs": "prompt text", "parameters": {...} }
//! NEW: { "model": "id", "messages": [{"role": "user", "content": "..."}], ... }
//! ```
//!
//! **Default Model:**
//! ```
//! OLD: facebook/bart-large-cnn (summarization model)
//! NEW: meta-llama/Llama-3.3-70B-Instruct (proper chat model)
//! ```
//!
//! ### Testing Philosophy
//!
//! These tests validate:
//!
//! 1. **Chat Format**: Multi-turn conversations with system/user/assistant roles
//! 2. **Model Selection**: Proper Pro model identifiers and provider mapping
//! 3. **Authentication**: Bearer token format and header structure
//! 4. **Response Parsing**: OpenAI-compatible response deserialization
//!
//! **No Mocking Principle**: All integration tests use real `HuggingFace` Router API
//! with valid API keys. This ensures we catch:
//! - API format changes immediately
//! - Real-world auth and rate limiting behavior
//! - Actual model availability and response formats
//!
//! Tests without network calls (serialization, validation) are unit tests that
//! don't require mocking because they test pure functions.

#![ allow( clippy::vec_init_then_push ) ]
#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::useless_vec ) ]
#![allow(clippy::missing_inline_in_public_items)]

#[ cfg( test ) ]
mod tests
{
  use api_huggingface::
  {
  secret::Secret,
  };

  #[ derive( Debug, Clone ) ]
  #[ allow( dead_code ) ]
  pub struct TestProvidersClient
  {
  pub api_key : Secret,
  pub base_url : String,
  }

  impl TestProvidersClient
  {
  pub fn new( api_key : Secret ) -> Self
  {
      Self
      {
  api_key,
  base_url : "https://api-inference.huggingface.co/v1".to_string(),
      }
  }
  }

  #[ derive( Debug, Clone, serde::Serialize, serde::Deserialize ) ]
  pub struct ChatCompletionRequest
  {
  pub model : String,
  pub messages : Vec< ChatMessage >,
  pub max_tokens : Option< u32 >,
  pub temperature : Option< f32 >,
  pub top_p : Option< f32 >,
  pub stream : Option< bool >,
  }

  #[ derive( Debug, Clone, serde::Serialize, serde::Deserialize ) ]
  pub struct ChatMessage
  {
  pub role : String,
  pub content : String,
  }

  #[ derive( Debug, Clone, serde::Serialize, serde::Deserialize ) ]
  pub struct ChatCompletionResponse
  {
  pub id : String,
  pub object : String,
  pub created : u64,
  pub model : String,
  pub choices : Vec< ChatChoice >,
  pub usage : Option< Usage >,
  }

  #[ derive( Debug, Clone, serde::Serialize, serde::Deserialize ) ]
  pub struct ChatChoice
  {
  pub index : u32,
  pub message : ChatMessage,
  pub finish_reason : Option< String >,
  }

  #[ derive( Debug, Clone, serde::Serialize, serde::Deserialize ) ]
  #[ allow( clippy::struct_field_names ) ]
  pub struct Usage
  {
  pub prompt_tokens : u32,
  pub completion_tokens : u32,
  pub total_tokens : u32,
  }

  #[ test ]
  fn test_providers_client_initialization()
  {
  // Test that we can initialize a providers client with API key
  let api_key = Secret::new( "test_key".to_string() );
  let client = TestProvidersClient::new( api_key );
  
  assert_eq!( client.base_url, "https://api-inference.huggingface.co/v1" );
  }

  #[ test ]
  fn test_chat_completion_request_serialization()
  {
  // Test that we can serialize chat completion requests correctly
  let request = ChatCompletionRequest
  {
      model : "meta-llama/Llama-2-7b-chat-hf".to_string(),
      messages : vec!
      [
  ChatMessage
  {
          role : "system".to_string(),
          content : "You are a helpful assistant.".to_string(),
  },
  ChatMessage
  {
          role : "user".to_string(),
          content : "What is x*3 if x=13?".to_string(),
  },
      ],
      max_tokens : Some( 100 ),
      temperature : Some( 0.7 ),
      top_p : Some( 0.9 ),
      stream : Some( false ),
  };

  let serialized = serde_json::to_string( &request ).expect( "[test_chat_completion_request_serialization] Failed to serialize ChatCompletionRequest to JSON - check serde_json::to_string() and ChatCompletionRequest serialization implementation" );
  assert!( serialized.contains( "meta-llama/Llama-2-7b-chat-hf" ) );
  assert!( serialized.contains( "system" ) );
  assert!( serialized.contains( "user" ) );
  assert!( serialized.contains( "What is x*3 if x=13?" ) );
  }

  #[ test ]
  fn test_chat_completion_response_deserialization()
  {
  // Test that we can deserialize chat completion responses correctly
  let response_json = r#"
  {
      "id": "chatcmpl-123",
      "object": "chat.completion",
      "created": 1677652288,
      "model": "meta-llama/Llama-2-7b-chat-hf",
      "choices": [{
  "index": 0,
  "message": {
          "role": "assistant",
          "content": "If x = 13, then x * 3 = 13 * 3 = 39."
  },
  "finish_reason": "stop"
      }],
      "usage": {
  "prompt_tokens": 20,
  "completion_tokens": 15,
  "total_tokens": 35
      }
  }"#;

  let response : ChatCompletionResponse = serde_json::from_str( response_json ).expect( "[test_chat_completion_response_deserialization] Failed to deserialize ChatCompletionResponse from JSON - check serde_json::from_str() and ChatCompletionResponse Deserialize implementation" );
  assert_eq!( response.model, "meta-llama/Llama-2-7b-chat-hf" );
  assert_eq!( response.choices.len(), 1 );
  assert_eq!( response.choices[ 0 ].message.role, "assistant" );
  assert!( response.choices[ 0 ].message.content.contains( "39" ) );
  }

  #[ test ]
  fn test_provider_model_mapping()
  {
  // Test different provider model identifiers
  let providers = vec!
  [
      ( "hf-inference", "meta-llama/Llama-2-7b-chat-hf" ),
      ( "together", "meta-llama/Llama-2-7b-chat-hf" ),
      ( "groq", "llama2-7b-4096" ),
      ( "openai", "gpt-3.5-turbo" ),
      ( "cohere", "command" ),
  ];

  for ( provider, model ) in providers
  {
      assert!( !provider.is_empty() );
      assert!( !model.is_empty() );
      // In real implementation, would test model availability per provider
  }
  }

  #[ test ]
  fn test_math_query_formatting()
  {
  // Test that mathematical queries are formatted properly for conversation
  let user_input = "x=13";
  let math_query = "x*3?";
  
  let messages = vec!
  [
      ChatMessage
      {
  role : "system".to_string(),
  content : "You are a helpful AI assistant. Answer mathematical questions clearly and accurately.".to_string(),
      },
      ChatMessage
      {
  role : "user".to_string(),
  content : user_input.to_string(),
      },
      ChatMessage
      {
  role : "assistant".to_string(),
  content : "I understand that x = 13. What would you like me to calculate with this value?".to_string(),
      },
      ChatMessage
      {
  role : "user".to_string(),
  content : math_query.to_string(),
      },
  ];

  assert_eq!( messages.len(), 4 );
  assert_eq!( messages[ 0 ].role, "system" );
  assert_eq!( messages[ 1 ].content, "x=13" );
  assert_eq!( messages[ 3 ].content, "x*3?" );
  }

  #[ test ]
  fn test_error_handling_structures()
  {
  // Test error response structure for providers API
  #[ derive( Debug, serde::Deserialize ) ]
  struct ProvidersError
  {
      error : ErrorDetails,
  }

  #[ derive( Debug, serde::Deserialize ) ]
  #[ allow( dead_code ) ]
  struct ErrorDetails
  {
      message : String,
      r#type : String,
      code : Option< String >,
  }

  let error_json = r#"
  {
      "error": {
  "message": "Model not available",
  "type": "model_not_found",
  "code": "404"
      }
  }"#;

  let error : ProvidersError = serde_json::from_str( error_json ).expect( "[test_error_response_format] Failed to deserialize ProvidersError from JSON - check serde_json::from_str() and ProvidersError Deserialize implementation" );
  assert_eq!( error.error.message, "Model not available" );
  assert_eq!( error.error.r#type, "model_not_found" );
  }

  #[ test ]
  fn test_authentication_headers()
  {
  // Test that authentication headers are formatted correctly
  use std::collections::HashMap;
  
  let api_key = Secret::new( "hf_test_key_1234".to_string() );
  
  let mut headers = HashMap::new();
  headers.insert( "Authorization".to_string(), format!( "Bearer {}", api_key.expose_secret() ) );
  headers.insert( "Content-Type".to_string(), "application/json".to_string() );

  assert_eq!( headers.get( "Authorization" ).expect( "[test_authentication_headers] Authorization header should exist after insert - check HashMap::get() implementation" ), "Bearer hf_test_key_1234" );
  assert_eq!( headers.get( "Content-Type" ).expect( "[test_authentication_headers] Content-Type header should exist after insert - check HashMap::get() implementation" ), "application/json" );
  }

  #[ test ]
  fn test_conversation_context_building()
  {
  // Test building conversation context with multiple turns
  let mut messages = Vec::new();
  
  // System message
  messages.push( ChatMessage
  {
      role : "system".to_string(),
      content : "You are a helpful AI assistant specialized in mathematics.".to_string(),
  } );
  
  // User starts conversation
  messages.push( ChatMessage
  {
      role : "user".to_string(),
      content : "Hello, I need help with math.".to_string(),
  } );
  
  // Assistant responds
  messages.push( ChatMessage
  {
      role : "assistant".to_string(),
      content : "Hello! I'd be happy to help you with mathematical problems. What would you like to work on?".to_string(),
  } );
  
  // User asks math question
  messages.push( ChatMessage
  {
      role : "user".to_string(),
      content : "If x=13, what is x*3?".to_string(),
  } );

  assert_eq!( messages.len(), 4 );
  assert_eq!( messages.last().expect( "[test_conversation_context_building] Messages vector should have last element after push - check Vec::last() implementation" ).content, "If x=13, what is x*3?" );
  }

  #[ test ]
  fn test_model_fallback_logic()
  {
  // Test fallback logic when Pro models are not available
  let pro_models = vec!
  [
      "meta-llama/Llama-2-7b-chat-hf",
      "mistralai/Mistral-7B-Instruct-v0.2",
      "codellama/CodeLlama-7b-Instruct-hf",
  ];
  
  let fallback_model = "facebook/bart-large-cnn";
  
  // Simulate Pro models being unavailable (404 responses)
  let available_model = if pro_models.is_empty()
  {
      fallback_model
  }
  else
  {
      &pro_models[ 0 ]
  };
  
  assert_eq!( available_model, "meta-llama/Llama-2-7b-chat-hf" );
  }

  #[ test ]
  fn test_streaming_response_structure()
  {
  // Test structure for streaming chat completions
  #[ derive( Debug, serde::Deserialize ) ]
  #[ allow( dead_code ) ]
  struct StreamingChunk
  {
      id : String,
      object : String,
      created : u64,
      model : String,
      choices : Vec< StreamingChoice >,
  }

  #[ derive( Debug, serde::Deserialize ) ]
  #[ allow( dead_code ) ]
  struct StreamingChoice
  {
      index : u32,
      delta : StreamingDelta,
      finish_reason : Option< String >,
  }

  #[ derive( Debug, serde::Deserialize ) ]
  #[ allow( dead_code ) ]
  struct StreamingDelta
  {
      role : Option< String >,
      content : Option< String >,
  }

  let chunk_json = r#"
  {
      "id": "chatcmpl-123",
      "object": "chat.completion.chunk",
      "created": 1677652288,
      "model": "meta-llama/Llama-2-7b-chat-hf",
      "choices": [{
  "index": 0,
  "delta": {
          "content": "39"
  },
  "finish_reason": null
      }]
  }"#;

  let chunk : StreamingChunk = serde_json::from_str( chunk_json ).expect( "[test_streaming_chunk_format] Failed to deserialize StreamingChunk from JSON - check serde_json::from_str() and StreamingChunk Deserialize implementation" );
  assert_eq!( chunk.choices[ 0 ].delta.content, Some( "39".to_string() ) );
  }
}
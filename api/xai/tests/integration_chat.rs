//! Integration tests for chat completion API.
//!
//! # Purpose
//!
//! These tests validate real API interactions with XAI's Grok models.
//! They require valid API credentials and network access.
//!
//! # Key Insights
//!
//! - **Model Selection**: Using `grok-3` (current stable). Previous `grok-beta`
//!   was deprecated 2025-09-15. Always check XAI docs for latest models.
//!
//! - **URL Construction**: Base URL must have trailing slash (`https://api.x.ai/v1/`)
//!   and endpoint paths must NOT have leading slash (`chat/completions`).
//!   This is due to `Url::join()` behavior - leading slash replaces entire path.
//!
//! - **Error Handling**: Tests fail loudly if API key is missing or invalid.
//!   This is intentional - integration tests should not silently pass.
//!
//! # Running Tests
//!
//! ```bash
//! cargo test --features integration --test integration_chat
//! ```

#![ cfg( feature = "integration" ) ]

mod inc;
use inc::test_helpers::create_test_client;

use api_xai::{ ChatCompletionRequest, Message, ClientApiAccessors };

#[ tokio::test ]
async fn test_chat_completion_basic()
{
  let client = create_test_client();

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Say hello" ) ] )
    .max_tokens( 20u32 )
    .form();

  let response = client.chat().create( request ).await
    .expect( "Chat completion should succeed" );

  // Verify response structure
  assert!( !response.id.is_empty(), "Response should have an ID" );
  assert_eq!( response.object, "chat.completion", "Object type should be chat.completion" );
  assert!( !response.choices.is_empty(), "Response should have at least one choice" );

  // Verify content
  let first_choice = &response.choices[ 0 ];
  assert_eq!( first_choice.index, 0, "First choice should have index 0" );
  assert!( first_choice.message.content.is_some(), "Message should have content" );

  let content = first_choice.message.content.as_ref().unwrap();
  assert!( !content.is_empty(), "Content should not be empty" );

  // Verify usage
  assert!( response.usage.prompt_tokens > 0, "Should have prompt tokens" );
  assert!( response.usage.completion_tokens > 0, "Should have completion tokens" );
  assert_eq!(
    response.usage.total_tokens,
    response.usage.prompt_tokens + response.usage.completion_tokens,
    "Total tokens should equal sum of prompt and completion tokens"
  );

  println!( "✅ Basic chat completion test passed" );
  println!( "Response : {content}" );
  println!( "Usage : {} prompt + {} completion = {} total tokens",
    response.usage.prompt_tokens,
    response.usage.completion_tokens,
    response.usage.total_tokens
  );
}

#[ tokio::test ]
async fn test_chat_completion_with_system_message()
{
  let client = create_test_client();

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![
      Message::system( "You are a helpful assistant that speaks like a pirate" ),
      Message::user( "Say hello" ),
    ] )
    .max_tokens( 30u32 )
    .form();

  let response = client.chat().create( request ).await
    .expect( "Chat completion with system message should succeed" );

  assert!( !response.choices.is_empty() );
  let content = response.choices[ 0 ].message.content.as_ref().unwrap();
  assert!( !content.is_empty() );

  println!( "✅ Chat with system message test passed" );
  println!( "Pirate response : {content}" );
}

#[ tokio::test ]
async fn test_chat_completion_with_temperature()
{
  let client = create_test_client();

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Count from 1 to 3" ) ] )
    .temperature( 0.2 ) // Low temperature for deterministic output
    .max_tokens( 20u32 )
    .form();

  let response = client.chat().create( request ).await
    .expect( "Chat completion with temperature should succeed" );

  assert!( !response.choices.is_empty() );
  let content = response.choices[ 0 ].message.content.as_ref().unwrap();
  assert!( !content.is_empty() );

  println!( "✅ Chat with temperature test passed" );
  println!( "Response : {content}" );
}

#[ tokio::test ]
async fn test_chat_completion_with_max_tokens()
{
  let client = create_test_client();

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Write a long story" ) ] )
    .max_tokens( 10u32 ) // Very limited
    .form();

  let response = client.chat().create( request ).await
    .expect( "Chat completion with max_tokens should succeed" );

  assert!( !response.choices.is_empty() );

  // Should respect max_tokens
  assert!(
    response.usage.completion_tokens <= 10,
    "Completion tokens should not exceed max_tokens"
  );

  // Might finish with "length" reason due to token limit
  let finish_reason = response.choices[ 0 ].finish_reason.as_deref();
  assert!(
    finish_reason == Some( "length" ) || finish_reason == Some( "stop" ),
    "Finish reason should be 'length' or 'stop', got : {finish_reason:?}"
  );

  println!( "✅ Chat with max_tokens test passed" );
  println!( "Finish reason : {finish_reason:?}" );
  println!( "Completion tokens : {}", response.usage.completion_tokens );
}

#[ tokio::test ]
async fn test_chat_completion_with_multiple_messages()
{
  let client = create_test_client();

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![
      Message::user( "My name is Alice" ),
      Message::assistant( "Hello Alice! Nice to meet you." ),
      Message::user( "What is my name?" ),
    ] )
    .max_tokens( 20u32 )
    .form();

  let response = client.chat().create( request ).await
    .expect( "Chat completion with conversation history should succeed" );

  assert!( !response.choices.is_empty() );
  let content = response.choices[ 0 ].message.content.as_ref().unwrap();

  // The model should remember the name from the conversation
  // (though we can't guarantee exact phrasing)
  assert!( !content.is_empty() );

  println!( "✅ Chat with conversation history test passed" );
  println!( "Response : {content}" );
}

#[ tokio::test ]
async fn test_chat_completion_model_grok_beta()
{
  let client = create_test_client();

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Hello" ) ] )
    .max_tokens( 10u32 )
    .form();

  let response = client.chat().create( request ).await
    .expect( "Chat with grok-3 should succeed" );

  // Verify the model used
  assert!(
    response.model.contains( "grok" ),
    "Response model should contain 'grok', got : {}",
    response.model
  );

  println!( "✅ Grok-beta model test passed" );
  println!( "Model used : {}", response.model );
}

#[ tokio::test ]
async fn test_chat_completion_error_handling_invalid_model()
{
  let client = create_test_client();

  let request = ChatCompletionRequest::former()
    .model( "nonexistent-model-12345".to_string() )
    .messages( vec![ Message::user( "Hello" ) ] )
    .form();

  let result = client.chat().create( request ).await;

  // Should return an error for invalid model
  assert!( result.is_err(), "Should fail with invalid model" );

  let error = result.unwrap_err();
  let error_str = format!( "{error:?}" );

  println!( "✅ Invalid model error handling test passed" );
  println!( "Error : {error_str}" );
}

#[ tokio::test ]
async fn test_chat_completion_empty_message_handling()
{
  let client = create_test_client();

  // XAI might accept empty messages unlike some other APIs
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "" ) ] )
    .max_tokens( 10u32 )
    .form();

  let result = client.chat().create( request ).await;

  // Log the result (API might accept or reject empty messages)
  match result {
    Ok( response ) => {
      println!( "✅ Empty message test : API accepted empty message" );
      println!( "Response : {:?}", response.choices[ 0 ].message.content );
    }
    Err( e ) => {
      println!( "✅ Empty message test : API rejected empty message (expected)" );
      println!( "Error : {e:?}" );
    }
  }
}

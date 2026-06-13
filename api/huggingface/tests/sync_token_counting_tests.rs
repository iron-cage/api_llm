//! Integration tests for synchronous token counting

#![ cfg( feature = "sync" ) ]

use api_huggingface::sync::SyncClient;
use api_huggingface::token_counter::CountingStrategy;
use api_huggingface::providers::ChatMessage;

#[ test ]
fn test_sync_token_counter_basic()
{
  let client = SyncClient::new( "test_key".to_string() )
  .expect( "Failed to create client" );

  let counter = client.token_counter();
  let count = counter.count_tokens( "Hello world" );

  assert!( count.total > 0, "Should count tokens" );
  assert!( count.characters > 0, "Should count characters" );
}

#[ test ]
fn test_sync_token_counter_with_strategy()
{
  let client = SyncClient::new( "test_key".to_string() )
  .expect( "Failed to create client" );

  let counter = client.token_counter_with_strategy( CountingStrategy::CharacterBased );
  let count = counter.count_tokens( "Hello world" );

  assert!( count.total > 0, "Should count tokens" );
  assert_eq!( count.strategy, CountingStrategy::CharacterBased );
}

#[ test ]
fn test_sync_token_counter_empty_text()
{
  let client = SyncClient::new( "test_key".to_string() )
  .expect( "Failed to create client" );

  let counter = client.token_counter();
  let count = counter.count_tokens( "" );

  assert_eq!( count.total, 0, "Empty text should have 0 tokens" );
  assert_eq!( count.characters, 0, "Empty text should have 0 characters" );
}

#[ test ]
fn test_sync_token_counter_long_text()
{
  let client = SyncClient::new( "test_key".to_string() )
  .expect( "Failed to create client" );

  let long_text = "This is a very long text. ".repeat( 100 );

  let counter = client.token_counter();
  let count = counter.count_tokens( &long_text );

  assert!( count.total > 100, "Long text should have many tokens" );
}

#[ test ]
fn test_sync_token_counter_unicode()
{
  let client = SyncClient::new( "test_key".to_string() )
  .expect( "Failed to create client" );

  let counter = client.token_counter();
  let count = counter.count_tokens( "Hello 世界 🌍" );

  assert!( count.total > 0, "Should count unicode tokens" );
  assert!( count.characters > 0, "Should count unicode characters" );
}

#[ test ]
fn test_sync_token_counter_multiple_texts()
{
  let client = SyncClient::new( "test_key".to_string() )
  .expect( "Failed to create client" );

  let texts = vec![ "Hello", "world", "!" ];

  let counter = client.token_counter();
  let count = counter.count_texts( &texts );

  assert!( count.total > 0, "Should count tokens from multiple texts" );
}

#[ test ]
fn test_sync_token_counter_messages()
{
  let client = SyncClient::new( "test_key".to_string() )
  .expect( "Failed to create client" );

  let messages = vec![
  ChatMessage
  {
      role : "user".to_string(),
      content : "Hello".to_string(),
      tool_calls : None,
      tool_call_id : None,
  },
  ChatMessage
  {
      role : "assistant".to_string(),
      content : "Hi there!".to_string(),
      tool_calls : None,
      tool_call_id : None,
  },
  ];

  let counter = client.token_counter();
  let count = counter.count_messages( &messages );

  assert!( count.total > 0, "Should count tokens from messages" );
}

#[ test ]
fn test_sync_token_counter_all_strategies()
{
  let client = SyncClient::new( "test_key".to_string() )
  .expect( "Failed to create client" );

  let text = "This is a test message";

  // Test Estimation strategy
  let counter_estimation = client.token_counter_with_strategy( CountingStrategy::Estimation );
  let count_estimation = counter_estimation.count_tokens( text );

  // Test WordBased strategy
  let counter_word = client.token_counter_with_strategy( CountingStrategy::WordBased );
  let count_word = counter_word.count_tokens( text );

  // Test CharacterBased strategy
  let counter_char = client.token_counter_with_strategy( CountingStrategy::CharacterBased );
  let count_char = counter_char.count_tokens( text );

  // All should count something
  assert!( count_estimation.total > 0 );
  assert!( count_word.total > 0 );
  assert!( count_char.total > 0 );

  // Strategies might produce different counts
  assert_eq!( count_estimation.strategy, CountingStrategy::Estimation );
  assert_eq!( count_word.strategy, CountingStrategy::WordBased );
  assert_eq!( count_char.strategy, CountingStrategy::CharacterBased );
}

#[ test ]
fn test_sync_token_counter_reusable()
{
  let client = SyncClient::new( "test_key".to_string() )
  .expect( "Failed to create client" );

  // Get counter multiple times
  let counter1 = client.token_counter();
  let count1 = counter1.count_tokens( "First" );

  let counter2 = client.token_counter();
  let count2 = counter2.count_tokens( "Second" );

  assert!( count1.total > 0 );
  assert!( count2.total > 0 );
}

#[ test ]
fn test_sync_token_counter_cost_calculation()
{
  let client = SyncClient::new( "test_key".to_string() )
  .expect( "Failed to create client" );

  let counter = client.token_counter();
  let count = counter.count_tokens( "Hello world" );

  let cost = count.cost_units();
  assert!( cost > 0.0, "Should calculate cost units" );
}

#[ test ]
fn test_sync_token_counter_whitespace()
{
  let client = SyncClient::new( "test_key".to_string() )
  .expect( "Failed to create client" );

  let counter = client.token_counter();

  let _count_spaces = counter.count_tokens( "   " );
  let count_mixed = counter.count_tokens( "  hello  world  " );

  // Spaces count is valid (no assertion needed for non-negative usize)
  assert!( count_mixed.total > 0 );
}

#[ test ]
fn test_sync_token_counter_special_characters()
{
  let client = SyncClient::new( "test_key".to_string() )
  .expect( "Failed to create client" );

  let counter = client.token_counter();
  let count = counter.count_tokens( "!@#$%^&*()" );

  // Token count is valid (no assertion needed for non-negative usize)
  assert!( count.characters > 0, "Should count special characters" );
}

#[ test ]
fn test_sync_token_counter_newlines()
{
  let client = SyncClient::new( "test_key".to_string() )
  .expect( "Failed to create client" );

  let counter = client.token_counter();
  let count = counter.count_tokens( "Line 1\nLine 2\nLine 3" );

  assert!( count.total > 0, "Should handle newlines" );
}

#[ test ]
fn test_sync_token_counter_tabs()
{
  let client = SyncClient::new( "test_key".to_string() )
  .expect( "Failed to create client" );

  let counter = client.token_counter();
  let count = counter.count_tokens( "Column\tColumn\tColumn" );

  assert!( count.total > 0, "Should handle tabs" );
}

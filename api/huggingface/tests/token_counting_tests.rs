//! Integration tests for Token Counting
//!
//! These tests verify the token counting functionality with various strategies and inputs.
//!
//! ## Test Coverage
//!
//! - All counting strategies ( Estimation, `WordBased`, `CharacterBased` )
//! - Message token counting
//! - Multi-text counting
//! - Cost estimation
//! - Large text handling
//! - Unicode support
//! - Edge cases

#![ allow( clippy::float_cmp ) ]
#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::uninlined_format_args ) ]

use api_huggingface::token_counter::{TokenCounter, CountingStrategy};
use api_huggingface::providers::ChatMessage;

// ============================================================================
// Strategy Tests
// ============================================================================

#[ test ]
fn test_estimation_strategy_simple_text() 
{
  let counter = TokenCounter::new( CountingStrategy::Estimation );
  let text = "The quick brown fox jumps over the lazy dog";
  let count = counter.count_tokens( text );

  assert_eq!( count.strategy, CountingStrategy::Estimation );
  assert!( count.total > 0 );
  assert_eq!( count.characters, 43 );
  assert_eq!( count.total, 11 ); // ceil( 43 / 4 ) = 11
}

#[ test ]
fn test_word_based_strategy() 
{
  let counter = TokenCounter::new( CountingStrategy::WordBased );
  let text = "Hello world from Rust programming";
  let count = counter.count_tokens( text );

  assert_eq!( count.strategy, CountingStrategy::WordBased );
  assert_eq!( count.total, 7 ); // 5 words * 1.3 = 6.5, ceil = 7
}

#[ test ]
fn test_character_based_strategy() 
{
  let counter = TokenCounter::new( CountingStrategy::CharacterBased );
  let text = "Test message";
  let count = counter.count_tokens( text );

  assert_eq!( count.strategy, CountingStrategy::CharacterBased );
  assert_eq!( count.characters, 12 );
  assert_eq!( count.total, 4 ); // ceil( 12 / 3.5 ) = 4
}

#[ test ]
fn test_all_strategies_on_same_text() 
{
  let text = "The quick brown fox jumps";

  let est = TokenCounter::new( CountingStrategy::Estimation ).count_tokens( text );
  let word = TokenCounter::new( CountingStrategy::WordBased ).count_tokens( text );
  let char = TokenCounter::new( CountingStrategy::CharacterBased ).count_tokens( text );

  // All should count same characters
  assert_eq!( est.characters, word.characters );
  assert_eq!( word.characters, char.characters );

  // CharacterBased should differ from others ( most accurate )
  assert_ne!( char.total, est.total );
}

// ============================================================================
// Message Counting Tests
// ============================================================================

#[ test ]
fn test_count_single_message() 
{
  let counter = TokenCounter::new( CountingStrategy::CharacterBased );
  let messages = vec![
  ChatMessage {
      role : "user".to_string( ),
      content : "Hello, how are you?".to_string( ),
      tool_calls : None,
      tool_call_id : None,
  }
  ];

  let count = counter.count_messages( &messages );

  // Should include role + content + overhead
  assert!( count.total > 0 );
  // Role is "user" ( 4 chars ) + content ( 19 chars ) + overhead ( 4 tokens )
  // = ( 4 + 19 ) / 3.5 + 4 = ceil( 6.57 ) + 4 = 7 + 4 = 11
  assert!( count.total >= 10 );
}

#[ test ]
fn test_count_multiple_messages() 
{
  let counter = TokenCounter::new( CountingStrategy::CharacterBased );
  let messages = vec![
  ChatMessage {
      role : "user".to_string( ),
      content : "Hello".to_string( ),
      tool_calls : None,
      tool_call_id : None,
  },
  ChatMessage {
      role : "assistant".to_string( ),
      content : "Hi there!".to_string( ),
      tool_calls : None,
      tool_call_id : None,
  },
  ChatMessage {
      role : "user".to_string( ),
      content : "How are you?".to_string( ),
      tool_calls : None,
      tool_call_id : None,
  },
  ];

  let count = counter.count_messages( &messages );

  // 3 messages with overhead
  assert!( count.total > 10 );
  assert!( count.characters > 20 );
}

#[ test ]
fn test_empty_messages() 
{
  let counter = TokenCounter::new( CountingStrategy::Estimation );
  let messages : Vec< ChatMessage > = vec![ ];

  let count = counter.count_messages( &messages );

  assert_eq!( count.total, 0 );
  assert_eq!( count.characters, 0 );
}

#[ test ]
fn test_message_with_empty_content() 
{
  let counter = TokenCounter::new( CountingStrategy::CharacterBased );
  let messages = vec![
  ChatMessage {
      role : "user".to_string( ),
      content : String::new( ),
      tool_calls : None,
      tool_call_id : None,
  }
  ];

  let count = counter.count_messages( &messages );

  // Should still have role + overhead
  assert!( count.total >= 4 );
}

// ============================================================================
// Multi-Text Counting Tests
// ============================================================================

#[ test ]
fn test_count_multiple_texts() 
{
  let counter = TokenCounter::new( CountingStrategy::Estimation );
  let texts = vec!["Hello", "world", "from", "Rust" ];

  let count = counter.count_texts( &texts );

  // Combined : "Helloworldfromrust" = 18 chars
  assert_eq!( count.characters, 18 );
  assert_eq!( count.total, 5 ); // ceil( 18 / 4 ) = 5
}

#[ test ]
fn test_count_empty_texts() 
{
  let counter = TokenCounter::new( CountingStrategy::WordBased );
  let texts : Vec< &str > = vec![ ];

  let count = counter.count_texts( &texts );

  assert_eq!( count.total, 0 );
  assert_eq!( count.characters, 0 );
}

#[ test ]
fn test_count_texts_with_empty_strings() 
{
  let counter = TokenCounter::new( CountingStrategy::CharacterBased );
  let texts = vec!["", "", "hello", "" ];

  let count = counter.count_texts( &texts );

  assert_eq!( count.characters, 5 ); // Only "hello"
}

// ============================================================================
// Cost Estimation Tests
// ============================================================================

#[ test ]
fn test_cost_units_calculation() 
{
  let counter = TokenCounter::new( CountingStrategy::Estimation );
  let text = "a".repeat( 4000 ); // 4000 chars = 1000 tokens

  let count = counter.count_tokens( &text );
  let cost = count.cost_units( );

  assert_eq!( count.total, 1000 );
  assert_eq!( cost, 1.0 ); // 1000 / 1000 = 1.0
}

#[ test ]
fn test_cost_units_fractional() 
{
  let counter = TokenCounter::new( CountingStrategy::Estimation );
  let text = "a".repeat( 2000 ); // 2000 chars = 500 tokens

  let count = counter.count_tokens( &text );
  let cost = count.cost_units( );

  assert_eq!( count.total, 500 );
  assert_eq!( cost, 0.5 ); // 500 / 1000 = 0.5
}

#[ test ]
fn test_cost_units_large_text() 
{
  let counter = TokenCounter::new( CountingStrategy::CharacterBased );
  let text = "a".repeat( 35000 ); // 35000 chars = 10000 tokens

  let count = counter.count_tokens( &text );
  let cost = count.cost_units( );

  assert_eq!( count.total, 10000 );
  assert_eq!( cost, 10.0 ); // 10000 / 1000 = 10.0
}

// ============================================================================
// Large Text Tests
// ============================================================================

#[ test ]
fn test_very_large_text() 
{
  let counter = TokenCounter::new( CountingStrategy::Estimation );
  let text = "a".repeat( 100_000 );

  let count = counter.count_tokens( &text );

  assert_eq!( count.characters, 100_000 );
  assert_eq!( count.total, 25000 ); // 100000 / 4 = 25000
}

#[ test ]
fn test_large_text_word_based() 
{
  let counter = TokenCounter::new( CountingStrategy::WordBased );
  let text = ( 0..10000 ).map( |_| "word " ).collect::< String >( );

  let count = counter.count_tokens( &text );

  // 10000 words * 1.3 = 13000
  assert_eq!( count.total, 13000 );
}

// ============================================================================
// Unicode Tests
// ============================================================================

#[ test ]
fn test_unicode_characters() 
{
  let counter = TokenCounter::new( CountingStrategy::CharacterBased );
  let text = "Hello 世界 🌍";

  let count = counter.count_tokens( text );

  // 10 Unicode characters ( including spaces and emoji )
  assert_eq!( count.characters, 10 );
  assert_eq!( count.total, 3 ); // ceil( 10 / 3.5 ) = 3
}

#[ test ]
fn test_emoji_text() 
{
  let counter = TokenCounter::new( CountingStrategy::Estimation );
  let text = "🚀🌟💻🎉";

  let count = counter.count_tokens( text );

  assert_eq!( count.characters, 4 );
  assert_eq!( count.total, 1 ); // ceil( 4 / 4 ) = 1
}

#[ test ]
fn test_mixed_scripts() 
{
  let counter = TokenCounter::new( CountingStrategy::WordBased );
  let text = "Hello мир world 世界";

  let count = counter.count_tokens( text );

  // 4 words * 1.3 = 5.2, ceil = 6
  assert_eq!( count.total, 6 );
}

// ============================================================================
// Edge Cases
// ============================================================================

#[ test ]
fn test_empty_string() 
{
  let counter = TokenCounter::new( CountingStrategy::Estimation );
  let count = counter.count_tokens( "" );

  assert_eq!( count.total, 0 );
  assert_eq!( count.characters, 0 );
}

#[ test ]
fn test_whitespace_only() 
{
  let counter = TokenCounter::new( CountingStrategy::WordBased );
  let text = "     ";

  let count = counter.count_tokens( text );

  // No words, so 0 tokens
  assert_eq!( count.total, 0 );
}

#[ test ]
fn test_newlines_and_tabs() 
{
  let counter = TokenCounter::new( CountingStrategy::CharacterBased );
  let text = "Hello\n\tWorld";

  let count = counter.count_tokens( text );

  assert_eq!( count.characters, 12 ); // Including \n and \t
}

#[ test ]
fn test_single_character() 
{
  let counter = TokenCounter::new( CountingStrategy::Estimation );
  let count = counter.count_tokens( "a" );

  assert_eq!( count.characters, 1 );
  assert_eq!( count.total, 1 ); // ceil( 1 / 4 ) = 1
}

// ============================================================================
// Strategy Change Tests
// ============================================================================

#[ test ]
fn test_change_strategy() 
{
  let mut counter = TokenCounter::new( CountingStrategy::Estimation );
  let text = "Hello world from Rust programming language";

  let count1 = counter.count_tokens( text );
  assert_eq!( count1.strategy, CountingStrategy::Estimation );

  counter.set_strategy( CountingStrategy::CharacterBased );

  let count2 = counter.count_tokens( text );
  assert_eq!( count2.strategy, CountingStrategy::CharacterBased );

  // Different strategies should give different results
  assert_ne!( count1.total, count2.total );
}

#[ test ]
fn test_default_counter() 
{
  let counter = TokenCounter::default( );

  assert_eq!( counter.strategy( ), CountingStrategy::CharacterBased );
}

// ============================================================================
// Comparison Tests
// ============================================================================

#[ test ]
fn test_estimation_vs_character_based() 
{
  let text = "The quick brown fox";

  let est = TokenCounter::new( CountingStrategy::Estimation ).count_tokens( text );
  let char = TokenCounter::new( CountingStrategy::CharacterBased ).count_tokens( text );

  // CharacterBased should generally be more accurate ( slightly higher )
  assert!( char.total <= est.total + 2 );
}

#[ test ]
fn test_word_based_with_punctuation() 
{
  let counter = TokenCounter::new( CountingStrategy::WordBased );
  let text1 = "Hello, world!";
  let text2 = "Hello world";

  let count1 = counter.count_tokens( text1 );
  let count2 = counter.count_tokens( text2 );

  // Both should count 2 words
  assert_eq!( count1.total, count2.total );
}

// ============================================================================
// Performance Tests
// ============================================================================

#[ test ]
fn test_repeated_counting_same_text() 
{
  let counter = TokenCounter::new( CountingStrategy::CharacterBased );
  let text = "This is a test message";

  let count1 = counter.count_tokens( text );
  let count2 = counter.count_tokens( text );

  assert_eq!( count1.total, count2.total );
  assert_eq!( count1.characters, count2.characters );
}

#[ test ]
fn test_count_long_conversation() 
{
  let counter = TokenCounter::new( CountingStrategy::CharacterBased );
  let mut messages = vec![ ];

  for i in 0..100
  {
  messages.push( ChatMessage {
      role : if i % 2 == 0 { "user" } else { "assistant" }.to_string( ),
      content : format!( "This is message number {}", i ),
      tool_calls : None,
      tool_call_id : None,
  } );
  }

  let count = counter.count_messages( &messages );

  // 100 messages should have substantial token count
  assert!( count.total > 200 );
  assert!( count.characters > 1000 );
}

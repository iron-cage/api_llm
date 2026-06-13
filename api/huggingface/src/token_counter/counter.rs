//! Token Counter Implementation
//!
//! Provides token counting with multiple strategies.
//!
//! ## Strategies
//!
//! - **Estimation**: Characters / 4 ( fast, rough estimate )
//! - **`WordBased`**: Word count * 1.3 ( moderate accuracy )
//! - **`CharacterBased`**: Characters / 3.5 ( better accuracy )
//!
//! ## Usage
//!
//! ```no_run
//! use api_huggingface::token_counter::{TokenCounter, CountingStrategy};
//!
//! let counter = TokenCounter::new( CountingStrategy::CharacterBased );
//! let count = counter.count_tokens( "Your text here" );
//! println!( "Tokens : {}", count.total );
//! ```

use crate::providers::ChatMessage;

/// Token counting strategy
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum CountingStrategy 
{
  /// Fast estimation : characters / 4
  Estimation,
  /// Word-based : word count * 1.3
  WordBased,
  /// Character-based : characters / 3.5 ( more accurate )
  CharacterBased,
}

impl Default for CountingStrategy 
{
  #[ inline ]
  fn default() -> Self 
  {
  Self::CharacterBased
  }
}

/// Token count result
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub struct TokenCount 
{
  /// Total token count
  pub total : usize,
  /// Character count ( for reference )
  pub characters : usize,
  /// Strategy used for counting
  pub strategy : CountingStrategy,
}

impl TokenCount 
{
  /// Create new token count
  #[ inline ]
  #[ must_use ]
  pub fn new( total : usize, characters : usize, strategy : CountingStrategy ) -> Self 
  {
  Self {
      total,
      characters,
      strategy,
  }
  }

  /// Get estimated cost multiplier ( tokens / 1000 )
  #[ inline ]
  #[ must_use ]
  pub fn cost_units( &self ) -> f64 
  {
  self.total as f64 / 1000.0
  }
}

/// Token counter
#[ derive( Debug, Clone ) ]
pub struct TokenCounter 
{
  strategy : CountingStrategy,
}

impl TokenCounter 
{
  /// Create new token counter with given strategy
  #[ inline ]
  #[ must_use ]
  pub fn new( strategy : CountingStrategy ) -> Self 
  {
  Self { strategy }
  }

  /// Count tokens in text
  #[ inline ]
  #[ must_use ]
  pub fn count_tokens( &self, text : &str ) -> TokenCount 
  {
  let characters = text.chars( ).count( );

  let total = match self.strategy
  {
      CountingStrategy::Estimation => Self::estimate_tokens( text ),
      CountingStrategy::WordBased => Self::word_based_count( text ),
      CountingStrategy::CharacterBased => Self::character_based_count( text ),
  };

  TokenCount::new( total, characters, self.strategy )
  }

  /// Count tokens in multiple texts
  #[ inline ]
  #[ must_use ]
  pub fn count_texts( &self, texts : &[&str ] ) -> TokenCount 
  {
  let total_text = texts.join( "" );
  self.count_tokens( &total_text )
  }

  /// Count tokens in chat messages
  #[ inline ]
  #[ must_use ]
  pub fn count_messages( &self, messages : &[ChatMessage ] ) -> TokenCount 
  {
  let mut total_chars = 0;
  let mut total_tokens = 0;

  for message in messages
  {
      // Count role ( adds overhead )
      let role_count = self.count_tokens( &message.role );
      total_chars += role_count.characters;
      total_tokens += role_count.total;

      // Count content
      let content_count = self.count_tokens( &message.content );
      total_chars += content_count.characters;
      total_tokens += content_count.total;

      // Add message overhead ( formatting tokens )
      total_tokens += 4; // Typical overhead per message
  }

  TokenCount::new( total_tokens, total_chars, self.strategy )
  }

  /// Estimate tokens ( fast, rough )
  #[ inline ]
  #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
  fn estimate_tokens( text : &str ) -> usize 
  {
  let chars = text.chars( ).count( );
  ( chars as f64 / 4.0 ).ceil( ) as usize
  }

  /// Word-based count
  #[ inline ]
  #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
  fn word_based_count( text : &str ) -> usize 
  {
  let words = text.split_whitespace( ).count( );
  ( words as f64 * 1.3 ).ceil( ) as usize
  }

  /// Character-based count ( more accurate )
  #[ inline ]
  #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
  fn character_based_count( text : &str ) -> usize 
  {
  let chars = text.chars( ).count( );
  ( chars as f64 / 3.5 ).ceil( ) as usize
  }

  /// Get current strategy
  #[ inline ]
  #[ must_use ]
  pub fn strategy( &self ) -> CountingStrategy 
  {
  self.strategy
  }

  /// Change counting strategy
  #[ inline ]
  pub fn set_strategy( &mut self, strategy : CountingStrategy ) 
  {
  self.strategy = strategy;
  }
}

impl Default for TokenCounter 
{
  #[ inline ]
  fn default() -> Self 
  {
  Self::new( CountingStrategy::default( ))
  }
}

/// Token counting errors
#[ derive( Debug ) ]
pub enum TokenCountError 
{
  /// Text is too large to count
  TextTooLarge {
  /// Size of the text
  size : usize,
  /// Maximum allowed size
  max_size : usize,
  },
}

impl core::fmt::Display for TokenCountError 
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result 
  {
  match self
  {
      Self::TextTooLarge { size, max_size } => {
  write!( f, "Text too large : {size} characters ( max : {max_size} )" )
      }
  }
  }
}

impl std::error::Error for TokenCountError {}

#[ cfg( test ) ]
mod tests {
  use super::*;

  #[ test ]
  fn test_estimation_strategy() 
  {
  let counter = TokenCounter::new( CountingStrategy::Estimation );
  let text = "Hello, world!"; // 13 characters
  let count = counter.count_tokens( text );

  assert_eq!( count.characters, 13 );
  assert_eq!( count.total, 4 ); // ceil( 13 / 4 ) = 4
  assert_eq!( count.strategy, CountingStrategy::Estimation );
  }

  #[ test ]
  fn test_word_based_strategy() 
  {
  let counter = TokenCounter::new( CountingStrategy::WordBased );
  let text = "Hello world from Rust"; // 4 words
  let count = counter.count_tokens( text );

  assert_eq!( count.total, 6 ); // ceil( 4 * 1.3 ) = 6
  }

  #[ test ]
  fn test_character_based_strategy() 
  {
  let counter = TokenCounter::new( CountingStrategy::CharacterBased );
  let text = "Hello!"; // 6 characters
  let count = counter.count_tokens( text );

  assert_eq!( count.characters, 6 );
  assert_eq!( count.total, 2 ); // ceil( 6 / 3.5 ) = 2
  }

  #[ test ]
  fn test_empty_text() 
  {
  let counter = TokenCounter::new( CountingStrategy::Estimation );
  let count = counter.count_tokens( "" );

  assert_eq!( count.total, 0 );
  assert_eq!( count.characters, 0 );
  }

  #[ test ]
  fn test_count_multiple_texts() 
  {
  let counter = TokenCounter::new( CountingStrategy::Estimation );
  let texts = vec!["Hello", "world" ];
  let count = counter.count_texts( &texts );

  // Combined : "Helloworld" = 10 characters
  assert_eq!( count.characters, 10 );
  assert_eq!( count.total, 3 ); // ceil( 10 / 4 ) = 3
  }

  #[ test ]
  fn test_count_messages() 
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
  ];

  let count = counter.count_messages( &messages );

  // Should include role + content + overhead for each message
  assert!( count.total > 0 );
  assert!( count.characters > 0 );
  }

  #[ test ]
  #[ allow( clippy::float_cmp ) ]
  fn test_cost_units() 
  {
  let count = TokenCount::new( 1500, 5000, CountingStrategy::Estimation );
  let cost = count.cost_units( );

  assert_eq!( cost, 1.5 ); // 1500 / 1000 = 1.5
  }

  #[ test ]
  fn test_strategy_change() 
  {
  let mut counter = TokenCounter::new( CountingStrategy::Estimation );
  assert_eq!( counter.strategy( ), CountingStrategy::Estimation );

  counter.set_strategy( CountingStrategy::WordBased );
  assert_eq!( counter.strategy( ), CountingStrategy::WordBased );
  }

  #[ test ]
  fn test_default_strategy() 
  {
  let counter = TokenCounter::default( );
  assert_eq!( counter.strategy( ), CountingStrategy::CharacterBased );
  }

  #[ test ]
  fn test_large_text() 
  {
  let counter = TokenCounter::new( CountingStrategy::Estimation );
  let text = "a".repeat( 10000 );
  let count = counter.count_tokens( &text );

  assert_eq!( count.characters, 10000 );
  assert_eq!( count.total, 2500 ); // ceil( 10000 / 4 ) = 2500
  }

  #[ test ]
  fn test_unicode_text() 
  {
  let counter = TokenCounter::new( CountingStrategy::CharacterBased );
  let text = "Hello 世界 🌍"; // Mixed ASCII, Chinese, emoji
  let count = counter.count_tokens( text );

  assert!( count.total > 0 );
  assert!( count.characters > 0 );
  }

  #[ test ]
  fn test_token_count_equality() 
  {
  let count1 = TokenCount::new( 100, 350, CountingStrategy::CharacterBased );
  let count2 = TokenCount::new( 100, 350, CountingStrategy::CharacterBased );
  let count3 = TokenCount::new( 101, 350, CountingStrategy::CharacterBased );

  assert_eq!( count1, count2 );
  assert_ne!( count1, count3 );
  }

  #[ test ]
  fn test_strategy_enum_equality() 
  {
  assert_eq!( CountingStrategy::Estimation, CountingStrategy::Estimation );
  assert_ne!( CountingStrategy::Estimation, CountingStrategy::WordBased );
  }

  #[ test ]
  fn test_whitespace_handling() 
  {
  let counter = TokenCounter::new( CountingStrategy::WordBased );
  let text1 = "Hello   world"; // Multiple spaces
  let text2 = "Hello world";

  let count1 = counter.count_tokens( text1 );
  let count2 = counter.count_tokens( text2 );

  // Both should have same word count
  assert_eq!( count1.total, count2.total );
  }
}

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
  fn estimate_tokens( text : &str ) -> usize
  {
  let chars = text.chars( ).count( );
  ( chars + 3 ) / 4 // ceiling of chars/4 in integer arithmetic
  }

  /// Word-based count
  #[ inline ]
  fn word_based_count( text : &str ) -> usize
  {
  let words = text.split_whitespace( ).count( );
  ( words * 13 + 9 ) / 10 // ceiling of words*1.3 in integer arithmetic
  }

  /// Character-based count ( more accurate )
  #[ inline ]
  fn character_based_count( text : &str ) -> usize
  {
  let chars = text.chars( ).count( );
  ( chars * 2 + 6 ) / 7 // ceiling of chars/3.5 = ceiling of 2*chars/7 in integer arithmetic
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

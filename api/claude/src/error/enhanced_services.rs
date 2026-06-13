//! Enhanced error service types and implementations
//!
//! Backoff calculation for rate limit errors.

#[ allow( clippy::missing_inline_in_public_items ) ]
mod private
{
  use super::super::core::orphan::*;
  use std::time::Duration;

/// Backoff calculator for rate limiting
#[ derive( Debug ) ]
pub struct BackoffCalculator;

/// Backoff strategy details
#[ derive( Debug, Clone ) ]
pub struct BackoffStrategyDetails
{
  /// Initial delay
  initial_delay : Duration,
  /// Backoff type
  backoff_type : BackoffType,
  /// Maximum retries
  max_retries : u32,
  /// Whether jitter is enabled
  jitter_enabled : bool,
  /// Suggested batch size reduction
  suggested_batch_size_reduction : Option< f32 >,
}

impl BackoffCalculator
{
  /// Calculate backoff strategy for a rate limit error.
  ///
  /// # Errors
  ///
  /// Returns `NotImplemented` when the `retry-logic` feature is disabled.
  #[ cfg( feature = "retry-logic" ) ]
  pub fn calculate_backoff( error : &RateLimitError ) -> AnthropicResult< BackoffStrategyDetails >
  {
    let base_delay = if let Some( retry_after ) = error.retry_after()
    {
      ( *retry_after * 1000 ).max( 1000 )
    }
    else
    {
      match error.limit_type()
      {
        "authentication" => 5000,
        "tokens" => 2000,
        _ => 1000,
      }
    };

    Ok( BackoffStrategyDetails
    {
      initial_delay : Duration::from_millis( base_delay ),
      backoff_type : BackoffType::Linear,
      max_retries : 5,
      jitter_enabled : true,
      suggested_batch_size_reduction : Some( 0.5 ),
    } )
  }

  /// # Errors
  ///
  /// Always returns `NotImplemented` — enable the `retry-logic` feature for a real implementation.
  #[ cfg( not( feature = "retry-logic" ) ) ]
  pub fn calculate_backoff( _error : &RateLimitError ) -> AnthropicResult< BackoffStrategyDetails >
  {
    Err( AnthropicError::NotImplemented( "BackoffCalculator requires retry-logic feature".to_string() ) )
  }
}

impl BackoffStrategyDetails
{
  /// Get initial delay
  #[ must_use ]
  pub fn initial_delay( &self ) -> Duration
  {
    self.initial_delay
  }

  /// Get backoff type
  #[ must_use ]
  pub fn backoff_type( &self ) -> BackoffType
  {
    self.backoff_type.clone()
  }

  /// Get max retries
  #[ must_use ]
  pub fn max_retries( &self ) -> u32
  {
    self.max_retries
  }

  /// Check if jitter enabled
  #[ must_use ]
  pub fn jitter_enabled( &self ) -> bool
  {
    self.jitter_enabled
  }

  /// Get suggested batch size reduction
  #[ must_use ]
  pub fn suggested_batch_size_reduction( &self ) -> &Option< f32 >
  {
    &self.suggested_batch_size_reduction
  }
}

}

crate::mod_interface!
{
  #[ cfg( feature = "error-handling" ) ]
  exposed use
  {
    BackoffCalculator,
    BackoffStrategyDetails,
  };
}

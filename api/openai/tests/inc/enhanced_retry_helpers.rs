//! Enhanced Retry Test Helpers
//!
//! Shared test infrastructure for retry logic tests including:
//! - `EnhancedRetryConfig` struct and builder
//! - `RetryState` for state management
//! - `MockHttpClient` for controlled failure scenarios
//! - `EnhancedRetryExecutor` for retry coordination
//! - Helper functions for retry testing

#![allow(clippy::missing_inline_in_public_items)]

#[ cfg( feature = "retry" ) ]
#[ allow( missing_docs ) ]
pub mod helpers
{
  use api_openai::
  {
    error ::{ OpenAIError, Result },
  };

  use std::
  {
    sync ::{ Arc, Mutex },
  };
  use core::time::Duration;
  use std::time::Instant;

  use serde::{ Serialize, Deserialize };
  use tokio::time::sleep;
  use rand::{ rng, RngExt };

  /// Enhanced retry configuration for HTTP requests
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct EnhancedRetryConfig
  {
    /// Maximum number of retry attempts
    pub max_attempts : u32,
    /// Base delay in milliseconds between retries
    pub base_delay_ms : u64,
    /// Maximum delay in milliseconds between retries
    pub max_delay_ms : u64,
    /// Maximum elapsed time for all retry attempts in milliseconds
    pub max_elapsed_time_ms : u64,
    /// Jitter amount in milliseconds to add randomness
    pub jitter_ms : u64,
    /// Multiplier for exponential backoff (default : 2.0)
    pub backoff_multiplier : f64,
  }

  impl Default for EnhancedRetryConfig
  {
    fn default() -> Self
    {
      Self
      {
        max_attempts : 3,
        base_delay_ms : 1000,
        max_delay_ms : 30000,
        max_elapsed_time_ms : 120_000,
        jitter_ms : 100,
        backoff_multiplier : 2.0,
      }
    }
  }

  impl EnhancedRetryConfig
  {
    /// Create a new retry configuration
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Set maximum retry attempts
    #[ must_use ]
    pub fn with_max_attempts( mut self, max_attempts : u32 ) -> Self
    {
      self.max_attempts = max_attempts;
      self
    }

    /// Set base delay between retries
    #[ must_use ]
    pub fn with_base_delay( mut self, base_delay_ms : u64 ) -> Self
    {
      self.base_delay_ms = base_delay_ms;
      self
    }

    /// Set maximum delay between retries
    #[ must_use ]
    pub fn with_max_delay( mut self, max_delay_ms : u64 ) -> Self
    {
      self.max_delay_ms = max_delay_ms;
      self
    }

    /// Set maximum elapsed time for all attempts
    #[ must_use ]
    pub fn with_max_elapsed_time( mut self, max_elapsed_time_ms : u64 ) -> Self
    {
      self.max_elapsed_time_ms = max_elapsed_time_ms;
      self
    }

    /// Set jitter amount for randomization
    #[ must_use ]
    pub fn with_jitter( mut self, jitter_ms : u64 ) -> Self
    {
      self.jitter_ms = jitter_ms;
      self
    }

    /// Set backoff multiplier for exponential backoff
    #[ must_use ]
    pub fn with_backoff_multiplier( mut self, multiplier : f64 ) -> Self
    {
      self.backoff_multiplier = multiplier;
      self
    }

    /// Calculate retry delay with exponential backoff and jitter
    /// Formula : `base_delay` * `backoff_multiplier^attempt` + random(0, `jitter_ms`)
    #[ must_use ]
    pub fn calculate_delay( &self, attempt : u32 ) -> Duration
    {
      let max_delay = Duration::from_millis( self.max_delay_ms );

      // Calculate exponential backoff : base_delay * multiplier^attempt
      #[ allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_possible_wrap) ]
      let exponential_delay = ( self.base_delay_ms as f64 * self.backoff_multiplier.powi( attempt as i32 ) ) as u64;

      // Add jitter : random value between 0 and jitter_ms
      let mut rng = rng();
      let jitter = rng.random_range( 0..=self.jitter_ms );

      let total_delay_ms = exponential_delay + jitter;
      let total_delay = Duration::from_millis( total_delay_ms );

      // Ensure delay doesn't exceed maximum
      core ::cmp::min( total_delay, max_delay )
    }

    /// Check if an error is retryable
    #[ must_use ]
    pub fn is_retryable_error( &self, error : &OpenAIError ) -> bool
    {
      match error
      {
        // Retryable errors : network, timeout, rate limiting, stream, and websocket errors
        OpenAIError::Network( _ ) | OpenAIError::Timeout( _ ) | OpenAIError::RateLimit( _ ) | OpenAIError::Stream( _ ) | OpenAIError::Ws( _ ) => true,
        // HTTP errors : check if message contains server error or rate limiting
        OpenAIError::Http( message ) =>
        {
          message.contains( '5' ) || message.contains( "429" ) || message.contains( "500" ) || message.contains( "502" ) || message.contains( "503" ) || message.contains( "504" )
        },
        // Non-retryable errors : all client-side errors, invalid messages, and unknown errors
        _ => false,
      }
    }

    /// Validate configuration parameters
    ///
    /// # Errors
    /// Returns an error if any configuration values are invalid.
    pub fn validate( &self ) -> core::result::Result< (), String >
    {
      if self.max_attempts == 0
      {
        return Err( "max_attempts must be greater than 0".to_string() );
      }

      if self.base_delay_ms == 0
      {
        return Err( "base_delay_ms must be greater than 0".to_string() );
      }

      if self.max_delay_ms < self.base_delay_ms
      {
        return Err( "max_delay_ms must be greater than or equal to base_delay_ms".to_string() );
      }

      if self.max_elapsed_time_ms == 0
      {
        return Err( "max_elapsed_time_ms must be greater than 0".to_string() );
      }

      if self.backoff_multiplier <= 0.0
      {
        return Err( "backoff_multiplier must be greater than 0".to_string() );
      }

      Ok( () )
    }
  }

  /// Thread-safe retry state management
  #[ derive( Debug ) ]
  pub struct RetryState
  {
    /// Current attempt number (0-indexed)
    pub attempt : u32,
    /// Total attempts made
    pub total_attempts : u32,
    /// Start time of first attempt
    pub start_time : Instant,
    /// Last error encountered
    pub last_error : Option< String >,
    /// Total elapsed time
    pub elapsed_time : Duration,
  }

  impl Default for RetryState
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl RetryState
  {
    /// Create new retry state
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        attempt : 0,
        total_attempts : 0,
        start_time : Instant::now(),
        last_error : None,
        elapsed_time : Duration::ZERO,
      }
    }

    /// Increment attempt counter
    pub fn next_attempt( &mut self )
    {
      self.attempt += 1;
      self.total_attempts += 1;
      self.elapsed_time = self.start_time.elapsed();
    }

    /// Set last error
    pub fn set_error( &mut self, error : String )
    {
      self.last_error = Some( error );
    }

    /// Reset state for new request
    pub fn reset( &mut self )
    {
      self.attempt = 0;
      self.total_attempts = 0;
      self.start_time = Instant::now();
      self.last_error = None;
      self.elapsed_time = Duration::ZERO;
    }

    /// Check if max elapsed time exceeded
    #[ must_use ]
    pub fn is_elapsed_time_exceeded( &self, max_elapsed_time : Duration ) -> bool
    {
      self.elapsed_time >= max_elapsed_time
    }
  }

  /// Test harness for controlled retry behavior validation
  ///
  /// This is NOT a mock of the `OpenAI` API. It's a controlled test harness that simulates
  /// specific failure sequences to validate retry logic in isolation.
  ///
  /// # Purpose
  ///
  /// Allows testing retry coordinator behavior with predetermined failure patterns:
  /// - Transient network failures (connection errors)
  /// - Server errors (5xx responses)
  /// - Mixed failure types
  /// - Success after N failures
  ///
  /// # Usage in Tests
  ///
  /// Used exclusively for unit testing retry mechanism components:
  /// - Exponential backoff calculation
  /// - Error classification (retryable vs non-retryable)
  /// - State management (attempt counters, elapsed time)
  /// - Max attempts enforcement
  /// - Max elapsed time enforcement
  ///
  /// ⚠️ CODEBASE HYGIENE VIOLATION: This mock violates no-mocking rule
  /// Justification : Testing retry logic requires controlled failure sequences
  /// that cannot be reliably reproduced with real API calls
  ///
  /// Mitigation : Corresponding integration tests must verify retry behavior
  /// with real `OpenAI` API calls under actual failure conditions
  ///
  /// `TODO(hygiene-006)`: Create integration tests for:
  /// - Real network failures with retry recovery
  /// - Real 500/503 errors with exponential backoff
  /// - Real timeout scenarios with retry limits
  /// - Real rate limit (429) handling with retry
  ///
  /// Integration test file : `tests/retry_integration_tests.rs` (to be created)
  #[ derive( Debug ) ]
  pub struct MockHttpClient
  {
    response_sequence : Arc< Mutex< Vec< Result< String > > > >,
    call_count : Arc< Mutex< u32 > >,
  }

  impl MockHttpClient
  {
    /// Create mock client with predetermined response sequence
    #[ must_use ]
    pub fn new( responses : Vec< Result< String > > ) -> Self
    {
      Self
      {
        response_sequence : Arc::new( Mutex::new( responses ) ),
        call_count : Arc::new( Mutex::new( 0 ) ),
      }
    }

    /// Simulate HTTP request that may fail
    ///
    /// # Errors
    ///
    /// Returns error if the predefined response sequence contains an error for this call.
    ///
    /// # Panics
    ///
    /// Panics if mutex is poisoned.
    #[ allow(clippy::unused_async) ]
    pub async fn make_request( &self ) -> Result< String >
    {
      let mut count = self.call_count.lock().unwrap();
      *count += 1;
      let call_index = *count - 1;
      drop( count );

      let mut responses = self.response_sequence.lock().unwrap();
      if call_index < u32::try_from(responses.len()).unwrap_or(0)
      {
        responses.remove( 0 )
      }
      else
      {
        // If we've exhausted predefined responses, return success
        Ok( "success".to_string() )
      }
    }

    /// Get total number of calls made
    ///
    /// # Panics
    ///
    /// Panics if mutex is poisoned.
    #[ must_use ]
    pub fn get_call_count( &self ) -> u32
    {
      *self.call_count.lock().unwrap()
    }
  }

  /// Enhanced retry executor with comprehensive retry logic
  #[ derive( Debug ) ]
  pub struct EnhancedRetryExecutor
  {
    config : EnhancedRetryConfig,
    state : Arc< Mutex< RetryState > >,
  }

  impl EnhancedRetryExecutor
  {
    /// Create new retry executor with configuration
    ///
    /// # Errors
    /// Returns an error if the provided configuration is invalid.
    pub fn new( config : EnhancedRetryConfig ) -> core::result::Result< Self, String >
    {
      config.validate()?;

      Ok( Self
      {
        config,
        state : Arc::new( Mutex::new( RetryState::new() ) ),
      } )
    }

    /// Execute operation with retry logic
    ///
    /// # Errors
    /// Returns an error if all retry attempts fail or if the operation fails with a non-retryable error.
    ///
    /// # Panics
    /// Panics if the retry state mutex is poisoned.
    pub async fn execute< F, Fut, T >( &self, operation : F ) -> Result< T >
    where
      F : Fn() -> Fut,
      Fut : core::future::Future< Output = Result< T > >,
    {
      // Reset state for new execution
      {
        let mut state = self.state.lock().unwrap();
        state.reset();
      }

      let max_elapsed_time = Duration::from_millis( self.config.max_elapsed_time_ms );
      #[ allow(unused_assignments) ]
      let mut last_error : Option< error_tools::untyped::Error > = None;

      loop
      {
        // Check if max elapsed time exceeded
        {
          let state = self.state.lock().unwrap();
          if state.is_elapsed_time_exceeded( max_elapsed_time )
          {
            return Err( error_tools::untyped::Error::msg( format!( "Max elapsed time exceeded : {max_elapsed_time:?}" ) ) );
          }
        }

        // Increment attempt counter
        {
          let mut state = self.state.lock().unwrap();
          state.next_attempt();
        }

        // Get current attempt number
        let current_attempt = {
          let state = self.state.lock().unwrap();
          state.attempt
        };

        // Execute operation
        match operation().await
        {
          Ok( result ) => return Ok( result ),
          Err( error ) =>
          {
            // Store error in state
            {
              let mut state = self.state.lock().unwrap();
              state.set_error( error.to_string() );
            }

            // Try to downcast to OpenAIError for retry checking
            let is_retryable = if let Some( openai_error ) = error.downcast_ref::< OpenAIError >()
            {
              self.config.is_retryable_error( openai_error )
            }
            else
            {
              // If not OpenAIError, assume retryable for network/timeout-like errors
              let error_msg = error.to_string().to_lowercase();
              error_msg.contains( "network" ) || error_msg.contains( "timeout" ) || error_msg.contains( "connection" )
            };

            last_error = Some( error );

            // Check if error is retryable
            if !is_retryable
            {
              return Err( last_error.unwrap() );
            }

            // Check if max attempts reached
            if current_attempt >= self.config.max_attempts
            {
              return Err( last_error.unwrap() );
            }

            // Calculate delay for next attempt (0-indexed for calculation)
            let delay = self.config.calculate_delay( current_attempt - 1 );

            // Wait before next attempt
            sleep( delay ).await;
          }
        }
      }
    }

    /// Get current retry state (for testing)
    ///
    /// # Panics
    /// Panics if the retry state mutex is poisoned.
    #[ must_use ]
    pub fn get_state( &self ) -> RetryState
    {
      let state = self.state.lock().unwrap();
      RetryState
      {
        attempt : state.attempt,
        total_attempts : state.total_attempts,
        start_time : state.start_time,
        last_error : state.last_error.clone(),
        elapsed_time : state.elapsed_time,
      }
    }
  }
}

// Re-export for convenience
#[ cfg( feature = "retry" ) ]
pub use helpers::*;

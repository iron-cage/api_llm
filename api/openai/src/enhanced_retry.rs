//! Enhanced Retry Logic Module
//!
//! This module provides enhanced retry functionality for HTTP requests with
//! exponential backoff, jitter, and configurable retry policies. All functionality
//! is feature-gated to ensure zero overhead when disabled.

#![ allow( clippy::missing_inline_in_public_items ) ]

#[ cfg( feature = "retry" ) ]
mod private
{
  use crate::
  {
    error ::{ OpenAIError, Result },
  };

  use core::time::Duration;
  use std::
  {
    sync ::{ Arc, Mutex },
    time ::Instant,
  };

  use serde::{ Serialize, Deserialize };
  use tokio::time::sleep;
  use rand::RngExt;

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
    /// Formula : `base_delay` * `backoff_multiplier`^attempt + random(0, `jitter_ms`)
    #[ must_use ]
    pub fn calculate_delay( &self, attempt : u32 ) -> Duration
    {
      let max_delay = Duration::from_millis( self.max_delay_ms );

      // Calculate exponential backoff : base_delay * multiplier^attempt
      let base_delay_f64 = self.base_delay_ms as f64;
      let attempt_i32 = i32::try_from( attempt ).unwrap_or( i32::MAX );
      let exponential_f64 = base_delay_f64 * self.backoff_multiplier.powi( attempt_i32 );
      #[ allow(clippy::cast_possible_truncation, clippy::cast_sign_loss) ]
      let exponential_delay = exponential_f64.min( u64::MAX as f64 ).max( 0.0 ) as u64;

      // Add jitter : random value between 0 and jitter_ms
      let mut rng = rand::rng();
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
        // Network, timeout, rate limiting, stream, and WebSocket errors are retryable
        OpenAIError::Network( _ ) | OpenAIError::Timeout( _ ) | OpenAIError::RateLimit( _ ) | OpenAIError::Stream( _ ) | OpenAIError::Ws( _ ) => true,
        // HTTP errors : check if message contains server error or rate limiting
        OpenAIError::Http( message ) =>
        {
          message.contains( '5' ) || message.contains( "429" ) || message.contains( "500" ) || message.contains( "502" ) || message.contains( "503" ) || message.contains( "504" )
        },
        // All other errors are not retryable
        OpenAIError::Api( _ ) | OpenAIError::WsInvalidMessage( _ ) | OpenAIError::Internal( _ ) |
        OpenAIError::InvalidArgument( _ ) | OpenAIError::MissingArgument( _ ) | OpenAIError::MissingEnvironment( _ ) |
        OpenAIError::MissingHeader( _ ) | OpenAIError::MissingFile( _ ) | OpenAIError::File( _ ) | OpenAIError::Unknown( _ ) => false,
      }
    }

    /// Validate configuration parameters
    ///
    /// # Errors
    ///
    /// Returns an error if any configuration parameter is invalid (e.g., zero values where positive values are required).
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
    ///
    /// Returns an error if the configuration validation fails.
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
    ///
    /// Returns an error if the operation fails after all retry attempts or if time limits are exceeded.
    ///
    /// # Panics
    ///
    /// Panics if the internal state mutex is poisoned.
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

            // Check if error is retryable
            if !is_retryable
            {
              return Err( error );
            }

            // Check if max attempts reached
            if current_attempt >= self.config.max_attempts
            {
              return Err( error );
            }

            // Calculate delay for next attempt (0-indexed for calculation)
            let delay = self.config.calculate_delay( current_attempt - 1 );

            // Log retry attempt (only when retry feature is enabled)
            #[ cfg( feature = "retry" ) ]
            {
              tracing ::debug!( "Retrying request attempt {} after {:?} delay", current_attempt, delay );
            }

            // Wait before next attempt
            sleep( delay ).await;
          }
        }
      }
    }

    /// Get current retry state (for testing and metrics)
    ///
    /// # Panics
    ///
    /// Panics if the internal state mutex is poisoned.
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

    /// Get retry configuration
    #[ must_use ]
    pub fn config( &self ) -> &EnhancedRetryConfig
    {
      &self.config
    }
  }
}

// Re-export retry functionality only when feature is enabled
#[ cfg( feature = "retry" ) ]
pub use private::
{
  EnhancedRetryConfig,
  RetryState,
  EnhancedRetryExecutor,
};

// Provide no-op implementations when retry feature is disabled
#[ cfg( not( feature = "retry" ) ) ]
/// No-op retry configuration module when retry feature is disabled
pub mod private
{
  /// No-op retry configuration when feature is disabled
  #[ derive( Debug, Clone ) ]
  pub struct EnhancedRetryConfig;

  impl EnhancedRetryConfig
  {
    /// Create a new no-op configuration
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
    }
  }

  impl Default for EnhancedRetryConfig
  {
    fn default() -> Self
    {
      Self
    }
  }
}

#[ cfg( not( feature = "retry" ) ) ]
pub use private::EnhancedRetryConfig;

// Export for mod_interface
crate ::mod_interface!
{
  #[ cfg( feature = "retry" ) ]
  exposed use
  {
    EnhancedRetryConfig,
    RetryState,
    EnhancedRetryExecutor,
  };

  #[ cfg( not( feature = "retry" ) ) ]
  exposed use
  {
    EnhancedRetryConfig,
  };
}
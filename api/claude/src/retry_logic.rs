//!
//! Retry logic implementation for the Anthropic API client
//!
//! This module provides comprehensive retry functionality with exponential backoff,
//! jitter, and error-specific retry policies for robust API communication.
//!

#[ cfg( feature = "retry-logic" ) ]
// Allow missing inline attributes for retry logic module
#[ allow( clippy::missing_inline_in_public_items ) ]
mod private
{
  #[ cfg( feature = "error-handling" ) ]
  use crate::error::{AnthropicError, AnthropicResult, RateLimitError};

  #[ cfg( not( feature = "error-handling" ) ) ]
  use error_tools::{err, Result as AnthropicResult};

  #[ cfg( not( feature = "error-handling" ) ) ]
  type AnthropicError = error_tools::Error;

  // Provide minimal RateLimitError type when error-handling feature is disabled
  #[ cfg( not( feature = "error-handling" ) ) ]
  #[ derive( Debug, Clone ) ]
  pub struct RateLimitError
  {
    message : String,
    retry_after : Option< u64 >,
    limit_type : String,
  }

  #[ cfg( not( feature = "error-handling" ) ) ]
  impl RateLimitError
  {
    pub fn new(limit_type : String, retry_after : Option< u64 >, message : String) -> Self
    {
      Self { message, retry_after, limit_type }
    }

    pub fn retry_after(&self) -> Option< u64 >
    {
      self.retry_after
    }

    pub fn limit_type(&self) -> &str
    {
      &self.limit_type
    }
  }

  // Provide minimal BackoffCalculator when error-handling feature is disabled
  #[ cfg( not( feature = "error-handling" ) ) ]
  pub struct BackoffCalculator;

  #[ cfg( not( feature = "error-handling" ) ) ]
  impl BackoffCalculator
  {
    pub fn calculate_backoff(_error : &RateLimitError) -> AnthropicResult< BackoffStrategyDetails >
    {
      Err(err!("BackoffCalculator not available without error-handling feature"))
    }
  }

  use core::time::Duration;
  use std::collections::HashMap;

  /// Configuration for retry behavior
  #[ derive( Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize ) ]
  pub struct RetryConfig
  {
    max_attempts : u32,
    base_delay_ms : u64,
    max_delay_ms : u64,
    backoff_multiplier : f64,
    jitter_enabled : bool,
  }

  impl Default for RetryConfig
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl RetryConfig
  {
    /// Create retry configuration with explicit parameters (no defaults)
    ///
    /// # Arguments
    ///
    /// * `max_attempts` - Maximum number of retry attempts (must be >= 1)
    /// * `base_delay_ms` - Base delay in milliseconds (must be > 0)
    /// * `max_delay_ms` - Maximum delay in milliseconds (must be >= `base_delay_ms`)
    /// * `backoff_multiplier` - Multiplier for exponential backoff (must be >= 1.0)
    /// * `jitter_enabled` - Whether to apply jitter to delays
    pub fn with_explicit_config( max_attempts : u32, base_delay_ms : u64, max_delay_ms : u64, backoff_multiplier : f64, jitter_enabled : bool ) -> Self
    {
      Self {
        max_attempts,
        base_delay_ms,
        max_delay_ms,
        backoff_multiplier,
        jitter_enabled,
      }
    }
  }

  impl RetryConfig
  {
    /// Create new retry configuration (compatibility wrapper)
    ///
    /// NOTE: This is a compatibility wrapper with sensible defaults. For explicit control, use `with_explicit_config()`
    pub fn new() -> Self
    {
      // Compatibility wrapper with sensible defaults for retry logic
      Self::with_explicit_config(
        3,      // max_attempts : 3 attempts total
        1000,   // base_delay_ms : 1 second base delay
        60000,  // max_delay_ms : 60 seconds max delay
        2.0,    // backoff_multiplier : exponential backoff
        true,   // jitter_enabled : add jitter to delays
      )
    }

    /// Set maximum number of retry attempts
    #[ must_use ]
    pub fn with_max_attempts( mut self, max_attempts : u32 ) -> Self
    {
      self.max_attempts = max_attempts;
      self
    }

    /// Set base delay in milliseconds
    #[ must_use ]
    pub fn with_base_delay_ms( mut self, base_delay_ms : u64 ) -> Self
    {
      self.base_delay_ms = base_delay_ms;
      self
    }

    /// Set maximum delay in milliseconds
    #[ must_use ]
    pub fn with_max_delay_ms( mut self, max_delay_ms : u64 ) -> Self
    {
      self.max_delay_ms = max_delay_ms;
      self
    }

    /// Set initial delay as Duration (convenience method)
    #[ must_use ]
    #[ allow( clippy::cast_possible_truncation ) ]
    pub fn with_initial_delay( self, delay : Duration ) -> Self
    {
      self.with_base_delay_ms( delay.as_millis() as u64 )
    }

    /// Set maximum delay as Duration (convenience method)
    #[ must_use ]
    #[ allow( clippy::cast_possible_truncation ) ]
    pub fn with_max_delay( self, delay : Duration ) -> Self
    {
      self.with_max_delay_ms( delay.as_millis() as u64 )
    }

    /// Set backoff multiplier for exponential backoff
    #[ must_use ]
    pub fn with_backoff_multiplier( mut self, backoff_multiplier : f64 ) -> Self
    {
      self.backoff_multiplier = backoff_multiplier;
      self
    }

    /// Enable or disable jitter
    #[ must_use ]
    pub fn with_jitter( mut self, jitter_enabled : bool ) -> Self
    {
      self.jitter_enabled = jitter_enabled;
      self
    }

    /// Get maximum attempts
    pub fn max_attempts( &self ) -> u32
    {
      self.max_attempts
    }

    /// Get base delay in milliseconds
    pub fn base_delay_ms( &self ) -> u64
    {
      self.base_delay_ms
    }

    /// Get initial delay as Duration (same as base delay)
    #[ must_use ]
    pub fn initial_delay( &self ) -> Duration
    {
      Duration::from_millis( self.base_delay_ms )
    }

    /// Get maximum delay in milliseconds
    pub fn max_delay_ms( &self ) -> u64
    {
      self.max_delay_ms
    }

    /// Get backoff multiplier
    pub fn backoff_multiplier( &self ) -> f64
    {
      self.backoff_multiplier
    }

    /// Check if jitter is enabled
    pub fn jitter_enabled( &self ) -> bool
    {
      self.jitter_enabled
    }

    /// Validate configuration parameters
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid
    pub fn validate( &self ) -> AnthropicResult< () >
    {
      if self.max_attempts == 0
      {
        return Err( AnthropicError::InvalidArgument( "max_attempts must be >= 1".to_string() ) );
      }

      if self.base_delay_ms == 0
      {
        return Err( AnthropicError::InvalidArgument( "base_delay_ms must be > 0".to_string() ) );
      }

      if self.max_delay_ms < self.base_delay_ms
      {
        return Err( AnthropicError::InvalidArgument( "max_delay_ms must be >= base_delay_ms".to_string() ) );
      }

      if self.backoff_multiplier < 1.0
      {
        return Err( AnthropicError::InvalidArgument( "backoff_multiplier must be >= 1.0".to_string() ) );
      }

      Ok(())
    }

    /// Check if configuration is valid
    #[ must_use ]
    pub fn is_valid( &self ) -> bool
    {
      self.validate().is_ok()
    }
  }

  /// Types of retry strategies
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum RetryStrategyType
  {
    /// Exponential backoff strategy (delay multiplied by factor each attempt)
    ExponentialBackoff,
    /// Linear backoff strategy (delay increased linearly each attempt)
    LinearBackoff,
    /// Fixed delay strategy (same delay for all attempts)
    FixedDelay,
  }

  /// Retry strategy implementation
  #[ derive( Debug, Clone ) ]
  pub struct RetryStrategy
  {
    strategy_type : RetryStrategyType,
    config : RetryConfig,
  }

  impl RetryStrategy
  {
    /// Create exponential backoff strategy with explicit configuration
    pub fn exponential_backoff_with_config( config : RetryConfig ) -> Self
    {
      Self {
        strategy_type : RetryStrategyType::ExponentialBackoff,
        config,
      }
    }

    /// Create linear backoff strategy with explicit configuration
    pub fn linear_backoff_with_config( config : RetryConfig ) -> Self
    {
      Self {
        strategy_type : RetryStrategyType::LinearBackoff,
        config,
      }
    }

    /// Create fixed delay strategy with explicit configuration
    pub fn fixed_delay_with_config( config : RetryConfig ) -> Self
    {
      Self {
        strategy_type : RetryStrategyType::FixedDelay,
        config,
      }
    }

    /// Create exponential backoff strategy (compatibility wrapper)
    ///
    /// NOTE: This is a compatibility wrapper with default config. For explicit control, use `exponential_backoff_with_config()`
    pub fn exponential_backoff() -> Self
    {
      Self::exponential_backoff_with_config( RetryConfig::new() )
    }

    /// Create linear backoff strategy (compatibility wrapper)
    ///
    /// NOTE: This is a compatibility wrapper with default config. For explicit control, use `linear_backoff_with_config()`
    pub fn linear_backoff() -> Self
    {
      Self::linear_backoff_with_config( RetryConfig::new() )
    }

    /// Create fixed delay strategy (compatibility wrapper)
    ///
    /// NOTE: This is a compatibility wrapper with default config. For explicit control, use `fixed_delay_with_config()`
    pub fn fixed_delay() -> Self
    {
      Self::fixed_delay_with_config( RetryConfig::new() )
    }

    /// Set custom configuration
    #[ must_use ]
    pub fn with_config( mut self, config : RetryConfig ) -> Self
    {
      self.config = config;
      self
    }

    /// Get strategy type
    pub fn strategy_type( &self ) -> RetryStrategyType
    {
      self.strategy_type
    }

    /// Get configuration
    pub fn config( &self ) -> &RetryConfig
    {
      &self.config
    }

    /// Determine if an error should be retried
    pub fn should_retry( &self, error : &AnthropicError, attempt : u32 ) -> bool
    {
      // Check if max attempts reached
      if attempt >= self.config.max_attempts
      {
        return false;
      }

      // Check if error is retryable
      #[ cfg( feature = "error-handling" ) ]
      {
        match error
        {
          AnthropicError::Http( http_error ) =>
          {
            // Retry on 5xx errors but not 4xx, and on connection/timeout issues
            if let Some( status_code ) = http_error.status_code()
            {
              (500..600).contains(&status_code)
            }
            else
            {
              // Retry on connection/timeout issues (no status code)
              let message = http_error.message().to_lowercase();
              message.contains("timeout") || message.contains("connection") || message.contains("failed") || message.contains("temporary")
            }
          },
          // Retry on rate limits, stream errors, and internal errors (often transient)
          AnthropicError::RateLimit( _ ) | AnthropicError::Stream( _ ) | AnthropicError::Internal( _ ) => true,
          // Don't retry on authentication, validation, or argument errors
          _ => false,
        }
      }

      #[ cfg( not( feature = "error-handling" ) ) ]
      {
        // When error-handling feature is disabled, use basic string matching on error message
        let error_msg = error.to_string().to_lowercase();
        error_msg.contains("timeout") ||
        error_msg.contains("network") ||
        error_msg.contains("rate limit") ||
        error_msg.contains("5")  // Basic HTTP 5xx detection
      }
    }

    /// Calculate delay for a specific attempt with explicit jitter configuration
    ///
    /// # Panics
    ///
    /// This method will panic if jitter is enabled in config but jitter factors are not provided
    #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
    pub fn calculate_delay_with_jitter_config( &self, attempt : u32, jitter_min_factor : Option< f64 >, jitter_max_factor : Option< f64 > ) -> u64
    {
      let base_delay = self.config.base_delay_ms;

      let calculated_delay = match self.strategy_type
      {
        RetryStrategyType::ExponentialBackoff =>
        {
          let exponent = f64::from(attempt - 1);
          let delay = base_delay as f64 * self.config.backoff_multiplier.powf( exponent );
          delay as u64
        },
        RetryStrategyType::LinearBackoff =>
        {
          base_delay * u64::from(attempt)
        },
        RetryStrategyType::FixedDelay =>
        {
          base_delay
        },
      };

      // Apply max delay cap
      let capped_delay = calculated_delay.min( self.config.max_delay_ms );

      // Apply jitter if explicitly configured
      if self.config.jitter_enabled
      {
        if let ( Some( min_factor ), Some( max_factor ) ) = ( jitter_min_factor, jitter_max_factor )
        {
          Self::apply_jitter_with_config( capped_delay, min_factor, max_factor )
        }
        else
        {
          capped_delay
        }
      }
      else
      {
        capped_delay
      }
    }

    /// Calculate delay for a specific attempt (compatibility wrapper)
    ///
    /// NOTE: This is a compatibility wrapper with default jitter settings. For explicit control, use `calculate_delay_with_jitter_config()`
    #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
    pub fn calculate_delay( &self, attempt : u32 ) -> u64
    {
      // Compatibility wrapper : apply jitter if enabled in config, using ±10% variation (original behavior)
      if self.config.jitter_enabled
      {
        self.calculate_delay_with_jitter_config( attempt, Some( 0.9 ), Some( 1.1 ) )
      }
      else
      {
        self.calculate_delay_with_jitter_config( attempt, None, None )
      }
    }

    /// Calculate delay for a specific error with explicit configuration
    pub fn calculate_delay_for_error_with_config(
      &self,
      error : &RateLimitError,
      attempt : u32,
      jitter_min_factor : Option< f64 >,
      jitter_max_factor : Option< f64 >,
    ) -> u64
    {
      let base_delay = self.calculate_delay_with_jitter_config( attempt, jitter_min_factor, jitter_max_factor );

      // If error has retry-after header, use the longer of the two
      if let Some( retry_after ) = error.retry_after()
      {
        let retry_after_ms = *retry_after * 1000;
        base_delay.max( retry_after_ms )
      }
      else
      {
        base_delay
      }
    }

    /// Calculate delay for a specific error (compatibility wrapper)
    pub fn calculate_delay_for_error( &self, error : &RateLimitError, attempt : u32 ) -> u64
    {
      // Compatibility wrapper with no jitter (None values)
      self.calculate_delay_for_error_with_config( error, attempt, None, None )
    }

    /// Apply jitter to delay with explicit configuration
    #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
    fn apply_jitter_with_config( delay : u64, jitter_min_factor : f64, jitter_max_factor : f64 ) -> u64
    {
      use core::hash::{Hash, Hasher};
      use std::collections::hash_map::DefaultHasher;

      // Use deterministic pseudo-random based on current time for testing
      let mut hasher = DefaultHasher::new();
      std::time::SystemTime::now().duration_since( std::time::UNIX_EPOCH )
        .unwrap_or_default().as_nanos().hash( &mut hasher );

      let hash = hasher.finish();
      let factor_range = jitter_max_factor - jitter_min_factor;
      let jitter_factor = jitter_min_factor + ( hash % 1000 ) as f64 / 1000.0 * factor_range;

      ( delay as f64 * jitter_factor ) as u64
    }

  }

  /// Retry executor for running operations with retry logic
  #[ derive( Debug ) ]
  pub struct RetryExecutor
  {
    strategy : RetryStrategy,
    current_attempt : u32,
  }

  impl RetryExecutor
  {
    /// Create new retry executor
    pub fn new( strategy : RetryStrategy ) -> Self
    {
      Self {
        strategy,
        current_attempt : 0,
      }
    }

    /// Get retry strategy
    pub fn strategy( &self ) -> &RetryStrategy
    {
      &self.strategy
    }

    /// Get current attempt number
    pub fn current_attempt( &self ) -> u32
    {
      self.current_attempt
    }

    /// Check if max attempts exceeded
    pub fn has_exceeded_max_attempts( &self ) -> bool
    {
      self.current_attempt >= self.strategy.config.max_attempts
    }

    /// Execute operation with retry logic
    ///
    /// # Errors
    ///
    /// Returns an error if all retry attempts fail
    pub async fn execute< F, Fut, T >( &self, operation : F ) -> AnthropicResult< T >
    where
      F: Fn() -> Fut,
      Fut : core::future::Future< Output = AnthropicResult< T > >,
    {
      let mut attempt = 1;

      loop
      {
        match operation().await
        {
          Ok( result ) => return Ok( result ),
          Err( error ) =>
          {
            // Check if we should retry
            if !self.strategy.should_retry( &error, attempt )
            {
              return Err( error );
            }

            // Calculate and apply delay
            let delay_ms = {
              #[ cfg( feature = "error-handling" ) ]
              {
                match &error
                {
                  AnthropicError::RateLimit( rate_limit_error ) =>
                  {
                    self.strategy.calculate_delay_for_error( rate_limit_error, attempt )
                  },
                  _ => self.strategy.calculate_delay( attempt ),
                }
              }

              #[ cfg( not( feature = "error-handling" ) ) ]
              {
                self.strategy.calculate_delay( attempt )
              }
            };

            tokio::time::sleep( Duration::from_millis( delay_ms ) ).await;
            attempt += 1;
          }
        }
      }
    }
  }

  /// Backoff strategy types for error handling
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum BackoffStrategyType
  {
    /// Exponential backoff strategy for error handling
    ExponentialBackoff,
    /// Linear backoff strategy for error handling
    LinearBackoff,
    /// Fixed delay strategy for error handling
    FixedDelay,
  }

  /// Detailed backoff strategy information
  #[ derive( Debug, Clone ) ]
  pub struct BackoffStrategyDetails
  {
    strategy_type : BackoffStrategyType,
    suggested_delay_ms : u64,
    jitter_enabled : bool,
  }

  impl BackoffStrategyDetails
  {
    /// Create new backoff strategy details
    pub fn new( strategy_type : BackoffStrategyType, suggested_delay_ms : u64, jitter_enabled : bool ) -> Self
    {
      Self {
        strategy_type,
        suggested_delay_ms,
        jitter_enabled,
      }
    }

    /// Get strategy type
    pub fn strategy_type( &self ) -> BackoffStrategyType
    {
      self.strategy_type
    }

    /// Get suggested delay in milliseconds
    pub fn suggested_delay_ms( &self ) -> u64
    {
      self.suggested_delay_ms
    }

    /// Check if jitter is enabled
    pub fn jitter_enabled( &self ) -> bool
    {
      self.jitter_enabled
    }
  }

  // Enhanced BackoffCalculator implementation is provided in error.rs when error-handling feature is enabled

  /// Metrics tracking for retry attempts
  #[ derive( Debug, Clone ) ]
  pub struct RetryMetrics
  {
    total_attempts : u64,
    successful_retries : u64,
    failed_attempts : u64,
    total_delay_ms : u64,
    error_counts : HashMap< String, u64 >,
  }

  impl Default for RetryMetrics 
  {
    fn default() -> Self 
    {
      Self::new()
    }
  }

  impl RetryMetrics
  {
    /// Create new retry metrics tracker
    pub fn new() -> Self
    {
      Self {
        total_attempts : 0,
        successful_retries : 0,
        failed_attempts : 0,
        total_delay_ms : 0,
        error_counts : HashMap::new(),
      }
    }

    /// Record a retry attempt
    pub fn record_attempt( &mut self, attempt : u32, delay_ms : u64 )
    {
      self.total_attempts += 1;
      self.total_delay_ms += delay_ms;

      if attempt > 1
      {
        // Only count as retry if it's not the first attempt
        // This will be updated when we know the outcome
      }
    }

    /// Record successful retry
    pub fn record_success( &mut self, final_attempt : u32 )
    {
      if final_attempt > 1
      {
        self.successful_retries += 1;
      }
    }

    /// Record failed retry
    pub fn record_failure( &mut self, error : &AnthropicError )
    {
      self.failed_attempts += 1;

      let error_type = {
        #[ cfg( feature = "error-handling" ) ]
        {
          match error
          {
            AnthropicError::RateLimit( _ ) => "rate_limit",
            AnthropicError::Http( _ ) => "http",
            AnthropicError::Stream( _ ) => "stream",
            AnthropicError::Internal( _ ) => "internal",
            AnthropicError::Authentication( _ ) => "authentication",
            AnthropicError::InvalidArgument( _ ) => "invalid_argument",
            AnthropicError::InvalidRequest( _ ) => "invalid_request",
            AnthropicError::Api( _ ) => "api",
            AnthropicError::File( _ ) => "file",
            AnthropicError::Parsing( _ ) => "parsing",
            AnthropicError::MissingEnvironment( _ ) => "missing_environment",
            AnthropicError::NotImplemented( _ ) => "not_implemented",
            _ => "other",
          }
        }

        #[ cfg( not( feature = "error-handling" ) ) ]
        {
          let error_msg = error.to_string().to_lowercase();
          if error_msg.contains("rate limit") { "rate_limit" }
          else if error_msg.contains("timeout") { "timeout" }
          else if error_msg.contains("network") { "network" }
          else if error_msg.contains("http") { "http" }
          else if error_msg.contains("auth") { "authentication" }
          else if error_msg.contains("invalid") { "invalid_argument" }
          else { "other" }
        }
      };

      *self.error_counts.entry( error_type.to_string() ).or_insert( 0 ) += 1;
    }

    /// Get total attempts
    pub fn total_attempts( &self ) -> u64
    {
      self.total_attempts
    }

    /// Get successful retries
    pub fn successful_retries( &self ) -> u64
    {
      self.successful_retries
    }

    /// Get failed attempts
    pub fn failed_attempts( &self ) -> u64
    {
      self.failed_attempts
    }

    /// Get total delay in milliseconds
    pub fn total_delay_ms( &self ) -> u64
    {
      self.total_delay_ms
    }

    /// Get average delay per attempt
    pub fn average_delay_ms( &self ) -> f64
    {
      if self.total_attempts == 0
      {
        0.0
      }
      else
      {
        self.total_delay_ms as f64 / self.total_attempts as f64
      }
    }

    /// Reset all metrics
    pub fn reset( &mut self )
    {
      self.total_attempts = 0;
      self.successful_retries = 0;
      self.failed_attempts = 0;
      self.total_delay_ms = 0;
      self.error_counts.clear();
    }
  }
}

#[ cfg( feature = "retry-logic" ) ]
crate::mod_interface!
{
  // Core retry types
  exposed use RetryConfig;
  exposed use RetryStrategy;
  exposed use RetryStrategyType;
  exposed use RetryExecutor;

  // Backoff strategy types
  exposed use BackoffStrategyType;
  exposed use BackoffStrategyDetails;

  // Metrics
  exposed use RetryMetrics;
}
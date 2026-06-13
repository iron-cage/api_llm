//! Explicit retry builder and client monitoring types
//!
//! `ExplicitRetryBuilder`, `RateLimitInfo`, and `HealthStatus` for transparent
//! client-side retry control and request monitoring.

#[ allow( clippy::missing_inline_in_public_items ) ]
mod private
{
  use super::super::implementation::orphan::Client;
  #[ cfg( feature = "error-handling" ) ]
  use crate::error::{ AnthropicError, AnthropicResult };
  use std::time::Duration;

  impl Client
  {
    /// Get explicit rate limit information for manual control
    ///
    /// # Governing Principle Compliance
    ///
    /// This method follows the "Thin Client, Rich API" principle by:
    /// - **Information vs Action**: Provides rate limit data without making automatic decisions
    /// - **Zero Automatic Behavior**: No hidden rate limiting or magic throttling
    /// - **Explicit Control**: Developers can use rate limit information to make their own decisions
    /// - **Transparent Operations**: All rate limit metrics are visible and accessible
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use api_claude::{ Client, Secret };
    ///
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "sk-ant-api03-example".to_string() ).unwrap();
    /// let client = Client::new( secret );
    ///
    /// // Check rate limit information for manual decision making
    /// let rate_info = client.rate_limit_info();
    /// if rate_info.remaining_requests() < 10 {
    ///     // Developer decides to wait or use alternative strategy
    ///     println!( "Rate limit approaching : {} requests remaining", rate_info.remaining_requests() );
    /// }
    /// # Ok( () )
    /// # }
    /// ```
    pub fn rate_limit_info( &self ) -> RateLimitInfo
    {
      RateLimitInfo::new()
    }

    /// Get explicit health monitoring information for manual decision making
    ///
    /// # Governing Principle Compliance
    ///
    /// This method follows the "Thin Client, Rich API" principle by:
    /// - **Information vs Action**: Provides health information without making automatic decisions
    /// - **Zero Automatic Behavior**: No hidden health-based request blocking or filtering
    /// - **Explicit Control**: Developers can use health information to make their own decisions
    /// - **Transparent Operations**: All health data is visible and accessible
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use api_claude::{ Client, Secret };
    ///
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "sk-ant-api03-example".to_string() ).unwrap();
    /// let client = Client::new( secret );
    ///
    /// // Check health information for manual decision making
    /// let health = client.health();
    /// if health.consecutive_failures() > 5 {
    ///     // Developer decides to wait or use alternative strategy
    ///     println!( "High failure rate detected : {} failures", health.consecutive_failures() );
    /// }
    /// # Ok( () )
    /// # }
    /// ```
    pub fn health( &self ) -> HealthStatus
    {
      HealthStatus::new()
    }

    /// Create an explicit retry builder for this client
    ///
    /// # Governing Principle Compliance
    ///
    /// This method follows the "Thin Client, Rich API" principle by:
    /// - **Explicit Configuration**: All retry behavior must be explicitly configured by developers
    /// - **Zero Automatic Behavior**: No hidden retry logic or magic thresholds
    /// - **Transparent Configuration**: All retry parameters are explicitly specified
    /// - **Information vs Action**: Provides retry capability without imposing retry decisions
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use api_claude::{ Client, Secret };
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "sk-ant-api03-example".to_string() ).unwrap();
    /// let client = Client::new( secret );
    ///
    /// // Explicit retry with manual configuration
    /// let response = client
    ///   .explicit_retry()
    ///   .with_attempts( 3 )
    ///   .with_delay( Duration::from_secs( 1 ) )
    ///   .execute( | _client | async move {
    ///     // Your API operation here
    ///     Ok( "operation result".to_string() )
    ///   } )
    ///   .await?;
    /// # Ok( () )
    /// # }
    /// ```
    pub fn explicit_retry( &self ) -> ExplicitRetryBuilder< '_ >
    {
      ExplicitRetryBuilder::new( self )
    }
  }

  /// Rate limit information for explicit control
  ///
  /// # Governing Principle Compliance
  ///
  /// This struct follows the "Thin Client, Rich API" principle by:
  /// - **Information vs Action**: Provides rate limit data without making automatic decisions
  /// - **Zero Automatic Behavior**: No hidden rate limiting or magic throttling thresholds
  /// - **Explicit Control**: Developers can use this information to make their own timing decisions
  /// - **Transparent Operations**: All rate limit metrics are visible and accessible
  #[ derive( Debug, Clone ) ]
  pub struct RateLimitInfo
  {
    remaining_requests : u32,
    total_limit : u32,
    reset_time : Option< std::time::SystemTime >,
    window_duration : std::time::Duration,
  }

  impl RateLimitInfo
  {
    /// Create new rate limit info (currently returns placeholder data)
    ///
    /// # Note
    ///
    /// This is a placeholder implementation. In a real implementation, this would
    /// track actual rate limit headers from API responses. For now, it provides the
    /// interface for explicit rate limit information access without automatic behavior.
    pub fn new() -> Self
    {
      Self
      {
        remaining_requests : 1000, // Placeholder
        total_limit : 1000,        // Placeholder
        reset_time : None,         // Placeholder
        window_duration : std::time::Duration::from_secs( 3600 ), // Placeholder : 1 hour
      }
    }

    /// Get remaining requests in current window
    #[ inline ]
    #[ must_use ]
    pub fn remaining_requests( &self ) -> u32
    {
      self.remaining_requests
    }

    /// Get total rate limit for current window
    #[ inline ]
    #[ must_use ]
    pub fn total_limit( &self ) -> u32
    {
      self.total_limit
    }

    /// Get time when rate limit window resets
    #[ inline ]
    #[ must_use ]
    pub fn reset_time( &self ) -> Option< std::time::SystemTime >
    {
      self.reset_time
    }

    /// Get duration of rate limit window
    #[ inline ]
    #[ must_use ]
    pub fn window_duration( &self ) -> std::time::Duration
    {
      self.window_duration
    }

  }

  impl Default for RateLimitInfo
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl RateLimitInfo
  {
    /// Calculate usage percentage (0.0 to 1.0)
    #[ inline ]
    #[ must_use ]
    pub fn usage_percentage( &self ) -> f64
    {
      if self.total_limit == 0
      {
        0.0
      }
      else
      {
        let used = self.total_limit.saturating_sub( self.remaining_requests );
        f64::from( used ) / f64::from( self.total_limit )
      }
    }

    /// Check if rate limit is approaching based on explicit criteria
    ///
    /// # Arguments
    ///
    /// * `threshold_percentage` - Percentage threshold (0.0 to 1.0) for considering rate limit approaching
    ///
    /// # Returns
    ///
    /// Returns true if usage percentage exceeds the specified threshold (explicit developer-defined threshold)
    #[ inline ]
    #[ must_use ]
    pub fn is_approaching_limit_with_threshold( &self, threshold_percentage : f64 ) -> bool
    {
      self.usage_percentage() >= threshold_percentage
    }

    /// Calculate suggested delay for manual rate control
    ///
    /// # Arguments
    ///
    /// * `desired_requests_per_minute` - Target request rate for manual pacing
    ///
    /// # Returns
    ///
    /// Returns suggested delay duration for achieving the desired rate (developer-controlled pacing)
    #[ inline ]
    #[ must_use ]
    pub fn suggested_delay_for_rate( &self, desired_requests_per_minute : u32 ) -> std::time::Duration
    {
      if desired_requests_per_minute == 0
      {
        std::time::Duration::from_secs( 60 ) // Safe fallback
      }
      else
      {
        let seconds_per_request = 60.0 / f64::from( desired_requests_per_minute );
        std::time::Duration::from_secs_f64( seconds_per_request )
      }
    }
  }

  /// Health status information for explicit monitoring
  ///
  /// # Governing Principle Compliance
  ///
  /// This struct follows the "Thin Client, Rich API" principle by:
  /// - **Information vs Action**: Provides health data without making automatic decisions
  /// - **Zero Automatic Behavior**: No hidden health-based logic or magic thresholds
  /// - **Explicit Control**: Developers can use this information to make their own decisions
  /// - **Transparent Operations**: All health metrics are visible and accessible
  #[ derive( Debug, Clone ) ]
  pub struct HealthStatus
  {
    consecutive_failures : u32,
    total_requests : u64,
    total_failures : u64,
    last_error : Option< String >,
  }

  impl HealthStatus
  {
    /// Create new health status (currently returns placeholder data)
    ///
    /// # Note
    ///
    /// This is a placeholder implementation. In a real implementation, this would
    /// track actual request metrics. For now, it provides the interface for explicit
    /// health monitoring without automatic behavior.
    pub fn new() -> Self
    {
      Self
      {
        consecutive_failures : 0,
        total_requests : 0,
        total_failures : 0,
        last_error : None,
      }
    }

  }

  impl Default for HealthStatus
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl HealthStatus
  {
    /// Get consecutive failures count
    #[ inline ]
    #[ must_use ]
    pub fn consecutive_failures( &self ) -> u32
    {
      self.consecutive_failures
    }

    /// Get total requests count
    #[ inline ]
    #[ must_use ]
    pub fn total_requests( &self ) -> u64
    {
      self.total_requests
    }

    /// Get total failures count
    #[ inline ]
    #[ must_use ]
    pub fn total_failures( &self ) -> u64
    {
      self.total_failures
    }

    /// Get success rate (0.0 to 1.0)
    #[ inline ]
    #[ must_use ]
    pub fn success_rate( &self ) -> f64
    {
      if self.total_requests == 0
      {
        1.0
      }
      else
      {
        let successes = self.total_requests.saturating_sub( self.total_failures );
        successes as f64 / self.total_requests as f64
      }
    }

    /// Get last error message if any
    #[ inline ]
    #[ must_use ]
    pub fn last_error( &self ) -> Option< &str >
    {
      self.last_error.as_deref()
    }

    /// Check if the service appears healthy based on explicit criteria
    ///
    /// # Arguments
    ///
    /// * `max_consecutive_failures` - Maximum consecutive failures before considering unhealthy
    /// * `min_success_rate` - Minimum success rate (0.0 to 1.0) before considering unhealthy
    ///
    /// # Returns
    ///
    /// Returns true if health criteria are met (explicit developer-defined thresholds)
    #[ inline ]
    #[ must_use ]
    pub fn is_healthy_with_criteria( &self, max_consecutive_failures : u32, min_success_rate : f64 ) -> bool
    {
      self.consecutive_failures <= max_consecutive_failures && self.success_rate() >= min_success_rate
    }
  }

  /// Type alias for retry predicate function
  type RetryPredicate = Box< dyn Fn( &AnthropicError, u32 ) -> bool + Send + Sync >;

  /// Builder for explicit retry operations
  ///
  /// # Governing Principle Compliance
  ///
  /// This builder follows the "Thin Client, Rich API" principle by:
  /// - **Explicit Configuration**: All retry behavior must be explicitly configured by developers
  /// - **Zero Magic**: No automatic retry decisions or hidden retry logic
  /// - **Transparent Control**: Every retry parameter is visible and configurable
  /// - **Information vs Action**: Provides retry information without making retry decisions
  pub struct ExplicitRetryBuilder< 'a >
  {
    client : &'a Client,
    max_attempts : Option< u32 >,
    delay : Option< Duration >,
    should_retry_fn : Option< RetryPredicate >,
  }

  impl core::fmt::Debug for ExplicitRetryBuilder< '_ >
  {
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      f.debug_struct( "ExplicitRetryBuilder" )
      .field( "max_attempts", &self.max_attempts )
      .field( "delay", &self.delay )
      .field( "has_custom_retry_fn", &self.should_retry_fn.is_some() )
      .finish()
    }
  }

  impl< 'a > ExplicitRetryBuilder< 'a >
  {
    /// Create new explicit retry builder
    fn new( client : &'a Client ) -> Self
    {
      Self
      {
        client,
        max_attempts : None,
        delay : None,
        should_retry_fn : None,
      }
    }

    /// Set maximum number of retry attempts (explicit configuration required)
    ///
    /// # Arguments
    ///
    /// * `attempts` - Maximum number of attempts (must be >= 1)
    #[ must_use ]
    pub fn with_attempts( mut self, attempts : u32 ) -> Self
    {
      self.max_attempts = Some( attempts );
      self
    }

    /// Set delay between retry attempts (explicit configuration required)
    ///
    /// # Arguments
    ///
    /// * `delay` - Duration to wait between attempts
    #[ must_use ]
    pub fn with_delay( mut self, delay : Duration ) -> Self
    {
      self.delay = Some( delay );
      self
    }

    /// Set custom retry condition (explicit configuration required)
    ///
    /// # Arguments
    ///
    /// * `should_retry` - Function that determines if an error should be retried
    #[ must_use ]
    pub fn with_retry_condition< F >( mut self, should_retry : F ) -> Self
    where
      F : Fn( &AnthropicError, u32 ) -> bool + Send + Sync + 'static,
    {
      self.should_retry_fn = Some( Box::new( should_retry ) );
      self
    }

    /// Execute operation with explicit retry configuration
    ///
    /// # Arguments
    ///
    /// * `operation` - Function that performs the operation to retry
    ///
    /// # Errors
    ///
    /// Returns the last error if all retry attempts fail, or validation errors if configuration is invalid
    ///
    /// # Panics
    ///
    /// This function contains an internal unwrap that is guaranteed safe due to program logic
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use api_claude::{ Client, Secret };
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "sk-ant-api03-example".to_string() ).unwrap();
    /// let client = Client::new( secret );
    ///
    /// // Execute with explicit retry configuration
    /// let result = client
    ///   .explicit_retry()
    ///   .with_attempts( 3 )
    ///   .with_delay( Duration::from_secs( 2 ) )
    ///   .execute( | client | async move {
    ///     // Your API call here
    ///     Ok( "result".to_string() )
    ///   } )
    ///   .await?;
    /// # Ok( () )
    /// # }
    /// ```
    pub async fn execute< F, Fut, T >( self, operation : F ) -> AnthropicResult< T >
    where
      F : Fn( &Client ) -> Fut,
      Fut : core::future::Future< Output = AnthropicResult< T > >,
    {
      // Validate configuration
      let max_attempts = self.max_attempts
        .ok_or_else( || AnthropicError::InvalidArgument( "max_attempts must be explicitly configured".to_string() ) )?;

      let delay = self.delay
        .ok_or_else( || AnthropicError::InvalidArgument( "delay must be explicitly configured".to_string() ) )?;

      if max_attempts == 0
      {
        return Err( AnthropicError::InvalidArgument( "max_attempts must be >= 1".to_string() ) );
      }

      let should_retry = self.should_retry_fn.unwrap_or_else( || {
        Box::new( | error : &AnthropicError, _attempt : u32 | -> bool {
          // Default retry condition for common retryable errors
          #[ cfg( feature = "error-handling" ) ]
          {
            match error
            {
              AnthropicError::RateLimit( _ ) |
              AnthropicError::Stream( _ ) |
              AnthropicError::Internal( _ ) => true,
              AnthropicError::Http( http_error ) => {
                http_error.status_code().map_or( true, | code | ( 500..600 ).contains( &code ) )
              },
              _ => false,
            }
          }
          #[ cfg( not( feature = "error-handling" ) ) ]
          {
            let error_msg = error.to_string().to_lowercase();
            error_msg.contains( "timeout" ) ||
            error_msg.contains( "rate limit" ) ||
            error_msg.contains( "5" ) // Basic HTTP 5xx detection
          }
        } )
      } );

      let mut last_error = None;

      for attempt in 1..=max_attempts
      {
        match operation( self.client ).await
        {
          Ok( result ) => return Ok( result ),
          Err( error ) =>
          {
            last_error = Some( error );

            // Check if we should retry
            if attempt < max_attempts && should_retry( last_error.as_ref().unwrap(), attempt )
            {
              tokio::time::sleep( delay ).await;
            }
            else
            {
              break;
            }
          }
        }
      }

      Err( last_error.unwrap_or_else( ||
        AnthropicError::Internal( "Unexpected retry failure".to_string() )
      ) )
    }
  }
}

crate::mod_interface!
{
  exposed use RateLimitInfo;
  exposed use HealthStatus;
  exposed use ExplicitRetryBuilder;
}

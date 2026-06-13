//! Circuit breaker implementation for the Anthropic API client
//!
//! This module provides circuit breaker pattern functionality to handle
//! service failures gracefully and prevent cascading failures.

#[ cfg( feature = "circuit-breaker" ) ]
mod private
{
  use std::time::Instant;
  use std::sync::{ Arc, Mutex };

  #[ cfg( feature = "error-handling" ) ]
  use crate::error::AnthropicError;

  #[ cfg( not( feature = "error-handling" ) ) ]
  type AnthropicError = crate::error_tools::Error;

  /// Circuit breaker states
  #[ derive( Debug, Clone, PartialEq, Eq ) ]
  pub enum CircuitState
  {
    /// Circuit is closed, requests are allowed
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, limited requests are allowed for testing
    HalfOpen,
  }

  /// Circuit breaker configuration
  #[ derive( Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize ) ]
  pub struct CircuitBreakerConfig
  {
    /// Number of failures required to open the circuit
    failure_threshold : u32,
    /// Number of successes required to close the circuit from half-open
    success_threshold : u32,
    /// Timeout before allowing recovery attempt (milliseconds)
    timeout_ms : u64,
    /// Timeout for half-open state (milliseconds)
    half_open_timeout_ms : u64,
    /// Whether to ignore authentication errors
    ignore_auth_errors : bool,
    /// Whether to ignore rate limit errors
    ignore_rate_limit_errors : bool,
    /// Whether to ignore validation errors
    ignore_validation_errors : bool,
    /// Size of the sliding window for failure tracking
    window_size : usize,
  }

  impl Default for CircuitBreakerConfig 
  {
    fn default() -> Self 
    {
      Self::new()
    }
  }

  impl CircuitBreakerConfig
  {
    /// Create circuit breaker configuration with explicit parameters (no defaults)
    ///
    /// # Arguments
    ///
    /// * `failure_threshold` - Number of failures required to open the circuit (must be > 0)
    /// * `success_threshold` - Number of successes required to close the circuit from half-open (must be > 0)
    /// * `timeout_ms` - Timeout before allowing recovery attempt in milliseconds (must be > 0)
    /// * `half_open_timeout_ms` - Timeout for half-open state in milliseconds
    /// * `ignore_auth_errors` - Whether to ignore authentication errors in failure counting
    /// * `ignore_rate_limit_errors` - Whether to ignore rate limit errors in failure counting
    /// * `ignore_validation_errors` - Whether to ignore validation errors in failure counting
    /// * `window_size` - Size of the sliding window for failure tracking
    #[ allow( clippy::too_many_arguments ) ]
    pub fn with_explicit_config(
      failure_threshold : u32,
      success_threshold : u32,
      timeout_ms : u64,
      half_open_timeout_ms : u64,
      ignore_auth_errors : bool,
      ignore_rate_limit_errors : bool,
      ignore_validation_errors : bool,
      window_size : usize,
    ) -> Self
    {
      Self {
        failure_threshold,
        success_threshold,
        timeout_ms,
        half_open_timeout_ms,
        ignore_auth_errors,
        ignore_rate_limit_errors,
        ignore_validation_errors,
        window_size,
      }
    }

    /// Create a new circuit breaker configuration (compatibility wrapper)
    ///
    /// NOTE: This is a compatibility wrapper with sensible defaults. For explicit control, use `with_explicit_config()`
    #[ must_use ]
    pub fn new() -> Self
    {
      // Compatibility wrapper with sensible defaults for circuit breaker
      Self::with_explicit_config(
        5,     // failure_threshold : 5 failures to open
        3,     // success_threshold : 3 successes to close
        60000, // timeout_ms : 1 minute
        30000, // half_open_timeout_ms : 30 seconds
        true,  // ignore_auth_errors : true (auth errors don't count as circuit failures)
        true,  // ignore_rate_limit_errors : true (rate limits don't count as circuit failures)
        true,  // ignore_validation_errors : true (validation errors don't count as circuit failures)
        100,   // window_size : 100 requests for tracking
      )
    }

    /// Set the failure threshold
    #[ must_use ]
    pub fn with_failure_threshold( mut self, threshold : u32 ) -> Self
    {
      self.failure_threshold = threshold;
      self
    }

    /// Set the success threshold
    #[ must_use ]
    pub fn with_success_threshold( mut self, threshold : u32 ) -> Self
    {
      self.success_threshold = threshold;
      self
    }

    /// Set the timeout in milliseconds
    #[ must_use ]
    pub fn with_timeout_ms( mut self, timeout : u64 ) -> Self
    {
      self.timeout_ms = timeout;
      self
    }

    /// Set the half-open timeout in milliseconds
    #[ must_use ]
    pub fn with_half_open_timeout_ms( mut self, timeout : u64 ) -> Self
    {
      self.half_open_timeout_ms = timeout;
      self
    }

    /// Set whether to ignore authentication errors
    #[ must_use ]
    pub fn with_ignore_auth_errors( mut self, ignore : bool ) -> Self
    {
      self.ignore_auth_errors = ignore;
      self
    }

    /// Set whether to ignore rate limit errors
    #[ must_use ]
    pub fn with_ignore_rate_limit_errors( mut self, ignore : bool ) -> Self
    {
      self.ignore_rate_limit_errors = ignore;
      self
    }

    /// Set whether to ignore validation errors
    #[ must_use ]
    pub fn with_ignore_validation_errors( mut self, ignore : bool ) -> Self
    {
      self.ignore_validation_errors = ignore;
      self
    }

    /// Set the window size for failure tracking
    #[ must_use ]
    pub fn with_window_size( mut self, size : usize ) -> Self
    {
      self.window_size = size;
      self
    }

    /// Validate the configuration
    #[ must_use ]
    pub fn is_valid( &self ) -> bool
    {
      self.failure_threshold > 0 &&
      self.success_threshold > 0 &&
      self.timeout_ms > 0
    }

    /// Get the failure threshold
    #[ must_use ]
    pub fn failure_threshold( &self ) -> u32
    {
      self.failure_threshold
    }

    /// Get the success threshold
    #[ must_use ]
    pub fn success_threshold( &self ) -> u32
    {
      self.success_threshold
    }

    /// Get the timeout in milliseconds
    #[ must_use ]
    pub fn timeout_ms( &self ) -> u64
    {
      self.timeout_ms
    }

    /// Get the half-open timeout in milliseconds
    #[ must_use ]
    pub fn half_open_timeout_ms( &self ) -> u64
    {
      self.half_open_timeout_ms
    }
  }

  // No Default implementation - explicit configuration required per governing principle

  /// Circuit breaker metrics
  #[ derive( Debug, Clone ) ]
  pub struct CircuitBreakerMetrics
  {
    total_requests : u64,
    success_count : u64,
    failure_count : u64,
    state_changes : u64,
  }

  impl Default for CircuitBreakerMetrics 
  {
    fn default() -> Self 
    {
      Self::new()
    }
  }

  impl CircuitBreakerMetrics
  {
    /// Create new metrics
    #[ must_use ]
    pub fn new() -> Self
    {
      Self {
        total_requests : 0,
        success_count : 0,
        failure_count : 0,
        state_changes : 0,
      }
    }

    /// Get total requests
    #[ must_use ]
    pub fn total_requests( &self ) -> u64
    {
      self.total_requests
    }

    /// Get success count
    #[ must_use ]
    pub fn success_count( &self ) -> u64
    {
      self.success_count
    }

    /// Get failure count
    #[ must_use ]
    pub fn failure_count( &self ) -> u64
    {
      self.failure_count
    }

    /// Get state changes
    #[ must_use ]
    pub fn state_changes( &self ) -> u64
    {
      self.state_changes
    }

    /// Get success rate
    #[ must_use ]
    pub fn success_rate( &self ) -> f64
    {
      if self.total_requests == 0
      {
        1.0
      }
      else
      {
        self.success_count as f64 / self.total_requests as f64
      }
    }

    /// Export metrics in Prometheus format
    #[ must_use ]
    pub fn export_prometheus_format( &self ) -> String
    {
      format!(
        "circuit_breaker_requests_total {}\ncircuit_breaker_failures_total {}\ncircuit_breaker_state 0\ncircuit_breaker_success_rate {}",
        self.total_requests,
        self.failure_count,
        self.success_rate()
      )
    }

    /// Export metrics as JSON
    #[ must_use ]
    pub fn to_json( &self ) -> String
    {
      format!(
        r#"{{"total_requests": {}, "success_count": {}, "failure_count": {}, "state_changes": {}, "success_rate": {}}}"#,
        self.total_requests,
        self.success_count,
        self.failure_count,
        self.state_changes,
        self.success_rate()
      )
    }
  }

  /// Circuit breaker implementation
  #[ derive( Debug ) ]
  pub struct CircuitBreaker
  {
    config : CircuitBreakerConfig,
    state : Arc< Mutex< CircuitBreakerState > >,
  }

  #[ derive( Debug ) ]
  struct CircuitBreakerState
  {
    current_state : CircuitState,
    failure_count : u32,
    success_count : u32,
    last_failure_time : Option< Instant >,
    last_state_change : Instant,
    metrics : CircuitBreakerMetrics,
  }

  impl CircuitBreaker
  {
    /// Create a new circuit breaker
    pub fn new( config : CircuitBreakerConfig ) -> Self
    {
      let state = CircuitBreakerState {
        current_state : CircuitState::Closed,
        failure_count : 0,
        success_count : 0,
        last_failure_time : None,
        last_state_change : Instant::now(),
        metrics : CircuitBreakerMetrics::new(),
      };

      Self {
        config,
        state : Arc::new( Mutex::new( state ) ),
      }
    }

    /// Get current circuit state
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn state( &self ) -> CircuitState
    {
      let state = self.state.lock().unwrap();
      state.current_state.clone()
    }

    /// Check if requests can be executed
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn can_execute( &self ) -> bool
    {
      let state = self.state.lock().unwrap();
      match state.current_state
      {
        CircuitState::Closed | CircuitState::HalfOpen => true,
        CircuitState::Open => false,
      }
    }

    /// Check if circuit can attempt recovery
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn can_attempt_recovery( &self ) -> bool
    {
      let state = self.state.lock().unwrap();
      if state.current_state != CircuitState::Open
      {
        return false;
      }

      if let Some( last_failure ) = state.last_failure_time
      {
        let elapsed = last_failure.elapsed();
        elapsed.as_millis() >= u128::from(self.config.timeout_ms)
      }
      else
      {
        false
      }
    }

    /// Attempt to recover (transition to half-open)
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn attempt_recovery( &self )
    {
      let mut state = self.state.lock().unwrap();
      if state.current_state == CircuitState::Open && self.can_attempt_recovery()
      {
        state.current_state = CircuitState::HalfOpen;
        state.success_count = 0;
        state.last_state_change = Instant::now();
        state.metrics.state_changes += 1;
      }
    }

    /// Record a successful operation
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn record_success( &self )
    {
      let mut state = self.state.lock().unwrap();
      state.success_count += 1;
      state.metrics.total_requests += 1;
      state.metrics.success_count += 1;

      if state.current_state == CircuitState::HalfOpen && state.success_count >= self.config.success_threshold
      {
        state.current_state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
        state.last_state_change = Instant::now();
        state.metrics.state_changes += 1;
      }
    }

    /// Record a failed operation with explicit error classification
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn record_failure_with_config(
      &self,
      error : &AnthropicError,
      failure_status_codes : Option< &std::ops::Range< u16 > >,
      consider_network_errors_as_failures : bool,
      default_error_classification : bool,
    )
    {
      if !self.is_failure_with_config( error, failure_status_codes, consider_network_errors_as_failures, default_error_classification )
      {
        return; // Don't count this as a failure
      }

      let mut state = self.state.lock().unwrap();
      state.failure_count += 1;
      state.metrics.total_requests += 1;
      state.metrics.failure_count += 1;
      state.last_failure_time = Some( Instant::now() );

      match state.current_state
      {
        CircuitState::Closed =>
        {
          if state.failure_count >= self.config.failure_threshold
          {
            state.current_state = CircuitState::Open;
            state.last_state_change = Instant::now();
            state.metrics.state_changes += 1;
          }
        },
        CircuitState::HalfOpen =>
        {
          // Any failure in half-open immediately opens the circuit
          state.current_state = CircuitState::Open;
          state.failure_count = 1;
          state.success_count = 0;
          state.last_state_change = Instant::now();
          state.metrics.state_changes += 1;
        },
        CircuitState::Open => {},
      }
    }

    /// Check if an error should be considered a failure with explicit error classification
    pub fn is_failure_with_config(
      &self,
      error : &AnthropicError,
      failure_status_codes : Option< &std::ops::Range< u16 > >,
      consider_network_errors_as_failures : bool,
      default_error_classification : bool,
    ) -> bool
    {
      match error
      {
        #[ cfg( feature = "error-handling" ) ]
        AnthropicError::Authentication( _ ) => !self.config.ignore_auth_errors,
        #[ cfg( feature = "error-handling" ) ]
        AnthropicError::RateLimit( _ ) => !self.config.ignore_rate_limit_errors,
        #[ cfg( feature = "error-handling" ) ]
        AnthropicError::InvalidArgument( _ ) | AnthropicError::InvalidRequest( _ ) => !self.config.ignore_validation_errors,
        #[ cfg( feature = "error-handling" ) ]
        AnthropicError::Http( http_error ) =>
        {
          // Use explicit status code configuration instead of magic ranges
          if let Some( status_code ) = http_error.status_code()
          {
            if let Some( failure_range ) = failure_status_codes
            {
              failure_range.contains( &status_code )
            }
            else
            {
              default_error_classification
            }
          }
          else
          {
            // Use explicit network error classification instead of automatic decision
            consider_network_errors_as_failures
          }
        },
        _ => default_error_classification, // Use explicit default instead of automatic assumption
      }
    }

    /// Record a failed operation (compatibility wrapper with sensible defaults)
    ///
    /// NOTE: This is a compatibility wrapper. For explicit control, use `record_failure_with_config()`
    pub fn record_failure( &self, error : &AnthropicError )
    {
      // Compatibility wrapper with sensible defaults for HTTP errors
      let http_5xx_range = 500..600;
      self.record_failure_with_config(
        error,
        Some( &http_5xx_range ),
        true, // Consider network errors as failures
        true, // Default error classification : consider as failure
      );
    }

    /// Check if an error should be considered a failure (compatibility wrapper with sensible defaults)
    ///
    /// NOTE: This is a compatibility wrapper. For explicit control, use `is_failure_with_config()`
    pub fn is_failure( &self, error : &AnthropicError ) -> bool
    {
      // Compatibility wrapper with sensible defaults for HTTP errors
      let http_5xx_range = 500..600;
      self.is_failure_with_config(
        error,
        Some( &http_5xx_range ),
        true, // Consider network errors as failures
        true, // Default error classification : consider as failure
      )
    }

    /// Get circuit breaker metrics
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn metrics( &self ) -> CircuitBreakerMetrics
    {
      let state = self.state.lock().unwrap();
      state.metrics.clone()
    }

    /// Reset the circuit breaker
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn reset( &self )
    {
      let mut state = self.state.lock().unwrap();
      state.current_state = CircuitState::Closed;
      state.failure_count = 0;
      state.success_count = 0;
      state.last_failure_time = None;
      state.last_state_change = Instant::now();
      state.metrics = CircuitBreakerMetrics::new();
    }
  }

  // Clone shares Arc<Mutex<CircuitBreakerState>> — cloned instances share circuit state
  impl Clone for CircuitBreaker
  {
    fn clone( &self ) -> Self
    {
      Self {
        config : self.config.clone(),
        state : self.state.clone(),
      }
    }
  }

}

#[ cfg( feature = "circuit-breaker" ) ]
crate::mod_interface!
{
  exposed use
  {
    CircuitBreaker,
    CircuitBreakerConfig,
    CircuitBreakerMetrics,
    CircuitState,
  };
}
//! Enterprise Configuration Module
//!
//! Provides unified configuration builder for all enterprise reliability features:
//! - Retry logic
//! - Circuit breaker
//! - Rate limiting
//! - Failover
//! - Health checks

mod private
{
  use serde::{ Serialize, Deserialize };
  use std::time::Duration;

/// Enterprise configuration combining all reliability features
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct EnterpriseConfig
{
  #[ cfg( feature = "retry-logic" ) ]
  retry : Option< crate::retry_logic::RetryConfig >,
  #[ cfg( feature = "circuit-breaker" ) ]
  circuit_breaker : Option< crate::circuit_breaker::CircuitBreakerConfig >,
  #[ cfg( feature = "rate-limiting" ) ]
  rate_limiting : Option< crate::rate_limiting::RateLimiterConfig >,
  #[ cfg( feature = "failover" ) ]
  failover : Option< crate::failover::FailoverConfig >,
  #[ cfg( feature = "health-checks" ) ]
  health_checks : Option< crate::health_checks::HealthCheckConfig >,
}

impl EnterpriseConfig
{
  /// Check if configuration is valid
  pub fn is_valid( &self ) -> bool
  {
    self.validate().is_ok()
  }

  /// Validate the enterprise configuration
  ///
  /// # Errors
  ///
  /// Returns an error if any of the enabled feature configurations are invalid
  pub fn validate( &self ) -> Result< (), String >
  {
    // Validate retry config
    #[ cfg( feature = "retry-logic" ) ]
    if let Some( ref retry ) = self.retry
    {
      if !retry.is_valid()
      {
        return Err( "Invalid retry configuration".to_string() );
      }
    }

    // Validate circuit breaker
    #[ cfg( feature = "circuit-breaker" ) ]
    if let Some( ref cb ) = self.circuit_breaker
    {
      if !cb.is_valid()
      {
        return Err( "Invalid circuit breaker configuration".to_string() );
      }
    }

    // Validate rate limiting
    #[ cfg( feature = "rate-limiting" ) ]
    if let Some( ref rl ) = self.rate_limiting
    {
      if !rl.is_valid()
      {
        return Err( "Invalid rate limiting configuration".to_string() );
      }
    }

    Ok(())
  }

  /// Check if retry is enabled
  #[ cfg( feature = "retry-logic" ) ]
  pub fn retry_enabled( &self ) -> bool
  {
    self.retry.is_some()
  }

  /// Check if retry is enabled (stub when feature disabled)
  #[ cfg( not( feature = "retry-logic" ) ) ]
  pub fn retry_enabled( &self ) -> bool
  {
    false
  }

  /// Get retry configuration
  #[ cfg( feature = "retry-logic" ) ]
  pub fn retry_config( &self ) -> Option< &crate::retry_logic::RetryConfig >
  {
    self.retry.as_ref()
  }

  /// Check if circuit breaker is enabled
  #[ cfg( feature = "circuit-breaker" ) ]
  pub fn circuit_breaker_enabled( &self ) -> bool
  {
    self.circuit_breaker.is_some()
  }

  /// Check if circuit breaker is enabled (stub for non-feature builds)
  #[ cfg( not( feature = "circuit-breaker" ) ) ]
  pub fn circuit_breaker_enabled( &self ) -> bool
  {
    false
  }

  /// Get circuit breaker configuration
  #[ cfg( feature = "circuit-breaker" ) ]
  pub fn circuit_breaker_config( &self ) -> Option< &crate::circuit_breaker::CircuitBreakerConfig >
  {
    self.circuit_breaker.as_ref()
  }

  /// Check if rate limiting is enabled
  #[ cfg( feature = "rate-limiting" ) ]
  pub fn rate_limiting_enabled( &self ) -> bool
  {
    self.rate_limiting.is_some()
  }

  /// Check if rate limiting is enabled (stub for non-feature builds)
  #[ cfg( not( feature = "rate-limiting" ) ) ]
  pub fn rate_limiting_enabled( &self ) -> bool
  {
    false
  }

  /// Get rate limiting configuration
  #[ cfg( feature = "rate-limiting" ) ]
  pub fn rate_limiting_config( &self ) -> Option< &crate::rate_limiting::RateLimiterConfig >
  {
    self.rate_limiting.as_ref()
  }

  /// Check if failover is enabled
  #[ cfg( feature = "failover" ) ]
  pub fn failover_enabled( &self ) -> bool
  {
    self.failover.is_some()
  }

  /// Check if failover is enabled (stub for non-feature builds)
  #[ cfg( not( feature = "failover" ) ) ]
  pub fn failover_enabled( &self ) -> bool
  {
    false
  }

  /// Check if health checks are enabled
  #[ cfg( feature = "health-checks" ) ]
  pub fn health_checks_enabled( &self ) -> bool
  {
    self.health_checks.is_some()
  }

  /// Check if health checks are enabled (stub for non-feature builds)
  #[ cfg( not( feature = "health-checks" ) ) ]
  pub fn health_checks_enabled( &self ) -> bool
  {
    false
  }
}

/// Builder for enterprise configuration
#[ derive( Debug, Clone ) ]
pub struct EnterpriseConfigBuilder
{
  #[ cfg( feature = "retry-logic" ) ]
  retry : Option< crate::retry_logic::RetryConfig >,
  #[ cfg( feature = "circuit-breaker" ) ]
  circuit_breaker : Option< crate::circuit_breaker::CircuitBreakerConfig >,
  #[ cfg( feature = "rate-limiting" ) ]
  rate_limiting : Option< crate::rate_limiting::RateLimiterConfig >,
  #[ cfg( feature = "failover" ) ]
  failover : Option< crate::failover::FailoverConfig >,
  #[ cfg( feature = "health-checks" ) ]
  health_checks : Option< crate::health_checks::HealthCheckConfig >,
}

impl EnterpriseConfigBuilder
{
  /// Create a new enterprise configuration builder
  pub fn new() -> Self
  {
    Self
    {
      #[ cfg( feature = "retry-logic" ) ]
      retry : None,
      #[ cfg( feature = "circuit-breaker" ) ]
      circuit_breaker : None,
      #[ cfg( feature = "rate-limiting" ) ]
      rate_limiting : None,
      #[ cfg( feature = "failover" ) ]
      failover : None,
      #[ cfg( feature = "health-checks" ) ]
      health_checks : None,
    }
  }

  /// Configure retry logic
  #[ cfg( feature = "retry-logic" ) ]
  #[ must_use ]
  pub fn with_retry( mut self, config : crate::retry_logic::RetryConfig ) -> Self
  {
    self.retry = Some( config );
    self
  }

  /// Configure circuit breaker
  #[ cfg( feature = "circuit-breaker" ) ]
  #[ must_use ]
  pub fn with_circuit_breaker( mut self, config : crate::circuit_breaker::CircuitBreakerConfig ) -> Self
  {
    self.circuit_breaker = Some( config );
    self
  }

  /// Configure rate limiting
  #[ cfg( feature = "rate-limiting" ) ]
  #[ must_use ]
  pub fn with_rate_limiting( mut self, config : crate::rate_limiting::RateLimiterConfig ) -> Self
  {
    self.rate_limiting = Some( config );
    self
  }

  /// Configure failover
  #[ cfg( feature = "failover" ) ]
  #[ must_use ]
  pub fn with_failover( mut self, config : crate::failover::FailoverConfig ) -> Self
  {
    self.failover = Some( config );
    self
  }

  /// Configure health checks
  #[ cfg( feature = "health-checks" ) ]
  #[ must_use ]
  pub fn with_health_checks( mut self, config : crate::health_checks::HealthCheckConfig ) -> Self
  {
    self.health_checks = Some( config );
    self
  }

  /// Build the enterprise configuration
  pub fn build( self ) -> EnterpriseConfig
  {
    EnterpriseConfig
    {
      #[ cfg( feature = "retry-logic" ) ]
      retry : self.retry,
      #[ cfg( feature = "circuit-breaker" ) ]
      circuit_breaker : self.circuit_breaker,
      #[ cfg( feature = "rate-limiting" ) ]
      rate_limiting : self.rate_limiting,
      #[ cfg( feature = "failover" ) ]
      failover : self.failover,
      #[ cfg( feature = "health-checks" ) ]
      health_checks : self.health_checks,
    }
  }

  /// Try to build with validation
  ///
  /// # Errors
  ///
  /// Returns an error if the enterprise configuration is invalid
  pub fn try_build( self ) -> Result< EnterpriseConfig, String >
  {
    let config = self.build();
    config.validate()?;
    Ok( config )
  }

  /// Create a conservative enterprise profile (high safety, low performance impact)
  pub fn conservative() -> EnterpriseConfig
  {
    let mut builder = Self::new();

    #[ cfg( feature = "retry-logic" ) ]
    {
      builder = builder.with_retry( crate::retry_logic::RetryConfig::new()
        .with_max_attempts( 3 )
        .with_initial_delay( Duration::from_millis( 100 ) )
        .with_max_delay( Duration::from_secs( 5 ) )
        .with_backoff_multiplier( 2.0 ) );
    }

    #[ cfg( feature = "circuit-breaker" ) ]
    {
      builder = builder.with_circuit_breaker( crate::circuit_breaker::CircuitBreakerConfig::new()
        .with_failure_threshold( 5 )
        .with_success_threshold( 2 ) );
    }

    builder.build()
  }

  /// Create a balanced enterprise profile (moderate safety and performance)
  pub fn balanced() -> EnterpriseConfig
  {
    let mut builder = Self::new();

    #[ cfg( feature = "retry-logic" ) ]
    {
      builder = builder.with_retry( crate::retry_logic::RetryConfig::new()
        .with_max_attempts( 5 )
        .with_initial_delay( Duration::from_millis( 50 ) )
        .with_max_delay( Duration::from_secs( 10 ) )
        .with_backoff_multiplier( 2.0 ) );
    }

    #[ cfg( feature = "circuit-breaker" ) ]
    {
      builder = builder.with_circuit_breaker( crate::circuit_breaker::CircuitBreakerConfig::new()
        .with_failure_threshold( 3 )
        .with_success_threshold( 2 ) );
    }

    #[ cfg( feature = "rate-limiting" ) ]
    {
      builder = builder.with_rate_limiting( crate::rate_limiting::RateLimiterConfig::new()
        .with_tokens_per_second( 10.0 )
        .with_bucket_capacity( 100 ) );
    }

    builder.build()
  }

  /// Create an aggressive enterprise profile (maximum reliability features)
  pub fn aggressive() -> EnterpriseConfig
  {
    let mut builder = Self::new();

    #[ cfg( feature = "retry-logic" ) ]
    {
      builder = builder.with_retry( crate::retry_logic::RetryConfig::new()
        .with_max_attempts( 10 )
        .with_initial_delay( Duration::from_millis( 25 ) )
        .with_max_delay( Duration::from_secs( 30 ) )
        .with_backoff_multiplier( 2.5 ) );
    }

    #[ cfg( feature = "circuit-breaker" ) ]
    {
      builder = builder.with_circuit_breaker( crate::circuit_breaker::CircuitBreakerConfig::new()
        .with_failure_threshold( 2 )
        .with_success_threshold( 3 ) );
    }

    #[ cfg( feature = "rate-limiting" ) ]
    {
      builder = builder.with_rate_limiting( crate::rate_limiting::RateLimiterConfig::new()
        .with_tokens_per_second( 5.0 )
        .with_bucket_capacity( 50 ) );
    }

    #[ cfg( feature = "failover" ) ]
    {
      builder = builder.with_failover( crate::failover::FailoverConfig::new()
        .with_strategy( crate::failover::FailoverStrategy::Priority ) );
    }

    #[ cfg( feature = "health-checks" ) ]
    {
      builder = builder.with_health_checks( crate::health_checks::HealthCheckConfig::new()
        .with_interval( Duration::from_secs( 30 ) )
        .with_timeout( Duration::from_secs( 5 ) ) );
    }

    builder.build()
  }

  /// Reset the builder to empty state
  #[ must_use ]
  pub fn reset( self ) -> Self
  {
    Self::new()
  }
}

impl Default for EnterpriseConfigBuilder
{
  fn default() -> Self
  {
    Self::new()
  }
}

} // end mod private

crate::mod_interface!
{
  exposed use EnterpriseConfig;
  exposed use EnterpriseConfigBuilder;
}

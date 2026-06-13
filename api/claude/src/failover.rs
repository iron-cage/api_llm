//! Failover Module
//!
//! This module provides stateless failover utilities for Anthropic Claude API requests.
//! Following the "Thin Client, Rich API" principle, this module offers failover patterns
//! and endpoint management tools without automatic behaviors or magic thresholds.
//!
//! # Key Principles
//!
//! - **Explicit Configuration**: All failover behavior must be explicitly configured
//! - **No Automatic Decisions**: Developer controls when and how failover occurs
//! - **Transparent Operation**: Clear visibility into failover state and decisions
//! - **Stateless Design**: No persistence beyond runtime state

mod private
{
  use std::
  {
    sync::{ Arc, Mutex },
    time::Instant,
  };
  use core::time::Duration;
  use serde::{ Deserialize, Serialize };

  /// Endpoint health status
  #[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
  pub enum EndpointHealth
  {
    /// Endpoint is healthy and available
    Healthy,
    /// Endpoint is degraded but still usable
    Degraded,
    /// Endpoint is unhealthy and should be avoided
    Unhealthy,
    /// Endpoint health is unknown
    Unknown,
  }

  /// Failover strategy for selecting endpoints
  #[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
  pub enum FailoverStrategy
  {
    /// Round-robin through available endpoints
    RoundRobin,
    /// Priority-based selection (highest priority first)
    Priority,
    /// Random selection from available endpoints
    Random,
    /// Sticky to first healthy endpoint
    Sticky,
  }

  /// Endpoint configuration for failover
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct FailoverEndpoint
  {
    /// Unique identifier for the endpoint
    pub id : String,
    /// Endpoint URL (e.g., "<https://api.anthropic.com>")
    pub url : String,
    /// Priority level (higher = more preferred)
    pub priority : i32,
    /// Maximum timeout for requests to this endpoint
    pub timeout : Duration,
    /// Current health status
    pub health : EndpointHealth,
    /// Last health check timestamp
    #[ serde( skip, default = "Instant::now" ) ]
    pub last_checked : Instant,
  }

  impl FailoverEndpoint
  {
    /// Create a new failover endpoint
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the endpoint
    /// * `url` - Full URL for the endpoint
    /// * `priority` - Priority level (higher = more preferred)
    /// * `timeout` - Maximum timeout duration
    ///
    /// # Example
    ///
    /// ```ignore
    /// use api_claude::{ FailoverEndpoint, Duration };
    ///
    /// let endpoint = FailoverEndpoint::new(
    ///   "primary".to_string(),
    ///   "https://api.anthropic.com".to_string(),
    ///   100,
    ///   Duration::from_secs( 30 )
    /// );
    /// ```
    #[ inline ]
    #[ must_use ]
    pub fn new( id : String, url : String, priority : i32, timeout : Duration ) -> Self
    {
      Self
      {
        id,
        url,
        priority,
        timeout,
        health : EndpointHealth::Unknown,
        last_checked : Instant::now(),
      }
    }

    /// Check if the endpoint is available (healthy or degraded)
    #[ inline ]
    #[ must_use ]
    pub fn is_available( &self ) -> bool
    {
      matches!( self.health, EndpointHealth::Healthy | EndpointHealth::Degraded )
    }

    /// Check if the endpoint is preferred (healthy only)
    #[ inline ]
    #[ must_use ]
    pub fn is_preferred( &self ) -> bool
    {
      matches!( self.health, EndpointHealth::Healthy )
    }

    /// Update the health status of the endpoint
    #[ inline ]
    pub fn update_health( &mut self, health : EndpointHealth )
    {
      self.health = health;
      self.last_checked = Instant::now();
    }

    /// Get time since last health check
    #[ inline ]
    #[ must_use ]
    pub fn time_since_check( &self ) -> Duration
    {
      self.last_checked.elapsed()
    }
  }

  /// Failover configuration and policy
  #[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
  pub struct FailoverConfig
  {
    /// Strategy for selecting endpoints
    pub strategy : FailoverStrategy,
    /// Maximum number of retry attempts
    pub max_retries : u32,
    /// Base delay between retries (in milliseconds)
    pub retry_delay_ms : u64,
    /// Maximum delay between retries (in milliseconds)
    pub max_retry_delay_ms : u64,
    /// Health check interval (in milliseconds)
    pub health_check_interval_ms : u64,
    /// Timeout for failover operations (in milliseconds)
    pub failover_timeout_ms : u64,
  }

  impl Default for FailoverConfig
  {
    #[ inline ]
    fn default() -> Self
    {
      Self
      {
        strategy : FailoverStrategy::Priority,
        max_retries : 3,
        retry_delay_ms : 1000,
        max_retry_delay_ms : 30000,
        health_check_interval_ms : 30000,
        failover_timeout_ms : 10000,
      }
    }
  }

  impl FailoverConfig
  {
    /// Create a new failover configuration with defaults
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Set the failover strategy
    #[ inline ]
    #[ must_use ]
    pub fn with_strategy( mut self, strategy : FailoverStrategy ) -> Self
    {
      self.strategy = strategy;
      self
    }

    /// Set maximum retry attempts
    #[ inline ]
    #[ must_use ]
    pub fn with_max_retries( mut self, max_retries : u32 ) -> Self
    {
      self.max_retries = max_retries;
      self
    }

    /// Set base retry delay
    #[ inline ]
    #[ must_use ]
    pub fn with_retry_delay_ms( mut self, delay_ms : u64 ) -> Self
    {
      self.retry_delay_ms = delay_ms;
      self
    }

    /// Set maximum retry delay
    #[ inline ]
    #[ must_use ]
    pub fn with_max_retry_delay_ms( mut self, max_delay_ms : u64 ) -> Self
    {
      self.max_retry_delay_ms = max_delay_ms;
      self
    }

    /// Set health check interval
    #[ inline ]
    #[ must_use ]
    pub fn with_health_check_interval_ms( mut self, interval_ms : u64 ) -> Self
    {
      self.health_check_interval_ms = interval_ms;
      self
    }

    /// Set failover timeout
    #[ inline ]
    #[ must_use ]
    pub fn with_failover_timeout_ms( mut self, timeout_ms : u64 ) -> Self
    {
      self.failover_timeout_ms = timeout_ms;
      self
    }

    /// Validate the configuration
    #[ inline ]
    #[ must_use ]
    pub fn is_valid( &self ) -> bool
    {
      self.max_retries > 0
      && self.retry_delay_ms > 0
      && self.max_retry_delay_ms >= self.retry_delay_ms
      && self.health_check_interval_ms > 0
      && self.failover_timeout_ms > 0
    }
  }

  /// Failover context representing the current state of a failover attempt
  #[ derive( Debug, Clone ) ]
  pub struct FailoverContext
  {
    /// Current attempt number (1-indexed)
    pub attempt : u32,
    /// Endpoint being attempted
    pub endpoint : FailoverEndpoint,
    /// Time when the attempt started
    pub started_at : Instant,
    /// Previous failed endpoints in this context
    pub failed_endpoints : Vec< String >,
  }

  impl FailoverContext
  {
    /// Create a new failover context
    #[ inline ]
    #[ must_use ]
    pub fn new( endpoint : FailoverEndpoint ) -> Self
    {
      Self
      {
        attempt : 1,
        endpoint,
        started_at : Instant::now(),
        failed_endpoints : Vec::new(),
      }
    }

    /// Create next attempt with different endpoint
    #[ inline ]
    #[ must_use ]
    pub fn next_attempt( mut self, endpoint : FailoverEndpoint ) -> Self
    {
      self.failed_endpoints.push( self.endpoint.id.clone() );
      self.attempt += 1;
      self.endpoint = endpoint;
      self.started_at = Instant::now();
      self
    }

    /// Check if maximum retries exceeded
    #[ inline ]
    #[ must_use ]
    pub fn is_exhausted( &self, max_retries : u32 ) -> bool
    {
      self.attempt > max_retries
    }

    /// Get elapsed time for current attempt
    #[ inline ]
    #[ must_use ]
    pub fn elapsed( &self ) -> Duration
    {
      self.started_at.elapsed()
    }

    /// Check if endpoint was already tried
    #[ inline ]
    #[ must_use ]
    pub fn was_endpoint_tried( &self, endpoint_id : &str ) -> bool
    {
      self.failed_endpoints.contains( &endpoint_id.to_string() ) || self.endpoint.id == endpoint_id
    }
  }

  /// Failover manager for endpoint selection and health tracking
  #[ derive( Debug ) ]
  pub struct FailoverManager
  {
    /// Configuration for failover behavior
    config : FailoverConfig,
    /// List of available endpoints
    endpoints : Vec< FailoverEndpoint >,
    /// Round-robin index for round-robin strategy
    round_robin_index : Arc< Mutex< usize > >,
  }

  impl FailoverManager
  {
    /// Create a new failover manager
    ///
    /// # Example
    ///
    /// ```ignore
    /// use api_claude::{ FailoverManager, FailoverConfig, FailoverEndpoint, FailoverStrategy };
    /// use std::time::Duration;
    ///
    /// let config = FailoverConfig::new()
    ///   .with_strategy( FailoverStrategy::Priority )
    ///   .with_max_retries( 3 );
    ///
    /// let endpoints = vec![
    ///   FailoverEndpoint::new(
    ///     "primary".to_string(),
    ///     "https://api.anthropic.com".to_string(),
    ///     100,
    ///     Duration::from_secs( 30 )
    ///   ),
    ///   FailoverEndpoint::new(
    ///     "backup".to_string(),
    ///     "https://api-backup.anthropic.com".to_string(),
    ///     50,
    ///     Duration::from_secs( 45 )
    ///   ),
    /// ];
    ///
    /// let manager = FailoverManager::new( config, endpoints );
    /// ```
    #[ inline ]
    #[ must_use ]
    pub fn new( config : FailoverConfig, endpoints : Vec< FailoverEndpoint > ) -> Self
    {
      Self
      {
        config,
        endpoints,
        round_robin_index : Arc::new( Mutex::new( 0 ) ),
      }
    }

    /// Get the failover configuration
    #[ inline ]
    #[ must_use ]
    pub fn config( &self ) -> &FailoverConfig
    {
      &self.config
    }

    /// Get all endpoints
    #[ inline ]
    #[ must_use ]
    pub fn endpoints( &self ) -> &Vec< FailoverEndpoint >
    {
      &self.endpoints
    }

    /// Update the health of a specific endpoint
    #[ inline ]
    pub fn update_endpoint_health( &mut self, endpoint_id : &str, health : EndpointHealth )
    {
      if let Some( endpoint ) = self.endpoints.iter_mut().find( | e | e.id == endpoint_id )
      {
        endpoint.update_health( health );
      }
    }

    /// Get the next endpoint according to the failover strategy
    #[ inline ]
    #[ must_use ]
    pub fn select_endpoint( &self, context : Option< &FailoverContext > ) -> Option< FailoverEndpoint >
    {
      let available_endpoints : Vec< &FailoverEndpoint > = self.endpoints
        .iter()
        .filter( | e | e.is_available() )
        .filter( | e | context.map_or( true, | ctx | !ctx.was_endpoint_tried( &e.id ) ) )
        .collect();

      if available_endpoints.is_empty()
      {
        return None;
      }

      match self.config.strategy
      {
        FailoverStrategy::RoundRobin => self.select_round_robin( &available_endpoints ),
        FailoverStrategy::Priority => Self::select_priority( &available_endpoints ),
        FailoverStrategy::Random => Self::select_random( &available_endpoints ),
        FailoverStrategy::Sticky => Self::select_sticky( &available_endpoints ),
      }
    }

    /// Select endpoint using round-robin strategy
    fn select_round_robin( &self, endpoints : &[ &FailoverEndpoint ] ) -> Option< FailoverEndpoint >
    {
      if endpoints.is_empty()
      {
        return None;
      }

      let mut index = self.round_robin_index.lock().ok()?;
      let selected = endpoints[ *index % endpoints.len() ];
      *index = ( *index + 1 ) % endpoints.len();
      Some( selected.clone() )
    }

    /// Select endpoint using priority strategy
    fn select_priority( endpoints : &[ &FailoverEndpoint ] ) -> Option< FailoverEndpoint >
    {
      endpoints
        .iter()
        .max_by_key( | e | e.priority )
        .map( | e | ( *e ).clone() )
    }

    /// Select endpoint using random strategy
    fn select_random( endpoints : &[ &FailoverEndpoint ] ) -> Option< FailoverEndpoint >
    {
      use std::collections::hash_map::RandomState;
      use std::hash::{ BuildHasher, Hash, Hasher };

      let mut hasher = RandomState::new().build_hasher();
      Instant::now().hash( &mut hasher );
      let hash = hasher.finish();
      let index = usize::try_from( hash ).unwrap_or( 0 ) % endpoints.len();
      endpoints.get( index ).map( | e | ( *e ).clone() )
    }

    /// Select endpoint using sticky strategy (first available)
    fn select_sticky( endpoints : &[ &FailoverEndpoint ] ) -> Option< FailoverEndpoint >
    {
      endpoints
        .iter()
        .find( | e | e.is_preferred() )
        .or_else( || endpoints.first() )
        .map( | e | ( *e ).clone() )
    }

    /// Get count of healthy endpoints
    #[ inline ]
    #[ must_use ]
    pub fn healthy_count( &self ) -> usize
    {
      self.endpoints.iter().filter( | e | e.is_preferred() ).count()
    }

    /// Get count of available endpoints (healthy or degraded)
    #[ inline ]
    #[ must_use ]
    pub fn available_count( &self ) -> usize
    {
      self.endpoints.iter().filter( | e | e.is_available() ).count()
    }

    /// Check if any endpoints are available
    #[ inline ]
    #[ must_use ]
    pub fn has_available_endpoints( &self ) -> bool
    {
      self.available_count() > 0
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    EndpointHealth,
    FailoverStrategy,
    FailoverEndpoint,
    FailoverConfig,
    FailoverContext,
    FailoverManager,
  };
}

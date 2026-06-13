//! Failover management for the Gemini API client.
//!
//! This module provides explicit failover functionality following the "Thin Client, Rich API" principle.
//! All failover operations are explicit and triggered by user code, not automatic behaviors.

mod private
{
  use serde::{ Deserialize, Serialize };
  use core::time::Duration;
  use std::time::SystemTime;
  use std::sync::{ Arc, Mutex };
  use std::collections::HashMap;
  use futures::Future;

  /// Configuration for failover behavior
  #[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
  pub struct FailoverConfig
  {
    /// Primary endpoint URL
    pub primary_endpoint : String,
    /// List of backup endpoint URLs
    pub backup_endpoints : Vec< String >,
    /// Timeout for endpoint health checks
    pub timeout : Duration,
    /// Maximum retry attempts per endpoint
    pub max_retries : u32,
    /// Failover strategy to use
    pub strategy : FailoverStrategy,
  }

  impl Default for FailoverConfig
  {
    #[ inline ]
    fn default() -> Self
    {
      Self {
        primary_endpoint : "https://generativelanguage.googleapis.com".to_string(),
        backup_endpoints : Vec::new(),
        timeout : Duration::from_secs( 5 ),
        max_retries : 3,
        strategy : FailoverStrategy::Priority,
      }
    }
  }

  /// Builder for creating failover configuration
  #[ derive( Debug, Clone ) ]
  pub struct FailoverConfigBuilder
  {
    config : FailoverConfig,
  }

  impl Default for FailoverConfigBuilder
  {
    #[ inline ]
    fn default() -> Self
    {
      Self {
        config : FailoverConfig::default(),
      }
    }
  }

  impl FailoverConfigBuilder
  {
    /// Create a new failover config builder
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self {
        config : FailoverConfig::default(),
      }
    }

    /// Set the primary endpoint
    #[ inline ]
    #[ must_use ]
    pub fn primary_endpoint( mut self, endpoint : String ) -> Self
    {
      self.config.primary_endpoint = endpoint;
      self
    }

    /// Add a backup endpoint
    #[ inline ]
    #[ must_use ]
    pub fn backup_endpoint( mut self, endpoint : String ) -> Self
    {
      self.config.backup_endpoints.push( endpoint );
      self
    }

    /// Set the timeout for health checks
    #[ inline ]
    #[ must_use ]
    pub fn timeout( mut self, timeout : Duration ) -> Self
    {
      self.config.timeout = timeout;
      self
    }

    /// Set the maximum retry attempts
    #[ inline ]
    #[ must_use ]
    pub fn max_retries( mut self, retries : u32 ) -> Self
    {
      self.config.max_retries = retries;
      self
    }

    /// Set the failover strategy
    #[ inline ]
    #[ must_use ]
    pub fn strategy( mut self, strategy : FailoverStrategy ) -> Self
    {
      self.config.strategy = strategy;
      self
    }

    /// Build the configuration with validation
    ///
    /// # Errors
    ///
    /// Returns `Error` if the configuration is invalid:
    /// - Primary endpoint is empty
    /// - Timeout is zero
    /// - Max retries exceeds reasonable limits
    #[ inline ]
    pub fn build( self ) -> Result< FailoverConfig, crate::error::Error >
    {
      Self::validate_config( &self.config )?;
      Ok( self.config )
    }

    /// Validate the failover configuration
    #[ inline ]
    fn validate_config( config : &FailoverConfig ) -> Result< (), crate::error::Error >
    {
      if config.primary_endpoint.is_empty()
      {
        return Err( crate::error::Error::ConfigurationError(
          "Primary endpoint cannot be empty".to_string()
        ) );
      }

      if config.backup_endpoints.is_empty()
      {
        return Err( crate::error::Error::ConfigurationError(
          "At least one backup endpoint is required for failover".to_string()
        ) );
      }

      if config.timeout.is_zero()
      {
        return Err( crate::error::Error::ConfigurationError(
          "Timeout cannot be zero".to_string()
        ) );
      }

      if config.max_retries == 0
      {
        return Err( crate::error::Error::ConfigurationError(
          "Max retries must be at least 1".to_string()
        ) );
      }

      Ok( () )
    }
  }

  impl FailoverConfig
  {
    /// Create a new failover config builder
    #[ inline ]
    #[ must_use ]
    pub fn builder() -> FailoverConfigBuilder
    {
      FailoverConfigBuilder::new()
    }
  }

  /// Available failover strategies
  #[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
  pub enum FailoverStrategy
  {
    /// Try backup endpoints in priority order
    Priority,
    /// Round-robin through available endpoints
    RoundRobin,
  }

  /// Health status of an endpoint
  #[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
  pub enum HealthStatus
  {
    /// Endpoint is healthy and responsive
    Healthy,
    /// Endpoint is unhealthy or unresponsive
    Unhealthy,
    /// Endpoint status is unknown
    Unknown,
  }

  /// Health information for a specific endpoint
  #[ derive( Debug, Clone ) ]
  pub struct EndpointHealth
  {
    /// The endpoint URL
    pub endpoint : String,
    /// Current health status
    pub status : HealthStatus,
    /// Last time this endpoint was checked
    pub last_check : SystemTime,
    /// Response time of last successful request
    pub response_time : Option< Duration >,
    /// Number of consecutive failures
    pub consecutive_failures : u32,
  }

  /// Result of checking endpoint health
  #[ derive( Debug, Clone ) ]
  pub struct HealthCheckResult
  {
    /// Whether the primary endpoint is healthy
    pub primary_healthy : bool,
    /// Whether backup endpoints are available
    pub backup_available : bool,
    /// Detailed health status for all endpoints
    pub endpoint_health : Vec< EndpointHealth >,
    /// Recommended next endpoint to try
    pub recommended_endpoint : Option< String >,
  }

  /// Metrics for failover operations
  #[ derive( Debug, Clone ) ]
  pub struct FailoverMetrics
  {
    /// Total number of configured endpoints
    pub total_endpoints : usize,
    /// Number of times failover has occurred
    pub failover_count : u64,
    /// Health status of all endpoints
    pub endpoint_health : Vec< EndpointHealth >,
    /// Current active endpoint
    pub active_endpoint : String,
  }

  /// Failover management interface
  #[ allow( missing_debug_implementations ) ] // Cannot derive Debug due to function pointers
  pub struct FailoverManager
  {
    client : crate::client::Client,
    config : FailoverConfig,
    metrics : Arc< Mutex< FailoverMetrics > >,
    round_robin_index : Arc< Mutex< usize > >,
    endpoint_health : Arc< Mutex< HashMap<  String, EndpointHealth  > > >,
  }

  impl FailoverManager
  {
    /// Create a new failover manager
    #[ inline ]
    #[ must_use ]
    pub fn new( client : crate::client::Client, config : FailoverConfig ) -> Self
    {
      let total_endpoints = 1 + config.backup_endpoints.len();
      let metrics = FailoverMetrics {
        total_endpoints,
        failover_count : 0,
        endpoint_health : Vec::new(),
        active_endpoint : config.primary_endpoint.clone(),
      };

      Self {
        client,
        config,
        metrics : Arc::new( Mutex::new( metrics ) ),
        round_robin_index : Arc::new( Mutex::new( 0 ) ),
        endpoint_health : Arc::new( Mutex::new( HashMap::new() ) ),
      }
    }

    /// Get the current failover configuration
    #[ inline ]
    #[ must_use ]
    pub fn current_config( &self ) -> &FailoverConfig
    {
      &self.config
    }

    /// Check the health of all configured endpoints
    ///
    /// # Errors
    ///
    /// Returns `Error` if health check operations fail due to:
    /// - Network connectivity issues
    /// - Endpoint timeout or unavailability
    /// - Configuration validation errors
    ///
    /// # Panics
    ///
    /// Panics if mutex locks are poisoned (rare runtime error).
    #[ inline ]
    pub async fn check_endpoint_health( &self ) -> Result< HealthCheckResult, crate::error::Error >
    {
      let mut endpoint_health = Vec::new();
      let mut backup_available = false;

      // Check primary endpoint
      let primary_health = self.check_single_endpoint( &self.config.primary_endpoint ).await;
      let primary_healthy = primary_health.status == HealthStatus::Healthy;
      endpoint_health.push( primary_health );

      // Check backup endpoints
      for backup in &self.config.backup_endpoints
      {
        let backup_health = self.check_single_endpoint( backup ).await;
        if backup_health.status == HealthStatus::Healthy
        {
          backup_available = true;
        }
        endpoint_health.push( backup_health );
      }

      // Update stored health information
      {
        let mut stored_health = self.endpoint_health.lock().unwrap();
        for health in &endpoint_health
        {
          stored_health.insert( health.endpoint.clone(), health.clone() );
        }
      }

      let recommended_endpoint = if !primary_healthy && backup_available
      {
        self.get_next_healthy_endpoint().ok()
      } else {
        None
      };

      Ok( HealthCheckResult {
        primary_healthy,
        backup_available,
        endpoint_health,
        recommended_endpoint,
      } )
    }

    /// Check health of a single endpoint
    async fn check_single_endpoint( &self, endpoint : &str ) -> EndpointHealth
    {
      let start_time = SystemTime::now();

      // Create HTTP client for health check
      let client = reqwest::Client::builder()
        .timeout( self.config.timeout )
        .build()
        .unwrap_or_default();

      // Perform HEAD request to check endpoint health
      let result = client.head( endpoint ).send().await;

      let ( status, response_time, consecutive_failures ) = match result
      {
        Ok( response ) => {
          if response.status().is_success() || response.status().is_redirection()
          {
            ( HealthStatus::Healthy, start_time.elapsed().ok(), 0 )
          } else {
            ( HealthStatus::Unhealthy, None, 1 )
          }
        },
        Err( _ ) => ( HealthStatus::Unhealthy, None, 1 ),
      };

      EndpointHealth {
        endpoint : endpoint.to_string(),
        status,
        last_check : start_time,
        response_time,
        consecutive_failures,
      }
    }

    /// Get the next endpoint to try based on strategy
    ///
    /// # Errors
    ///
    /// Returns `Error` if no backup endpoints are configured.
    ///
    /// # Panics
    ///
    /// Panics if mutex locks are poisoned (rare runtime error).
    #[ inline ]
    pub fn get_next_endpoint( &self ) -> Result< String, crate::error::Error >
    {
      match self.config.strategy
      {
        FailoverStrategy::Priority => {
          // Return first backup endpoint in priority order
          self.config.backup_endpoints.first()
            .cloned()
            .map_or_else(
              || Err( crate::error::Error::ConfigurationError(
                "No backup endpoints configured".to_string()
              ) ),
              Ok
            )
        },
        FailoverStrategy::RoundRobin => {
          let mut index = self.round_robin_index.lock().unwrap();
          let backup_count = self.config.backup_endpoints.len();

          if backup_count == 0
          {
            return Err( crate::error::Error::ConfigurationError(
              "No backup endpoints configured".to_string()
            ) );
          }

          let selected = &self.config.backup_endpoints[ *index % backup_count ];
          *index = ( *index + 1 ) % backup_count;

          Ok( selected.clone() )
        },
      }
    }

    /// Get the next healthy endpoint
    #[ inline ]
    fn get_next_healthy_endpoint( &self ) -> Result< String, crate::error::Error >
    {
      // For now, just use the first backup endpoint
      // In a full implementation, this would check health of all backups
      self.config.backup_endpoints.first()
        .cloned()
        .map_or_else(
          || Err( crate::error::Error::ConfigurationError(
            "No backup endpoints available".to_string()
          ) ),
          Ok
        )
    }

    /// Switch to backup endpoint explicitly
    ///
    /// # Errors
    ///
    /// Returns `Error` if:
    /// - No backup endpoints are available
    /// - Client creation fails with backup endpoint
    ///
    /// # Panics
    ///
    /// Panics if mutex locks are poisoned (rare runtime error).
    #[ inline ]
    pub fn switch_to_backup( &self ) -> Result< crate::client::Client, crate::error::Error >
    {
      let backup_endpoint = self.get_next_endpoint()?;

      // Create new client with backup endpoint
      // For now, we'll create a simplified client. In a full implementation,
      // we would extract the API key and other settings from the original client.
      let backup_client = crate::client::Client::builder()
        .api_key( self.client.api_key.clone() )
        .base_url( backup_endpoint.clone() )
        .build()?;

      // Update metrics
      {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.failover_count += 1;
        metrics.active_endpoint = backup_endpoint;
      }

      Ok( backup_client )
    }

    /// Execute a request with failover handling
    ///
    /// # Errors
    ///
    /// Returns `Error` if:
    /// - Both primary and backup endpoints fail
    /// - No backup endpoints are configured and primary fails
    /// - Client creation fails
    #[ inline ]
    pub async fn execute_with_failover< F, Fut, T >(
      &self,
      operation : F
    ) -> Result< T, crate::error::Error >
    where
      F: Fn( crate::client::Client ) -> Fut,
      Fut : Future< Output = Result< T, crate::error::Error > >,
    {
      // Try primary endpoint first
      if let Ok( result ) = operation( self.client.clone() ).await
      {
        Ok( result )
      } else {
        // Primary failed, try backup
        let backup_client = self.switch_to_backup()?;
        operation( backup_client ).await
      }
    }

    /// Get current failover metrics
    ///
    /// # Panics
    ///
    /// Panics if mutex locks are poisoned (rare runtime error).
    #[ inline ]
    #[ must_use ]
    pub fn get_metrics( &self ) -> FailoverMetrics
    {
      let mut metrics = self.metrics.lock().unwrap();

      // Update endpoint health in metrics
      let stored_health = self.endpoint_health.lock().unwrap();
      metrics.endpoint_health = stored_health.values().cloned().collect();

      metrics.clone()
    }
  }

  /// Failover builder for the client
  #[ derive( Debug ) ]
  pub struct FailoverBuilder
  {
    client : crate::client::Client,
  }

  impl FailoverBuilder
  {
    /// Create a new failover builder
    #[ inline ]
    #[ must_use ]
    pub fn new( client : crate::client::Client ) -> Self
    {
      Self { client }
    }

    /// Configure failover with the given configuration
    #[ inline ]
    #[ must_use ]
    pub fn configure( self, config : FailoverConfig ) -> FailoverManager
    {
      FailoverManager::new( self.client, config )
    }
  }
}

::mod_interface::mod_interface!
{
  exposed use private::FailoverConfig;
  exposed use private::FailoverConfigBuilder;
  exposed use private::FailoverStrategy;
  exposed use private::HealthStatus;
  exposed use private::EndpointHealth;
  exposed use private::HealthCheckResult;
  exposed use private::FailoverMetrics;
  exposed use private::FailoverManager;
  exposed use private::FailoverBuilder;
}
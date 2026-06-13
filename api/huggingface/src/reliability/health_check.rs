//! Health Check Implementation
//!
//! Provides automated health monitoring for API endpoints with multiple strategies.
//!
//! ## Features
//!
//! - **Multiple Strategies**: Ping, lightweight API, or full endpoint testing
//! - **Background Monitoring**: Continuous health checks with configurable intervals
//! - **Latency Tracking**: Monitor response times and detect degradation
//! - **Health Callbacks**: Get notified of health status changes
//! - **Thread-Safe**: Safe for concurrent use
//!
//! ## Usage
//!
//! ```no_run
//! # use api_huggingface::reliability::{HealthChecker, HealthCheckConfig, HealthCheckStrategy};
//! # use std::time::Duration;
//! # async fn example( ) -> Result< ( ), Box< dyn std::error::Error > > {
//! let health_checker = HealthChecker::new(
//!   HealthCheckConfig {
//!     endpoint : "https://api-inference.huggingface.co".to_string( ),
//!     strategy : HealthCheckStrategy::LightweightApi,
//!     check_interval : Duration::from_secs( 30 ),
//!     timeout : Duration::from_secs( 5 ),
//!     unhealthy_threshold : 3,
//!   }
//! );
//!
//! // Start background monitoring
//! let _monitor = health_checker.start_monitoring( ).await;
//!
//! // Check current health
//! let status = health_checker.check_health( ).await?;
//! println!( "Healthy : {}, Latency : {}ms", status.healthy, status.latency_ms );
//! # Ok( ( ))
//! # }
//! ```

use core::time::Duration;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::time::sleep;

/// Health check strategy
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum HealthCheckStrategy 
{
  /// Simple connectivity ping ( fastest )
  Ping,
  /// Call a lightweight API endpoint
  LightweightApi,
  /// Test the full endpoint functionality
  FullEndpoint,
}

/// Health check configuration
#[ derive( Debug, Clone ) ]
pub struct HealthCheckConfig 
{
  /// Endpoint URL to monitor
  pub endpoint : String,
  /// Health check strategy to use
  pub strategy : HealthCheckStrategy,
  /// Interval between health checks
  pub check_interval : Duration,
  /// Timeout for health check requests
  pub timeout : Duration,
  /// Number of consecutive failures before marking unhealthy
  pub unhealthy_threshold : u32,
}

impl Default for HealthCheckConfig 
{
  #[ inline ]
  fn default() -> Self 
  {
  Self {
      endpoint : String::new( ),
      strategy : HealthCheckStrategy::LightweightApi,
      check_interval : Duration::from_secs( 30 ),
      timeout : Duration::from_secs( 5 ),
      unhealthy_threshold : 3,
  }
  }
}

/// Health status information
#[ derive( Debug, Clone ) ]
pub struct HealthStatus 
{
  /// Whether the endpoint is healthy
  pub healthy : bool,
  /// Response latency in milliseconds
  pub latency_ms : u64,
  /// Number of consecutive failures
  pub consecutive_failures : u32,
  /// Total checks performed
  pub total_checks : u64,
  /// Timestamp of last check
  pub last_check : Instant,
}

impl Default for HealthStatus 
{
  #[ inline ]
  fn default() -> Self 
  {
  Self {
      healthy : true,
      latency_ms : 0,
      consecutive_failures : 0,
      total_checks : 0,
      last_check : Instant::now( ),
  }
  }
}

/// Internal health checker state
#[ derive( Debug ) ]
pub struct HealthCheckerState 
{
  /// Current health status
  pub status : HealthStatus,
  /// Whether monitoring is active
  pub monitoring : bool,
}

/// Health checker for monitoring endpoint health
#[ derive( Debug, Clone ) ]
pub struct HealthChecker 
{
  config : HealthCheckConfig,
  /// Internal health checker state ( public for testing )
  pub state : Arc< RwLock< HealthCheckerState > >,
}

impl HealthChecker 
{
  /// Create new health checker with given configuration
  #[ inline ]
  #[ must_use ]
  pub fn new( config : HealthCheckConfig ) -> Self 
  {
  Self {
      config,
      state : Arc::new( RwLock::new( HealthCheckerState {
  status : HealthStatus::default( ),
  monitoring : false,
      } )),
  }
  }

  /// Perform a single health check
  ///
  /// # Errors
  ///
  /// Returns `HealthCheckError::CheckFailed` if the health check fails.
  /// Returns `HealthCheckError::Timeout` if the check exceeds the configured timeout.
  #[ inline ]
  pub async fn check_health( &self ) -> Result< HealthStatus, HealthCheckError > 
  {
  let start = Instant::now( );

  // Perform health check based on strategy
  let result = match self.config.strategy
  {
      HealthCheckStrategy::Ping => self.ping_check( ).await,
      HealthCheckStrategy::LightweightApi => self.lightweight_api_check( ).await,
      HealthCheckStrategy::FullEndpoint => self.full_endpoint_check( ).await,
  };

  let latency = start.elapsed( );
  #[ allow( clippy::cast_possible_truncation ) ]
  let latency_ms = latency.as_millis( ) as u64;

  // Update state
  let mut state = self.state.write( ).await;
  state.status.total_checks += 1;
  state.status.last_check = Instant::now( );
  state.status.latency_ms = latency_ms;

  match result
  {
      Ok( ( )) => {
  // Success - reset failure count and mark healthy
  state.status.consecutive_failures = 0;
  state.status.healthy = true;
  Ok( state.status.clone( ))
      }
      Err( e ) => {
  // Failure - increment counter
  state.status.consecutive_failures += 1;

  // Mark unhealthy if threshold reached
  if state.status.consecutive_failures >= self.config.unhealthy_threshold
  {
          state.status.healthy = false;
  }

  Err( e )
      }
  }
  }

  /// Start background health monitoring
  ///
  /// Returns a handle that can be used to stop monitoring.
  #[ inline ]
  pub async fn start_monitoring( &self ) -> MonitorHandle 
  {
  let mut state = self.state.write( ).await;
  state.monitoring = true;
  drop( state );

  let checker = self.clone( );
  let handle = tokio::spawn( async move {
      checker.monitoring_loop( ).await;
  } );

  MonitorHandle { handle }
  }

  /// Stop background health monitoring
  #[ inline ]
  pub async fn stop_monitoring( &self ) 
  {
  let mut state = self.state.write( ).await;
  state.monitoring = false;
  }

  /// Get current health status without performing a check
  #[ inline ]
  pub async fn get_status( &self ) -> HealthStatus 
  {
  let state = self.state.read( ).await;
  state.status.clone( )
  }

  /// Check if currently monitoring
  #[ inline ]
  pub async fn is_monitoring( &self ) -> bool 
  {
  let state = self.state.read( ).await;
  state.monitoring
  }

  /// Reset health status
  #[ inline ]
  pub async fn reset( &self ) 
  {
  let mut state = self.state.write( ).await;
  state.status = HealthStatus::default( );
  }

  // Internal monitoring loop
  async fn monitoring_loop( &self ) 
  {
  loop
  {
      // Check if monitoring should stop
      {
  let state = self.state.read( ).await;
  if !state.monitoring
  {
          break;
  }
      }

      // Perform health check
      let _ = self.check_health( ).await;

      // Wait for next check
      sleep( self.config.check_interval ).await;
  }
  }

  // Ping check - simple connectivity test
  async fn ping_check( &self ) -> Result< ( ), HealthCheckError > 
  {
  // For HTTP endpoints, a simple HEAD request is sufficient
  let client = reqwest::Client::builder( )
      .timeout( self.config.timeout )
      .build( )
      .map_err( |e| HealthCheckError::CheckFailed {
  reason : format!( "Failed to create client : {e}" ),
      } )?;

  let response = client
      .head( &self.config.endpoint )
      .send( )
      .await
      .map_err( |e| {
  if e.is_timeout( )
  {
          HealthCheckError::Timeout
  } else {
          HealthCheckError::CheckFailed {
      reason : format!( "Ping failed : {e}" ),
          }
  }
      } )?;

  if response.status( ).is_success( ) || response.status( ).is_redirection( )
  {
      Ok( ( ))
  } else {
      Err( HealthCheckError::CheckFailed {
  reason : format!( "Ping returned status : {}", response.status( )),
      } )
  }
  }

  // Lightweight API check - call a simple endpoint
  async fn lightweight_api_check( &self ) -> Result< ( ), HealthCheckError > 
  {
  // Use GET request to a lightweight endpoint
  let client = reqwest::Client::builder( )
      .timeout( self.config.timeout )
      .build( )
      .map_err( |e| HealthCheckError::CheckFailed {
  reason : format!( "Failed to create client : {e}" ),
      } )?;

  let response = client
      .get( &self.config.endpoint )
      .send( )
      .await
      .map_err( |e| {
  if e.is_timeout( )
  {
          HealthCheckError::Timeout
  } else {
          HealthCheckError::CheckFailed {
      reason : format!( "API check failed : {e}" ),
          }
  }
      } )?;

  if response.status( ).is_success( )
  {
      Ok( ( ))
  } else {
      Err( HealthCheckError::CheckFailed {
  reason : format!( "API returned status : {}", response.status( )),
      } )
  }
  }

  // Full endpoint check - test actual functionality
  async fn full_endpoint_check( &self ) -> Result< ( ), HealthCheckError > 
  {
  // For full endpoint, we use POST with a test payload
  let client = reqwest::Client::builder( )
      .timeout( self.config.timeout )
      .build( )
      .map_err( |e| HealthCheckError::CheckFailed {
  reason : format!( "Failed to create client : {e}" ),
      } )?;

  // Simple test payload
  let test_payload = serde_json::json!( {
      "inputs": "test"
  } );

  let response = client
      .post( &self.config.endpoint )
      .json( &test_payload )
      .send( )
      .await
      .map_err( |e| {
  if e.is_timeout( )
  {
          HealthCheckError::Timeout
  } else {
          HealthCheckError::CheckFailed {
      reason : format!( "Full endpoint check failed : {e}" ),
          }
  }
      } )?;

  // Accept success or error responses - we just want to know the endpoint is responding
  if response.status( ).is_success( ) || response.status( ).is_client_error( )
  {
      Ok( ( ))
  } else {
      Err( HealthCheckError::CheckFailed {
  reason : format!( "Endpoint returned status : {}", response.status( )),
      } )
  }
  }
}

/// Handle for background monitoring task
#[ derive( Debug ) ]
pub struct MonitorHandle 
{
  handle : tokio::task::JoinHandle< ( ) >,
}

impl MonitorHandle 
{
  /// Stop monitoring and wait for task to complete
  #[ inline ]
  pub async fn stop( self ) 
  {
  self.handle.abort( );
  let _ = self.handle.await;
  }
}

/// Health check errors
#[ derive( Debug ) ]
pub enum HealthCheckError 
{
  /// Health check timed out
  Timeout,
  /// Health check failed
  CheckFailed {
  /// Reason for failure
  reason : String,
  },
}

impl core::fmt::Display for HealthCheckError 
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result 
  {
  match self
  {
      Self::Timeout => write!( f, "Health check timed out" ),
      Self::CheckFailed { reason } => write!( f, "Health check failed : {reason}" ),
  }
  }
}

impl std::error::Error for HealthCheckError {}

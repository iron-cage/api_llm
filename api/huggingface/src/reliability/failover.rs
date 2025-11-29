//! Failover Implementation
//!
//! Provides automatic failover to backup endpoints when primary endpoints fail.
//!
//! ## Strategies
//!
//! - **Priority**: Try endpoints in order until one succeeds
//! - **`RoundRobin`**: Distribute requests evenly across endpoints
//! - **Random**: Select random endpoint for each request
//! - **Sticky**: Stick to one endpoint until it fails
//!
//! ## Usage
//!
//! ```no_run
//! # use api_huggingface::reliability::{FailoverManager, FailoverConfig, FailoverStrategy};
//! # use std::time::Duration;
//! # async fn example( ) -> Result< ( ), Box< dyn std::error::Error > > {
//! let failover = FailoverManager::new(
//!   FailoverConfig {
//!     endpoints : vec![
//!       "https://api-inference.huggingface.co".to_string( ),
//!       "https://api-inference-backup.huggingface.co".to_string( ),
//!     ],
//!     strategy : FailoverStrategy::Priority,
//!     max_retries : 3,
//!     failure_window : Duration::from_secs( 300 ),
//!     failure_threshold : 5,
//!   }
//! ).map_err( |e| format!( "{:?}", e ))?;
//!
//! let _endpoint = failover.select_endpoint( ).await.map_err( |e| format!( "{:?}", e ))?;
//! # Ok( ( ))
//! # }
//! ```

use std::sync::Arc;
use std::time::Instant;
use core::time::Duration;
use tokio::sync::RwLock;
use rand::Rng;

/// Failover strategy
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum FailoverStrategy 
{
  /// Try endpoints in priority order
  Priority,
  /// Distribute requests evenly across endpoints
  RoundRobin,
  /// Select random endpoint for each request
  Random,
  /// Stick to one endpoint until it fails
  Sticky,
}

/// Failover configuration
#[ derive( Debug, Clone ) ]
pub struct FailoverConfig 
{
  /// List of endpoint URLs in priority order
  pub endpoints : Vec< String >,
  /// Failover strategy to use
  pub strategy : FailoverStrategy,
  /// Maximum retry attempts per request
  pub max_retries : u32,
  /// Time window for tracking endpoint failures
  pub failure_window : Duration,
  /// Number of failures before marking endpoint unhealthy
  pub failure_threshold : u32,
}

impl Default for FailoverConfig 
{
  #[ inline ]
  fn default() -> Self 
  {
  Self {
      endpoints : vec![ ],
      strategy : FailoverStrategy::Priority,
      max_retries : 3,
      failure_window : Duration::from_secs( 60 ),
      failure_threshold : 5,
  }
  }
}

/// Endpoint health status
#[ derive( Debug, Clone ) ]
struct EndpointHealth 
{
  /// Endpoint URL
  url : String,
  /// Recent failure timestamps
  failures : Vec< Instant >,
  /// Total requests sent to this endpoint
  requests : u64,
  /// Total successful requests
  successes : u64,
  /// Last successful request time
  last_success : Option< Instant >,
  /// Is endpoint currently healthy
  healthy : bool,
}

impl EndpointHealth 
{
  /// Create new endpoint health tracker
  #[ inline ]
  fn new( url : String ) -> Self 
  {
  Self {
      url,
      failures : Vec::new( ),
      requests : 0,
      successes : 0,
      last_success : None,
      healthy : true,
  }
  }

  /// Record a successful request
  #[ inline ]
  fn record_success( &mut self ) 
  {
  self.requests += 1;
  self.successes += 1;
  self.last_success = Some( Instant::now( ));
  self.healthy = true;
  }

  /// Record a failed request
  #[ inline ]
  fn record_failure( &mut self, failure_window : Duration, failure_threshold : u32 ) 
  {
  self.requests += 1;
  let now = Instant::now( );
  self.failures.push( now );

  // Remove failures outside the window
  self.failures.retain( |&t| now.duration_since( t ) < failure_window );

  // Update health status
  if self.failures.len( ) >= failure_threshold as usize
  {
      self.healthy = false;
  }
  }

  /// Get failure count in current window
  #[ inline ]
  fn failure_count( &mut self, failure_window : Duration ) -> usize 
  {
  let now = Instant::now( );
  self.failures.retain( |&t| now.duration_since( t ) < failure_window );
  self.failures.len( )
  }

  /// Get success rate
  #[ inline ]
  fn success_rate( &self ) -> f64
  {
  if self.requests == 0
  {
      1.0
  } else {
      self.successes as f64 / self.requests as f64
  }
  }
}

/// Failover manager state
#[ derive( Debug ) ]
struct FailoverState 
{
  /// Endpoint health tracking
  endpoints : Vec< EndpointHealth >,
  /// Current index for round-robin
  round_robin_index : usize,
  /// Current sticky endpoint index
  sticky_index : Option< usize >,
}

/// Failover manager for multi-endpoint failover
#[ derive( Debug, Clone ) ]
pub struct FailoverManager 
{
  config : FailoverConfig,
  state : Arc< RwLock< FailoverState > >,
}

impl FailoverManager 
{
  /// Create new failover manager with given configuration
  ///
  /// # Errors
  ///
  /// Returns `FailoverError::NoEndpoints` if no endpoints are configured.
  #[ inline ]
  pub fn new( config : FailoverConfig ) -> Result< Self, FailoverError > 
  {
  if config.endpoints.is_empty( )
  {
      return Err( FailoverError::NoEndpoints );
  }

  let endpoints = config.endpoints.iter( )
      .map( |url| EndpointHealth::new( url.clone( )) )
      .collect( );

  Ok( Self {
      config,
      state : Arc::new( RwLock::new( FailoverState {
  endpoints,
  round_robin_index : 0,
  sticky_index : None,
      } )),
  } )
  }

  /// Select next endpoint based on strategy
  ///
  /// # Errors
  ///
  /// Returns `FailoverError::AllEndpointsUnhealthy` if no healthy endpoints available.
  #[ inline ]
  pub async fn select_endpoint( &self ) -> Result< String, FailoverError > 
  {
  let mut state = self.state.write( ).await;

  match self.config.strategy
  {
      FailoverStrategy::Priority => {
  // Try endpoints in order, prefer healthy ones
  for endpoint in &state.endpoints
  {
          if endpoint.healthy
          {
      return Ok( endpoint.url.clone( ));
          }
  }
  // If all unhealthy, try first one anyway
  state.endpoints.first( )
          .map( |e| e.url.clone( ))
          .ok_or( FailoverError::NoEndpoints )
      }

      FailoverStrategy::RoundRobin => {
  let healthy_count = state.endpoints.iter( ).filter( |e| e.healthy ).count( );
  if healthy_count == 0
  {
          return Err( FailoverError::AllEndpointsUnhealthy );
  }

  // Find next healthy endpoint
  let start_index = state.round_robin_index;
  loop
  {
          let current_index = state.round_robin_index;
          let endpoint_url = state.endpoints[current_index ].url.clone( );
          let endpoint_healthy = state.endpoints[current_index ].healthy;

          state.round_robin_index = ( state.round_robin_index + 1 ) % state.endpoints.len( );

          if endpoint_healthy
          {
      return Ok( endpoint_url );
          }

          // Prevent infinite loop
          if state.round_robin_index == start_index
          {
      return Err( FailoverError::AllEndpointsUnhealthy );
          }
  }
      }

      FailoverStrategy::Random => {
  let healthy : Vec< _ > = state.endpoints.iter( )
          .filter( |e| e.healthy )
          .collect( );

  if healthy.is_empty( )
  {
          return Err( FailoverError::AllEndpointsUnhealthy );
  }

  let mut rng = rand::rng( );
  let index = rng.random_range( 0..healthy.len( ));
  Ok( healthy[index ].url.clone( ))
      }

      FailoverStrategy::Sticky => {
  // If no sticky endpoint or it's unhealthy, select new one
  if let Some( idx ) = state.sticky_index
  {
          if idx < state.endpoints.len( ) && state.endpoints[idx ].healthy
          {
      return Ok( state.endpoints[idx ].url.clone( ));
          }
  }

  // Select first healthy endpoint
  for i in 0..state.endpoints.len( )
  {
          if state.endpoints[i ].healthy
          {
      let url = state.endpoints[i ].url.clone( );
      state.sticky_index = Some( i );
      return Ok( url );
          }
  }

  Err( FailoverError::AllEndpointsUnhealthy )
      }
  }
  }

  /// Execute operation with failover retry logic
  ///
  /// Tries the operation with different endpoints until success or max retries reached.
  ///
  /// # Errors
  ///
  /// Returns `FailoverError::AllRetriesFailed` if all retry attempts fail.
  /// Returns `FailoverError::Operation` wrapping the underlying error if operation fails.
  #[ inline ]
  pub async fn execute_with_failover< F, T, E >( &self, mut f : F ) -> Result< T, FailoverError< E > >
  where
  F : FnMut( String ) -> core::pin::Pin< Box< dyn core::future::Future< Output = Result< T, E > > + Send > >,
  E: core::fmt::Display,
  {
  let mut attempts = 0;
  let mut last_error = None;

  while attempts <= self.config.max_retries
  {
      attempts += 1;

      let endpoint = match self.select_endpoint( ).await
      {
  Ok( ep ) => ep,
  Err( e ) => return Err( FailoverError::SelectionFailed( e.to_string( )) ),
      };

      let endpoint_clone = endpoint.clone( );
      match f( endpoint.clone( )).await
      {
  Ok( result ) => {
          self.record_success( &endpoint_clone ).await;
          return Ok( result );
  }
  Err( e ) => {
          self.record_failure( &endpoint_clone ).await;
          last_error = Some( e );

          // Exponential backoff before retry: 500ms, 1s, 2s, 4s (capped at 5s)
          // This gives the API time to recover from rate limiting or transient issues
          if attempts <= self.config.max_retries
          {
            let delay_ms = 500 * 2u64.pow( attempts - 1 );
            tokio::time::sleep( Duration::from_millis( delay_ms.min( 5000 ) ) ).await;
          }
  }
      }
  }

  if let Some( err ) = last_error
  {
      Err( FailoverError::AllRetriesFailed {
  attempts,
  last_error : err.to_string( ),
      } )
  } else {
      Err( FailoverError::AllEndpointsUnhealthy )
  }
  }

  /// Record successful request for endpoint
  #[ inline ]
  pub async fn record_success( &self, endpoint : &str ) 
  {
  let mut state = self.state.write( ).await;
  if let Some( ep ) = state.endpoints.iter_mut( ).find( |e| e.url == endpoint )
  {
      ep.record_success( );
  }
  }

  /// Record failed request for endpoint
  #[ inline ]
  pub async fn record_failure( &self, endpoint : &str ) 
  {
  let mut state = self.state.write( ).await;
  if let Some( ep ) = state.endpoints.iter_mut( ).find( |e| e.url == endpoint )
  {
      ep.record_failure( self.config.failure_window, self.config.failure_threshold );
  }
  }

  /// Get health status of all endpoints
  #[ inline ]
  pub async fn health_status( &self ) -> Vec< EndpointHealthStatus > 
  {
  let mut state = self.state.write( ).await;
  state.endpoints.iter_mut( ).map( |ep| {
      EndpointHealthStatus {
  url : ep.url.clone( ),
  healthy : ep.healthy,
  requests : ep.requests,
  successes : ep.successes,
  success_rate : ep.success_rate( ),
  failure_count : ep.failure_count( self.config.failure_window ),
      }
  } ).collect( )
  }

  /// Reset all endpoint health status
  #[ inline ]
  pub async fn reset( &self ) 
  {
  let mut state = self.state.write( ).await;
  for endpoint in &mut state.endpoints
  {
      endpoint.failures.clear( );
      endpoint.requests = 0;
      endpoint.successes = 0;
      endpoint.last_success = None;
      endpoint.healthy = true;
  }
  state.round_robin_index = 0;
  state.sticky_index = None;
  }
}

/// Endpoint health status ( public view )
#[ derive( Debug, Clone, PartialEq ) ]
pub struct EndpointHealthStatus 
{
  /// Endpoint URL
  pub url : String,
  /// Is endpoint healthy
  pub healthy : bool,
  /// Total requests
  pub requests : u64,
  /// Successful requests
  pub successes : u64,
  /// Success rate ( 0.0 - 1.0 )
  pub success_rate : f64,
  /// Current failure count in window
  pub failure_count : usize,
}

/// Failover errors
#[ derive( Debug ) ]
pub enum FailoverError< E = String > 
{
  /// No endpoints configured
  NoEndpoints,
  /// All endpoints are unhealthy
  AllEndpointsUnhealthy,
  /// Endpoint selection failed
  SelectionFailed( String ),
  /// All retry attempts failed
  AllRetriesFailed {
  /// Number of attempts made
  attempts : u32,
  /// Last error message
  last_error : String,
  },
  /// Operation failed
  Operation( E ),
}

impl< E > core::fmt::Display for FailoverError< E >
where
  E: core::fmt::Display,
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result 
  {
  match self
  {
      Self::NoEndpoints => write!( f, "No endpoints configured" ),
      Self::AllEndpointsUnhealthy => write!( f, "All endpoints are unhealthy" ),
      Self::SelectionFailed( msg ) => write!( f, "Endpoint selection failed : {msg}" ),
      Self::AllRetriesFailed { attempts, last_error } => {
  write!( f, "All {attempts} retry attempts failed, last error : {last_error}" )
      }
      Self::Operation( e ) => write!( f, "Operation failed : {e}" ),
  }
  }
}

impl< E > std::error::Error for FailoverError< E >
where
  E: std::error::Error + 'static,
{
  #[ inline ]
  fn source( &self ) -> Option< &( dyn std::error::Error + 'static ) > 
  {
  match self
  {
      Self::Operation( e ) => Some( e ),
      _ => None,
  }
  }
}

#[ cfg( test ) ]
mod tests {
  use super::*;

  #[ tokio::test ]
  async fn test_failover_priority_strategy() 
  {
  let config = FailoverConfig {
      endpoints : vec![
  "https://primary.example.com".to_string( ),
  "https://backup.example.com".to_string( ),
      ],
      strategy : FailoverStrategy::Priority,
      ..Default::default( )
  };

  let failover = FailoverManager::new( config ).unwrap( );

  // Should select first endpoint
  let endpoint = failover.select_endpoint( ).await.unwrap( );
  assert_eq!( endpoint, "https://primary.example.com" );

  // Mark first as unhealthy
  for _ in 0..5
  {
      failover.record_failure( "https://primary.example.com" ).await;
  }

  // Should select second endpoint
  let endpoint = failover.select_endpoint( ).await.unwrap( );
  assert_eq!( endpoint, "https://backup.example.com" );
  }

  #[ tokio::test ]
  async fn test_failover_round_robin_strategy() 
  {
  let config = FailoverConfig {
      endpoints : vec![
  "https://server1.example.com".to_string( ),
  "https://server2.example.com".to_string( ),
  "https://server3.example.com".to_string( ),
      ],
      strategy : FailoverStrategy::RoundRobin,
      ..Default::default( )
  };

  let failover = FailoverManager::new( config ).unwrap( );

  // Should cycle through endpoints
  let e1 = failover.select_endpoint( ).await.unwrap( );
  let e2 = failover.select_endpoint( ).await.unwrap( );
  let e3 = failover.select_endpoint( ).await.unwrap( );
  let e4 = failover.select_endpoint( ).await.unwrap( );

  assert_eq!( e1, "https://server1.example.com" );
  assert_eq!( e2, "https://server2.example.com" );
  assert_eq!( e3, "https://server3.example.com" );
  assert_eq!( e4, "https://server1.example.com" ); // Wraps around
  }

  #[ tokio::test ]
  async fn test_failover_sticky_strategy() 
  {
  let config = FailoverConfig {
      endpoints : vec![
  "https://server1.example.com".to_string( ),
  "https://server2.example.com".to_string( ),
      ],
      strategy : FailoverStrategy::Sticky,
      ..Default::default( )
  };

  let failover = FailoverManager::new( config ).unwrap( );

  // Should stick to first endpoint
  let e1 = failover.select_endpoint( ).await.unwrap( );
  let e2 = failover.select_endpoint( ).await.unwrap( );
  let e3 = failover.select_endpoint( ).await.unwrap( );

  assert_eq!( e1, e2 );
  assert_eq!( e2, e3 );
  }

  #[ tokio::test ]
  #[ allow( clippy::float_cmp ) ]
  async fn test_failover_health_tracking() 
  {
  let config = FailoverConfig {
      endpoints : vec!["https://example.com".to_string( ) ],
      strategy : FailoverStrategy::Priority,
      failure_threshold : 3,
      ..Default::default( )
  };

  let failover = FailoverManager::new( config ).unwrap( );

  // Record successes
  for _ in 0..5
  {
      failover.record_success( "https://example.com" ).await;
  }

  let status = failover.health_status( ).await;
  assert_eq!( status[0 ].requests, 5 );
  assert_eq!( status[0 ].successes, 5 );
  assert_eq!( status[0 ].success_rate, 1.0 );
  assert!( status[0 ].healthy );

  // Record failures
  for _ in 0..3
  {
      failover.record_failure( "https://example.com" ).await;
  }

  let status = failover.health_status( ).await;
  assert_eq!( status[0 ].requests, 8 );
  assert_eq!( status[0 ].failure_count, 3 );
  assert!( !status[0 ].healthy );
  }

  #[ tokio::test ]
  async fn test_failover_reset() 
  {
  let config = FailoverConfig {
      endpoints : vec!["https://example.com".to_string( ) ],
      strategy : FailoverStrategy::Priority,
      ..Default::default( )
  };

  let failover = FailoverManager::new( config ).unwrap( );

  // Make some requests
  failover.record_success( "https://example.com" ).await;
  failover.record_failure( "https://example.com" ).await;

  // Reset
  failover.reset( ).await;

  let status = failover.health_status( ).await;
  assert_eq!( status[0 ].requests, 0 );
  assert_eq!( status[0 ].successes, 0 );
  assert!( status[0 ].healthy );
  }

  #[ tokio::test ]
  async fn test_failover_no_endpoints_error() 
  {
  let config = FailoverConfig {
      endpoints : vec![ ],
      strategy : FailoverStrategy::Priority,
      ..Default::default( )
  };

  let result = FailoverManager::new( config );
  assert!( result.is_err( ));
  }

  #[ tokio::test ]
  #[ allow( clippy::float_cmp ) ]
  async fn test_endpoint_health_success_rate() 
  {
  let mut health = EndpointHealth::new( "https://example.com".to_string( ));

  assert_eq!( health.success_rate( ), 1.0 ); // No requests = 100%

  health.record_success( );
  health.record_success( );
  health.record_failure( Duration::from_secs( 60 ), 5 );

  assert_eq!( health.success_rate( ), 2.0 / 3.0 );
  }
}

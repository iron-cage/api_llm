//! Circuit Breaker Implementation
//!
//! Provides automatic service degradation protection using a state machine pattern.
//!
//! ## States
//!
//! - **Closed**: Normal operation, requests pass through
//! - **Open**: Service is failing, requests are rejected immediately
//! - **`HalfOpen`**: Testing if service has recovered
//!
//! ## State Transitions
//!
//! ```text
//! Closed --[failures >= threshold ]--> Open
//! Open --[timeout elapsed ]--> HalfOpen
//! HalfOpen --[success ]--> Closed
//! HalfOpen --[failure ]--> Open
//! ```
//!
//! ## Usage
//!
//! ```no_run
//! # use api_huggingface::reliability::{CircuitBreaker, CircuitBreakerConfig};
//! # use std::time::Duration;
//! # async fn example( ) -> Result< ( ), Box< dyn std::error::Error > > {
//! let circuit_breaker = CircuitBreaker::new(
//!   CircuitBreakerConfig {
//!     failure_threshold : 5,
//!     success_threshold : 2,
//!     timeout : Duration::from_secs( 60 ),
//!   }
//! );
//!
//! let _result = circuit_breaker.execute( async {
//!   Ok::< String, Box< dyn std::error::Error > >( "response".to_string( ))
//! } ).await;
//! # Ok( ( ))
//! # }
//! ```

use std::sync::Arc;
use std::time::Instant; // Instant only available in std, not core
use core::time::Duration;
use tokio::sync::RwLock;

/// Circuit breaker configuration
#[ derive( Debug, Clone ) ]
pub struct CircuitBreakerConfig 
{
  /// Number of consecutive failures before opening the circuit
  pub failure_threshold : u32,
  /// Number of consecutive successes in half-open state before closing
  pub success_threshold : u32,
  /// Duration to wait before transitioning from open to half-open
  pub timeout : Duration,
}

impl Default for CircuitBreakerConfig 
{
  #[ inline ]
  fn default() -> Self 
  {
  Self {
      failure_threshold : 5,
      success_threshold : 2,
      timeout : Duration::from_secs( 60 ),
  }
  }
}

/// Circuit breaker state
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum CircuitState 
{
  /// Normal operation - requests pass through
  Closed,
  /// Service failing - requests rejected immediately
  Open,
  /// Testing recovery - limited requests allowed
  HalfOpen,
}

/// Internal state with counters and timestamps
#[ derive( Debug ) ]
struct CircuitBreakerState 
{
  state : CircuitState,
  failure_count : u32,
  success_count : u32,
  last_failure_time : Option< Instant >,
}

/// Circuit breaker for automatic failure handling
#[ derive( Debug, Clone ) ]
pub struct CircuitBreaker 
{
  config : CircuitBreakerConfig,
  state : Arc< RwLock< CircuitBreakerState > >,
}

impl CircuitBreaker 
{
  /// Create new circuit breaker with given configuration
  #[ inline ]
  #[ must_use ]
  pub fn new( config : CircuitBreakerConfig ) -> Self 
  {
  Self {
      config,
      state : Arc::new( RwLock::new( CircuitBreakerState {
  state : CircuitState::Closed,
  failure_count : 0,
  success_count : 0,
  last_failure_time : None,
      } )),
  }
  }

  /// Execute operation with circuit breaker protection
  ///
  /// Returns `Err` immediately if circuit is open, otherwise executes the operation
  /// and updates circuit state based on result.
  ///
  /// # Errors
  ///
  /// Returns `CircuitBreakerError::CircuitOpen` if the circuit is currently open.
  /// Returns `CircuitBreakerError::Operation( E )` if the operation fails with error `E`.
  #[ inline ]
  pub async fn execute< F, T, E >( &self, f : F ) -> Result< T, CircuitBreakerError< E > >
  where
  F: core::future::Future< Output = Result< T, E > >,
  {
  // Check if we should allow the request
  {
      let mut state = self.state.write( ).await;

      match state.state
      {
  CircuitState::Open => {
          // Check if timeout has elapsed
          if let Some( last_failure ) = state.last_failure_time
          {
      if last_failure.elapsed( ) >= self.config.timeout
      {
              // Transition to half-open
              state.state = CircuitState::HalfOpen;
              state.success_count = 0;
              state.failure_count = 0;
      } else {
              // Circuit still open, reject request
              return Err( CircuitBreakerError::CircuitOpen );
      }
          }
  }
  CircuitState::Closed | CircuitState::HalfOpen => {
          // Allow request in closed and half-open states
  }
      }
  }

  // Execute the operation
  let result = f.await;

  // Update state based on result
  {
      let mut state = self.state.write( ).await;

      match result
      {
  Ok( ref _value ) => {
          // Success
          match state.state
          {
      CircuitState::Closed => {
              // Reset failure count on success
              state.failure_count = 0;
      }
      CircuitState::HalfOpen => {
              // Increment success count
              state.success_count += 1;

              // Check if we should close the circuit
              if state.success_count >= self.config.success_threshold
              {
        state.state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
              }
      }
      CircuitState::Open => {
              // Should not happen, but handle gracefully
              state.state = CircuitState::HalfOpen;
              state.success_count = 1;
              state.failure_count = 0;
      }
          }
  }
  Err( ref _e ) => {
          // Failure
          state.failure_count += 1;
          state.last_failure_time = Some( Instant::now( ));

          match state.state
          {
      CircuitState::Closed => {
              // Check if we should open the circuit
              if state.failure_count >= self.config.failure_threshold
              {
        state.state = CircuitState::Open;
              }
      }
      CircuitState::HalfOpen => {
              // Any failure in half-open state reopens the circuit
              state.state = CircuitState::Open;
              state.success_count = 0;
      }
      CircuitState::Open => {
              // Already open, just update timestamp
      }
          }
  }
      }
  }

  result.map_err( CircuitBreakerError::Operation )
  }

  /// Get current circuit state
  #[ inline ]
  pub async fn state( &self ) -> CircuitState 
  {
  self.state.read( ).await.state
  }

  /// Check if circuit is open
  #[ inline ]
  pub async fn is_open( &self ) -> bool 
  {
  self.state.read( ).await.state == CircuitState::Open
  }

  /// Check if circuit is closed
  #[ inline ]
  pub async fn is_closed( &self ) -> bool 
  {
  self.state.read( ).await.state == CircuitState::Closed
  }

  /// Check if circuit is half-open
  #[ inline ]
  pub async fn is_half_open( &self ) -> bool 
  {
  self.state.read( ).await.state == CircuitState::HalfOpen
  }

  /// Get current failure count
  #[ inline ]
  pub async fn failure_count( &self ) -> u32 
  {
  self.state.read( ).await.failure_count
  }

  /// Get current success count ( relevant in half-open state )
  #[ inline ]
  pub async fn success_count( &self ) -> u32 
  {
  self.state.read( ).await.success_count
  }

  /// Reset circuit breaker to closed state
  #[ inline ]
  pub async fn reset( &self ) 
  {
  let mut state = self.state.write( ).await;
  state.state = CircuitState::Closed;
  state.failure_count = 0;
  state.success_count = 0;
  state.last_failure_time = None;
  }
}

/// Circuit breaker errors
#[ derive( Debug ) ]
pub enum CircuitBreakerError< E > 
{
  /// Circuit is open, request rejected
  CircuitOpen,

  /// Operation failed
  Operation( E ),
}

impl< E > core::fmt::Display for CircuitBreakerError< E >
where
  E: core::fmt::Display,
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result 
  {
  match self
  {
      Self::CircuitOpen => write!( f, "Circuit breaker is open" ),
      Self::Operation( e ) => write!( f, "Operation failed : {e}" ),
  }
  }
}

impl< E > std::error::Error for CircuitBreakerError< E >
where
  E: std::error::Error + 'static,
{
  #[ inline ]
  fn source( &self ) -> Option< &( dyn std::error::Error + 'static ) > 
  {
  match self
  {
      Self::CircuitOpen => None,
      Self::Operation( e ) => Some( e ),
  }
  }
}


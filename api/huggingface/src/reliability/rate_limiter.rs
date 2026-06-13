//! Rate Limiting Implementation
//!
//! Provides automatic rate limiting using a token bucket algorithm with multiple time windows.
//!
//! ## Features
//!
//! - **Per-Second Limiting**: Control requests per second
//! - **Per-Minute Limiting**: Control requests per minute
//! - **Per-Hour Limiting**: Control requests per hour
//! - **Token Bucket**: Smooth rate limiting with burst capacity
//! - **Thread-Safe**: Safe for concurrent use
//!
//! ## Usage
//!
//! ```no_run
//! # use api_huggingface::reliability::{RateLimiter, RateLimiterConfig};
//! # async fn example( ) -> Result< ( ), Box< dyn std::error::Error > > {
//! let rate_limiter = RateLimiter::new(
//!   RateLimiterConfig {
//!     requests_per_second : Some( 10 ),
//!     requests_per_minute : Some( 500 ),
//!     requests_per_hour : Some( 10000 ),
//!   }
//! );
//!
//! // Acquire permission before making request
//! rate_limiter.acquire( ).await?;
//! // ... make your request ...
//! # Ok( ( ))
//! # }
//! ```

use std::sync::Arc;
use std::time::Instant;
use core::time::Duration;
use tokio::sync::RwLock;

/// Rate limiter configuration
#[ derive( Debug, Clone, Copy ) ]
pub struct RateLimiterConfig 
{
  /// Maximum requests per second ( None = unlimited )
  pub requests_per_second : Option< u32 >,
  /// Maximum requests per minute ( None = unlimited )
  pub requests_per_minute : Option< u32 >,
  /// Maximum requests per hour ( None = unlimited )
  pub requests_per_hour : Option< u32 >,
}

impl Default for RateLimiterConfig 
{
  #[ inline ]
  fn default() -> Self 
  {
  Self {
      requests_per_second : Some( 10 ),
      requests_per_minute : Some( 500 ),
      requests_per_hour : Some( 10000 ),
  }
  }
}

/// Token bucket for a single time window
#[ derive( Debug ) ]
struct TokenBucket 
{
  /// Maximum tokens ( capacity )
  capacity : u32,
  /// Current available tokens
  tokens : f64,
  /// Last refill time
  last_refill : Instant,
  /// Refill rate ( tokens per second )
  refill_rate : f64,
}

impl TokenBucket 
{
  /// Create new token bucket
  #[ inline ]
  fn new( capacity : u32, refill_duration : Duration ) -> Self 
  {
  let refill_rate = f64::from( capacity ) / refill_duration.as_secs_f64( );
  Self {
      capacity,
      tokens : f64::from( capacity ),
      last_refill : Instant::now( ),
      refill_rate,
  }
  }

  /// Refill tokens based on elapsed time
  #[ inline ]
  fn refill( &mut self ) 
  {
  let now = Instant::now( );
  let elapsed = now.duration_since( self.last_refill ).as_secs_f64( );

  // Add tokens based on elapsed time
  self.tokens = ( self.tokens + elapsed * self.refill_rate ).min( f64::from( self.capacity ));
  self.last_refill = now;
  }

  /// Try to consume one token
  #[ inline ]
  fn try_consume( &mut self ) -> bool 
  {
  self.refill( );

  if self.tokens >= 1.0
  {
      self.tokens -= 1.0;
      true
  } else {
      false
  }
  }

  /// Get current token count
  #[ inline ]
  #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
  fn available_tokens( &mut self ) -> u32 
  {
  self.refill( );
  self.tokens.floor( ) as u32
  }

  /// Time until next token is available
  #[ inline ]
  fn time_until_token( &mut self ) -> Option< Duration > 
  {
  self.refill( );

  if self.tokens >= 1.0
  {
      None
  } else {
      let tokens_needed = 1.0 - self.tokens;
      let seconds = tokens_needed / self.refill_rate;
      // Fix(bug-rl-01): guard against refill_rate=0.0 (capacity=0) producing +Infinity.
      // Root cause: f64 divides by 0.0 silently to +Inf; Duration::from_secs_f64(+Inf) panics.
      // Pitfall: always call is_finite() before passing float seconds to Duration conversion.
      if seconds.is_finite( )
      {
        Some( Duration::from_secs_f64( seconds ))
      } else {
        Some( Duration::MAX )
      }
  }
  }
}

/// Rate limiter for controlling request rates
#[ derive( Debug, Clone ) ]
#[ allow( clippy::struct_field_names ) ]
pub struct RateLimiter
{
  per_second : Option< Arc< RwLock< TokenBucket > > >,
  per_minute : Option< Arc< RwLock< TokenBucket > > >,
  per_hour : Option< Arc< RwLock< TokenBucket > > >,
}

impl RateLimiter 
{
  /// Create new rate limiter with given configuration
  #[ inline ]
  #[ must_use ]
  pub fn new( config : RateLimiterConfig ) -> Self 
  {
  Self {
      per_second : config.requests_per_second.map( |capacity| {
  Arc::new( RwLock::new( TokenBucket::new( capacity, Duration::from_secs( 1 )) ))
      } ),
      per_minute : config.requests_per_minute.map( |capacity| {
  Arc::new( RwLock::new( TokenBucket::new( capacity, Duration::from_secs( 60 )) ))
      } ),
      per_hour : config.requests_per_hour.map( |capacity| {
  Arc::new( RwLock::new( TokenBucket::new( capacity, Duration::from_secs( 3600 )) ))
      } ),
  }
  }

  /// Acquire permission to make a request ( blocks until available )
  ///
  /// This method waits until tokens are available in all configured time windows.
  ///
  /// # Errors
  ///
  /// Returns `RateLimitError` if rate limit cannot be satisfied.
  #[ inline ]
  pub async fn acquire( &self ) -> Result< ( ), RateLimitError > 
  {
  loop
  {
      // Try to acquire from all buckets
      let mut max_wait : Option< Duration > = None;

      if let Some( ref bucket ) = self.per_second
      {
  let mut b = bucket.write( ).await;
  if !b.try_consume( )
  {
          if let Some( wait ) = b.time_until_token( )
          {
      max_wait = Some( max_wait.map_or( wait, |m : Duration| m.max( wait )) );
          }
  }
      }

      if let Some( ref bucket ) = self.per_minute
      {
  let mut b = bucket.write( ).await;
  if !b.try_consume( )
  {
          if let Some( wait ) = b.time_until_token( )
          {
      max_wait = Some( max_wait.map_or( wait, |m : Duration| m.max( wait )) );
          }
  }
      }

      if let Some( ref bucket ) = self.per_hour
      {
  let mut b = bucket.write( ).await;
  if !b.try_consume( )
  {
          if let Some( wait ) = b.time_until_token( )
          {
      max_wait = Some( max_wait.map_or( wait, |m : Duration| m.max( wait )) );
          }
  }
      }

      // If all buckets succeeded, we're done
      if max_wait.is_none( )
      {
  return Ok( ( ));
      }

      // Otherwise, wait for the longest required duration
      if let Some( wait_duration ) = max_wait
      {
  tokio::time::sleep( wait_duration ).await;
      }
  }
  }

  /// Try to acquire permission without blocking
  ///
  /// Returns `Ok( ( ))` if permission granted, `Err` if rate limited.
  ///
  /// # Errors
  ///
  /// Returns `RateLimitError::RateLimitExceeded` if any time window is exhausted.
  #[ inline ]
  pub async fn try_acquire( &self ) -> Result< ( ), RateLimitError > 
  {
  // Try per-second bucket
  if let Some( ref bucket ) = self.per_second
  {
      let mut b = bucket.write( ).await;
      if !b.try_consume( )
      {
  return Err( RateLimitError::RateLimitExceeded {
          window : "per_second".to_string( ),
          retry_after : b.time_until_token( ),
  } );
      }
  }

  // Try per-minute bucket
  if let Some( ref bucket ) = self.per_minute
  {
      let mut b = bucket.write( ).await;
      if !b.try_consume( )
      {
  return Err( RateLimitError::RateLimitExceeded {
          window : "per_minute".to_string( ),
          retry_after : b.time_until_token( ),
  } );
      }
  }

  // Try per-hour bucket
  if let Some( ref bucket ) = self.per_hour
  {
      let mut b = bucket.write( ).await;
      if !b.try_consume( )
      {
  return Err( RateLimitError::RateLimitExceeded {
          window : "per_hour".to_string( ),
          retry_after : b.time_until_token( ),
  } );
      }
  }

  Ok( ( ))
  }

  /// Get current available tokens for all time windows
  #[ inline ]
  pub async fn available_tokens( &self ) -> AvailableTokens 
  {
  AvailableTokens {
      per_second : if let Some( ref bucket ) = self.per_second
      {
  Some( bucket.write( ).await.available_tokens( ))
      } else {
  None
      },
      per_minute : if let Some( ref bucket ) = self.per_minute
      {
  Some( bucket.write( ).await.available_tokens( ))
      } else {
  None
      },
      per_hour : if let Some( ref bucket ) = self.per_hour
      {
  Some( bucket.write( ).await.available_tokens( ))
      } else {
  None
      },
  }
  }

  /// Reset all rate limit buckets to full capacity
  #[ inline ]
  pub async fn reset( &self ) 
  {
  if let Some( ref bucket ) = self.per_second
  {
      let mut b = bucket.write( ).await;
      b.tokens = f64::from( b.capacity );
      b.last_refill = Instant::now( );
  }

  if let Some( ref bucket ) = self.per_minute
  {
      let mut b = bucket.write( ).await;
      b.tokens = f64::from( b.capacity );
      b.last_refill = Instant::now( );
  }

  if let Some( ref bucket ) = self.per_hour
  {
      let mut b = bucket.write( ).await;
      b.tokens = f64::from( b.capacity );
      b.last_refill = Instant::now( );
  }
  }
}

/// Available tokens in each time window
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub struct AvailableTokens 
{
  /// Tokens available in per-second window
  pub per_second : Option< u32 >,
  /// Tokens available in per-minute window
  pub per_minute : Option< u32 >,
  /// Tokens available in per-hour window
  pub per_hour : Option< u32 >,
}

/// Rate limit errors
#[ derive( Debug ) ]
pub enum RateLimitError 
{
  /// Rate limit exceeded for a time window
  RateLimitExceeded {
  /// Which time window was exceeded
  window : String,
  /// Time to wait before retry
  retry_after : Option< Duration >,
  },
}

impl core::fmt::Display for RateLimitError 
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result 
  {
  match self
  {
      Self::RateLimitExceeded { window, retry_after } => {
  if let Some( duration ) = retry_after
  {
          write!( f, "Rate limit exceeded for {window}, retry after {duration:?}" )
  } else {
          write!( f, "Rate limit exceeded for {window}" )
  }
      }
  }
  }
}

impl std::error::Error for RateLimitError {}

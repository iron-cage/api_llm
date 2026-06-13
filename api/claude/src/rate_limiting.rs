//! Rate limiting implementation for Anthropic Claude API
//!
//! This module provides rate limiting functionality to control API request rates
//! and prevent hitting API rate limits.

#[ cfg( feature = "rate-limiting" ) ]
#[ allow( clippy::std_instead_of_core ) ]
// Allow missing inline attributes for rate limiting module
#[ allow( clippy::missing_inline_in_public_items ) ]
mod private
{
  use crate::CreateMessageRequest;
  use std::time::{ Duration, Instant };
  use std::sync::{ Arc, Mutex };

  /// Configuration for rate limiting
  #[ derive( Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize ) ]
  pub struct RateLimiterConfig
  {
    /// Tokens per second rate
    tokens_per_second : f64,
    /// Maximum bucket capacity
    bucket_capacity : u32,
    /// Initial number of tokens
    initial_tokens : u32,
  }

  // No Default implementation - explicit configuration required per governing principle

  impl RateLimiterConfig
  {
    /// Create rate limiter configuration with explicit parameters (no defaults)
    ///
    /// # Arguments
    ///
    /// * `tokens_per_second` - Rate at which tokens are added to the bucket (must be > 0.0)
    /// * `bucket_capacity` - Maximum number of tokens the bucket can hold (must be > 0)
    /// * `initial_tokens` - Initial number of tokens in the bucket (must be <= `bucket_capacity`)
    pub fn with_explicit_config( tokens_per_second : f64, bucket_capacity : u32, initial_tokens : u32 ) -> Self
    {
      Self {
        tokens_per_second,
        bucket_capacity,
        initial_tokens,
      }
    }
  }

  impl Default for RateLimiterConfig 
  {
    fn default() -> Self 
    {
      Self::new()
    }
  }

  impl RateLimiterConfig
  {
    /// Create a new rate limiter configuration (compatibility wrapper)
    ///
    /// NOTE: This is a compatibility wrapper with sensible defaults. For explicit control, use `with_explicit_config()`
    pub fn new() -> Self
    {
      // Compatibility wrapper with sensible defaults for rate limiting
      Self::with_explicit_config(
        10.0, // tokens_per_second : 10 tokens per second
        100,  // bucket_capacity : 100 tokens maximum
        50,   // initial_tokens : start with half capacity
      )
    }

    /// Set tokens per second
    #[ must_use ]
    pub fn with_tokens_per_second( mut self, tokens_per_second : f64 ) -> Self
    {
      self.tokens_per_second = tokens_per_second;
      self
    }

    /// Set bucket capacity
    #[ must_use ]
    pub fn with_bucket_capacity( mut self, bucket_capacity : u32 ) -> Self
    {
      self.bucket_capacity = bucket_capacity;
      self
    }

    /// Set initial tokens
    #[ must_use ]
    pub fn with_initial_tokens( mut self, initial_tokens : u32 ) -> Self
    {
      self.initial_tokens = initial_tokens;
      self
    }

    /// Validate configuration
    pub fn is_valid( &self ) -> bool
    {
      self.tokens_per_second > 0.0 &&
      self.bucket_capacity > 0 &&
      self.initial_tokens <= self.bucket_capacity
    }

    /// Get tokens per second
    pub fn tokens_per_second( &self ) -> f64
    {
      self.tokens_per_second
    }

    /// Get bucket capacity
    pub fn bucket_capacity( &self ) -> u32
    {
      self.bucket_capacity
    }

    /// Get initial tokens
    pub fn initial_tokens( &self ) -> u32
    {
      self.initial_tokens
    }
  }

  /// Rate limiter state
  #[ derive( Debug, Clone ) ]
  struct RateLimiterState
  {
    tokens : f64,
    last_refill : Instant,
  }

  /// Rate limiter metrics
  #[ derive( Debug, Clone ) ]
  pub struct RateLimiterMetrics
  {
    requests_allowed : u64,
    requests_blocked : u64,
    total_tokens_consumed : u64,
  }

  impl RateLimiterMetrics
  {
    /// Get number of requests allowed
    pub fn requests_allowed( &self ) -> u64
    {
      self.requests_allowed
    }

    /// Get number of requests blocked
    pub fn requests_blocked( &self ) -> u64
    {
      self.requests_blocked
    }

    /// Get total tokens consumed
    pub fn total_tokens_consumed( &self ) -> u64
    {
      self.total_tokens_consumed
    }

    /// Calculate current throughput
    pub fn current_throughput( &self ) -> f64
    {
      // Placeholder implementation
      0.0
    }
  }

  /// Token bucket rate limiter
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "rate-limiting" ) ]
  /// # {
  /// use api_claude::{ RateLimiter, RateLimiterConfig };
  ///
  /// // Create a rate limiter with default configuration
  /// let config = RateLimiterConfig::new()
  ///   .with_tokens_per_second( 10.0 )
  ///   .with_bucket_capacity( 100 );
  /// let limiter = RateLimiter::new( config );
  ///
  /// // Check if we can make a request
  /// if limiter.can_make_request( 1 ) {
  ///   // Try to consume tokens for the request
  ///   let success = limiter.try_consume( 1 );
  ///   assert!( success );
  /// }
  ///
  /// // Get current metrics
  /// let metrics = limiter.metrics();
  /// assert_eq!( metrics.requests_allowed(), 1 );
  /// # }
  /// ```
  #[ derive( Debug, Clone ) ]
  pub struct RateLimiter
  {
    config : RateLimiterConfig,
    state : Arc< Mutex< RateLimiterState > >,
    metrics : Arc< Mutex< RateLimiterMetrics > >,
  }

  /// Configuration for request cost calculation
  #[ derive( Debug, Clone, Copy ) ]
  pub struct RequestCostConfig
  {
    /// Base token cost for any request
    pub base_cost : u32,
    /// Divisor applied to `max_tokens` to compute cost contribution; `None` = skip
    pub max_tokens_divisor : Option< f64 >,
    /// Flat cost added per message in the conversation; `None` = skip
    pub message_cost_per_count : Option< u32 >,
    /// Divisor applied to total system prompt length; `None` = skip
    pub system_prompt_divisor : Option< f64 >,
    /// Divisor applied to text content length; `None` = skip
    pub content_length_divisor : Option< f64 >,
    /// Flat cost for non-text content blocks (images, etc.); `None` = skip
    pub non_text_content_cost : Option< u32 >,
    /// Floor applied after all other costs are summed; `None` = no floor
    pub minimum_cost : Option< u32 >,
  }

  impl RateLimiter
  {
    /// Create a new rate limiter
    pub fn new( config : RateLimiterConfig ) -> Self
    {
      let now = Instant::now();
      Self {
        config,
        state : Arc::new( Mutex::new( RateLimiterState {
          tokens : f64::from(config.initial_tokens),
          last_refill : now,
        } ) ),
        metrics : Arc::new( Mutex::new( RateLimiterMetrics {
          requests_allowed : 0,
          requests_blocked : 0,
          total_tokens_consumed : 0,
        } ) ),
      }
    }

    /// Get available tokens
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn available_tokens( &self ) -> u32
    {
      let state = self.state.lock().unwrap();
      #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
      {
        state.tokens.max(0.0) as u32
      }
    }

    /// Check if request can be made
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn can_make_request( &self, tokens : u32 ) -> bool
    {
      let state = self.state.lock().unwrap();
      if tokens == 0
      {
        true
      } else {
        state.tokens >= f64::from(tokens)
      }
    }

    /// Try to consume tokens
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn try_consume( &self, tokens : u32 ) -> bool
    {
      let mut state = self.state.lock().unwrap();
      if state.tokens > 0.0
      {
        let tokens_to_consume = f64::from(tokens).min( state.tokens );
        state.tokens -= tokens_to_consume;

        // Update metrics
        let mut metrics = self.metrics.lock().unwrap();
        metrics.requests_allowed += 1;
        #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
        {
          metrics.total_tokens_consumed += tokens_to_consume.max(0.0) as u64;
        }

        true
      } else {
        // Update metrics
        let mut metrics = self.metrics.lock().unwrap();
        metrics.requests_blocked += 1;

        false
      }
    }

    /// Refill tokens based on elapsed time
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn refill( &self )
    {
      let mut state = self.state.lock().unwrap();
      let now = Instant::now();
      let elapsed = now.duration_since( state.last_refill ).as_secs_f64();

      if elapsed > 0.0
      {
        let tokens_to_add = elapsed * self.config.tokens_per_second;
        state.tokens = ( state.tokens + tokens_to_add ).min( f64::from(self.config.bucket_capacity) );
        state.last_refill = now;
      }
    }

    /// Get time until tokens are available
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn time_until_tokens_available( &self, tokens : u32 ) -> Duration
    {
      let state = self.state.lock().unwrap();

      if state.tokens >= f64::from(tokens)
      {
        return Duration::from_secs( 0 );
      }

      let tokens_needed = f64::from(tokens) - state.tokens;
      let seconds_needed = tokens_needed / self.config.tokens_per_second;

      Duration::from_secs_f64( seconds_needed )
    }

    /// Wait for tokens with explicit timeout and check interval
    pub fn wait_for_tokens_with_config( &self, tokens : u32, timeout : Duration, check_interval : Duration ) -> bool
    {
      let start = Instant::now();

      loop
      {
        if self.can_make_request( tokens )
        {
          return true;
        }

        if start.elapsed() >= timeout
        {
          return false;
        }

        // Sleep for the specified interval before checking again
        std::thread::sleep( check_interval );
      }
    }

    /// Wait for tokens with optional timeout (compatibility wrapper)
    ///
    /// NOTE: This is a compatibility wrapper with sensible defaults. For explicit control, use `wait_for_tokens_with_config()`
    pub fn wait_for_tokens( &self, tokens : u32, timeout : Option< Duration > ) -> bool
    {
      // Compatibility wrapper with sensible defaults
      let actual_timeout = timeout.unwrap_or( Duration::from_secs( 60 ) ); // Default 60 second timeout
      let check_interval = Duration::from_millis( 10 ); // Default 10ms check interval
      self.wait_for_tokens_with_config( tokens, actual_timeout, check_interval )
    }

    /// Calculate request cost in tokens with explicit cost configuration
    pub fn calculate_request_cost_with_config(
      &self,
      request : &CreateMessageRequest,
      config : RequestCostConfig,
    ) -> u32
    {
      let mut cost = config.base_cost;

      // Add cost based on max_tokens (if configured)
      if let Some( divisor ) = config.max_tokens_divisor
      {
        #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
        {
          cost += ( f64::from(request.max_tokens) / divisor ).ceil() as u32;
        }
      }

      // Add cost based on number of messages (if configured)
      if let Some( per_message_cost ) = config.message_cost_per_count
      {
        cost += u32::try_from(request.messages.len()).unwrap_or(0) * per_message_cost;
      }

      // Add cost based on system prompt length (if configured)
      if let ( Some( ref system_blocks ), Some( divisor ) ) = ( &request.system, config.system_prompt_divisor )
      {
        #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
        {
          let total_system_len : usize = system_blocks.iter().map( | block | block.text.len() ).sum();
          cost += ( total_system_len as f64 / divisor ).ceil() as u32;
        }
      }

      // Add cost based on message content length (if configured)
      if config.content_length_divisor.is_some() || config.non_text_content_cost.is_some()
      {
        for message in &request.messages
        {
          for content in &message.content
          {
            let content_cost = match content
            {
              crate::messages::Content::Text { text, .. } => {
                if let Some( divisor ) = config.content_length_divisor
                {
                  #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
                  {
                    ( text.len() as f64 / divisor ).ceil() as u32
                  }
                }
                else
                {
                  0
                }
              },
              _ => config.non_text_content_cost.unwrap_or(0),
            };
            cost += content_cost;
          }
        }
      }

      // Apply minimum cost (if configured)
      config.minimum_cost.map_or( cost, | min | cost.max( min ) )
    }

    /// Calculate request cost in tokens (compatibility wrapper)
    ///
    /// NOTE: This is a compatibility wrapper with sensible defaults. For explicit control, use `calculate_request_cost_with_config()`
    pub fn calculate_request_cost( &self, request : &CreateMessageRequest ) -> u32
    {
      self.calculate_request_cost_with_config( request, RequestCostConfig
      {
        base_cost : 1,
        max_tokens_divisor : Some( 1000.0 ),
        message_cost_per_count : Some( 1 ),
        system_prompt_divisor : Some( 100.0 ),
        content_length_divisor : Some( 100.0 ),
        non_text_content_cost : Some( 50 ),
        minimum_cost : Some( 1 ),
      } )
    }

    /// Get metrics
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn metrics( &self ) -> RateLimiterMetrics
    {
      let metrics = self.metrics.lock().unwrap();
      metrics.clone()
    }

    /// Serialize state for persistence
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn serialize_state( &self ) -> String
    {
      let state = self.state.lock().unwrap();
      let metrics = self.metrics.lock().unwrap();

      // Serialize to JSON-like format
      format!(
        "{{\"tokens\":{},\"last_refill_nanos\":{},\"requests_allowed\":{},\"requests_blocked\":{},\"total_tokens_consumed\":{}}}",
        state.tokens,
        state.last_refill.elapsed().as_nanos(),
        metrics.requests_allowed,
        metrics.requests_blocked,
        metrics.total_tokens_consumed
      )
    }

    /// Create rate limiter from saved state
    ///
    /// # Errors
    ///
    /// Returns an error if the state string cannot be parsed or contains invalid data
    #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
    pub fn from_state( config : RateLimiterConfig, state_str : &str ) -> Result< Self, String >
    {
      if state_str.is_empty() || state_str == "{}"
      {
        return Ok( Self::new( config ) );
      }

      // Simple JSON-like parsing
      let tokens = Self::extract_json_field( state_str, "tokens" )?;
      let last_refill_nanos = Self::extract_json_field( state_str, "last_refill_nanos" )?;
      let requests_allowed = Self::extract_json_field( state_str, "requests_allowed" )?.max( 0.0 ) as u64;
      let requests_blocked = Self::extract_json_field( state_str, "requests_blocked" )?.max( 0.0 ) as u64;
      let total_tokens_consumed = Self::extract_json_field( state_str, "total_tokens_consumed" )?.max( 0.0 ) as u64;

      let now = Instant::now();
      let last_refill = now.checked_sub( Duration::from_nanos( last_refill_nanos.max( 0.0 ) as u64 ) ).unwrap_or( now );

      Ok( Self {
        config,
        state : Arc::new( Mutex::new( RateLimiterState {
          tokens,
          last_refill,
        } ) ),
        metrics : Arc::new( Mutex::new( RateLimiterMetrics {
          requests_allowed,
          requests_blocked,
          total_tokens_consumed,
        } ) ),
      } )
    }

    /// Extract a numeric field from JSON-like string
    fn extract_json_field( json : &str, field : &str ) -> Result< f64, String >
    {
      let pattern = format!( "\"{field}\":" );
      if let Some( start ) = json.find( &pattern )
      {
        let start = start + pattern.len();
        let end = json[ start.. ].find( ',' ).or_else( || json[ start.. ].find( '}' ) );
        if let Some( end ) = end
        {
          let value_str = &json[ start..start + end ];
          value_str.parse::< f64 >().map_err( |_| format!( "Failed to parse {field} field" ) )
        } else {
          Err( format!( "Could not find end of {field} field" ) )
        }
      } else {
        Err( format!( "Field {field} not found" ) )
      }
    }

    /// Get configuration
    pub fn config( &self ) -> &RateLimiterConfig
    {
      &self.config
    }
  }

  /// Configuration for adaptive rate limiting
  #[ derive( Debug, Clone ) ]
  pub struct AdaptiveRateLimiterConfig
  {
    base_tokens_per_second : f64,
    max_tokens_per_second : f64,
    min_tokens_per_second : f64,
    adjustment_factor : f64,
  }

  impl Default for AdaptiveRateLimiterConfig 
  {
    fn default() -> Self 
    {
      Self::new()
    }
  }

  impl AdaptiveRateLimiterConfig
  {
    /// Create new adaptive rate limiter config with explicit parameters (no defaults)
    ///
    /// # Arguments
    ///
    /// * `base_tokens_per_second` - Initial rate for token generation
    /// * `max_tokens_per_second` - Maximum rate limit
    /// * `min_tokens_per_second` - Minimum rate limit
    /// * `adjustment_factor` - Factor for rate adjustments (must be > 0.0 and < 1.0)
    pub fn with_explicit_config( base_tokens_per_second : f64, max_tokens_per_second : f64, min_tokens_per_second : f64, adjustment_factor : f64 ) -> Self
    {
      Self {
        base_tokens_per_second,
        max_tokens_per_second,
        min_tokens_per_second,
        adjustment_factor,
      }
    }

    /// Create new adaptive rate limiter config (compatibility wrapper)
    ///
    /// NOTE: This is a compatibility wrapper with sensible defaults. For explicit control, use `with_explicit_config()`
    pub fn new() -> Self
    {
      // Compatibility wrapper with sensible defaults for adaptive rate limiting
      Self::with_explicit_config(
        10.0, // base_tokens_per_second : start at 10 TPS
        50.0, // max_tokens_per_second : max 50 TPS
        1.0,  // min_tokens_per_second : min 1 TPS
        0.1,  // adjustment_factor : 10% adjustments
      )
    }

    /// Set base tokens per second
    #[ must_use ]
    pub fn with_base_tokens_per_second( mut self, rate : f64 ) -> Self
    {
      self.base_tokens_per_second = rate;
      self
    }

    /// Set maximum tokens per second
    #[ must_use ]
    pub fn with_max_tokens_per_second( mut self, rate : f64 ) -> Self
    {
      self.max_tokens_per_second = rate;
      self
    }

    /// Set minimum tokens per second
    #[ must_use ]
    pub fn with_min_tokens_per_second( mut self, rate : f64 ) -> Self
    {
      self.min_tokens_per_second = rate;
      self
    }

    /// Set adjustment factor
    #[ must_use ]
    pub fn with_adjustment_factor( mut self, factor : f64 ) -> Self
    {
      self.adjustment_factor = factor;
      self
    }
  }

  /// Adaptive rate limiter that adjusts based on API responses
  #[ derive( Debug, Clone ) ]
  pub struct AdaptiveRateLimiter
  {
    config : AdaptiveRateLimiterConfig,
    current_rate : Arc< Mutex< f64 > >,
  }

  impl AdaptiveRateLimiter
  {
    /// Create new adaptive rate limiter
    pub fn new( config : AdaptiveRateLimiterConfig ) -> Self
    {
      let current_rate = config.base_tokens_per_second;
      Self {
        config,
        current_rate : Arc::new( Mutex::new( current_rate ) ),
      }
    }

    /// Get current rate
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn current_rate( &self ) -> f64
    {
      let current_rate = self.current_rate.lock().unwrap();
      *current_rate
    }

    /// Record successful request with explicit adjustment configuration
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn record_success_with_config( &self, increase_factor : f64 )
    {
      let mut current_rate = self.current_rate.lock().unwrap();
      let new_rate = *current_rate * ( 1.0 + increase_factor );
      *current_rate = new_rate.min( self.config.max_tokens_per_second );
    }

    /// Record rate limit hit with explicit adjustment configuration
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn record_rate_limit_hit_with_config( &self, decrease_factor : f64 )
    {
      let mut current_rate = self.current_rate.lock().unwrap();
      let new_rate = *current_rate * ( 1.0 - decrease_factor );
      *current_rate = new_rate.max( self.config.min_tokens_per_second );
    }

    /// Record error with explicit adjustment configuration
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned
    pub fn record_error_with_config( &self, decrease_factor : f64 )
    {
      let mut current_rate = self.current_rate.lock().unwrap();
      let new_rate = *current_rate * ( 1.0 - decrease_factor );
      *current_rate = new_rate.max( self.config.min_tokens_per_second );
    }

    /// Record successful request (compatibility wrapper)
    ///
    /// NOTE: This is a compatibility wrapper with default adjustment factor. For explicit control, use `record_success_with_config()`
    pub fn record_success( &self )
    {
      // Compatibility wrapper using the config's adjustment factor
      self.record_success_with_config( self.config.adjustment_factor );
    }

    /// Record rate limit hit (compatibility wrapper)
    ///
    /// NOTE: This is a compatibility wrapper with default adjustment factor. For explicit control, use `record_rate_limit_hit_with_config()`
    pub fn record_rate_limit_hit( &self )
    {
      // Compatibility wrapper using double the config's adjustment factor for rate limits
      self.record_rate_limit_hit_with_config( self.config.adjustment_factor * 2.0 );
    }

    /// Record error (compatibility wrapper)
    ///
    /// NOTE: This is a compatibility wrapper with default adjustment factor. For explicit control, use `record_error_with_config()`
    pub fn record_error( &self )
    {
      // Compatibility wrapper using the config's adjustment factor
      self.record_error_with_config( self.config.adjustment_factor );
    }
  }
}

#[ cfg( feature = "rate-limiting" ) ]
crate::mod_interface!
{
  exposed use
  {
    RateLimiterConfig,
    RequestCostConfig,
    RateLimiter,
    RateLimiterMetrics,
    AdaptiveRateLimiterConfig,
    AdaptiveRateLimiter,
  };
}

#[ cfg( not( feature = "rate-limiting" ) ) ]
crate::mod_interface!
{
  // Empty when rate-limiting feature is disabled
}
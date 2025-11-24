//! Quota Management Module
//!
//! This module handles request quotas, rate limiting, and usage enforcement
//! for enterprise `OpenAI` API usage.

use serde::{ Deserialize, Serialize };
use std::
{
  collections ::HashMap,
  sync ::{ Arc, Mutex },
  time ::Instant,
};
use core::time::Duration;
use crate::error::{ Result, OpenAIError };

/// Quota status for a request
#[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
pub enum QuotaStatus
{
  /// Request is allowed
  Allowed,
  /// Daily quota exceeded
  DailyQuotaExceeded,
  /// Hourly quota exceeded
  HourlyQuotaExceeded,
  /// Concurrent request limit exceeded
  ConcurrentLimitExceeded,
  /// Rate limit exceeded
  RateLimitExceeded,
}

/// Quota reservation for a request
#[ derive( Debug, Clone ) ]
pub struct QuotaReservation
{
  /// Whether this reservation affects concurrent count
  pub concurrent : bool,
  /// Timestamp when reservation was made
  pub timestamp : Instant,
  /// Estimated tokens for this request
  pub estimated_tokens : Option< u64 >,
}

/// Request metadata for quota calculations
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct RequestMetadata
{
  /// Estimated token count for the request
  pub estimated_tokens : u64,
  /// Model being used
  pub model : String,
  /// Request type (chat, completion, etc.)
  pub request_type : String,
  /// Priority level (0-10, higher is more important)
  pub priority : u8,
  /// User identifier for per-user quotas
  pub user_id : Option< String >,
}

/// Usage tracking data
#[ derive( Debug, Clone ) ]
pub struct QuotaUsageTracker
{
  /// Daily request count
  pub daily_requests : u64,
  /// Hourly request count
  pub hourly_requests : u64,
  /// Current concurrent requests
  pub concurrent_requests : u64,
  /// Daily token count
  pub daily_tokens : u64,
  /// Hourly token count
  pub hourly_tokens : u64,
  /// Per-user usage tracking
  pub user_usage : HashMap<  String, UserUsage  >,
  /// Last reset timestamps
  pub last_daily_reset : Instant,
  /// Last hourly reset timestamp
  pub last_hourly_reset : Instant,
}

/// Per-user usage tracking
#[ derive( Debug, Clone, Default ) ]
pub struct UserUsage
{
  /// User's daily requests
  pub daily_requests : u64,
  /// User's hourly requests
  pub hourly_requests : u64,
  /// User's daily tokens
  pub daily_tokens : u64,
  /// User's hourly tokens
  pub hourly_tokens : u64,
}

/// Comprehensive quota usage information
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct QuotaUsage
{
  /// Current daily usage
  pub daily : QuotaUsageDetails,
  /// Current hourly usage
  pub hourly : QuotaUsageDetails,
  /// Current concurrent usage
  pub concurrent : ConcurrentUsageDetails,
  /// Per-user usage breakdown
  pub per_user : HashMap<  String, UserQuotaUsage  >,
  /// Usage efficiency metrics
  pub efficiency_metrics : UsageEfficiencyMetrics,
}

/// Detailed quota usage for a time period
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct QuotaUsageDetails
{
  /// Requests used
  pub requests_used : u64,
  /// Requests limit
  pub requests_limit : Option< u64 >,
  /// Tokens used
  pub tokens_used : u64,
  /// Tokens limit
  pub tokens_limit : Option< u64 >,
  /// Usage percentage (0.0-1.0)
  pub usage_percentage : f64,
  /// Time until reset
  pub time_until_reset_seconds : u64,
}

/// Current concurrent usage details
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ConcurrentUsageDetails
{
  /// Current concurrent requests
  pub current_requests : u64,
  /// Maximum concurrent requests allowed
  pub max_requests : Option< u64 >,
  /// Peak concurrent requests in current period
  pub peak_requests : u64,
  /// Average concurrent requests
  pub average_requests : f64,
}

/// Per-user quota usage summary
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct UserQuotaUsage
{
  /// User identifier
  pub user_id : String,
  /// Daily usage
  pub daily_requests : u64,
  /// Hourly usage
  pub hourly_requests : u64,
  /// Daily tokens
  pub daily_tokens : u64,
  /// Hourly tokens
  pub hourly_tokens : u64,
  /// Usage rank among all users (1 = highest usage)
  pub usage_rank : u32,
}

/// Usage efficiency and optimization metrics
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct UsageEfficiencyMetrics
{
  /// Average tokens per request
  pub avg_tokens_per_request : f64,
  /// Peak usage efficiency (requests/hour at peak)
  pub peak_efficiency : f64,
  /// Request success rate
  pub success_rate : f64,
  /// Average response time in milliseconds
  pub avg_response_time_ms : f64,
  /// Quota utilization efficiency (0.0-1.0)
  pub quota_utilization : f64,
}

/// Main quota manager for enterprise usage control
#[ derive( Debug ) ]
pub struct QuotaManager
{
  /// Daily request limit
  daily_request_limit : Option< u64 >,
  /// Hourly request limit
  hourly_request_limit : Option< u64 >,
  /// Concurrent request limit
  concurrent_request_limit : Option< u64 >,
  /// Daily token limit
  daily_token_limit : Option< u64 >,
  /// Hourly token limit
  hourly_token_limit : Option< u64 >,
  /// Usage tracking
  usage_tracker : Arc< Mutex< QuotaUsageTracker > >,
}

impl QuotaManager
{
  /// Create new quota manager with specified limits
  #[ inline ]
  #[ must_use ]
  pub fn new(
    daily_request_limit : Option< u64 >,
    hourly_request_limit : Option< u64 >,
    concurrent_request_limit : Option< u64 >,
    daily_token_limit : Option< u64 >,
    hourly_token_limit : Option< u64 >,
  ) -> Self
  {
    Self
    {
      daily_request_limit,
      hourly_request_limit,
      concurrent_request_limit,
      daily_token_limit,
      hourly_token_limit,
      usage_tracker : Arc::new( Mutex::new( QuotaUsageTracker
      {
        daily_requests : 0,
        hourly_requests : 0,
        concurrent_requests : 0,
        daily_tokens : 0,
        hourly_tokens : 0,
        user_usage : HashMap::new(),
        last_daily_reset : Instant::now(),
        last_hourly_reset : Instant::now(),
      } ) ),
    }
  }

  /// Check if request is allowed under current quotas
  ///
  /// # Errors
  ///
  /// Returns an error if quota checking fails or quota limits are exceeded.
  ///
  /// # Panics
  ///
  /// Panics if the usage tracker mutex is poisoned.
  #[ inline ]
  pub fn check_quota( &self, request : &RequestMetadata ) -> Result< QuotaStatus >
  {
    let mut tracker = self.usage_tracker.lock().unwrap();
    Self::reset_counters_if_needed( &mut tracker );

    // Check daily limits
    if let Some( daily_limit ) = self.daily_request_limit
    {
      if tracker.daily_requests >= daily_limit
      {
        return Ok( QuotaStatus::DailyQuotaExceeded );
      }
    }

    if let Some( daily_token_limit ) = self.daily_token_limit
    {
      if tracker.daily_tokens + request.estimated_tokens > daily_token_limit
      {
        return Ok( QuotaStatus::DailyQuotaExceeded );
      }
    }

    // Check hourly limits
    if let Some( hourly_limit ) = self.hourly_request_limit
    {
      if tracker.hourly_requests >= hourly_limit
      {
        return Ok( QuotaStatus::HourlyQuotaExceeded );
      }
    }

    if let Some( hourly_token_limit ) = self.hourly_token_limit
    {
      if tracker.hourly_tokens + request.estimated_tokens > hourly_token_limit
      {
        return Ok( QuotaStatus::HourlyQuotaExceeded );
      }
    }

    // Check concurrent limits
    if let Some( concurrent_limit ) = self.concurrent_request_limit
    {
      if tracker.concurrent_requests >= concurrent_limit
      {
        return Ok( QuotaStatus::ConcurrentLimitExceeded );
      }
    }

    Ok( QuotaStatus::Allowed )
  }

  /// Reserve quota for a request
  ///
  /// # Errors
  ///
  /// Returns an error if quota reservation fails or limits are exceeded.
  ///
  /// # Panics
  ///
  /// Panics if the usage tracker mutex is poisoned.
  #[ inline ]
  pub fn reserve_quota( &self, request : &RequestMetadata ) -> Result< QuotaReservation >
  {
    let status = self.check_quota( request )?;
    if !matches!( status, QuotaStatus::Allowed )
    {
      return Err( OpenAIError::Internal( format!( "Quota exceeded : {status:?}" ) ).into() );
    }

    let mut tracker = self.usage_tracker.lock().unwrap();

    // Update counters
    tracker.daily_requests += 1;
    tracker.hourly_requests += 1;
    tracker.concurrent_requests += 1;
    tracker.daily_tokens += request.estimated_tokens;
    tracker.hourly_tokens += request.estimated_tokens;

    // Update user usage if user_id provided
    if let Some( ref user_id ) = request.user_id
    {
      let user_usage = tracker.user_usage.entry( user_id.clone() )
        .or_insert( UserUsage
        {
          daily_requests : 0,
          hourly_requests : 0,
          daily_tokens : 0,
          hourly_tokens : 0,
        } );

      user_usage.daily_requests += 1;
      user_usage.hourly_requests += 1;
      user_usage.daily_tokens += request.estimated_tokens;
      user_usage.hourly_tokens += request.estimated_tokens;
    }

    Ok( QuotaReservation
    {
      concurrent : true,
      timestamp : Instant::now(),
      estimated_tokens : Some( request.estimated_tokens ),
    } )
  }

  /// Release quota reservation (typically called when request completes)
  ///
  /// # Panics
  ///
  /// Panics if the usage tracker mutex is poisoned.
  #[ inline ]
  pub fn release_quota( &self, reservation : &QuotaReservation )
  {
    if reservation.concurrent
    {
      let mut tracker = self.usage_tracker.lock().unwrap();
      if tracker.concurrent_requests > 0
      {
        tracker.concurrent_requests -= 1;
      }
    }
  }

  /// Get current quota usage statistics
  ///
  /// # Errors
  ///
  /// Returns an error if usage statistics cannot be retrieved.
  ///
  /// # Panics
  ///
  /// Panics if the usage tracker mutex is poisoned.
  #[ inline ]
  pub fn get_quota_usage( &self ) -> Result< QuotaUsage >
  {
    let mut tracker = self.usage_tracker.lock().unwrap();
    Self::reset_counters_if_needed( &mut tracker );

    let daily_usage_pct = if let Some( limit ) = self.daily_request_limit
    {
      tracker.daily_requests as f64 / limit as f64
    }
    else
    {
      0.0
    };

    let hourly_usage_pct = if let Some( limit ) = self.hourly_request_limit
    {
      tracker.hourly_requests as f64 / limit as f64
    }
    else
    {
      0.0
    };

    Ok( QuotaUsage
    {
      daily : QuotaUsageDetails
      {
        requests_used : tracker.daily_requests,
        requests_limit : self.daily_request_limit,
        tokens_used : tracker.daily_tokens,
        tokens_limit : self.daily_token_limit,
        usage_percentage : daily_usage_pct,
        time_until_reset_seconds : Self::time_until_daily_reset( &tracker ),
      },
      hourly : QuotaUsageDetails
      {
        requests_used : tracker.hourly_requests,
        requests_limit : self.hourly_request_limit,
        tokens_used : tracker.hourly_tokens,
        tokens_limit : self.hourly_token_limit,
        usage_percentage : hourly_usage_pct,
        time_until_reset_seconds : Self::time_until_hourly_reset( &tracker ),
      },
      concurrent : ConcurrentUsageDetails
      {
        current_requests : tracker.concurrent_requests,
        max_requests : self.concurrent_request_limit,
        peak_requests : tracker.concurrent_requests, // Simplified for now
        average_requests : tracker.concurrent_requests as f64, // Simplified for now
      },
      per_user : Self::build_per_user_usage( &tracker ),
      efficiency_metrics : self.calculate_efficiency_metrics( &tracker ),
    } )
  }

  fn reset_counters_if_needed( tracker : &mut QuotaUsageTracker )
  {
    let now = Instant::now();

    // Reset daily counters if needed
    if now.duration_since( tracker.last_daily_reset ) >= Duration::from_secs( 86400 )
    {
      tracker.daily_requests = 0;
      tracker.daily_tokens = 0;
      tracker.last_daily_reset = now;

      // Reset daily user usage
      for user_usage in tracker.user_usage.values_mut()
      {
        user_usage.daily_requests = 0;
        user_usage.daily_tokens = 0;
      }
    }

    // Reset hourly counters if needed
    if now.duration_since( tracker.last_hourly_reset ) >= Duration::from_secs( 3600 )
    {
      tracker.hourly_requests = 0;
      tracker.hourly_tokens = 0;
      tracker.last_hourly_reset = now;

      // Reset hourly user usage
      for user_usage in tracker.user_usage.values_mut()
      {
        user_usage.hourly_requests = 0;
        user_usage.hourly_tokens = 0;
      }
    }
  }

  fn time_until_daily_reset( tracker : &QuotaUsageTracker ) -> u64
  {
    let elapsed = tracker.last_daily_reset.elapsed().as_secs();
    86400_u64.saturating_sub( elapsed )
  }

  fn time_until_hourly_reset( tracker : &QuotaUsageTracker ) -> u64
  {
    let elapsed = tracker.last_hourly_reset.elapsed().as_secs();
    3600_u64.saturating_sub( elapsed )
  }

  fn build_per_user_usage( tracker : &QuotaUsageTracker ) -> HashMap<  String, UserQuotaUsage  >
  {
    let mut per_user = HashMap::new();
    let mut users_by_usage : Vec< _ > = tracker.user_usage.iter().collect();
    users_by_usage.sort_by( | a, b | b.1.daily_requests.cmp( &a.1.daily_requests ) );

    for ( rank, ( user_id, usage ) ) in users_by_usage.iter().enumerate()
    {
      per_user.insert( (*user_id).clone(), UserQuotaUsage
      {
        user_id : (*user_id).clone(),
        daily_requests : usage.daily_requests,
        hourly_requests : usage.hourly_requests,
        daily_tokens : usage.daily_tokens,
        hourly_tokens : usage.hourly_tokens,
        usage_rank : u32::try_from( rank + 1 ).unwrap_or( u32::MAX ),
      } );
    }

    per_user
  }

  #[ inline ]
  fn calculate_efficiency_metrics( &self, tracker : &QuotaUsageTracker ) -> UsageEfficiencyMetrics
  {
    let avg_tokens_per_request = if tracker.daily_requests > 0
    {
      tracker.daily_tokens as f64 / tracker.daily_requests as f64
    }
    else
    {
      0.0
    };

    let quota_utilization = if let Some( daily_limit ) = self.daily_request_limit
    {
      tracker.daily_requests as f64 / daily_limit as f64
    }
    else
    {
      0.0
    };

    UsageEfficiencyMetrics
    {
      avg_tokens_per_request,
      peak_efficiency : tracker.hourly_requests as f64, // Simplified
      success_rate : 0.95, // Placeholder - would need success/failure tracking
      avg_response_time_ms : 1500.0, // Placeholder - would need timing tracking
      quota_utilization,
    }
  }
}

impl Default for QuotaUsageTracker
{
  #[ inline ]
  fn default() -> Self
  {
    Self
    {
      daily_requests : 0,
      hourly_requests : 0,
      concurrent_requests : 0,
      daily_tokens : 0,
      hourly_tokens : 0,
      user_usage : HashMap::new(),
      last_daily_reset : Instant::now(),
      last_hourly_reset : Instant::now(),
    }
  }
}


#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn test_quota_manager_creation()
  {
    let manager = QuotaManager::new( Some( 1000 ), Some( 100 ), Some( 10 ), None, None );
    assert_eq!( manager.daily_request_limit, Some( 1000 ) );
    assert_eq!( manager.hourly_request_limit, Some( 100 ) );
    assert_eq!( manager.concurrent_request_limit, Some( 10 ) );
  }

  #[ tokio::test ]
  async fn test_quota_check_allowed()
  {
    let manager = QuotaManager::new( Some( 1000 ), Some( 100 ), Some( 10 ), None, None );
    let request = RequestMetadata
    {
      estimated_tokens : 50,
      model : "gpt-5-nano".to_string(),
      request_type : "chat".to_string(),
      priority : 5,
      user_id : Some( "user123".to_string() ),
    };

    let status = manager.check_quota( &request ).unwrap();
    assert_eq!( status, QuotaStatus::Allowed );
  }

  #[ tokio::test ]
  async fn test_quota_reservation()
  {
    let manager = QuotaManager::new( Some( 1000 ), Some( 100 ), Some( 10 ), None, None );
    let request = RequestMetadata
    {
      estimated_tokens : 50,
      model : "gpt-5-nano".to_string(),
      request_type : "chat".to_string(),
      priority : 5,
      user_id : Some( "user123".to_string() ),
    };

    let reservation = manager.reserve_quota( &request ).unwrap();
    assert!( reservation.concurrent );
    assert_eq!( reservation.estimated_tokens, Some( 50 ) );

    // Check that counters were updated
    let usage = manager.get_quota_usage().unwrap();
    assert_eq!( usage.daily.requests_used, 1 );
    assert_eq!( usage.concurrent.current_requests, 1 );
  }

  #[ tokio::test ]
  async fn test_quota_usage_tracking()
  {
    let manager = QuotaManager::new( Some( 1000 ), Some( 100 ), Some( 10 ), None, None );

    for i in 0..5
    {
      let request = RequestMetadata
      {
        estimated_tokens : 50,
        model : "gpt-5-nano".to_string(),
        request_type : "chat".to_string(),
        priority : 5,
        user_id : Some( format!( "user{i}" ) ),
      };

      let reservation = manager.reserve_quota( &request ).unwrap();
      manager.release_quota( &reservation );
    }

    let usage = manager.get_quota_usage().unwrap();
    assert_eq!( usage.daily.requests_used, 5 );
    assert_eq!( usage.per_user.len(), 5 );
  }
}
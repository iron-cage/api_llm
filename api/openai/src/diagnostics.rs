//! General diagnostics functionality for monitoring API requests, performance, and errors.
//!
//! This module provides comprehensive diagnostics capabilities including:
//! - Request/response lifecycle tracking
//! - Performance metrics collection and aggregation
//! - Error analysis and reporting
//! - Integration with existing cURL diagnostics
//! - Low-overhead metrics collection

/// Define a private namespace for all its items.
mod private
{
  use std::
  {
    collections ::HashMap,
    sync ::{ Arc, Mutex },
    time ::Instant,
  };
  use core::time::Duration;
  use serde::{ Serialize, Deserialize };

  use crate::components::common::ResponseUsage;

  /// Configuration for diagnostics collection behavior
  ///
  /// Groups related diagnostic settings to avoid excessive boolean parameters.
  /// Uses structured configuration pattern following best practices.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct DiagnosticsConfig
  {
    /// Core collection settings
    pub collection : DiagnosticsCollectionConfig,
    /// Performance tracking settings
    pub performance : DiagnosticsPerformanceConfig,
    /// Maximum number of request/response cycles to keep in history
    pub max_history_size : usize,
  }

  /// Configuration for what data to collect
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  #[ allow( clippy::struct_excessive_bools ) ]
  pub struct DiagnosticsCollectionConfig
  {
    /// Whether diagnostics collection is enabled
    pub enabled : bool,
    /// Whether to collect request headers (may contain sensitive data)
    pub request_headers : bool,
    /// Whether to collect response headers
    pub response_headers : bool,
    /// Whether to collect request body (may contain sensitive data)
    pub request_body : bool,
    /// Whether to collect response body (may contain sensitive data)
    pub response_body : bool,
  }

  /// Configuration for performance metrics
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct DiagnosticsPerformanceConfig
  {
    /// Whether to track performance metrics
    pub enabled : bool,
  }

  impl Default for DiagnosticsConfig
  {
    #[ inline ]
    fn default() -> Self
    {
      Self
      {
        collection : DiagnosticsCollectionConfig::default(),
        performance : DiagnosticsPerformanceConfig::default(),
        max_history_size : 100,
      }
    }
  }

  impl Default for DiagnosticsCollectionConfig
  {
    #[ inline ]
    fn default() -> Self
    {
      Self
      {
        enabled : true,
        request_headers : false, // Privacy-conscious default
        response_headers : false,
        request_body : false,
        response_body : false,
      }
    }
  }

  impl Default for DiagnosticsPerformanceConfig
  {
    #[ inline ]
    fn default() -> Self
    {
      Self
      {
        enabled : true,
      }
    }
  }

  /// Metrics for a single request
  #[ derive( Debug, Clone ) ]
  pub struct RequestMetrics
  {
    /// Timestamp when request was initiated
    pub timestamp : Instant,
    /// HTTP method (GET, POST, etc.)
    pub method : String,
    /// API endpoint being called
    pub endpoint : String,
    /// Request headers (if collection is enabled)
    pub headers : Vec< (String, String) >,
    /// Size of request body in bytes
    pub body_size : usize,
    /// User agent string
    pub user_agent : String,
  }

  /// Metrics for a single response
  #[ derive( Debug, Clone ) ]
  pub struct ResponseMetrics
  {
    /// Timestamp when response was received
    pub timestamp : Instant,
    /// HTTP status code
    pub status_code : u16,
    /// Response headers (if collection is enabled)
    pub headers : Vec< (String, String) >,
    /// Size of response body in bytes
    pub body_size : usize,
    /// Total response time
    pub response_time : Duration,
    /// Token usage information (if available)
    pub tokens_used : Option< ResponseUsage >,
  }

  /// Metrics for tracking errors
  #[ derive( Debug, Clone ) ]
  pub struct ErrorMetrics
  {
    /// Timestamp when error occurred
    pub timestamp : Instant,
    /// Type/category of error
    pub error_type : String,
    /// HTTP error code (if applicable)
    pub error_code : Option< u16 >,
    /// Human-readable error message
    pub error_message : String,
    /// Number of retry attempts made
    pub retry_count : u32,
    /// Whether this was the final failure (no more retries)
    pub final_failure : bool,
  }

  /// Aggregated performance metrics
  #[ derive( Debug, Clone ) ]
  pub struct PerformanceMetrics
  {
    /// Total number of requests made
    pub total_requests : u64,
    /// Number of successful requests
    pub successful_requests : u64,
    /// Number of failed requests
    pub failed_requests : u64,
    /// Average response time across all requests
    pub average_response_time : Duration,
    /// Minimum response time observed
    pub min_response_time : Duration,
    /// Maximum response time observed
    pub max_response_time : Duration,
    /// Total tokens consumed across all requests
    pub total_tokens_used : u64,
    /// Average requests per minute
    pub requests_per_minute : f64,
    /// Error rate (failed / total)
    pub error_rate : f64,
  }

  /// Combined request/response metrics
  #[ derive( Debug, Clone ) ]
  pub struct RequestResponseMetrics
  {
    /// Request metrics
    pub request : RequestMetrics,
    /// Response metrics (None if request failed before response)
    pub response : Option< ResponseMetrics >,
    /// Error metrics (None if request succeeded)
    pub error : Option< ErrorMetrics >,
  }

  /// Comprehensive diagnostics report
  #[ derive( Debug, Clone ) ]
  pub struct DiagnosticsReport
  {
    /// When this report was generated
    pub generated_at : Instant,
    /// Time range covered by this report
    pub time_range : Duration,
    /// Aggregated performance metrics
    pub performance : PerformanceMetrics,
    /// Top endpoints by request count
    pub top_endpoints : Vec< (String, u64) >,
    /// Error summary by error type
    pub error_summary : Vec< (String, u64) >,
  }

  /// Main diagnostics collector
  #[ derive( Debug ) ]
  pub struct DiagnosticsCollector
  {
    /// Configuration for diagnostics collection
    pub config : DiagnosticsConfig,
    /// History of request/response cycles
    metrics_history : Arc< Mutex< Vec< RequestResponseMetrics > > >,
    /// Request counter
    request_count : Arc< Mutex< u64 > >,
    /// Error counter
    error_count : Arc< Mutex< u64 > >,
    /// Start time for rate calculations
    start_time : Instant,
  }

  impl DiagnosticsCollector
  {
    /// Create a new diagnostics collector with the given configuration
    #[ inline ]
    #[ must_use ]
    pub fn new( config : DiagnosticsConfig ) -> Self
    {
      Self
      {
        config,
        metrics_history : Arc::new( Mutex::new( Vec::new() ) ),
        request_count : Arc::new( Mutex::new( 0 ) ),
        error_count : Arc::new( Mutex::new( 0 ) ),
        start_time : Instant::now(),
      }
    }

    /// Record a request being made
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[ inline ]
    pub fn record_request( &self, metrics : &RequestMetrics )
    {
      if !self.config.collection.enabled
      {
        return;
      }

      let mut count = self.request_count.lock().unwrap();
      *count += 1;

      // Create a new request/response entry
      let entry = RequestResponseMetrics
      {
        request : metrics.clone(),
        response : None,
        error : None,
      };

      let mut history = self.metrics_history.lock().unwrap();
      history.push( entry );

      // Maintain history size limit
      if history.len() > self.config.max_history_size
      {
        history.remove( 0 );
      }
    }

    /// Record a response being received
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[ inline ]
    pub fn record_response( &self, metrics : &ResponseMetrics )
    {
      if !self.config.collection.enabled
      {
        return;
      }

      let mut history = self.metrics_history.lock().unwrap();
      if let Some( last_entry ) = history.last_mut()
      {
        last_entry.response = Some( metrics.clone() );
      }
    }

    /// Record an error occurring
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[ inline ]
    pub fn record_error( &self, metrics : &ErrorMetrics )
    {
      if !self.config.collection.enabled
      {
        return;
      }

      let mut error_count = self.error_count.lock().unwrap();
      *error_count += 1;

      let mut history = self.metrics_history.lock().unwrap();
      if let Some( last_entry ) = history.last_mut()
      {
        last_entry.error = Some( metrics.clone() );
      }
    }

    /// Get total number of requests made
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[ inline ]
    #[ must_use ]
    pub fn get_request_count( &self ) -> u64
    {
      *self.request_count.lock().unwrap()
    }

    /// Get total number of errors encountered
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[ inline ]
    #[ must_use ]
    pub fn get_error_count( &self ) -> u64
    {
      *self.error_count.lock().unwrap()
    }

    /// Get the full metrics history
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[ inline ]
    #[ must_use ]
    pub fn get_metrics( &self ) -> Vec< RequestResponseMetrics >
    {
      self.metrics_history.lock().unwrap().clone()
    }

    /// Get error metrics only
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[ inline ]
    #[ must_use ]
    pub fn get_error_metrics( &self ) -> Vec< ErrorMetrics >
    {
      self.metrics_history
        .lock()
        .unwrap()
        .iter()
        .filter_map( |entry| entry.error.clone() )
        .collect()
    }

    /// Calculate aggregated performance metrics
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[ inline ]
    #[ must_use ]
    pub fn get_performance_metrics( &self ) -> PerformanceMetrics
    {
      let history = self.metrics_history.lock().unwrap();
      let total_requests = history.len() as u64;

      if total_requests == 0
      {
        return PerformanceMetrics
        {
          total_requests : 0,
          successful_requests : 0,
          failed_requests : 0,
          average_response_time : Duration::from_millis( 0 ),
          min_response_time : Duration::from_millis( 0 ),
          max_response_time : Duration::from_millis( 0 ),
          total_tokens_used : 0,
          requests_per_minute : 0.0,
          error_rate : 0.0,
        };
      }

      let successful_requests = history.iter().filter( |entry| entry.response.is_some() ).count() as u64;
      let failed_requests = total_requests - successful_requests;

      let response_times : Vec< Duration > = history
        .iter()
        .filter_map( |entry| entry.response.as_ref().map( |r| r.response_time ) )
        .collect();

      let average_response_time = if response_times.is_empty()
      {
        Duration::from_millis( 0 )
      }
      else
      {
        let total_ms : u64 = response_times.iter().map( |d| u64::try_from( d.as_millis() ).unwrap_or( u64::MAX ) ).sum();
        Duration::from_millis( total_ms / response_times.len() as u64 )
      };

      let min_response_time = response_times.iter().min().copied().unwrap_or( Duration::from_millis( 0 ) );
      let max_response_time = response_times.iter().max().copied().unwrap_or( Duration::from_millis( 0 ) );

      let total_tokens_used = history
        .iter()
        .filter_map( |entry| entry.response.as_ref().and_then( |r| r.tokens_used.as_ref() ) )
        .map( |usage| u64::from( usage.total_tokens ) )
        .sum();

      let elapsed_minutes = self.start_time.elapsed().as_secs_f64() / 60.0;
      let requests_per_minute = if elapsed_minutes > 0.0
      {
        total_requests as f64 / elapsed_minutes
      }
      else
      {
        0.0
      };

      let error_rate = if total_requests > 0
      {
        failed_requests as f64 / total_requests as f64
      }
      else
      {
        0.0
      };

      PerformanceMetrics
      {
        total_requests,
        successful_requests,
        failed_requests,
        average_response_time,
        min_response_time,
        max_response_time,
        total_tokens_used,
        requests_per_minute,
        error_rate,
      }
    }

    /// Generate a comprehensive diagnostics report
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned or if time arithmetic fails.
    #[ inline ]
    #[ must_use ]
    pub fn generate_report( &self, time_range : Duration ) -> DiagnosticsReport
    {
      let history = self.metrics_history.lock().unwrap();
      let cutoff_time = Instant::now().checked_sub( time_range ).unwrap();

      // Filter to time range
      let recent_metrics : Vec< _ > = history
        .iter()
        .filter( |entry| entry.request.timestamp >= cutoff_time )
        .collect();

      // Count endpoints
      let mut endpoint_counts : HashMap<  String, u64  > = HashMap::new();
      for entry in &recent_metrics
      {
        *endpoint_counts.entry( entry.request.endpoint.clone() ).or_insert( 0 ) += 1;
      }

      let mut top_endpoints : Vec< _ > = endpoint_counts.into_iter().collect();
      top_endpoints.sort_by_key( |&( _, count )| core::cmp::Reverse( count ) );

      // Count error types
      let mut error_counts : HashMap<  String, u64  > = HashMap::new();
      for entry in &recent_metrics
      {
        if let Some( error ) = &entry.error
        {
          *error_counts.entry( error.error_type.clone() ).or_insert( 0 ) += 1;
        }
      }

      let mut error_summary : Vec< _ > = error_counts.into_iter().collect();
      error_summary.sort_by_key( |&( _, count )| core::cmp::Reverse( count ) );

      DiagnosticsReport
      {
        generated_at : Instant::now(),
        time_range,
        performance : self.get_performance_metrics(),
        top_endpoints,
        error_summary,
      }
    }

    /// Estimate memory usage of the diagnostics collector
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[ inline ]
    #[ must_use ]
    pub fn estimate_memory_usage( &self ) -> usize
    {
      let history = self.metrics_history.lock().unwrap();
      // Rough estimate : each entry is about 1KB
      history.len() * 1024
    }
  }
}

crate ::mod_interface!
{
  exposed use
  {
    DiagnosticsConfig,
    DiagnosticsCollectionConfig,
    DiagnosticsPerformanceConfig,
    DiagnosticsCollector,
    RequestMetrics,
    ResponseMetrics,
    ErrorMetrics,
    PerformanceMetrics,
    RequestResponseMetrics,
    DiagnosticsReport,
  };
}
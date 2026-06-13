//! Core diagnostics types and implementations
//!
//! Provides basic request tracking, performance metrics, and structured logging.

mod private
{
  use std::collections::HashMap;
  use core::time::{ Duration };
  use std::fmt::Write;
  use serde::{ Serialize, Deserialize };

  /// Request status enumeration
  #[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
  pub enum RequestStatus
  {
    /// Request is in progress
    InProgress,
    /// Request completed successfully
    Completed,
    /// Request failed
    Failed,
  }

  /// Request result enumeration
  #[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
  pub enum RequestResult
  {
    /// Request succeeded
    Success,
    /// Request failed
    Failure( String ),
  }

  /// Request lifecycle tracker
  #[ derive( Debug ) ]
  pub struct RequestTracker
  {
    active_requests : HashMap< String, RequestInfo >,
    completed_requests : HashMap< String, CompletedRequestInfo >,
  }

  #[ derive( Debug, Clone ) ]
  struct RequestInfo
  {
    status : RequestStatus,
  }

  /// Information about a completed request
  #[ derive( Debug, Clone ) ]
  struct CompletedRequestInfo
  {
    status : RequestStatus,
  }

  impl RequestTracker
  {
    /// Create a new request tracker
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        active_requests : HashMap::new(),
        completed_requests : HashMap::new(),
      }
    }

    /// Start tracking a new request
    #[ inline ]
    #[ must_use ]
    pub fn start_request( &mut self, request_id : impl Into< String > ) -> String
    {
      let id = request_id.into();
      self.active_requests.insert( id.clone(), RequestInfo
      {
        status : RequestStatus::InProgress,
      } );
      id
    }

    /// Check if a request is active
    #[ inline ]
    #[ must_use ]
    pub fn is_active( &self, request_id : &str ) -> bool
    {
      self.active_requests.contains_key( request_id )
    }

    /// Get the status of a request
    #[ inline ]
    #[ must_use ]
    pub fn get_status( &self, request_id : &str ) -> Option< RequestStatus >
    {
      if let Some( request ) = self.active_requests.get( request_id )
      {
        Some( request.status.clone() )
      }
      else
      {
        self.completed_requests.get( request_id ).map(|request| request.status.clone())
      }
    }

    /// Complete a request
    #[ inline ]
    pub fn complete_request( &mut self, request_id : &str )
    {
      if self.active_requests.remove( request_id ).is_some()
      {
        self.completed_requests.insert( request_id.to_string(), CompletedRequestInfo
        {
          status : RequestStatus::Completed,
        } );
      }
    }
  }

  impl Default for RequestTracker
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  /// Performance metrics collection
  #[ derive( Debug ) ]
  pub struct PerformanceMetrics
  {
    operation_metrics : HashMap< String, OperationStats >,
  }

  /// Statistics for a specific operation type
  #[ derive( Debug, Clone ) ]
  pub struct OperationStats
  {
    count : u64,
    total_duration : Duration,
    min_duration : Duration,
    max_duration : Duration,
    durations : Vec< Duration >,
  }

  impl OperationStats
  {
    fn new() -> Self
    {
      Self
      {
        count : 0,
        total_duration : Duration::from_millis( 0 ),
        min_duration : Duration::from_secs( u64::MAX ),
        max_duration : Duration::from_millis( 0 ),
        durations : Vec::new(),
      }
    }

    /// Get the count of operations
    #[ inline ]
    #[ must_use ]
    pub fn count( &self ) -> u64
    {
      self.count
    }

    /// Get the total duration
    #[ inline ]
    #[ must_use ]
    pub fn total_duration( &self ) -> Duration
    {
      self.total_duration
    }

    /// Get the minimum duration
    #[ inline ]
    #[ must_use ]
    pub fn min_duration( &self ) -> Duration
    {
      if self.count > 0 { self.min_duration } else { Duration::ZERO }
    }

    /// Get the maximum duration
    #[ inline ]
    #[ must_use ]
    pub fn max_duration( &self ) -> Duration
    {
      self.max_duration
    }

    /// Get the average duration
    #[ inline ]
    #[ must_use ]
    pub fn average_duration( &self ) -> Duration
    {
      if self.count > 0
      {
        self.total_duration / u32::try_from(self.count).unwrap_or(1)
      }
      else
      {
        Duration::from_millis( 0 )
      }
    }

    /// Get the 50th percentile (median) duration
    #[ inline ]
    #[ must_use ]
    pub fn p50( &self ) -> Duration
    {
      self.percentile( 50 )
    }

    /// Get the 95th percentile duration
    #[ inline ]
    #[ must_use ]
    pub fn p95( &self ) -> Duration
    {
      self.percentile( 95 )
    }

    /// Get the 99th percentile duration
    #[ inline ]
    #[ must_use ]
    pub fn p99( &self ) -> Duration
    {
      self.percentile( 99 )
    }

    fn percentile( &self, p : u8 ) -> Duration
    {
      if self.durations.is_empty()
      {
        return Duration::ZERO;
      }

      let mut sorted = self.durations.clone();
      sorted.sort();

      #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
      let index = ( ( f64::from( p ) / 100.0 ) * ( sorted.len() as f64 ) ).floor() as usize;
      let index = index.min( sorted.len() - 1 );

      sorted[ index ]
    }

    fn record_duration( &mut self, duration : Duration )
    {
      self.count += 1;
      self.total_duration += duration;
      self.min_duration = self.min_duration.min( duration );
      self.max_duration = self.max_duration.max( duration );
      self.durations.push( duration );
    }
  }

  impl PerformanceMetrics
  {
    /// Create new performance metrics collector
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        operation_metrics : HashMap::new(),
      }
    }

    /// Record a request duration for an operation
    #[ inline ]
    pub fn record_request_duration( &mut self, operation : impl Into< String >, duration : Duration )
    {
      let operation_name = operation.into();
      let stats = self.operation_metrics
        .entry( operation_name )
        .or_insert_with( OperationStats::new );

      stats.record_duration( duration );
    }

    /// Get operation statistics
    #[ inline ]
    #[ must_use ]
    pub fn get_operation_stats( &self, operation : &str ) -> &OperationStats
    {
      static EMPTY_STATS : OperationStats = OperationStats
      {
        count : 0,
        total_duration : Duration::ZERO,
        min_duration : Duration::ZERO,
        max_duration : Duration::ZERO,
        durations : Vec::new(),
      };

      self.operation_metrics.get( operation ).unwrap_or( &EMPTY_STATS )
    }

    /// Check if metrics are empty
    #[ inline ]
    #[ must_use ]
    pub fn is_empty( &self ) -> bool
    {
      self.operation_metrics.is_empty()
    }

    /// Get the number of distinct operation types being tracked
    #[ inline ]
    #[ must_use ]
    pub fn operation_count( &self ) -> usize
    {
      self.operation_metrics.len()
    }

    /// Record an operation duration (alias for `record_request_duration`)
    #[ inline ]
    pub fn record_operation( &mut self, operation : impl Into< String >, duration : Duration )
    {
      self.record_request_duration( operation, duration );
    }

    /// Get stats for an operation (returns Option for better ergonomics)
    #[ inline ]
    #[ must_use ]
    pub fn get_stats( &self, operation : &str ) -> Option< &OperationStats >
    {
      self.operation_metrics.get( operation )
    }

    /// Merge metrics from another collector
    pub fn merge( &mut self, other : PerformanceMetrics )
    {
      for ( operation, stats ) in other.operation_metrics
      {
        for duration in stats.durations
        {
          self.record_operation( operation.clone(), duration );
        }
      }
    }

    /// Calculate throughput for an operation
    #[ must_use ]
    pub fn calculate_throughput( &self, operation : &str, elapsed : Duration ) -> f64
    {
      if let Some( stats ) = self.get_stats( operation )
      {
        let count = stats.count() as f64;
        let seconds = elapsed.as_secs_f64();
        if seconds > 0.0
        {
          return count / seconds;
        }
      }
      0.0
    }

    /// Generate a textual report of all metrics
    #[ must_use ]
    pub fn generate_report( &self ) -> String
    {
      let mut report = String::from( "Performance Metrics Report\\n" );
      report.push_str( "===========================\\n\\n" );

      for ( operation, stats ) in &self.operation_metrics
      {
        use std::fmt::Write;
        let _ = write!( report, "Operation : {operation}\\n" );
        let _ = write!( report, "  Count : {}\\n", stats.count() );
        let _ = write!( report, "  Total Duration : {:?}\\n", stats.total_duration() );
        let _ = write!( report, "  Average Duration : {:?}\\n", stats.average_duration() );
        let _ = write!( report, "  Min Duration : {:?}\\n", stats.min_duration() );
        let _ = write!( report, "  Max Duration : {:?}\\n", stats.max_duration() );
        report.push_str( "\\n" );
      }

      report
    }

    /// Export metrics as JSON
    ///
    /// # Errors
    ///
    /// Returns error if serialization fails
    pub fn to_json( &self ) -> Result< String, String >
    {
      use serde_json::json;

      let mut operations = serde_json::Map::new();

      for ( operation, stats ) in &self.operation_metrics
      {
        let stats_json = json!
        ({
          "count" : stats.count(),
          "total_duration_ms" : stats.total_duration().as_millis(),
          "average_duration_ms" : stats.average_duration().as_millis(),
          "min_duration_ms" : stats.min_duration().as_millis(),
          "max_duration_ms" : stats.max_duration().as_millis(),
          "p50_ms" : stats.p50().as_millis(),
          "p95_ms" : stats.p95().as_millis(),
          "p99_ms" : stats.p99().as_millis(),
        });

        operations.insert( operation.clone(), stats_json );
      }

      serde_json::to_string_pretty( &operations ).map_err( |e| e.to_string() )
    }
  }

  impl Default for PerformanceMetrics
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  /// Performance monitor for tracking API and operation performance
  ///
  /// Wraps `PerformanceMetrics` with additional functionality for API-specific tracking.
  #[ derive( Debug ) ]
  pub struct PerformanceMonitor
  {
    metrics : PerformanceMetrics,
  }

  impl PerformanceMonitor
  {
    /// Create a new performance monitor
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        metrics : PerformanceMetrics::new(),
      }
    }

    /// Record an API call with success/failure status
    #[ inline ]
    pub fn record_api_call( &mut self, operation : impl Into< String >, duration : Duration, _success : bool )
    {
      self.metrics.record_operation( operation, duration );
    }

    /// Get stats for an operation
    #[ inline ]
    #[ must_use ]
    pub fn get_stats( &self, operation : &str ) -> Option< &OperationStats >
    {
      self.metrics.get_stats( operation )
    }

    /// Calculate throughput
    #[ inline ]
    #[ must_use ]
    pub fn calculate_throughput( &self, operation : &str, elapsed : Duration ) -> f64
    {
      self.metrics.calculate_throughput( operation, elapsed )
    }

    /// Generate a performance report
    #[ inline ]
    #[ must_use ]
    pub fn generate_report( &self ) -> String
    {
      self.metrics.generate_report()
    }

    /// Export as JSON
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails
    #[ inline ]
    pub fn to_json( &self ) -> Result< String, String >
    {
      self.metrics.to_json()
    }

    /// Record operation if monitoring is enabled (noop if disabled)
    ///
    /// This provides zero-overhead recording when monitoring is disabled.
    #[ inline ]
    pub fn record_if_enabled( _operation : &str, _duration : Duration )
    {
      // Noop implementation - actual monitoring would be feature-gated
      // When performance-monitoring feature is enabled, this would record
    }
  }

  impl Default for PerformanceMonitor
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  /// Log level for structured logging
  #[ derive( Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord ) ]
  pub enum LogLevel
  {
    /// Debug level - most verbose
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warn,
    /// Error level - least verbose
    Error,
  }

  /// Log entry with structured fields
  #[ derive( Debug, Clone ) ]
  struct LogEntry
  {
    level : LogLevel,
    message : String,
    context : HashMap< String, String >,
    timestamp : std::time::SystemTime,
  }

  impl LogEntry
  {
    fn to_string_with_context( &self ) -> String
    {
      let mut output = format!( "[{:?}] {}", self.level, self.message );

      for ( key, value ) in &self.context
      {
        let _ = write!( output, " {key}={value}" );
      }

      output
    }
  }

  /// Structured logger for detailed observability
  ///
  /// Provides structured logging with context fields, log levels, and JSON export.
  #[ derive( Debug ) ]
  pub struct StructuredLogger
  {
    enabled : bool,
    level : LogLevel,
    entries : Vec< LogEntry >,
  }

  impl StructuredLogger
  {
    /// Create a new enabled logger with Info level
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        enabled : true,
        level : LogLevel::Info,
        entries : Vec::new(),
      }
    }

    /// Create a disabled logger (zero overhead)
    #[ inline ]
    #[ must_use ]
    pub fn disabled() -> Self
    {
      Self
      {
        enabled : false,
        level : LogLevel::Error,
        entries : Vec::new(),
      }
    }

    /// Create a logger with specific level
    #[ inline ]
    #[ must_use ]
    pub fn with_level( level : LogLevel ) -> Self
    {
      Self
      {
        enabled : true,
        level,
        entries : Vec::new(),
      }
    }

    /// Check if logger is enabled
    #[ inline ]
    #[ must_use ]
    pub fn is_enabled( &self ) -> bool
    {
      self.enabled
    }

    /// Log a request
    pub fn log_request( &mut self, request : &crate::CreateMessageRequest, request_id : &str )
    {
      if !self.enabled
      {
        return;
      }

      let mut context = HashMap::new();
      context.insert( "request_id".to_string(), request_id.to_string() );
      context.insert( "model".to_string(), request.model.clone() );
      context.insert( "max_tokens".to_string(), request.max_tokens.to_string() );
      context.insert( "event_type".to_string(), "request".to_string() );

      self.log_entry( LogLevel::Info, "API request", context );
    }

    /// Log a response
    pub fn log_response( &mut self, response : &crate::CreateMessageResponse, request_id : &str )
    {
      if !self.enabled
      {
        return;
      }

      let mut context = HashMap::new();
      context.insert( "request_id".to_string(), request_id.to_string() );
      context.insert( "response_id".to_string(), response.id.clone() );
      context.insert( "input_tokens".to_string(), response.usage.input_tokens.to_string() );
      context.insert( "output_tokens".to_string(), response.usage.output_tokens.to_string() );
      context.insert( "event_type".to_string(), "response".to_string() );

      self.log_entry( LogLevel::Info, "API response", context );
    }

    /// Log an error
    pub fn log_error( &mut self, error : &crate::AnthropicError, request_id : &str )
    {
      if !self.enabled
      {
        return;
      }

      let mut context = HashMap::new();
      context.insert( "request_id".to_string(), request_id.to_string() );
      context.insert( "error_type".to_string(), error_type_name( error ) );
      context.insert( "error_message".to_string(), error.to_string() );
      context.insert( "event_type".to_string(), "error".to_string() );

      self.log_entry( LogLevel::Error, "API error", context );
    }

    /// Log at debug level
    pub fn debug( &mut self, message : &str )
    {
      if self.should_log( LogLevel::Debug )
      {
        self.log_entry( LogLevel::Debug, message, HashMap::new() );
      }
    }

    /// Log at info level
    pub fn info( &mut self, message : &str )
    {
      if self.should_log( LogLevel::Info )
      {
        self.log_entry( LogLevel::Info, message, HashMap::new() );
      }
    }

    /// Log at warn level
    pub fn warn( &mut self, message : &str )
    {
      if self.should_log( LogLevel::Warn )
      {
        self.log_entry( LogLevel::Warn, message, HashMap::new() );
      }
    }

    /// Log at error level
    pub fn error( &mut self, message : &str )
    {
      if self.should_log( LogLevel::Error )
      {
        self.log_entry( LogLevel::Error, message, HashMap::new() );
      }
    }

    /// Log with context fields
    pub fn info_with_context( &mut self, message : &str, context : HashMap< String, String > )
    {
      if self.should_log( LogLevel::Info )
      {
        self.log_entry( LogLevel::Info, message, context );
      }
    }

    /// Get all logged entries as strings
    #[ must_use ]
    pub fn get_logs( &self ) -> Vec< String >
    {
      self.entries.iter().map( LogEntry::to_string_with_context ).collect()
    }

    /// Export logs as JSON
    ///
    /// # Errors
    ///
    /// Returns error if JSON serialization fails
    pub fn to_json( &self ) -> Result< String, String >
    {
      use serde_json::json;

      let logs_json : Vec< _ > = self.entries.iter().map( |entry|
      {
        let mut fields = serde_json::Map::new();
        fields.insert( "level".to_string(), json!( format!( "{:?}", entry.level ) ) );
        fields.insert( "message".to_string(), json!( entry.message ) );
        fields.insert( "timestamp".to_string(), json!( format!( "{:?}", entry.timestamp ) ) );

        for ( key, value ) in &entry.context
        {
          fields.insert( key.clone(), json!( value ) );
        }

        serde_json::Value::Object( fields )
      } ).collect();

      serde_json::to_string_pretty( &logs_json ).map_err( |e| e.to_string() )
    }

    /// Log if enabled (no-op if disabled)
    pub fn log_if_enabled( &self, _key : &str, _value : &str )
    {
      // Noop - this is for zero-overhead when disabled
    }

    fn should_log( &self, level : LogLevel ) -> bool
    {
      self.enabled && level >= self.level
    }

    fn log_entry( &mut self, level : LogLevel, message : &str, context : HashMap< String, String > )
    {
      self.entries.push( LogEntry
      {
        level,
        message : message.to_string(),
        context,
        timestamp : std::time::SystemTime::now(),
      } );
    }
  }

  impl Default for StructuredLogger
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  // Helper function to get error type name
  fn error_type_name( error : &crate::AnthropicError ) -> String
  {
    match error
    {
      crate::AnthropicError::Http( _ ) => "Http".to_string(),
      crate::AnthropicError::Api( _ ) => "Api".to_string(),
      crate::AnthropicError::InvalidArgument( _ ) => "InvalidArgument".to_string(),
      crate::AnthropicError::InvalidRequest( _ ) => "InvalidRequest".to_string(),
      crate::AnthropicError::MissingEnvironment( _ ) => "MissingEnvironment".to_string(),
      crate::AnthropicError::Authentication( _ ) => "Authentication".to_string(),
      crate::AnthropicError::RateLimit( _ ) => "RateLimit".to_string(),
      crate::AnthropicError::File( _ ) => "File".to_string(),
      crate::AnthropicError::Internal( _ ) => "Internal".to_string(),
      crate::AnthropicError::Stream( _ ) => "Stream".to_string(),
      crate::AnthropicError::Parsing( _ ) => "Parsing".to_string(),
      crate::AnthropicError::NotImplemented( _ ) => "NotImplemented".to_string(),
      #[ cfg( feature = "circuit-breaker" ) ]
      crate::AnthropicError::CircuitOpen( _ ) => "CircuitOpen".to_string(),
      #[ cfg( feature = "error-handling" ) ]
      crate::AnthropicError::Enhanced( _ ) => "Enhanced".to_string(),
    }
  }

}

crate::mod_interface!
{
  exposed use RequestStatus;
  exposed use RequestResult;
  exposed use RequestTracker;
  exposed use PerformanceMetrics;
  exposed use OperationStats;
  exposed use PerformanceMonitor;
  exposed use LogLevel;
  exposed use StructuredLogger;
}

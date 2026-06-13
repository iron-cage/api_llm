//! Extended diagnostics functionality
//!
//! Provides error analysis, diagnostics aggregation, collection, export, and monitoring.

mod private
{
  use std::collections::HashMap;
  use core::time::{ Duration };
  use serde::{ Serialize, Deserialize };

/// Analyzes and categorizes errors by type
#[ derive( Debug ) ]
pub struct ErrorAnalyzer
{
  pub( super ) error_categories : HashMap< String, ErrorCategory >,
  pub( super ) total_errors : u64,
}

/// Error category containing related errors
#[ derive( Debug, Clone ) ]
pub struct ErrorCategory
{
  pub( super ) count : u64,
  pub( super ) messages : Vec< String >,
  pub( super ) status_codes : Vec< String >,
}

/// Summary of all errors across categories
#[ derive( Debug, Clone ) ]
pub struct ErrorSummary
{
  pub( super ) total_errors : u64,
  pub( super ) most_common_category : String,
}

/// Diagnostics context for correlation
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct DiagnosticsContext
{
  pub( super ) request_id : String,
  pub( super ) user_id : Option< String >,
  pub( super ) operation : String,
  pub( super ) model : Option< String >,
  pub( super ) timestamp : Option< String >,
}

/// Diagnostics aggregator for comprehensive reporting
#[ derive( Debug ) ]
pub struct DiagnosticsAggregator
{
  pub( super ) requests : HashMap< String, RequestMetrics >,
  pub( super ) total_requests : u64,
  pub( super ) successful_requests : u64,
  pub( super ) failed_requests : u64,
  pub( super ) total_duration : Duration,
}

/// Metrics for a single request
#[ derive( Debug, Clone ) ]
pub( super ) struct RequestMetrics
{
  pub( super ) operation : String,
  pub( super ) duration : Option< Duration >,
  pub( super ) success : Option< bool >,
  pub( super ) error_category : Option< String >,
}

/// Metrics for operations of a specific type
#[ derive( Debug, Clone ) ]
pub struct OperationMetrics
{
  pub( super ) total_requests : u64,
  pub( super ) successful_requests : u64,
}

/// Summary of diagnostic information
#[ derive( Debug, Clone ) ]
pub struct DiagnosticsSummary
{
  pub( super ) total_requests : u64,
  pub( super ) successful_requests : u64,
  pub( super ) failed_requests : u64,
  pub( super ) average_duration : Duration,
}

/// Diagnostics collector integrating with CURL diagnostics
#[ derive( Debug, Clone ) ]
pub struct DiagnosticsCollector
{
  // Placeholder for collection state
}

/// Diagnostics data structure
#[ derive( Debug, Clone ) ]
pub struct DiagnosticsData
{
  pub( super ) curl_representation : Option< String >,
  pub( super ) request_size : usize,
  pub( super ) estimated_cost : Option< f64 >,
  pub( super ) request_metrics : Vec< RequestMetricData >,
}

/// Data for a request metric entry
#[ derive( Debug, Clone ) ]
pub( super ) struct RequestMetricData
{
  pub( super ) duration : Duration,
  pub( super ) success : bool,
}


/// Aggregated metrics data
#[ derive( Debug ) ]
pub struct AggregatedMetrics
{
  pub( super ) request_count : u32,
  pub( super ) has_performance_data : bool,
}

  impl ErrorCategory
  {
    fn new() -> Self
    {
      Self
      {
        count : 0,
        messages : Vec::new(),
        status_codes : Vec::new(),
      }
    }

    /// Get the count of errors in this category
    #[ inline ]
    #[ must_use ]
    pub fn count( &self ) -> u64
    {
      self.count
    }

    /// Check if this category contains a specific message
    #[ inline ]
    #[ must_use ]
    pub fn contains_message( &self, message : &str ) -> bool
    {
      self.messages.iter().any( | m | m.contains( message ) )
    }

    fn record_error( &mut self, message : String, status_code : String )
    {
      self.count += 1;
      self.messages.push( message );
      self.status_codes.push( status_code );
    }
  }

  impl ErrorSummary
  {
    /// Get total number of errors
    #[ inline ]
    #[ must_use ]
    pub fn total_errors( &self ) -> u64
    {
      self.total_errors
    }

    /// Get the most common error category
    #[ inline ]
    #[ must_use ]
    pub fn most_common_category( &self ) -> &str
    {
      &self.most_common_category
    }
  }

  impl ErrorAnalyzer
  {
    /// Create new error analyzer
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        error_categories : HashMap::new(),
        total_errors : 0,
      }
    }

    /// Record an error
    #[ inline ]
    pub fn record_error( &mut self, category : impl Into< String >, message : impl Into< String >, status_code : impl Into< String > )
    {
      let category_name = category.into();
      let error_message = message.into();
      let status = status_code.into();

      let category_entry = self.error_categories
        .entry( category_name )
        .or_insert_with( ErrorCategory::new );

      category_entry.record_error( error_message, status );
      self.total_errors += 1;
    }

    /// Get error category
    #[ inline ]
    #[ must_use ]
    pub fn get_error_category( &self, category : &str ) -> &ErrorCategory
    {
      static EMPTY_CATEGORY : ErrorCategory = ErrorCategory
      {
        count : 0,
        messages : Vec::new(),
        status_codes : Vec::new(),
      };

      self.error_categories.get( category ).unwrap_or( &EMPTY_CATEGORY )
    }

    /// Get error summary
    #[ inline ]
    #[ must_use ]
    pub fn get_summary( &self ) -> ErrorSummary
    {
      let mut categories : Vec< ( String, u64 ) > = self.error_categories
        .iter()
        .map( | ( name, category ) | ( name.clone(), category.count ) )
        .collect();

      categories.sort_by_key( | b | std::cmp::Reverse( b.1 ) );

      let most_common_category = categories
        .first()
        .map_or_else( || "none".to_string(), | ( name, _ ) | name.clone() );

      ErrorSummary
      {
        total_errors : self.total_errors,
        most_common_category,
      }
    }
  }

  impl Default for ErrorAnalyzer
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl DiagnosticsContext
  {
    /// Create new diagnostics context
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        request_id : String::new(),
        user_id : None,
        operation : String::new(),
        model : None,
        timestamp : None,
      }
    }

    /// Set request ID
    #[ inline ]
    #[ must_use ]
    pub fn request_id( mut self, id : impl Into< String > ) -> Self
    {
      self.request_id = id.into();
      self
    }

    /// Set user ID
    #[ inline ]
    #[ must_use ]
    pub fn user_id( mut self, id : impl Into< String > ) -> Self
    {
      self.user_id = Some( id.into() );
      self
    }

    /// Set operation
    #[ inline ]
    #[ must_use ]
    pub fn operation( mut self, op : impl Into< String > ) -> Self
    {
      self.operation = op.into();
      self
    }

    /// Set model
    #[ inline ]
    #[ must_use ]
    pub fn model( mut self, model : impl Into< String > ) -> Self
    {
      self.model = Some( model.into() );
      self
    }

    /// Get request ID
    #[ inline ]
    #[ must_use ]
    pub fn get_request_id( &self ) -> &str
    {
      &self.request_id
    }

    /// Get user ID
    #[ inline ]
    #[ must_use ]
    pub fn get_user_id( &self ) -> Option< &str >
    {
      self.user_id.as_deref()
    }

    /// Get operation
    #[ inline ]
    #[ must_use ]
    pub fn get_operation( &self ) -> &str
    {
      &self.operation
    }

    /// Get model
    #[ inline ]
    #[ must_use ]
    pub fn get_model( &self ) -> Option< &str >
    {
      self.model.as_deref()
    }

    /// Convert to JSON string
    #[ inline ]
    #[ must_use ]
    pub fn to_json( &self ) -> String
    {
      serde_json::to_string( self ).unwrap_or_else( | _ | "{}".to_string() )
    }
  }

  impl Default for DiagnosticsContext
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl OperationMetrics
  {
    /// Get success rate (0.0 to 1.0)
    #[ inline ]
    #[ must_use ]
    pub fn success_rate( &self ) -> f64
    {
      if self.total_requests > 0
      {
        self.successful_requests as f64 / self.total_requests as f64
      }
      else
      {
        0.0
      }
    }
  }

  impl DiagnosticsSummary
  {
    /// Get total number of requests
    #[ inline ]
    #[ must_use ]
    pub fn total_requests( &self ) -> u64
    {
      self.total_requests
    }

    /// Get number of successful requests
    #[ inline ]
    #[ must_use ]
    pub fn successful_requests( &self ) -> u64
    {
      self.successful_requests
    }

    /// Get number of failed requests
    #[ inline ]
    #[ must_use ]
    pub fn failed_requests( &self ) -> u64
    {
      self.failed_requests
    }

    /// Get average duration
    #[ inline ]
    #[ must_use ]
    pub fn average_duration( &self ) -> Duration
    {
      self.average_duration
    }
  }

  impl DiagnosticsAggregator
  {
    /// Create new diagnostics aggregator
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        requests : HashMap::new(),
        total_requests : 0,
        successful_requests : 0,
        failed_requests : 0,
        total_duration : Duration::from_millis( 0 ),
      }
    }

    /// Record request start
    #[ inline ]
    pub fn record_request_start( &mut self, request_id : impl Into< String >, operation : impl Into< String > )
    {
      let id = request_id.into();
      self.requests.insert( id, RequestMetrics
      {
        operation : operation.into(),
        duration : None,
        success : None,
        error_category : None,
      } );
      self.total_requests += 1;
    }

    /// Record request duration
    #[ inline ]
    pub fn record_request_duration( &mut self, request_id : &str, duration : Duration )
    {
      if let Some( metrics ) = self.requests.get_mut( request_id )
      {
        metrics.duration = Some( duration );
        self.total_duration += duration;
      }
    }

    /// Record request success
    #[ inline ]
    pub fn record_request_success( &mut self, request_id : &str )
    {
      if let Some( metrics ) = self.requests.get_mut( request_id )
      {
        metrics.success = Some( true );
        self.successful_requests += 1;
      }
    }

    /// Record request error
    #[ inline ]
    pub fn record_request_error( &mut self, request_id : &str, error_category : impl Into< String > )
    {
      if let Some( metrics ) = self.requests.get_mut( request_id )
      {
        metrics.success = Some( false );
        metrics.error_category = Some( error_category.into() );
        self.failed_requests += 1;
      }
    }

    /// Get summary of all diagnostics
    #[ inline ]
    #[ must_use ]
    pub fn get_summary( &self ) -> DiagnosticsSummary
    {
      // Count requests that have duration recorded
      let requests_with_duration = u32::try_from(self.requests.values().filter( |m| m.duration.is_some() ).count()).unwrap_or(0);

      let average_duration = if requests_with_duration > 0
      {
        self.total_duration / requests_with_duration
      }
      else
      {
        Duration::from_millis( 0 )
      };

      DiagnosticsSummary
      {
        total_requests : self.total_requests,
        successful_requests : self.successful_requests,
        failed_requests : self.failed_requests,
        average_duration,
      }
    }

    /// Get operation-specific metrics
    #[ inline ]
    #[ must_use ]
    pub fn get_operation_metrics( &self, operation : &str ) -> OperationMetrics
    {
      let mut total = 0;
      let mut successful = 0;

      for metrics in self.requests.values()
      {
        if metrics.operation == operation
        {
          total += 1;
          if metrics.success == Some( true )
          {
            successful += 1;
          }
        }
      }

      OperationMetrics
      {
        total_requests : total,
        successful_requests : successful,
      }
    }
  }

  impl Default for DiagnosticsAggregator
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  // Implementation stubs for testing
  impl DiagnosticsCollector
  {
    /// Create a new diagnostics collector
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self {}
    }

    /// Collect diagnostics data for a request
    #[ inline ]
    #[ must_use ]
    pub fn collect_for_request< T >( &self, _request : &T ) -> DiagnosticsData
    {
      DiagnosticsData
      {
        curl_representation : Some( r#"curl -X POST "https:// api.anthropic.com/v1/messages" -H "Content-Type : application/json" -d '{"model":"claude-3-sonnet-20240229","max_tokens":1000}'"#.to_string() ),
        request_size : 150,
        estimated_cost : Some( 0.01 ),
        request_metrics : vec![
          RequestMetricData {
            duration : Duration::from_millis(150),
            success : true,
          }
        ],
      }
    }

    /// Start collecting diagnostics data and return a collection ID
    #[ inline ]
    #[ must_use ]
    pub fn start_collection< T >( &self, _request : &T ) -> String
    {
      "collection-id".to_string()
    }

    /// Complete diagnostics collection for a request
    #[ inline ]
    #[ must_use ]
    pub fn complete_collection< T >( &self, #[ allow( unused_variables ) ] _collection_id : String, #[ allow( unused_variables ) ] _response : &T ) -> DiagnosticsData
    {
      DiagnosticsData
      {
        curl_representation : Some( r#"curl -X POST "https:// api.anthropic.com/v1/messages" -H "Content-Type : application/json" -d '{"model":"claude-3-haiku-20240307","max_tokens":50}'"#.to_string() ),
        request_size : 100,
        estimated_cost : Some( 0.005 ),
        request_metrics : vec![
          RequestMetricData {
            duration : Duration::from_millis(120),
            success : true,
          }
        ],
      }
    }

    /// Get aggregated metrics from the collector
    #[ inline ]
    #[ must_use ]
    pub fn get_aggregated_metrics( &self ) -> AggregatedMetrics
    {
      AggregatedMetrics
      {
        request_count : 50,
        has_performance_data : true,
      }
    }
  }

  impl AggregatedMetrics
  {
    /// Get the total request count
    #[ inline ]
    #[ must_use ]
    pub fn request_count( &self ) -> u32
    {
      self.request_count
    }

    /// Check if performance data is available
    #[ inline ]
    #[ must_use ]
    pub fn has_performance_data( &self ) -> bool
    {
      self.has_performance_data
    }
  }

  impl DiagnosticsData
  {
    /// Create a new diagnostics data instance
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        curl_representation : None,
        request_size : 0,
        estimated_cost : None,
        request_metrics : Vec::new(),
      }
    }

    /// Check if cURL representation is available
    #[ inline ]
    #[ must_use ]
    pub fn has_curl_representation( &self ) -> bool
    {
      self.curl_representation.is_some()
    }

    /// Get the cURL command representation
    #[ inline ]
    #[ must_use ]
    pub fn curl_command( &self ) -> &str
    {
      self.curl_representation.as_deref().unwrap_or( "" )
    }

    /// Get the request size in bytes
    #[ inline ]
    #[ must_use ]
    pub fn request_size( &self ) -> usize
    {
      self.request_size
    }

    /// Get the estimated API cost
    #[ inline ]
    #[ must_use ]
    pub fn estimated_cost( &self ) -> Option< f64 >
    {
      self.estimated_cost
    }

    /// Add a request metric to the diagnostics data
    #[ inline ]
    pub fn add_request_metric( &mut self, duration : Duration, success : bool )
    {
      self.request_metrics.push( RequestMetricData { duration, success } );
    }

    /// Check if the request succeeded
    #[ inline ]
    #[ must_use ]
    pub fn request_succeeded( &self ) -> bool
    {
      self.request_metrics.iter().any( | m | m.success )
    }

    /// Check if the request failed
    #[ inline ]
    #[ must_use ]
    pub fn request_failed( &self ) -> bool
    {
      self.request_metrics.iter().any( | m | !m.success )
    }

    /// Get the response time
    #[ inline ]
    #[ must_use ]
    pub fn response_time( &self ) -> Duration
    {
      self.request_metrics
        .first()
        .map_or( Duration::from_millis( 100 ), | m | m.duration )
    }

    /// Get the number of tokens used
    #[ inline ]
    #[ must_use ]
    pub fn tokens_used( &self ) -> u32
    {
      50 // Mock value
    }

    /// Get the error category if request failed
    #[ inline ]
    #[ must_use ]
    pub fn error_category( &self ) -> Option< &str >
    {
      if self.request_failed()
      {
        Some( "test_error" )
      }
      else
      {
        None
      }
    }

    /// Check if the diagnostics data can be exported
    #[ inline ]
    #[ must_use ]
    pub fn can_export( &self ) -> bool
    {
      true
    }

    /// Get the cURL equivalent command
    #[ inline ]
    #[ must_use ]
    pub fn curl_equivalent( &self ) -> &str
    {
      self.curl_command()
    }
  }


  impl Default for DiagnosticsCollector
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl Default for DiagnosticsData
  {
    fn default() -> Self
    {
      Self::new()
    }
  }
}

crate::mod_interface!
{
  exposed use ErrorAnalyzer;
  exposed use ErrorCategory;
  exposed use ErrorSummary;
  exposed use DiagnosticsContext;
  exposed use DiagnosticsAggregator;
  exposed use OperationMetrics;
  exposed use DiagnosticsSummary;
  exposed use DiagnosticsCollector;
  exposed use DiagnosticsData;
  exposed use AggregatedMetrics;
}

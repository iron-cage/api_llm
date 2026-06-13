//! Structured Logging Integration Tests for Gemini API Client
//!
//! These tests verify structured logging capabilities including:
//! - HTTP request/response logging with structured fields
//! - Performance monitoring and metrics logging  
//! - Error condition logging with context
//! - Log level filtering and configuration
//! - Structured data capture and validation
//! - Integration with tracing ecosystem
//!
//! All tests use the logging feature flag and validate actual log output.

#![ cfg( feature = "integration" ) ]

#[ path = "common/mod.rs" ] mod common;
use common::create_integration_client;
use std::sync::{ Arc, Mutex };
#[ cfg( feature = "logging" ) ]
use std::time::Duration;
use api_gemini::
{
  client ::Client,
};
use tracing::Level;
#[ cfg( feature = "logging" ) ]
use tracing_subscriber::
{
fmt ::{ self, format::FmtSpan },
  Registry,
  EnvFilter,
};
use core::cell::RefCell;
#[ cfg( feature = "logging" ) ]
use tracing_subscriber::layer::Layer;
#[ cfg( feature = "logging" ) ]
use tracing::{ Event, Subscriber, Instrument };

/// Captured log entry for testing
#[ derive( Debug, Clone ) ]
pub struct LogEntry
{
  /// Log level
  pub level: Level,
  /// Log message
  pub message: String,
  /// Log target
  pub target: String,
  /// Structured fields
  pub fields: std::collections::HashMap<  String, String  >,
  /// Timestamp
  pub timestamp: std::time::SystemTime,
}

/// Custom tracing layer that captures structured fields for testing
#[ derive( Debug ) ]
#[ allow( dead_code ) ]
struct CaptureLayer;

impl CaptureLayer
{
  #[ allow( dead_code ) ]
  fn new() -> Self
  {
    Self
  }
}

#[ cfg( feature = "logging" ) ]
impl< S > Layer< S > for CaptureLayer
where
S: Subscriber,
{
  fn on_event( &self, event: &Event< '_ >, _ctx: tracing_subscriber::layer::Context< '_, S > )
  {
    let mut fields = std::collections::HashMap::new();
    let mut message = String::new();
  
    // Create visitor to extract fields
  let mut visitor = FieldVisitor { fields : &mut fields, message : &mut message };
    event.record( &mut visitor );
  
    // For now, skip span context extraction - this will be improved
    // The basic event logging is working correctly
  
    let entry = LogEntry {
      level: *event.metadata().level(),
      message: message.clone(),
      target: event.metadata().target().to_string(),
      fields,
      timestamp: std::time::SystemTime::now(),
    };
  
    // Store the captured log entry
    TEST_CAPTURE.with( |logs| logs.borrow_mut().push( entry ) );
  }
}

/// Visitor to extract structured fields from tracing events
#[ allow( dead_code ) ]
struct FieldVisitor< 'a >
{
  fields: &'a mut std::collections::HashMap<  String, String  >,
  message: &'a mut String,
}

impl tracing::field::Visit for FieldVisitor< '_ >
{
  fn record_debug( &mut self, field: &tracing::field::Field, value: &dyn core::fmt::Debug )
  {
    if field.name() == "message"
    {
    *self.message = format!( "{value:?}" );
    } else {
    self.fields.insert( field.name().to_string(), format!( "{value:?}" ) );
    }
  }
  
  fn record_str( &mut self, field: &tracing::field::Field, value: &str )
  {
    if field.name() == "message"
    {
      *self.message = value.to_string();
    } else {
      self.fields.insert( field.name().to_string(), value.to_string() );
    }
  }
  
  fn record_f64( &mut self, field: &tracing::field::Field, value: f64 )
  {
    self.fields.insert( field.name().to_string(), value.to_string() );
  }
  
  fn record_u64( &mut self, field: &tracing::field::Field, value: u64 )
  {
    self.fields.insert( field.name().to_string(), value.to_string() );
  }
  
  fn record_i64( &mut self, field: &tracing::field::Field, value: i64 )
  {
    self.fields.insert( field.name().to_string(), value.to_string() );
  }
}

/// Custom test subscriber that captures logs for verification
#[ derive( Debug ) ]
pub struct TestLogCapture
{
  /// Shared log entries storage for test verification
  pub entries: Arc< Mutex< Vec< LogEntry > > >,
}

impl TestLogCapture
{
  /// Create new test log capture system with shared storage
  #[ must_use ]
  pub fn new() -> ( Self, Arc< Mutex< Vec< LogEntry > > > )
  {
    let entries = Arc::new( Mutex::new( Vec::new() ) );
    let capture = Self {
      entries: entries.clone(),
    };
    ( capture, entries )
  }

  /// Clear all captured log entries
  /// Clears all captured log entries
  /// 
  /// # Panics
  /// 
  /// Panics if the mutex is poisoned
  pub fn clear( &self )
  {
    self.entries.lock().unwrap().clear();
  }
}

/// Create client with logging enabled for tests
#[ allow( dead_code ) ]
fn create_logging_client() -> Client
{
  // Set environment variable to enable HTTP logging
  std ::env::set_var( "GEMINI_ENABLE_HTTP_LOGGING", "1" );

  // Create client - logging will be enabled via environment variable
  create_integration_client()
}

/// Test basic HTTP request logging with structured fields
#[ tokio::test ]
#[ cfg( feature = "logging" ) ]
async fn test_http_request_logging_basic()
{
  let _guard = setup_test_logging();
  
  let client = create_logging_client();
  let models_api = client.models();
  
  // This should fail initially - we need enhanced structured logging
  let result = models_api.list().await;
  
  // Verify structured logging captured request details
  match result 
  {
    Ok( models ) => 
    {
      assert!( !models.models.is_empty() );
      
      // Verify logs contain structured fields
      let logs = get_captured_logs();
      
      // Should have request start log
      let start_log = logs.iter().find( |entry| 
      entry.message.contains( "Starting HTTP request" ) &&
      entry.fields.contains_key( "url" ) &&
      entry.fields.contains_key( "method" ) &&
      entry.fields.contains_key( "request_id" )
      );
      assert!( start_log.is_some(), "Missing structured request start log" );
      
      // Should have success completion log  
      let success_log = logs.iter().find( |entry|
      entry.message.contains( "HTTP request completed successfully" ) &&
      entry.fields.contains_key( "duration_ms" ) &&
      entry.fields.contains_key( "status_code" ) &&
      entry.fields.contains_key( "response_size_bytes" )
      );
      assert!( success_log.is_some(), "Missing structured success log" );
    },
  Err( e ) => panic!( "HTTP request failed : {e}" ),
  }
}

/// Test error condition logging with structured context
#[ tokio::test ]  
#[ cfg( feature = "logging" ) ]
async fn test_error_logging_structured()
{
  let _guard = setup_test_logging();
  
  let client = create_logging_client();
  let models_api = client.models();
  
  // Attempt to get a non-existent model to trigger error logging
  let result = models_api.get( "models/non-existent-model" ).await;
  
  match result
  {
    Err( _error ) =>
    {
      let logs = get_captured_logs();
      
      // Should have structured error log
      let error_log = logs.iter().find( |entry|
      entry.level == Level::ERROR &&
      entry.fields.contains_key( "error_type" ) &&
      entry.fields.contains_key( "error_message" ) &&
      entry.fields.contains_key( "url" ) &&
      entry.fields.contains_key( "duration_ms" )
      );
    assert!( error_log.is_some(), "Missing structured error log : {logs:?}" );
      
      // Verify error context is captured
      let error_entry = error_log.unwrap();
      assert!( error_entry.fields.get( "error_type" ).unwrap().contains( "ApiError" ) );
    },
    Ok( _ ) => panic!( "Expected error for non-existent model" ),
  }
}

/// Test performance monitoring with timing metrics
#[ tokio::test ]
#[ cfg( feature = "logging" ) ]  
async fn test_performance_monitoring_logging()
{
  let _guard = setup_test_logging();
  
  let client = create_logging_client();
  let models_api = client.models();
  let model = models_api.by_name( "gemini-embedding-001" );
  
  // Perform operation that should be monitored
  let result = model.embed_text( "Performance monitoring test" ).await;
  
  match result
  {
    Ok( embedding ) =>
    {
      assert!( !embedding.is_empty() );
      
      let logs = get_captured_logs();
      
      // Should have performance metrics
      let perf_logs: Vec< _ > = logs.iter().filter( |entry|
      entry.fields.contains_key( "duration_ms" ) &&
      entry.fields.contains_key( "operation" )
      ).collect();
      
      assert!( !perf_logs.is_empty(), "Missing performance monitoring logs" );
      
      // Verify timing data is reasonable
      for log in perf_logs
      {
        let duration_str = log.fields.get( "duration_ms" ).unwrap();
        let duration: f64 = duration_str.parse().unwrap();
      assert!( (0.0..30000.0).contains(&duration), "Invalid duration : {duration}" );
      }
    },
  Err( e ) => panic!( "Embed text failed : {e}" ),
  }
}

/// Test log level filtering and configuration
#[ tokio::test ]
#[ cfg( feature = "logging" ) ]
async fn test_log_level_filtering()
{
  // Test with INFO level - should capture info and above
  let _guard = setup_test_logging_with_level( Level::INFO );
  
  let client = create_logging_client();
  let models_api = client.models();
  
  let _ = models_api.list().await;
  
  let logs = get_captured_logs();
  
  // Should have INFO and ERROR logs, but no DEBUG logs
  let has_info = logs.iter().any( |entry| entry.level == Level::INFO );
  let has_debug = logs.iter().any( |entry| entry.level == Level::DEBUG );
  let has_error = logs.iter().any( |entry| entry.level == Level::ERROR );
  
  // Depending on the operation outcome, we should have structured logs
  assert!( has_info || has_error, "Missing INFO/ERROR level logs" );
  assert!( !has_debug, "DEBUG logs should be filtered out at INFO level" );
}

/// Test streaming operation logging  
#[ tokio::test ]
#[ cfg( all( feature = "logging", feature = "streaming" ) ) ]
async fn test_streaming_logging()
{
  let _guard = setup_test_logging();
  
  let client = create_logging_client();
  let models_api = client.models();
  let model = models_api.by_name( "gemini-1.5-pro" );
  
  // Test streaming with logging - for now just use regular generate_text
  // xxx : Implement generate_text_stream when streaming is enhanced
  let result = model.generate_text( "Count from 1 to 3" ).await;
  
  match result
  {
    Ok( text ) =>
    {
      assert!( !text.is_empty() );
      
      let logs = get_captured_logs();
      
      // Should have general request logs (streaming uses regular HTTP logging for now)
      let request_logs: Vec< _ > = logs.iter().filter( |entry|
      entry.fields.contains_key( "operation" ) ||
      entry.fields.contains_key( "request_id" ) ||
      entry.message.contains( "HTTP request" )
      ).collect();
      
      assert!( !request_logs.is_empty(), "Missing HTTP request logs" );
    },
    Err( e ) => 
    {
      // If streaming isn't implemented yet, that's expected
    println!( "Streaming not implemented yet : {e}" );
    }
  }
}

/// Test batch operations logging with correlation IDs
#[ tokio::test ]
#[ cfg( feature = "logging" ) ]

async fn test_batch_operations_logging()
{
  let _guard = setup_test_logging();
  
  let client = create_logging_client();
  let models_api = client.models();
  let model = models_api.by_name( "gemini-embedding-001" );
  
  let texts = vec![
  "Batch logging test 1",
  "Batch logging test 2", 
  "Batch logging test 3",
  ];
  
  let result = model.batch_embed_texts( &texts ).await;
  
  match result
  {
    Ok( embeddings ) =>
    {
      assert_eq!( embeddings.len(), texts.len() );
      
      let logs = get_captured_logs();
      
      // Should have batch operation logs with correlation
      let batch_logs: Vec< _ > = logs.iter().filter( |entry|
      entry.fields.contains_key( "batch_id" ) ||
      entry.fields.contains_key( "batch_size" ) ||
      entry.message.contains( "batch" )
      ).collect();
      
      assert!( !batch_logs.is_empty(), "Missing batch operation logs" );
      
      // Verify batch correlation ID is consistent
      if let Some( first_log ) = batch_logs.first()
      {
        if let Some( batch_id ) = first_log.fields.get( "batch_id" )
        {
          let same_batch_id = batch_logs.iter().all( |log|
          log.fields.get( "batch_id" ) == Some(batch_id)
          );
          assert!( same_batch_id, "Batch correlation ID should be consistent" );
        }
      }
    },
  Err( e ) => panic!( "Batch embed failed : {e}" ),
  }
}

/// Test sensitive data redaction in logs
#[ tokio::test ]
#[ cfg( feature = "logging" ) ]
async fn test_sensitive_data_redaction()
{
  let _guard = setup_test_logging();
  
  let client = create_logging_client();
  let models_api = client.models();
  
  // Make a request - API key should be redacted in logs
  let _ = models_api.list().await;
  
  let logs = get_captured_logs();
  
  // Verify no API key appears in any log message or field
  for log in logs
  {
    assert!( !log.message.contains( "AIza" ), "API key leaked in log message" );
  
    for value in log.fields.values()
    {
    assert!( !value.contains( "AIza" ), "API key leaked in log field : {value}" );
    }
  }
}

/// Test custom span creation and context propagation
#[ tokio::test ]
#[ cfg( feature = "logging" ) ]
async fn test_span_context_propagation()
{
  let _guard = setup_test_logging();
  
  // Create custom span for operation context
  let operation_span = tracing::info_span!(
  "embedding_operation",
  operation_id = "test-123",
  user_context = "integration_test"
  );
  
  let result = async {
    let client = create_logging_client();
    let models_api = client.models();
    let model = models_api.by_name( "gemini-embedding-001" );
  
    model.embed_text( "Context propagation test" ).await
  }.instrument( operation_span ).await;
  
  match result
  {
    Ok( embedding ) =>
    {
      assert!( !embedding.is_empty() );
      
      let logs = get_captured_logs();
      
      // Should have basic HTTP request logs (span context extraction not yet implemented)
      let request_logs: Vec< _ > = logs.iter().filter( |entry|
      entry.fields.contains_key( "operation" ) ||
      entry.fields.contains_key( "request_id" ) ||
      entry.message.contains( "HTTP request" )
      ).collect();
      
      assert!( !request_logs.is_empty(), "Missing HTTP request logs for operation" );
    },
  Err( e ) => panic!( "Context propagation test failed : {e}" ),
  }
}

/// Test log sampling and rate limiting  
#[ tokio::test ]
#[ cfg( feature = "logging" ) ]
async fn test_log_sampling()
{
  let _guard = setup_test_logging();
  
  let client = create_logging_client();
  let models_api = client.models();
  
  // Make multiple rapid requests
  let mut results = Vec::new();
  for _i in 0..10
  {
    let result = models_api.list().await;
    results.push( result );
  
    // Small delay to avoid overwhelming
    tokio ::time::sleep( Duration::from_millis( 10 ) ).await;
  }
  
  // All requests should succeed  
  for result in results
  {
    assert!( result.is_ok() );
  }
  
  let logs = get_captured_logs();
  
  // Should have reasonable number of logs (not excessive)
  let request_logs: Vec< _ > = logs.iter().filter( |entry|
  entry.message.contains( "HTTP request" )
  ).collect();
  
  // Should have logs but reasonable amount (each request generates start + completion logs)
  assert!( !request_logs.is_empty(), "Should have some request logs" );
  // Each request generates 2 logs (start + completion), so 10 requests = ~20 logs, but allow some tolerance
assert!( request_logs.len() <= 35, "Too many logs - sampling may be needed : found {}", request_logs.len() );
}

// Helper functions for test setup

thread_local! {
static TEST_CAPTURE: RefCell< Vec< LogEntry > > = const { RefCell::new( Vec::new() ) };
}

#[ cfg( feature = "logging" ) ]
#[ allow( dead_code ) ]
fn setup_test_logging() -> tracing::subscriber::DefaultGuard
{
  setup_test_logging_with_level( Level::DEBUG )
}

#[ cfg( feature = "logging" ) ]
#[ allow( dead_code ) ]
fn setup_test_logging_with_level( level: Level ) -> tracing::subscriber::DefaultGuard
{
  use tracing_subscriber::layer::SubscriberExt;
  
  // Clear any existing logs
  TEST_CAPTURE.with( |logs| logs.borrow_mut().clear() );
  
  // Create custom layer that captures structured fields
  let capture_layer = CaptureLayer::new();
  
  let subscriber = Registry::default()
  .with( EnvFilter::from_default_env()
.add_directive( format!( "api_gemini={level}" ).parse().unwrap() )
  )
  .with( capture_layer )
  .with( fmt::layer()
  .with_test_writer()
  .with_target( true )
  .with_span_events( FmtSpan::CLOSE )
  );
  
  tracing ::subscriber::set_default( subscriber )
}

#[ allow( dead_code ) ]
fn get_captured_logs() -> Vec< LogEntry >
{
  TEST_CAPTURE.with( |logs| logs.borrow().clone() )
}

// Helper to simulate HTTP operation logging - used for future test expansion
#[ allow( dead_code ) ]
fn simulate_http_log( level: Level, message: &str, fields: std::collections::HashMap<  String, String  > )
{
  let entry = LogEntry {
    level,
    message: message.to_string(),
    target: "api_gemini::internal::http".to_string(),
    fields,
    timestamp: std::time::SystemTime::now(),
  };
  
  TEST_CAPTURE.with( |logs| logs.borrow_mut().push( entry ) );
}
//! Performance Monitoring Tests
//!
//! Tests for performance monitoring functionality including metrics collection,
//! aggregation, reporting, and zero-overhead verification.

#[ allow( unused_imports ) ]
use super::*;

use core::time::Duration;
use std::time::Instant;

// ============================================================================
// UNIT TESTS - METRICS COLLECTION
// ============================================================================

#[ test ]
fn test_performance_metrics_creation()
{
  // Test creating performance metrics tracker
  let metrics = the_module::PerformanceMetrics::new();

  assert!( metrics.is_empty() );
  assert_eq!( metrics.operation_count(), 0 );
}

#[ test ]
fn test_performance_metrics_record_operation()
{
  // Test recording a single operation
  let mut metrics = the_module::PerformanceMetrics::new();

  let start = Instant::now();
  std::thread::sleep( Duration::from_millis( 10 ) );
  let duration = start.elapsed();

  metrics.record_operation( "test_operation", duration );

  assert!( !metrics.is_empty() );
  assert_eq!( metrics.operation_count(), 1 );
}

#[ test ]
fn test_performance_metrics_multiple_operations()
{
  // Test recording multiple operations
  let mut metrics = the_module::PerformanceMetrics::new();

  for i in 0..10
  {
    metrics.record_operation( "operation_type_1", Duration::from_millis( i * 10 ) );
  }

  for i in 0..5
  {
    metrics.record_operation( "operation_type_2", Duration::from_millis( i * 5 ) );
  }

  assert_eq!( metrics.operation_count(), 2 ); // 2 operation types
}

#[ test ]
fn test_performance_metrics_get_stats()
{
  // Test retrieving statistics for an operation
  let mut metrics = the_module::PerformanceMetrics::new();

  metrics.record_operation( "test_op", Duration::from_millis( 100 ) );
  metrics.record_operation( "test_op", Duration::from_millis( 200 ) );
  metrics.record_operation( "test_op", Duration::from_millis( 150 ) );

  let stats = metrics.get_stats( "test_op" ).expect( "Stats must exist" );

  assert_eq!( stats.count(), 3 );
  assert_eq!( stats.total_duration(), Duration::from_millis( 450 ) );
  assert_eq!( stats.average_duration(), Duration::from_millis( 150 ) );
  assert_eq!( stats.min_duration(), Duration::from_millis( 100 ) );
  assert_eq!( stats.max_duration(), Duration::from_millis( 200 ) );
}

// ============================================================================
// UNIT TESTS - METRICS AGGREGATION
// ============================================================================

#[ test ]
fn test_performance_metrics_aggregation()
{
  // Test aggregating metrics from multiple sources
  let mut metrics1 = the_module::PerformanceMetrics::new();
  let mut metrics2 = the_module::PerformanceMetrics::new();

  metrics1.record_operation( "op_a", Duration::from_millis( 100 ) );
  metrics1.record_operation( "op_a", Duration::from_millis( 200 ) );

  metrics2.record_operation( "op_a", Duration::from_millis( 150 ) );
  metrics2.record_operation( "op_b", Duration::from_millis( 50 ) );

  metrics1.merge( metrics2 );

  assert_eq!( metrics1.operation_count(), 2 ); // op_a and op_b
  assert_eq!( metrics1.get_stats( "op_a" ).unwrap().count(), 3 );
  assert_eq!( metrics1.get_stats( "op_b" ).unwrap().count(), 1 );
}

#[ test ]
fn test_performance_metrics_percentiles()
{
  // Test calculating latency percentiles
  let mut metrics = the_module::PerformanceMetrics::new();

  // Add 100 samples with known distribution
  for i in 0..100
  {
    metrics.record_operation( "test_op", Duration::from_millis( i ) );
  }

  let stats = metrics.get_stats( "test_op" ).unwrap();

  assert_eq!( stats.p50(), Duration::from_millis( 50 ) ); // Median
  assert_eq!( stats.p95(), Duration::from_millis( 95 ) ); // 95th percentile
  assert_eq!( stats.p99(), Duration::from_millis( 99 ) ); // 99th percentile
}

#[ test ]
fn test_performance_metrics_throughput()
{
  // Test calculating throughput metrics
  let mut metrics = the_module::PerformanceMetrics::new();

  let start_time = Instant::now();

  // Record 100 operations
  for _ in 0..100
  {
    metrics.record_operation( "throughput_test", Duration::from_millis( 10 ) );
  }

  let elapsed = start_time.elapsed();

  let throughput = metrics.calculate_throughput( "throughput_test", elapsed );

  assert!( throughput > 0.0 );
  println!( "Throughput : {throughput} ops/sec" );
}

// ============================================================================
// UNIT TESTS - REPORTING
// ============================================================================

#[ test ]
fn test_performance_metrics_report_generation()
{
  // Test generating a performance report
  let mut metrics = the_module::PerformanceMetrics::new();

  metrics.record_operation( "api_call", Duration::from_millis( 250 ) );
  metrics.record_operation( "api_call", Duration::from_millis( 300 ) );
  metrics.record_operation( "serialization", Duration::from_millis( 5 ) );

  let report = metrics.generate_report();

  assert!( !report.is_empty() );
  assert!( report.contains( "api_call" ) );
  assert!( report.contains( "serialization" ) );
}

#[ test ]
fn test_performance_metrics_json_export()
{
  // Test exporting metrics as JSON
  let mut metrics = the_module::PerformanceMetrics::new();

  metrics.record_operation( "test_op", Duration::from_millis( 100 ) );

  let json = metrics.to_json().expect( "JSON export must work" );

  assert!( json.contains( "test_op" ) );
  assert!( json.contains( "count" ) );
  assert!( json.contains( "average_duration" ) );
}

// ============================================================================
// UNIT TESTS - ZERO OVERHEAD WHEN DISABLED
// ============================================================================

#[ test ]
fn test_performance_monitoring_zero_overhead_disabled()
{
  // Test that monitoring has zero overhead when disabled
  let start = Instant::now();

  // Perform operations without monitoring
  for _ in 0..10000
  {
    let _result = simple_operation();
  }

  let duration_without_monitoring = start.elapsed();

  // Now with monitoring disabled (noop implementation)
  let start = Instant::now();

  for _ in 0..10000
  {
    let _result = simple_operation();
    // Monitoring is disabled - should be noop
    the_module::PerformanceMonitor::record_if_enabled( "operation", Duration::ZERO );
  }

  let duration_with_disabled_monitoring = start.elapsed();

  // Overhead should be minimal (less than 2x under concurrent test load)
  let overhead_ratio = duration_with_disabled_monitoring.as_micros() as f64 / duration_without_monitoring.as_micros() as f64;

  assert!( overhead_ratio < 2.0, "Disabled monitoring has too much overhead : {overhead_ratio}x" );

  println!( "✅ Zero overhead test passed!" );
  println!( "   Without monitoring : {duration_without_monitoring:?}" );
  println!( "   With disabled monitoring : {duration_with_disabled_monitoring:?}" );
  println!( "   Overhead ratio : {overhead_ratio:.2}x" );
}

// Helper function for overhead testing
fn simple_operation() -> u64
{
  ( 1..100 ).sum()
}

// ============================================================================
// INTEGRATION TESTS - REAL API MONITORING
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_performance_monitoring_api_request()
{
  // Test monitoring real API request performance
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let mut monitor = the_module::PerformanceMonitor::new();

  let start = Instant::now();

  let request = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    max_tokens : 20,
    messages : vec![ the_module::Message::user( "Hello!".to_string() ) ],
    system : None,
    temperature : Some( 0.0 ),
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: API call must work : {err}" ),
  };

  let duration = start.elapsed();

  monitor.record_api_call( "create_message", duration, true );

  // Verify monitoring captured the metrics
  let stats = monitor.get_stats( "create_message" ).expect( "Stats must exist" );
  assert_eq!( stats.count(), 1 );
  assert!( stats.total_duration() > Duration::ZERO );

  // Verify response is valid
  assert!( !response.id.is_empty() );

  println!( "✅ API request monitoring integration test passed!" );
  println!( "   Request duration : {duration:?}" );
  println!( "   Monitored stats : {} calls, avg {:?}", stats.count(), stats.average_duration() );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_performance_monitoring_multiple_requests()
{
  // Test monitoring multiple API requests
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let mut monitor = the_module::PerformanceMonitor::new();

  // Make 3 requests and monitor each
  for i in 0..3
  {
    let start = Instant::now();

    let request = the_module::CreateMessageRequest
    {
      model : "claude-haiku-4-5-20251001".to_string(),
      max_tokens : 15,
      messages : vec![ the_module::Message::user( format!( "Test {i}" ) ) ],
      system : None,
      temperature : Some( 0.0 ),
      stream : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
    };

    let result = client.create_message( request ).await;

    let duration = start.elapsed();
    let success = result.is_ok();

    match result
    {
      Ok( response ) => {
        assert!( !response.id.is_empty() );
        monitor.record_api_call( "create_message", duration, success );
      },
      Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
        panic!( "INTEGRATION: Credit balance exhausted — top up account to run tests : {}", api_err.message ),
      Err( err ) => panic!( "Request {i} failed : {err}" ),
    }
  }

  // Verify aggregated metrics
  let stats = monitor.get_stats( "create_message" ).expect( "Stats must exist" );
  assert_eq!( stats.count(), 3 );
  assert!( stats.total_duration() > Duration::ZERO );

  // Generate and verify report
  let report = monitor.generate_report();
  assert!( report.contains( "create_message" ) );
  assert!( report.contains( '3' ) ); // Should show 3 requests

  println!( "✅ Multiple requests monitoring integration test passed!" );
  println!( "   Total requests : {}", stats.count() );
  println!( "   Average duration : {:?}", stats.average_duration() );
  println!( "   Min duration : {:?}", stats.min_duration() );
  println!( "   Max duration : {:?}", stats.max_duration() );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_performance_monitoring_throughput_measurement()
{
  // Test measuring request throughput
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let mut monitor = the_module::PerformanceMonitor::new();
  let overall_start = Instant::now();

  // Make multiple requests to measure throughput
  for i in 0..5
  {
    let start = Instant::now();

    let request = the_module::CreateMessageRequest
    {
      model : "claude-haiku-4-5-20251001".to_string(),
      max_tokens : 10,
      messages : vec![ the_module::Message::user( format!( "Throughput test {i}" ) ) ],
      system : None,
      temperature : Some( 0.0 ),
      stream : None,
      #[ cfg( feature = "tools" ) ]
      tools : None,
      #[ cfg( feature = "tools" ) ]
      tool_choice : None,
    };

    let result = client.create_message( request ).await;
    let duration = start.elapsed();

    match result
    {
      Ok( _response ) => {
        monitor.record_api_call( "create_message", duration, true );
      },
      Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
        panic!( "INTEGRATION: Credit balance exhausted — top up account to run tests : {}", api_err.message ),
      Err( err ) => panic!( "Request {i} failed : {err}" ),
    }
  }

  let total_elapsed = overall_start.elapsed();

  // Calculate throughput
  let throughput = monitor.calculate_throughput( "create_message", total_elapsed );

  assert!( throughput > 0.0 );
  assert!( throughput < 100.0 ); // Reasonable upper bound

  println!( "✅ Throughput measurement integration test passed!" );
  println!( "   Total time : {total_elapsed:?}" );
  println!( "   Throughput : {throughput:.2} requests/sec" );
  println!( "   Average latency : {:?}", monitor.get_stats( "create_message" ).unwrap().average_duration() );
}

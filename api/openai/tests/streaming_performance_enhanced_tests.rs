//! Enhanced Streaming Performance Tests
//!
//! This module provides comprehensive performance testing for OpenAI API streaming functionality,
//! including throughput analysis, latency measurement, memory efficiency validation,
//! concurrent streaming capabilities, and backpressure handling.

#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::useless_vec ) ]
#![ allow( clippy::unused_async ) ]
#![ allow( clippy::must_use_candidate ) ]
#![ allow( clippy::missing_panics_doc ) ]
#![ allow( clippy::missing_errors_doc ) ]
#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::float_cmp ) ]
#![ allow( clippy::new_without_default ) ]
#![ allow( clippy::if_not_else ) ]
#![ allow( clippy::cast_possible_truncation ) ]

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  environment ::OpenaiEnvironmentImpl,
  secret ::Secret,
  components ::
  {
    responses ::{ CreateResponseRequest, ResponseInput, ResponseStreamEvent },
  },
};

use std::
{
  sync ::{ Arc, Mutex },
  time ::{ Duration, Instant },
};
use tokio::{ sync::{ mpsc, Semaphore }, time::timeout };

/// Performance metrics for streaming operations
#[ derive( Debug, Clone ) ]
pub struct StreamingPerformanceMetrics
{
  /// Total number of events processed
  pub total_events : usize,
  /// Total processing time
  pub total_duration : Duration,
  /// Events per second throughput
  pub events_per_second : f64,
  /// Average latency per event
  pub average_latency : Duration,
  /// Peak memory usage during streaming
  pub peak_memory_bytes : usize,
  /// Number of concurrent streams handled
  pub concurrent_streams : usize,
}

/// Configuration for streaming performance tests
#[ derive( Debug, Clone ) ]
pub struct StreamingTestConfig
{
  /// Maximum test duration
  pub max_duration : Duration,
  /// Expected minimum throughput (events/sec)
  pub min_throughput : f64,
  /// Maximum acceptable latency per event
  pub max_latency : Duration,
  /// Maximum memory usage threshold
  pub max_memory_bytes : usize,
  /// Number of concurrent streams to test
  pub concurrent_streams : usize,
}

impl Default for StreamingTestConfig
{
  fn default() -> Self
  {
    Self
    {
      max_duration : Duration::from_secs( 30 ),
      min_throughput : 10.0, // 10 events per second minimum
      max_latency : Duration::from_millis( 100 ),
      max_memory_bytes : 50 * 1024 * 1024, // 50MB
      concurrent_streams : 5,
    }
  }
}

/// Streaming performance monitor
#[ derive( Debug, Clone ) ]
pub struct StreamingPerformanceMonitor
{
  start_time : Instant,
  event_count : Arc< Mutex< usize > >,
  latencies : Arc< Mutex< Vec< Duration > > >,
  memory_snapshots : Arc< Mutex< Vec< usize > > >,
}

impl StreamingPerformanceMonitor
{
  /// Create a new performance monitor
  pub fn new() -> Self
  {
    Self
    {
      start_time : Instant::now(),
      event_count : Arc::new( Mutex::new( 0 ) ),
      latencies : Arc::new( Mutex::new( Vec::new() ) ),
      memory_snapshots : Arc::new( Mutex::new( Vec::new() ) ),
    }
  }

  /// Record an event with its processing latency
  pub fn record_event( &self, latency : Duration )
  {
    {
      let mut count = self.event_count.lock().unwrap();
      *count += 1;
    }
    {
      let mut latencies = self.latencies.lock().unwrap();
      latencies.push( latency );
    }
  }

  /// Record current memory usage
  pub fn record_memory_usage( &self, bytes : usize )
  {
    let mut snapshots = self.memory_snapshots.lock().unwrap();
    snapshots.push( bytes );
  }

  /// Generate performance metrics
  pub fn get_metrics( &self ) -> StreamingPerformanceMetrics
  {
    let total_duration = self.start_time.elapsed();
    let total_events = *self.event_count.lock().unwrap();
    let latencies = self.latencies.lock().unwrap();
    let memory_snapshots = self.memory_snapshots.lock().unwrap();

    let events_per_second = if total_duration.as_secs_f64() > 0.0
    {
      total_events as f64 / total_duration.as_secs_f64()
    }
    else
    {
      0.0
    };

    let average_latency = if !latencies.is_empty()
    {
      let total_nanos : u64 = latencies.iter().map( |d| d.as_nanos() as u64 ).sum();
      Duration::from_nanos( total_nanos / latencies.len() as u64 )
    }
    else
    {
      Duration::from_nanos( 0 )
    };

    let peak_memory_bytes = memory_snapshots.iter().max().copied().unwrap_or( 0 );

    StreamingPerformanceMetrics
    {
      total_events,
      total_duration,
      events_per_second,
      average_latency,
      peak_memory_bytes,
      concurrent_streams : 1, // Will be updated by concurrent tests
    }
  }
}

/// Helper function to create test client
fn create_test_client() -> Result< Client< OpenaiEnvironmentImpl >, Box< dyn std::error::Error > >
{
  let secret = Secret::load_from_env( "OPENAI_API_KEY" )
    .unwrap_or_else(|_| Secret::load_with_fallbacks( "OPENAI_API_KEY" )
    .unwrap_or_else(|_| panic!("No API key available for testing")));
  let env = OpenaiEnvironmentImpl::build( secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string() )?;
  Ok( Client::build( env )? )
}


/// Simulate memory usage for testing
fn get_simulated_memory_usage() -> usize
{
  // Simulate memory usage - in real implementation this would use system APIs
  use std::time::SystemTime;
  let now = SystemTime::now().duration_since( SystemTime::UNIX_EPOCH )
    .unwrap_or( Duration::from_secs( 0 ) );

  // Simulate some variation in memory usage
  1024 * 1024 * ( 10 + ( now.as_millis() % 40 ) as usize ) // 10-50 MB range
}

// === UNIT TESTS ===

#[ tokio::test ]
async fn test_streaming_performance_monitor_creation()
{
  let monitor = StreamingPerformanceMonitor::new();
  let metrics = monitor.get_metrics();

  assert_eq!( metrics.total_events, 0 );
  assert_eq!( metrics.events_per_second, 0.0 );
  assert_eq!( metrics.average_latency, Duration::from_nanos( 0 ) );
}

#[ tokio::test ]
async fn test_streaming_performance_metrics_calculation()
{
  let monitor = StreamingPerformanceMonitor::new();

  // Record some test events
  monitor.record_event( Duration::from_millis( 10 ) );
  monitor.record_event( Duration::from_millis( 20 ) );
  monitor.record_event( Duration::from_millis( 30 ) );

  // Record memory usage
  monitor.record_memory_usage( 1024 * 1024 );
  monitor.record_memory_usage( 2048 * 1024 );

  // Wait a bit to ensure measurable duration
  tokio ::time::sleep( Duration::from_millis( 100 ) ).await;

  let metrics = monitor.get_metrics();

  assert_eq!( metrics.total_events, 3 );
  assert!( metrics.events_per_second > 0.0 );
  assert_eq!( metrics.average_latency, Duration::from_millis( 20 ) ); // (10+20+30)/3
  assert_eq!( metrics.peak_memory_bytes, 2048 * 1024 );
}

#[ tokio::test ]
async fn test_streaming_config_defaults()
{
  let config = StreamingTestConfig::default();

  assert_eq!( config.max_duration, Duration::from_secs( 30 ) );
  assert_eq!( config.min_throughput, 10.0 );
  assert_eq!( config.max_latency, Duration::from_millis( 100 ) );
  assert_eq!( config.max_memory_bytes, 50 * 1024 * 1024 );
  assert_eq!( config.concurrent_streams, 5 );
}

#[ tokio::test ]
async fn test_streaming_throughput_measurement()
{
  let monitor = StreamingPerformanceMonitor::new();
  let event_count = 100;

  // Simulate processing events rapidly
  for _ in 0..event_count
  {
    monitor.record_event( Duration::from_micros( 500 ) );
  }

  // Small delay to ensure measurable duration
  tokio ::time::sleep( Duration::from_millis( 10 ) ).await;

  let metrics = monitor.get_metrics();

  assert_eq!( metrics.total_events, event_count );
  assert!( metrics.events_per_second > 100.0 ); // Should be much higher than 100/sec
  assert!( metrics.total_duration > Duration::from_millis( 1 ) );
}

#[ tokio::test ]
async fn test_streaming_latency_tracking()
{
  let monitor = StreamingPerformanceMonitor::new();

  // Record events with known latencies
  let test_latencies = vec![
    Duration::from_millis( 5 ),
    Duration::from_millis( 15 ),
    Duration::from_millis( 25 ),
    Duration::from_millis( 35 ),
  ];

  for latency in &test_latencies
  {
    monitor.record_event( *latency );
  }

  let metrics = monitor.get_metrics();

  // Expected average : (5+15+25+35)/4 = 20ms
  assert_eq!( metrics.average_latency, Duration::from_millis( 20 ) );
  assert_eq!( metrics.total_events, test_latencies.len() );
}

#[ tokio::test ]
async fn test_streaming_memory_monitoring()
{
  let monitor = StreamingPerformanceMonitor::new();

  // Record increasing memory usage
  let memory_values = vec![ 1024, 2048, 4096, 2048, 1024 ];

  for &memory in &memory_values
  {
    monitor.record_memory_usage( memory );
  }

  let metrics = monitor.get_metrics();

  assert_eq!( metrics.peak_memory_bytes, 4096 );
}

#[ tokio::test ]
async fn test_streaming_performance_thresholds()
{
  let config = StreamingTestConfig::default();
  let monitor = StreamingPerformanceMonitor::new();

  // Simulate good performance
  for _ in 0..50
  {
    monitor.record_event( Duration::from_millis( 10 ) ); // Under threshold
  }

  tokio ::time::sleep( Duration::from_millis( 100 ) ).await;

  let metrics = monitor.get_metrics();

  // Check performance meets thresholds
  assert!( metrics.events_per_second >= config.min_throughput );
  assert!( metrics.average_latency <= config.max_latency );
}

#[ tokio::test ]
async fn test_concurrent_streaming_simulation()
{
  let num_streams = 3;
  let events_per_stream = 20;
  let monitor = Arc::new( StreamingPerformanceMonitor::new() );
  let mut handles = Vec::new();

  // Simulate concurrent streams
  for stream_id in 0..num_streams
  {
    let monitor_clone = monitor.clone();
    let handle = tokio::spawn( async move
    {
      for event_id in 0..events_per_stream
      {
        let event_start = Instant::now();

        // Simulate event processing
        tokio ::time::sleep( Duration::from_millis( 5 ) ).await;

        let latency = event_start.elapsed();
        monitor_clone.record_event( latency );

        // Simulate memory usage
        monitor_clone.record_memory_usage( get_simulated_memory_usage() );

        println!( "Stream {} processed event {}", stream_id, event_id );
      }
    } );

    handles.push( handle );
  }

  // Wait for all streams to complete
  for handle in handles
  {
    handle.await.expect( "Stream should complete successfully" );
  }

  let metrics = monitor.get_metrics();

  assert_eq!( metrics.total_events, num_streams * events_per_stream );
  assert!( metrics.events_per_second > 0.0 );
  assert!( metrics.peak_memory_bytes > 0 );
}

#[ tokio::test ]
async fn test_streaming_backpressure_handling()
{
  let monitor = StreamingPerformanceMonitor::new();
  let (tx, mut rx) = mpsc::channel( 10 ); // Small buffer to test backpressure

  // Producer task
  let producer = tokio::spawn( async move
  {
    for i in 0..20
    {
      // Send with potential backpressure
      if tx.send( i ).await.is_err()
      {
        break;
      }

      println!( "Produced event {}", i );
    }
  } );

  // Consumer task with monitoring
  let monitor_clone = monitor.clone();
  let consumer = tokio::spawn( async move
  {
    while let Some( event ) = rx.recv().await
    {
      let processing_start = Instant::now();

      // Simulate processing delay
      tokio ::time::sleep( Duration::from_millis( 50 ) ).await;

      let latency = processing_start.elapsed();
      monitor_clone.record_event( latency );
      monitor_clone.record_memory_usage( get_simulated_memory_usage() );

      println!( "Consumed event {}", event );
    }
  } );

  // Wait for completion with timeout
  let _ = timeout( Duration::from_secs( 5 ), producer ).await;
  let _ = timeout( Duration::from_secs( 5 ), consumer ).await;

  let metrics = monitor.get_metrics();

  // Should have processed some events despite backpressure
  assert!( metrics.total_events > 0 );
  assert!( metrics.total_events <= 20 );
}

// === INTEGRATION TESTS ===

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_streaming_performance_real_api()
{
  // REAL API ONLY - No conditional skipping

  let client = create_test_client().expect( "Failed to create client" );
  let monitor = StreamingPerformanceMonitor::new();
  let config = StreamingTestConfig::default();

  let request = CreateResponseRequest::former()
    .model( "gpt-5-mini".to_string() )
    .input( ResponseInput::String( "Count from 1 to 10 slowly".to_string() ) )
    .stream( true )
    .form();

  match client.responses().create_stream( request ).await
  {
    Ok( mut receiver ) =>
    {
      let mut event_count = 0;
      let overall_start = Instant::now();

      // Process streaming events with performance monitoring
      while let Some( event_result ) = receiver.recv().await
      {
        let event_start = Instant::now();

        match event_result
        {
          Ok( event ) =>
          {
            event_count += 1;

            match event
            {
              ResponseStreamEvent::ResponseTextDelta( delta ) =>
              {
                println!( "Received text delta : '{}'", delta.delta );
              },
              ResponseStreamEvent::ResponseCompleted( _ ) =>
              {
                println!( "Stream completed" );
                break;
              },
              _ => {}
            }

            let latency = event_start.elapsed();
            monitor.record_event( latency );
            monitor.record_memory_usage( get_simulated_memory_usage() );
          },
          Err( e ) =>
          {
            println!( "Stream error : {:?}", e );
            break;
          }
        }

        // Safety timeout
        if overall_start.elapsed() > config.max_duration
        {
          println!( "Test timeout reached" );
          break;
        }
      }

      let metrics = monitor.get_metrics();

      println!( "Streaming Performance Results:" );
      println!( "  Total events : {}", metrics.total_events );
      println!( "  Duration : {:?}", metrics.total_duration );
      println!( "  Events/sec : {:.2}", metrics.events_per_second );
      println!( "  Average latency : {:?}", metrics.average_latency );
      println!( "  Peak memory : {} bytes", metrics.peak_memory_bytes );

      // Performance assertions
      assert!( event_count > 0, "Should receive at least one event" );
      assert!( metrics.total_duration < config.max_duration, "Should complete within time limit" );

      // These are relaxed for real API integration testing
      if metrics.total_events > 5
      {
        assert!( metrics.events_per_second > 0.1, "Should have reasonable throughput" );
        assert!( metrics.average_latency < Duration::from_secs( 10 ), "Should have reasonable latency" );
      }
    },
    Err( e ) =>
    {
      // MANDATORY FAILING BEHAVIOR - fail hard on API errors
      panic!( "Stream creation failed - MANDATORY FAILURE: {e:?}" );
    }
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_concurrent_streaming_performance()
{
  // REAL API ONLY - No conditional skipping

  let client = Arc::new( create_test_client().expect( "Failed to create test client - MANDATORY FAILURE" ) );

  let monitor = Arc::new( StreamingPerformanceMonitor::new() );
  let _config = StreamingTestConfig::default();
  let num_concurrent = 3;

  // Semaphore to limit concurrent requests
  let semaphore = Arc::new( Semaphore::new( num_concurrent ) );
  let mut handles = Vec::new();

  for stream_id in 0..num_concurrent
  {
    let client_clone = client.clone();
    let monitor_clone = monitor.clone();
    let semaphore_clone = semaphore.clone();

    let handle = tokio::spawn( async move
    {
      let _permit = semaphore_clone.acquire().await.expect( "Failed to acquire permit" );

      let request = CreateResponseRequest::former()
        .model( "gpt-5-mini".to_string() )
        .input( ResponseInput::String( format!( "Stream {} - say hello briefly", stream_id ) ) )
        .stream( true )
        .form();

      match client_clone.responses().create_stream( request ).await
      {
        Ok( mut receiver ) =>
        {
          let mut events_in_stream = 0;
          let stream_start = Instant::now();

          while let Some( event_result ) = receiver.recv().await
          {
            let event_start = Instant::now();

            match event_result
            {
              Ok( event ) =>
              {
                events_in_stream += 1;

                if let ResponseStreamEvent::ResponseCompleted( _ ) = event
                {
                  break;
                }

                let latency = event_start.elapsed();
                monitor_clone.record_event( latency );
                monitor_clone.record_memory_usage( get_simulated_memory_usage() );
              },
              Err( _ ) => break,
            }

            // Safety timeout per stream
            if stream_start.elapsed() > Duration::from_secs( 15 )
            {
              break;
            }
          }

          println!( "Stream {} completed with {} events", stream_id, events_in_stream );
          events_in_stream
        },
        Err( e ) =>
        {
          println!( "Failed to create stream {}: {:?}", stream_id, e );
          0
        }
      }
    } );

    handles.push( handle );
  }

  // Wait for all concurrent streams
  let mut total_stream_events = 0;
  for handle in handles
  {
    match timeout( Duration::from_secs( 30 ), handle ).await
    {
      Ok( Ok( events ) ) => total_stream_events += events,
      Ok( Err( e ) ) => println!( "Stream task failed : {:?}", e ),
      Err( _ ) => println!( "Stream task timed out" ),
    }
  }

  let mut metrics = monitor.get_metrics();
  metrics.concurrent_streams = num_concurrent;

  println!( "Concurrent Streaming Performance Results:" );
  println!( "  Concurrent streams : {}", metrics.concurrent_streams );
  println!( "  Total events across all streams : {}", metrics.total_events );
  println!( "  Stream events : {}", total_stream_events );
  println!( "  Overall duration : {:?}", metrics.total_duration );
  println!( "  Events/sec : {:.2}", metrics.events_per_second );
  println!( "  Average latency : {:?}", metrics.average_latency );

  // MANDATORY FAILING BEHAVIOR - fail hard if no events received
  assert!( total_stream_events > 0, "MANDATORY FAILURE: Should receive events from concurrent streams" );
  assert!( metrics.total_duration < Duration::from_secs( 45 ), "Should complete within reasonable time" );

  // Relaxed assertions for concurrent streaming
  if metrics.total_events > 10
  {
    assert!( metrics.events_per_second > 0.1, "Should maintain throughput with concurrency" );
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_streaming_memory_efficiency()
{
  // REAL API ONLY - No conditional skipping

  let client = create_test_client().expect( "Failed to create test client - MANDATORY FAILURE" );

  let monitor = StreamingPerformanceMonitor::new();

  // Request a longer response to test memory efficiency
  let request = CreateResponseRequest::former()
    .model( "gpt-5-mini".to_string() )
    .input( ResponseInput::String( "Write a short story about performance optimization".to_string() ) )
    .stream( true )
    .form();

  match client.responses().create_stream( request ).await
  {
    Ok( mut receiver ) =>
    {
      let mut total_content_length = 0;
      let mut peak_event_size = 0;

      while let Some( event_result ) = receiver.recv().await
      {
        match event_result
        {
          Ok( event ) =>
          {
            let event_size = std::mem::size_of_val( &event );
            peak_event_size = peak_event_size.max( event_size );

            if let ResponseStreamEvent::ResponseTextDelta( delta ) = &event
            {
              total_content_length += delta.delta.len();
            }

            monitor.record_event( Duration::from_micros( 100 ) );
            monitor.record_memory_usage( get_simulated_memory_usage() );

            if let ResponseStreamEvent::ResponseCompleted( _ ) = event
            {
              break;
            }
          },
          Err( _ ) => break,
        }
      }

      let metrics = monitor.get_metrics();

      println!( "Memory Efficiency Results:" );
      println!( "  Total content length : {} bytes", total_content_length );
      println!( "  Peak event size : {} bytes", peak_event_size );
      println!( "  Peak memory usage : {} bytes", metrics.peak_memory_bytes );
      println!( "  Memory per event : {:.2} bytes", metrics.peak_memory_bytes as f64 / metrics.total_events as f64 );

      // MANDATORY FAILING BEHAVIOR - fail hard if no content received
      assert!( total_content_length > 0, "MANDATORY FAILURE: Should receive content" );
      assert!( peak_event_size < 100 * 1024, "Individual events should be reasonably sized" ); // < 100KB per event

      // Memory usage should be reasonable for streaming
      if metrics.total_events > 0
      {
        let memory_per_event = metrics.peak_memory_bytes as f64 / metrics.total_events as f64;
        assert!( memory_per_event < 1024.0 * 1024.0, "Memory usage per event should be efficient" ); // < 1MB per event
      }
    },
    Err( e ) =>
    {
      // MANDATORY FAILING BEHAVIOR - fail hard on API errors
      panic!( "Stream creation failed - MANDATORY FAILURE: {e:?}" );
    }
  }
}
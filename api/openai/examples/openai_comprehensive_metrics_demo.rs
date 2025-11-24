//! Comprehensive Metrics Collection Framework Demo
//!
//! This example demonstrates the complete metrics collection framework for the OpenAI API client,
//! including connection management, response caching, circuit breaker integration, and
//! comprehensive performance monitoring and analysis.

use api_openai::ClientApiAccessors;
use api_openai::
{
  environment ::DevEnvironment,
  chat ::CreateChatCompletionRequest,
  enhanced_client ::{ EnhancedClient, EnhancedClientBuilder },
  connection_manager ::ConnectionConfig,
  response_cache ::CacheConfig,
  enhanced_circuit_breaker ::EnhancedCircuitBreakerConfig,
  metrics_framework ::MetricsConfig,
  error ::Result,
};
use std::time::Duration;

#[ tokio::main ]
async fn main() -> Result< () >
{
  println!( "📊 Comprehensive Metrics Collection Framework Demo\n" );

  // Create environment
  let environment = DevEnvironment::new();

  // Configure comprehensive metrics collection
  let metrics_config = MetricsConfig
  {
    collect_connection_metrics : true,
    collect_cache_metrics : true,
    collect_circuit_breaker_metrics : true,
    collect_timing_metrics : true,
    collect_error_metrics : true,
    max_entries : 1000,
    collection_interval : Duration::from_secs( 5 ), // Collect every 5 seconds
    retention_period : Duration::from_secs( 1800 ), // Retain for 30 minutes
    enable_streaming : false,
  };

  // Configure connection management for optimal performance
  let connection_config = ConnectionConfig
  {
    max_connections_per_host : 8,
    min_connections_per_host : 2,
    idle_timeout : Duration::from_secs( 300 ),
    adaptive_pooling : true,
    enable_connection_warming : true,
    health_check_interval : Duration::from_secs( 30 ),
  };

  // Configure intelligent response caching
  let cache_config = CacheConfig
  {
    max_entries : 200,
    default_ttl : Duration::from_secs( 900 ), // 15 minutes
    max_response_size : 1024 * 1024, // 1MB
    enable_compression : true,
    cache_errors : false,
    cleanup_interval : Duration::from_secs( 30 ),
  };

  // Configure circuit breaker for fault tolerance
  let circuit_breaker_config = EnhancedCircuitBreakerConfig
  {
    failure_threshold : 5, // Open after 5 consecutive failures
    recovery_timeout_ms : 10000, // Wait 10 seconds before attempting recovery
    success_threshold : 3, // Close after 3 successful requests in half-open state
    half_open_max_requests : 5, // Allow up to 5 requests in half-open state
    half_open_timeout_ms : 15000, // Half-open state times out after 15 seconds
  };

  // Build enhanced client with full configuration using the builder pattern
  let client = EnhancedClientBuilder::new()
    .max_connections_per_host( 8 )
    .min_connections_per_host( 2 )
    .idle_timeout( Duration::from_secs( 300 ) )
    .adaptive_pooling( true )
    .connection_warming( true )
    .health_check_interval( Duration::from_secs( 30 ) )
    .with_cache( cache_config )
    .with_circuit_breaker( circuit_breaker_config )
    .with_metrics( metrics_config )
    .build( environment )?;

  println!( "✅ Enhanced client created with comprehensive monitoring:" );
  println!( "   - Connection Management : ✓" );
  println!( "   - Response Caching : {}", if client.is_caching_enabled() { "✓" } else { "✗" } );
  println!( "   - Circuit Breaker : {}", if client.is_circuit_breaker_enabled() { "✓" } else { "✗" } );
  println!( "   - Metrics Collection : {}", if client.is_metrics_enabled() { "✓" } else { "✗" } );

  // Warm up connections
  println!( "\n🔄 Warming up connections..." );
  client.warm_up_connections( vec![ "api.openai.com" ], 3 ).await?;

  // Create test requests for metrics collection
  let request1 = CreateChatCompletionRequest::former()
    .model( "gpt-5-nano".to_string() )
    .messages( vec![ api_openai::chat::Message::user( "What is performance monitoring?".to_string() ) ] )
    .max_tokens( 150 )
    .form();

  let request2 = CreateChatCompletionRequest::former()
    .model( "gpt-5-nano".to_string() )
    .messages( vec![ api_openai::chat::Message::user( "Explain metrics collection in software systems".to_string() ) ] )
    .max_tokens( 200 )
    .form();

  println!( "\n🧪 Executing requests for metrics collection...\n" );

  // Execute multiple requests to generate meaningful metrics
  let mut successful_requests = 0;
  let mut failed_requests = 0;

  for i in 1..=15
  {
    println!( "Request #{}: ", i );

    let request = if i % 2 == 0 { &request2 } else { &request1 };
    let cache_ttl = if i % 3 == 0 { Some( Duration::from_secs( 600 ) ) } else { None };

    let start_time = std::time::Instant::now();

    match client.post_cached(
      "/chat/completions",
      request,
      cache_ttl
    ).await
    {
      Ok( _response ) =>
      {
        successful_requests += 1;
        let duration = start_time.elapsed();
        println!( "  ✅ Success ({:.3}s) {}", duration.as_secs_f64(), if cache_ttl.is_some() { "(cached)" } else { "" } );
      },
      Err( e ) =>
      {
        failed_requests += 1;
        println!( "  ❌ Failed : {}", e );
      }
    }

    // Collect and display metrics every 5 requests
    if i % 5 == 0
    {
      println!( "\n📊 Metrics Snapshot (after {} requests):", i );

      if let Some( snapshot ) = client.get_metrics_snapshot().await
      {
        // Connection metrics
        if let Some( ref conn_metrics ) = snapshot.connection_metrics
        {
          println!( "   🔗 Connection Performance:" );
          println!( "     - Efficiency Score : {:.1}%", conn_metrics.efficiency_score * 100.0 );
          println!( "     - Connection Reuse Ratio : {:.1}", conn_metrics.connection_reuse_ratio );
          println!( "     - Pool Utilization : {:.1}%", conn_metrics.average_pool_utilization * 100.0 );
          println!( "     - Active Connections : {}", conn_metrics.active_connections );
          println!( "     - Health Score : {:.1}", conn_metrics.health_score );
        }

        // Cache metrics
        if let Some( ref cache_metrics ) = snapshot.cache_metrics
        {
          println!( "   💾 Cache Performance:" );
          println!( "     - Hit Ratio : {:.1}%", cache_metrics.hit_ratio * 100.0 );
          println!( "     - Total Requests : {}", cache_metrics.total_requests );
          println!( "     - Cache Hits : {}", cache_metrics.cache_hits );
          println!( "     - Current Entries : {}", cache_metrics.current_entries );
          println!( "     - Efficiency Score : {:.1}", cache_metrics.efficiency_score );
        }

        // Timing metrics
        if let Some( ref timing_metrics ) = snapshot.timing_metrics
        {
          println!( "   ⏱️ Timing Performance:" );
          println!( "     - Average Duration : {:.1}ms", timing_metrics.average_duration_ms );
          println!( "     - Min Duration : {:.1}ms", timing_metrics.min_duration_ms );
          println!( "     - Max Duration : {:.1}ms", timing_metrics.max_duration_ms );
          println!( "     - P95 Duration : {:.1}ms", timing_metrics.p95_duration_ms );
          println!( "     - P99 Duration : {:.1}ms", timing_metrics.p99_duration_ms );
          println!( "     - Total Requests : {}", timing_metrics.total_requests );
        }

        // Error metrics
        if let Some( ref error_metrics ) = snapshot.error_metrics
        {
          println!( "   ⚠️ Error Tracking:" );
          println!( "     - Total Errors : {}", error_metrics.total_errors );
          println!( "     - Error Rate : {:.1}/min", error_metrics.error_rate_per_minute );
          println!( "     - Trend : {}", error_metrics.trend );
          if let Some( ref most_common ) = error_metrics.most_common_error
          {
            println!( "     - Most Common Error : {}", most_common );
          }
        }
      }

      println!();
    }

    // Small delay between requests
    tokio ::time::sleep( Duration::from_millis( 200 ) ).await;
  }

  // Generate comprehensive analysis report
  println!( "\n📈 Comprehensive Performance Analysis\n" );

  if let Some( analysis ) = client.get_metrics_analysis().await
  {
    println!( "🏆 Overall Performance Grade : {}", analysis.performance_grade );
    println!( "💯 Health Score : {:.1}%", analysis.health_score * 100.0 );
    println!( "🚨 Risk Level : {}", analysis.risk_level );

    if !analysis.kpis.is_empty()
    {
      println!( "\n📊 Key Performance Indicators:" );
      for kpi in &analysis.kpis
      {
        println!( "   • {}", kpi );
      }
    }

    if !analysis.trends.is_empty()
    {
      println!( "\n📈 Performance Trends:" );
      for trend in &analysis.trends
      {
        println!( "   • {}", trend );
      }
    }

    if !analysis.issues.is_empty()
    {
      println!( "\n⚠️ Issues Identified:" );
      for issue in &analysis.issues
      {
        println!( "   • {}", issue );
      }
    }

    if !analysis.recommendations.is_empty()
    {
      println!( "\n💡 Recommendations:" );
      for rec in &analysis.recommendations
      {
        println!( "   • {}", rec );
      }
    }
  }

  // Export metrics demonstrations
  println!( "\n📤 Metrics Export Capabilities\n" );

  // JSON export
  match client.export_metrics_json().await
  {
    Ok( json_data ) =>
    {
      let entry_count = json_data.matches( "timestamp" ).count();
      println!( "✅ JSON Export : {} metrics entries exported", entry_count );
    },
    Err( e ) =>
    {
      println!( "❌ JSON Export failed : {}", e );
    }
  }

  // Prometheus export
  let prometheus_data = client.export_metrics_prometheus().await;
  let metric_count = prometheus_data.matches( "# HELP" ).count();
  println!( "✅ Prometheus Export : {} metrics available", metric_count );

  // Final summary
  println!( "\n📋 Execution Summary:" );
  println!( "   Total Requests : {}", successful_requests + failed_requests );
  println!( "   Successful : {}", successful_requests );
  println!( "   Failed : {}", failed_requests );
  println!( "   Success Rate : {:.1}%", ( successful_requests as f64 / ( successful_requests + failed_requests ) as f64 ) * 100.0 );

  // Performance benefits summary
  println!( "\n🎯 Metrics Framework Benefits Demonstrated:" );
  println!( "   ✓ Real-time performance monitoring" );
  println!( "   ✓ Comprehensive connection pool analysis" );
  println!( "   ✓ Cache efficiency tracking" );
  println!( "   ✓ Request timing and latency analysis" );
  println!( "   ✓ Error pattern detection and tracking" );
  println!( "   ✓ Automated performance grading" );
  println!( "   ✓ Actionable recommendations" );
  println!( "   ✓ Multiple export formats (JSON, Prometheus)" );
  println!( "   ✓ Historical data retention and analysis" );

  println!( "\n✅ Comprehensive metrics collection framework demonstration completed!" );
  println!( "🔍 The system provides deep insights into API client performance," );
  println!( "   enabling data-driven optimization and proactive issue detection." );

  Ok( () )
}
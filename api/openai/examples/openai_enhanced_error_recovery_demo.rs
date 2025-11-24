//! Enhanced Error Recovery with Circuit Breaker Patterns
//!
//! This example demonstrates the enhanced OpenAI client with comprehensive error recovery
//! capabilities including circuit breaker patterns, connection management, and response caching.

use api_openai::ClientApiAccessors;
use api_openai::
{
  environment ::DevEnvironment,
  chat ::CreateChatCompletionRequest,
  enhanced_client ::{ EnhancedClient, EnhancedClientBuilder },
  connection_manager ::ConnectionConfig,
  response_cache ::CacheConfig,
  enhanced_circuit_breaker ::EnhancedCircuitBreakerConfig,
  error ::Result,
};
use std::time::Duration;

#[ tokio::main ]
async fn main() -> Result< () >
{
  println!( "🛡️ Enhanced Error Recovery with Circuit Breaker Demo\n" );

  // Create environment
  let environment = DevEnvironment::new();

  // Configure connection management for resilience
  let connection_config = ConnectionConfig
  {
    max_connections_per_host : 5,
    min_connections_per_host : 1,
    idle_timeout : Duration::from_secs( 180 ),
    adaptive_pooling : true,
    enable_connection_warming : true,
    health_check_interval : Duration::from_secs( 30 ),
  };

  // Configure response caching for efficiency
  let cache_config = CacheConfig
  {
    max_entries : 50,
    default_ttl : Duration::from_secs( 600 ), // 10 minutes
    max_response_size : 512 * 1024, // 512KB
    enable_compression : true,
    cache_errors : false,
    cleanup_interval : Duration::from_secs( 60 ),
  };

  // Configure circuit breaker for fault tolerance
  let circuit_breaker_config = EnhancedCircuitBreakerConfig
  {
    failure_threshold : 3, // Open after 3 consecutive failures
    recovery_timeout_ms : 5000, // Wait 5 seconds before attempting recovery
    success_threshold : 2, // Close after 2 successful requests in half-open state
    half_open_max_requests : 3, // Allow up to 3 requests in half-open state
    half_open_timeout_ms : 10000, // Half-open state times out after 10 seconds
  };

  // Build enhanced client using the builder pattern
  let client = EnhancedClientBuilder::new()
    .max_connections_per_host( 5 )
    .min_connections_per_host( 1 )
    .idle_timeout( Duration::from_secs( 180 ) )
    .adaptive_pooling( true )
    .connection_warming( true )
    .health_check_interval( Duration::from_secs( 30 ) )
    .with_cache( cache_config )
    .with_circuit_breaker( circuit_breaker_config )
    .build( environment )?;

  println!( "✅ Enhanced client created with all reliability features:" );
  println!( "   - Connection Management : Enabled" );
  println!( "   - Response Caching : {}", if client.is_caching_enabled() { "Enabled" } else { "Disabled" } );
  println!( "   - Circuit Breaker : {}", if client.is_circuit_breaker_enabled() { "Enabled" } else { "Disabled" } );

  // Display initial circuit breaker state
  #[ cfg( feature = "circuit_breaker" ) ]
  {
    if let Some( cb_state ) = client.get_circuit_breaker_state().await
    {
      println!( "   - Circuit Breaker State : {:?}", cb_state );
    }
  }

  // Warm up connections
  println!( "\n🔄 Warming up connections..." );
  client.warm_up_connections( vec![ "api.openai.com" ], 2 ).await?;

  // Create a test chat completion request
  let request = CreateChatCompletionRequest::former()
    .model( "gpt-5-nano".to_string() )
    .messages( vec![ api_openai::chat::Message::user( "Explain circuit breaker pattern in 2 sentences".to_string() ) ] )
    .max_tokens( 100 )
    .form();

  // Demonstrate error recovery with multiple requests
  println!( "\n🧪 Testing Error Recovery Patterns\n" );

  let mut successful_requests = 0;
  let mut failed_requests = 0;

  for i in 1..=10
  {
    println!( "Request #{}: ", i );

    match client.post_cached(
      "/chat/completions",
      &request,
      Some( Duration::from_secs( 300 ) )
    ).await
    {
      Ok( _response ) =>
      {
        successful_requests += 1;
        println!( "  ✅ Success (cached response possible)" );
      },
      Err( e ) =>
      {
        failed_requests += 1;
        println!( "  ❌ Failed : {}", e );

        // Check circuit breaker state after failure
        #[ cfg( feature = "circuit_breaker" ) ]
        {
          if let Some( cb_state ) = client.get_circuit_breaker_state().await
          {
            println!( "  🔌 Circuit Breaker State : {:?}", cb_state );
          }
        }
      }
    }

    // Show statistics periodically
    if i % 3 == 0
    {
      println!( "\n📊 Performance Statistics (after {} requests):", i );

      // Circuit breaker statistics
      #[ cfg( feature = "circuit_breaker" ) ]
      {
        if let Some( cb_stats ) = client.get_circuit_breaker_stats().await
        {
          println!( "   Circuit Breaker:" );
          println!( "     - Total Requests : {}", cb_stats.total_requests );
          println!( "     - Total Failures : {}", cb_stats.total_failures );
          println!( "     - Trip Count : {}", cb_stats.trip_count );
          println!( "     - Current State : {:?}", cb_stats.state );
        }
      }

      // Cache statistics
      if let Some( cache_stats ) = client.get_cache_statistics().await
      {
        println!( "   Response Cache:" );
        println!( "     - Total Requests : {}", cache_stats.total_requests );
        println!( "     - Cache Hits : {}", cache_stats.cache_hits );
        println!( "     - Hit Ratio : {:.1}%", cache_stats.hit_ratio * 100.0 );
        println!( "     - Current Entries : {}", cache_stats.current_entries );
      }

      // Connection statistics
      let conn_stats = client.get_connection_stats().await;
      println!( "   Connection Management:" );
      println!( "     - Efficiency Score : {:.1}%", conn_stats.efficiency_score * 100.0 );
      println!( "     - Total Requests : {}", conn_stats.total_requests_served );
      println!( "     - Avg Response Time : {:.3}s", conn_stats.average_response_time_seconds );

      println!();
    }

    // Small delay between requests
    tokio ::time::sleep( Duration::from_millis( 500 ) ).await;
  }

  // Final comprehensive report
  println!( "\n📈 Final Error Recovery Report:" );
  println!( "   Successful Requests : {}", successful_requests );
  println!( "   Failed Requests : {}", failed_requests );
  println!( "   Success Rate : {:.1}%", ( successful_requests as f64 / 10.0 ) * 100.0 );

  #[ cfg( feature = "circuit_breaker" ) ]
  {
    if let Some( final_cb_stats ) = client.get_circuit_breaker_stats().await
    {
      println!( "\n🔌 Circuit Breaker Final Statistics:" );
      println!( "   - Total Requests : {}", final_cb_stats.total_requests );
      println!( "   - Total Failures : {}", final_cb_stats.total_failures );
      println!( "   - Trip Count : {}", final_cb_stats.trip_count );
      println!( "   - Final State : {:?}", final_cb_stats.state );

      if final_cb_stats.trip_count > 0
      {
        println!( "   ⚠️  Circuit breaker activated {} times to protect system", final_cb_stats.trip_count );
      }
      else
      {
        println!( "   ✅ Circuit breaker remained stable throughout test" );
      }
    }
  }

  // Performance report
  let performance_report = client.generate_performance_report().await;
  println!( "\n📊 Performance Analysis:" );
  println!( "   Overall Grade : {}", performance_report.analysis.grade );

  if !performance_report.analysis.kpis.is_empty()
  {
    println!( "   Key Performance Indicators:" );
    for kpi in &performance_report.analysis.kpis
    {
      println!( "     • {}", kpi );
    }
  }

  if !performance_report.analysis.recommendations.is_empty()
  {
    println!( "   Recommendations:" );
    for rec in &performance_report.analysis.recommendations
    {
      println!( "     • {}", rec );
    }
  }

  if !performance_report.analysis.issues.is_empty()
  {
    println!( "   Issues Identified:" );
    for issue in &performance_report.analysis.issues
    {
      println!( "     ⚠️ {}", issue );
    }
  }

  // Demonstrate circuit breaker reset
  #[ cfg( feature = "circuit_breaker" ) ]
  {
    if client.reset_circuit_breaker().await
    {
      println!( "\n🔄 Circuit breaker has been reset to closed state" );
      if let Some( reset_state ) = client.get_circuit_breaker_state().await
      {
        println!( "   New state : {:?}", reset_state );
      }
    }
  }

  println!( "\n✅ Enhanced error recovery demonstration completed!" );
  println!( "🛡️ The system demonstrated resilience through:" );
  println!( "   - Connection pooling and health management" );
  println!( "   - Intelligent response caching" );
  println!( "   - Circuit breaker fault tolerance" );
  println!( "   - Comprehensive performance monitoring" );

  Ok( () )
}
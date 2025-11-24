//! Enhanced OpenAI Client with Connection Management and Response Caching
//!
//! This example demonstrates the usage of the enhanced OpenAI client that provides:
//! - Advanced HTTP connection pooling and management
//! - Intelligent response caching with TTL (Time To Live)
//! - Performance monitoring and analytics

use api_openai::ClientApiAccessors;
use api_openai::
{
  environment ::DevEnvironment,
  chat ::CreateChatCompletionRequest,
  enhanced_client ::{ EnhancedClient, EnhancedClientBuilder },
  connection_manager ::ConnectionConfig,
  response_cache ::CacheConfig,
  error ::Result,
};
use std::time::Duration;

#[ tokio::main ]
async fn main() -> Result< () >
{
  println!( "🚀 Enhanced OpenAI Client with Caching Demo\n" );

  // Create environment
  let environment = DevEnvironment::new();

  // Configure connection management
  let connection_config = ConnectionConfig
  {
    max_connections_per_host : 10,
    min_connections_per_host : 2,
    idle_timeout : Duration::from_secs( 300 ),
    adaptive_pooling : true,
    enable_connection_warming : true,
    health_check_interval : Duration::from_secs( 60 ),
  };

  // Configure response caching
  let cache_config = CacheConfig
  {
    max_entries : 100,
    default_ttl : Duration::from_secs( 300 ), // 5 minutes
    max_response_size : 1024 * 1024, // 1MB
    enable_compression : true,
    cache_errors : false,
    cleanup_interval : Duration::from_secs( 30 ),
  };

  // Build enhanced client with both connection management and caching
  let client = EnhancedClient::build_with_caching(
    environment,
    connection_config,
    cache_config
  ).await?;

  println!( "✅ Enhanced client created with caching enabled : {}", client.is_caching_enabled() );

  // Warm up connections to OpenAI
  println!( "\n🔄 Warming up connections..." );
  client.warm_up_connections( vec![ "api.openai.com" ], 3 ).await?;

  // Create a simple chat completion request
  let request = CreateChatCompletionRequest::former()
    .model( "gpt-5-nano".to_string() )
    .messages( vec![ api_openai::chat::Message::user( "What is Rust programming language?".to_string() ) ] )
    .max_tokens( 100 )
    .form();

  // First request - should miss cache and make actual API call
  println!( "\n📞 Making first request (cache miss expected)..." );
  let start_time = std::time::Instant::now();

  let response1 = client.post_cached(
    "/chat/completions",
    &request,
    Some( Duration::from_secs( 300 ) ) // Cache for 5 minutes
  ).await?;

  let first_duration = start_time.elapsed();
  println!( "⏱️ First request took : {:?}", first_duration );

  // Second identical request - should hit cache
  println!( "\n📞 Making second identical request (cache hit expected)..." );
  let start_time = std::time::Instant::now();

  let response2 = client.post_cached(
    "/chat/completions",
    &request,
    Some( Duration::from_secs( 300 ) )
  ).await?;

  let second_duration = start_time.elapsed();
  println!( "⏱️ Second request took : {:?}", second_duration );

  // Display cache statistics
  if let Some( cache_stats ) = client.get_cache_statistics().await
  {
    println!( "\n📊 Cache Performance Statistics:" );
    println!( "   Total requests : {}", cache_stats.total_requests );
    println!( "   Cache hits : {}", cache_stats.cache_hits );
    println!( "   Cache misses : {}", cache_stats.cache_misses );
    println!( "   Hit ratio : {:.1}%", cache_stats.hit_ratio * 100.0 );
    println!( "   Current entries : {}", cache_stats.current_entries );
    println!( "   Total cached bytes : {}", cache_stats.total_cached_bytes );
    println!( "   Average TTL: {:.1}s", cache_stats.average_ttl_seconds );
  }

  // Display connection statistics
  let connection_stats = client.get_connection_stats().await;
  println!( "\n🔗 Connection Performance Statistics:" );
  println!( "   Efficiency score : {:.1}%", connection_stats.efficiency_score * 100.0 );
  println!( "   Connection reuse ratio : {:.1}", connection_stats.connection_reuse_ratio );
  println!( "   Average pool utilization : {:.1}%", connection_stats.average_pool_utilization * 100.0 );
  println!( "   Total requests served : {}", connection_stats.total_requests_served );
  println!( "   Average response time : {:.3}s", connection_stats.average_response_time_seconds );

  // Generate comprehensive performance report
  let performance_report = client.generate_performance_report().await;
  println!( "\n📈 Performance Report:" );
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

  // Test different cache scenarios
  println!( "\n🧪 Testing cache scenarios..." );

  // Request with no TTL (should not be cached)
  println!( "   Making request without TTL (no caching)..." );
  let _response3 = client.post_cached(
    "/chat/completions",
    &request,
    None // No TTL, won't be cached
  ).await?;

  // GET request with caching
  println!( "   Making GET request with caching..." );
  let _models : serde_json::Value = client.get_cached(
    "/models",
    Some( Duration::from_secs( 600 ) ) // Cache for 10 minutes
  ).await?;

  // Final cache statistics
  if let Some( final_stats ) = client.get_cache_statistics().await
  {
    println!( "\n📊 Final Cache Statistics:" );
    println!( "   Total requests : {}", final_stats.total_requests );
    println!( "   Cache hits : {}", final_stats.cache_hits );
    println!( "   Cache misses : {}", final_stats.cache_misses );
    println!( "   Hit ratio : {:.1}%", final_stats.hit_ratio * 100.0 );
    println!( "   Current entries : {}", final_stats.current_entries );
  }

  println!( "\n✅ Enhanced client demo completed successfully!" );
  println!( "🔄 Performance improvement from caching : {:.1}x faster",
    first_duration.as_millis() as f64 / second_duration.as_millis() as f64 );

  Ok( () )
}
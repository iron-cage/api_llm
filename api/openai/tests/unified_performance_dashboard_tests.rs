//! Unified Performance Dashboard Tests
//!
//! Tests for the integrated performance monitoring and analytics dashboard.

use api_openai::enhanced_client ::
{
  UnifiedPerformanceDashboard,
  ConnectionPerformanceReport,
  PerformanceAnalysis,
};

#[ cfg( feature = "integration" ) ]
use api_openai::
{
  environment ::{ OpenaiEnvironmentImpl, OpenAIRecommended },
  secret ::Secret,
  enhanced_client ::EnhancedClientBuilder,
  response_cache ::CacheConfig,
  metrics_framework ::MetricsConfig,
};

#[ cfg( feature = "integration" ) ]
use secrecy::ExposeSecret;

#[ tokio::test ]
#[ cfg( feature = "integration" ) ]
async fn test_enhanced_client_builder()
{
  let secret = Secret::new_unchecked( "sk-test_enhanced_1234567890abcdef".to_string() );
  let environment = OpenaiEnvironmentImpl::build(
    secret,
    None,
    None,
    OpenAIRecommended::base_url().to_string(),
    OpenAIRecommended::realtime_base_url().to_string(),
  ).expect( "Environment creation should work" );

  let enhanced_client = EnhancedClientBuilder::new()
    .max_connections_per_host( 20 )
    .min_connections_per_host( 2 )
    .with_cache( CacheConfig::default() )
    .with_default_metrics()
    .build( environment )
    .expect( "Enhanced client creation should work" );

  // Verify client was created successfully
  let base_client = enhanced_client.base_client();
  assert!( !base_client.environment.api_key.expose_secret().is_empty() );
}

#[ tokio::test ]
#[ cfg( feature = "integration" ) ]
async fn test_unified_performance_dashboard_creation()
{
  let secret = Secret::new_unchecked( "sk-test_dashboard_1234567890abcdef".to_string() );
  let environment = OpenaiEnvironmentImpl::build(
    secret,
    None,
    None,
    OpenAIRecommended::base_url().to_string(),
    OpenAIRecommended::realtime_base_url().to_string(),
  ).expect( "Environment creation should work" );

  let enhanced_client = EnhancedClientBuilder::new()
    .with_cache( CacheConfig::default() )
    .with_default_metrics()
    .build( environment )
    .expect( "Enhanced client creation should work" );

  // Get unified performance dashboard
  let dashboard = enhanced_client.get_unified_performance_dashboard().await
    .expect( "Should be able to get performance dashboard" );

  // Verify dashboard structure
  assert!( dashboard.overall_performance_score >= 0.0 );
  assert!( dashboard.overall_performance_score <= 100.0 );
  assert!( !dashboard.recommendations.is_empty() ); // Should have some recommendations
}

#[ tokio::test ]
#[ cfg( feature = "integration" ) ]
async fn test_connection_performance_metrics()
{
  let secret = Secret::new_unchecked( "sk-test_connection_1234567890abcdef".to_string() );
  let environment = OpenaiEnvironmentImpl::build(
    secret,
    None,
    None,
    OpenAIRecommended::base_url().to_string(),
    OpenAIRecommended::realtime_base_url().to_string(),
  ).expect( "Environment creation should work" );

  let enhanced_client = EnhancedClientBuilder::new()
    .max_connections_per_host( 15 )
    .build( environment )
    .expect( "Enhanced client creation should work" );

  let performance_report = enhanced_client.generate_performance_report().await;

  // Verify performance report structure
  assert!( performance_report.efficiency_metrics.efficiency_score >= 0.0 );
  assert!( performance_report.efficiency_metrics.efficiency_score <= 1.0 );
  // pool_stats may be empty initially
  assert!( !performance_report.analysis.grade.is_empty() );
}

#[ tokio::test ]
#[ cfg( feature = "integration" ) ]
async fn test_cache_performance_integration()
{
  let secret = Secret::new_unchecked( "sk-test_cache_1234567890abcdef".to_string() );
  let environment = OpenaiEnvironmentImpl::build(
    secret,
    None,
    None,
    OpenAIRecommended::base_url().to_string(),
    OpenAIRecommended::realtime_base_url().to_string(),
  ).expect( "Environment creation should work" );

  let cache_config = CacheConfig
  {
    max_entries : 100,
    default_ttl : core::time::Duration::from_secs( 300 ),
    enable_compression : true,
    max_response_size : 1024 * 1024, // 1MB
    cache_errors : false,
    cleanup_interval : core::time::Duration::from_secs( 60 ),
  };

  let enhanced_client = EnhancedClientBuilder::new()
    .with_cache( cache_config )
    .build( environment )
    .expect( "Enhanced client creation should work" );

  let dashboard = enhanced_client.get_unified_performance_dashboard().await
    .expect( "Should get dashboard" );

  // Should include cache performance data
  assert!( dashboard.cache_performance.is_some() );

  if let Some( cache_stats ) = dashboard.cache_performance
  {
    assert!( cache_stats.hit_ratio >= 0.0 );
    assert!( cache_stats.hit_ratio <= 1.0 );
    assert_eq!( cache_stats.total_requests, cache_stats.cache_hits + cache_stats.cache_misses );
  }
}

#[ tokio::test ]
#[ cfg( feature = "integration" ) ]
async fn test_metrics_integration()
{
  let secret = Secret::new_unchecked( "sk-test_metrics_1234567890abcdef".to_string() );
  let environment = OpenaiEnvironmentImpl::build(
    secret,
    None,
    None,
    OpenAIRecommended::base_url().to_string(),
    OpenAIRecommended::realtime_base_url().to_string(),
  ).expect( "Environment creation should work" );

  let metrics_config = MetricsConfig
  {
    collect_circuit_breaker_metrics : true,
    collect_timing_metrics : true,
    collect_error_metrics : true,
    max_entries : 100,
    collection_interval : core::time::Duration::from_secs( 60 ),
    enable_streaming : false,
    collect_cache_metrics : true,
    collect_connection_metrics : true,
    retention_period : core::time::Duration::from_secs( 3600 ),
  };

  let enhanced_client = EnhancedClientBuilder::new()
    .with_metrics( metrics_config )
    .build( environment )
    .expect( "Enhanced client creation should work" );

  let dashboard = enhanced_client.get_unified_performance_dashboard().await
    .expect( "Should get dashboard" );

  // Should include metrics summary data
  assert!( dashboard.metrics_summary.is_some() );

  if let Some( metrics_snapshot ) = dashboard.metrics_summary
  {
    assert!( metrics_snapshot.timestamp > 0 );
    // Verify that snapshot contains some data
    assert!( metrics_snapshot.connection_metrics.is_some() ||
             metrics_snapshot.timing_metrics.is_some() ||
             metrics_snapshot.error_metrics.is_some() );
  }
}

#[ tokio::test ]
#[ cfg( feature = "integration" ) ]
async fn test_performance_score_calculation()
{
  let secret = Secret::new_unchecked( "sk-test_score_1234567890abcdef".to_string() );
  let environment = OpenaiEnvironmentImpl::build(
    secret,
    None,
    None,
    OpenAIRecommended::base_url().to_string(),
    OpenAIRecommended::realtime_base_url().to_string(),
  ).expect( "Environment creation should work" );

  // Create client with both caching and metrics
  let enhanced_client = EnhancedClientBuilder::new()
    .with_cache( CacheConfig::default() )
    .with_default_metrics()
    .build( environment )
    .expect( "Enhanced client creation should work" );

  let dashboard = enhanced_client.get_unified_performance_dashboard().await
    .expect( "Should get dashboard" );

  // Performance score should be calculated from available components
  assert!( dashboard.overall_performance_score >= 0.0 );
  assert!( dashboard.overall_performance_score <= 100.0 );

  // With both cache and metrics enabled, should have higher score potential
  // (though actual score depends on usage patterns)
}

#[ tokio::test ]
#[ cfg( feature = "integration" ) ]
async fn test_recommendation_generation()
{
  let secret = Secret::new_unchecked( "sk-test_recommendations_1234567890abcdef".to_string() );
  let environment = OpenaiEnvironmentImpl::build(
    secret,
    None,
    None,
    OpenAIRecommended::base_url().to_string(),
    OpenAIRecommended::realtime_base_url().to_string(),
  ).expect( "Environment creation should work" );

  // Create client without caching or metrics to generate recommendations
  let enhanced_client = EnhancedClientBuilder::new()
    .build( environment )
    .expect( "Enhanced client creation should work" );

  let dashboard = enhanced_client.get_unified_performance_dashboard().await
    .expect( "Should get dashboard" );

  // Should recommend enabling caching and metrics
  assert!( dashboard.recommendations.iter().any( | rec |
    rec.contains( "caching" ) || rec.contains( "cache" )
  ) );
  assert!( dashboard.recommendations.iter().any( | rec |
    rec.contains( "metrics" ) || rec.contains( "monitoring" )
  ) );
}

#[ tokio::test ]
#[ cfg( feature = "integration" ) ]
async fn test_performance_analysis_grading()
{
  let secret = Secret::new_unchecked( "sk-test_grading_1234567890abcdef".to_string() );
  let environment = OpenaiEnvironmentImpl::build(
    secret,
    None,
    None,
    OpenAIRecommended::base_url().to_string(),
    OpenAIRecommended::realtime_base_url().to_string(),
  ).expect( "Environment creation should work" );

  let enhanced_client = EnhancedClientBuilder::new()
    .build( environment )
    .expect( "Enhanced client creation should work" );

  let performance_report = enhanced_client.generate_performance_report().await;

  // Verify grading system
  let valid_grades = ["A", "B", "C", "D", "F"];
  assert!( valid_grades.contains( &performance_report.analysis.grade.as_str() ) );

  // Should have KPIs
  assert!( !performance_report.analysis.kpis.is_empty() );

  // Should have some analysis (recommendations or issues)
  let has_analysis = !performance_report.analysis.recommendations.is_empty()
    || !performance_report.analysis.issues.is_empty();
  assert!( has_analysis );
}

#[ tokio::test ]
async fn test_unified_dashboard_serialization()
{
  // Create a mock dashboard for serialization testing
  let dashboard = UnifiedPerformanceDashboard
  {
    overall_performance_score : 85.5,
    connection_performance : ConnectionPerformanceReport
    {
      efficiency_metrics : api_openai::connection_manager::ConnectionEfficiencyMetrics
      {
        efficiency_score : 0.85,
        connection_reuse_ratio : 15.2,
        average_pool_utilization : 0.65,
        total_connections_created : 100,
        total_requests_served : 1520,
        total_connections_destroyed : 20,
        active_pools : 3,
      },
      pool_stats : Vec::new(),
      analysis : PerformanceAnalysis
      {
        grade : "B".to_string(),
        kpis : vec![ "Efficiency Score : 85.0%".to_string() ],
        recommendations : vec![ "Consider connection pooling optimization".to_string() ],
        issues : Vec::new(),
      },
    },
    cache_performance : None,
    metrics_summary : None,
    recommendations : vec![
      "Enable response caching for better performance".to_string(),
      "Enable metrics collection for better monitoring".to_string(),
    ],
  };

  // Test serialization
  let json = serde_json::to_string( &dashboard ).expect( "Serialization should work" );
  let deserialized : UnifiedPerformanceDashboard = serde_json::from_str( &json )
    .expect( "Deserialization should work" );

  assert!( ( dashboard.overall_performance_score - deserialized.overall_performance_score ).abs() < f64::EPSILON );
  assert_eq!( dashboard.connection_performance.analysis.grade, deserialized.connection_performance.analysis.grade );
  assert_eq!( dashboard.recommendations.len(), deserialized.recommendations.len() );
}
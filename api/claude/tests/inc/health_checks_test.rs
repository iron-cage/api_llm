//! Health checks tests for Anthropic API client
//!
//! These tests cover the health check implementation including:
//! - Configuration and validation
//! - Health status determination
//! - Response time tracking
//! - Multiple check strategies
//! - Metrics aggregation
//! - Concurrent endpoint checking

use super::*;

/// Test health check configuration defaults
#[ test ]
fn test_health_check_config_defaults()
{
  let config = the_module::HealthCheckConfig::default();

  assert_eq!( config.timeout_ms, 5000 );
  assert_eq!( config.strategy, the_module::HealthCheckStrategy::LightweightApi );
  assert_eq!( config.degraded_threshold_ms, 1000 );
  assert_eq!( config.unhealthy_threshold_ms, 5000 );
  assert!( config.is_valid() );
}

/// Test health check configuration builder
#[ test ]
fn test_health_check_config_builder()
{
  let config = the_module::HealthCheckConfig::new()
    .with_timeout_ms( 3000 )
    .with_strategy( the_module::HealthCheckStrategy::Ping )
    .with_degraded_threshold_ms( 500 )
    .with_unhealthy_threshold_ms( 2000 );

  assert_eq!( config.timeout_ms, 3000 );
  assert_eq!( config.strategy, the_module::HealthCheckStrategy::Ping );
  assert_eq!( config.degraded_threshold_ms, 500 );
  assert_eq!( config.unhealthy_threshold_ms, 2000 );
  assert!( config.is_valid() );
}

/// Test health check configuration validation
#[ test ]
fn test_health_check_config_validation()
{
  // Valid configuration
  let valid_config = the_module::HealthCheckConfig::new()
    .with_timeout_ms( 5000 )
    .with_degraded_threshold_ms( 1000 )
    .with_unhealthy_threshold_ms( 3000 );
  assert!( valid_config.is_valid() );

  // Invalid : zero timeout
  let invalid_config = the_module::HealthCheckConfig::new()
    .with_timeout_ms( 0 );
  assert!( !invalid_config.is_valid() );

  // Invalid : zero degraded threshold
  let invalid_config = the_module::HealthCheckConfig::new()
    .with_degraded_threshold_ms( 0 );
  assert!( !invalid_config.is_valid() );

  // Invalid : unhealthy threshold less than degraded threshold
  let invalid_config = the_module::HealthCheckConfig::new()
    .with_degraded_threshold_ms( 2000 )
    .with_unhealthy_threshold_ms( 1000 );
  assert!( !invalid_config.is_valid() );

  // Invalid : timeout less than unhealthy threshold
  let invalid_config = the_module::HealthCheckConfig::new()
    .with_timeout_ms( 2000 )
    .with_unhealthy_threshold_ms( 5000 );
  assert!( !invalid_config.is_valid() );
}

/// Test health status types
#[ test ]
fn test_health_status_types()
{
  let healthy = the_module::EndpointHealthStatus::Healthy;
  let degraded = the_module::EndpointHealthStatus::Degraded;
  let unhealthy = the_module::EndpointHealthStatus::Unhealthy;

  assert_eq!( healthy, the_module::EndpointHealthStatus::Healthy );
  assert_eq!( degraded, the_module::EndpointHealthStatus::Degraded );
  assert_eq!( unhealthy, the_module::EndpointHealthStatus::Unhealthy );
}

/// Test health check result helpers
#[ test ]
fn test_health_check_result_helpers()
{
  let healthy_result = the_module::HealthCheckResult
  {
    endpoint_url : "https://api.anthropic.com".to_string(),
    status : the_module::EndpointHealthStatus::Healthy,
    response_time_ms : 250,
    error_message : None,
    timestamp : std::time::SystemTime::now(),
  };

  assert!( healthy_result.is_healthy() );
  assert!( healthy_result.is_available() );
  assert_eq!( healthy_result.response_time().as_millis(), 250 );

  let degraded_result = the_module::HealthCheckResult
  {
    endpoint_url : "https://api.anthropic.com".to_string(),
    status : the_module::EndpointHealthStatus::Degraded,
    response_time_ms : 1500,
    error_message : None,
    timestamp : std::time::SystemTime::now(),
  };

  assert!( !degraded_result.is_healthy() );
  assert!( degraded_result.is_available() );
  assert_eq!( degraded_result.response_time().as_millis(), 1500 );

  let unhealthy_result = the_module::HealthCheckResult
  {
    endpoint_url : "https://api.anthropic.com".to_string(),
    status : the_module::EndpointHealthStatus::Unhealthy,
    response_time_ms : 8000,
    error_message : Some( "Timeout".to_string() ),
    timestamp : std::time::SystemTime::now(),
  };

  assert!( !unhealthy_result.is_healthy() );
  assert!( !unhealthy_result.is_available() );
  assert_eq!( unhealthy_result.response_time().as_millis(), 8000 );
  assert!( unhealthy_result.error_message.is_some() );
}

/// Test health check strategies
#[ test ]
fn test_health_check_strategies()
{
  let ping_strategy = the_module::HealthCheckStrategy::Ping;
  let api_strategy = the_module::HealthCheckStrategy::LightweightApi;

  assert_eq!( ping_strategy, the_module::HealthCheckStrategy::Ping );
  assert_eq!( api_strategy, the_module::HealthCheckStrategy::LightweightApi );
}

/// Test health metrics with empty results
#[ test ]
fn test_health_metrics_empty()
{
  let results : Vec< the_module::HealthCheckResult > = vec![];
  let metrics = the_module::HealthMetrics::from_results( &results );

  assert_eq!( metrics.total_endpoints, 0 );
  assert_eq!( metrics.healthy_count, 0 );
  assert_eq!( metrics.degraded_count, 0 );
  assert_eq!( metrics.unhealthy_count, 0 );
  assert_eq!( metrics.average_response_time_ms, 0 );
  assert!( metrics.healthy_percentage().abs() < f64::EPSILON );
  assert!( metrics.available_percentage().abs() < f64::EPSILON );
}

/// Test health metrics with single result
#[ test ]
fn test_health_metrics_single_result()
{
  let results = vec![
    the_module::HealthCheckResult
    {
      endpoint_url : "https://api.anthropic.com".to_string(),
      status : the_module::EndpointHealthStatus::Healthy,
      response_time_ms : 300,
      error_message : None,
      timestamp : std::time::SystemTime::now(),
    },
  ];

  let metrics = the_module::HealthMetrics::from_results( &results );

  assert_eq!( metrics.total_endpoints, 1 );
  assert_eq!( metrics.healthy_count, 1 );
  assert_eq!( metrics.degraded_count, 0 );
  assert_eq!( metrics.unhealthy_count, 0 );
  assert_eq!( metrics.average_response_time_ms, 300 );
  assert_eq!( metrics.min_response_time_ms, 300 );
  assert_eq!( metrics.max_response_time_ms, 300 );
  assert!( ( metrics.healthy_percentage() - 100.0_f64 ).abs() < f64::EPSILON );
  assert!( ( metrics.available_percentage() - 100.0_f64 ).abs() < f64::EPSILON );
}

/// Test health metrics with multiple results
#[ test ]
fn test_health_metrics_multiple_results()
{
  let results = vec![
    the_module::HealthCheckResult
    {
      endpoint_url : "https://api1.anthropic.com".to_string(),
      status : the_module::EndpointHealthStatus::Healthy,
      response_time_ms : 200,
      error_message : None,
      timestamp : std::time::SystemTime::now(),
    },
    the_module::HealthCheckResult
    {
      endpoint_url : "https://api2.anthropic.com".to_string(),
      status : the_module::EndpointHealthStatus::Degraded,
      response_time_ms : 1200,
      error_message : None,
      timestamp : std::time::SystemTime::now(),
    },
    the_module::HealthCheckResult
    {
      endpoint_url : "https://api3.anthropic.com".to_string(),
      status : the_module::EndpointHealthStatus::Unhealthy,
      response_time_ms : 5000,
      error_message : Some( "Timeout".to_string() ),
      timestamp : std::time::SystemTime::now(),
    },
    the_module::HealthCheckResult
    {
      endpoint_url : "https://api4.anthropic.com".to_string(),
      status : the_module::EndpointHealthStatus::Healthy,
      response_time_ms : 400,
      error_message : None,
      timestamp : std::time::SystemTime::now(),
    },
  ];

  let metrics = the_module::HealthMetrics::from_results( &results );

  assert_eq!( metrics.total_endpoints, 4 );
  assert_eq!( metrics.healthy_count, 2 );
  assert_eq!( metrics.degraded_count, 1 );
  assert_eq!( metrics.unhealthy_count, 1 );

  // Average : (200 + 1200 + 5000 + 400) / 4 = 1700
  assert_eq!( metrics.average_response_time_ms, 1700 );
  assert_eq!( metrics.min_response_time_ms, 200 );
  assert_eq!( metrics.max_response_time_ms, 5000 );

  // Healthy percentage : 2/4 = 50%
  assert!( ( metrics.healthy_percentage() - 50.0 ).abs() < 0.01 );

  // Available percentage : (2+1)/4 = 75%
  assert!( ( metrics.available_percentage() - 75.0 ).abs() < 0.01 );
}

/// Test health metrics percentage calculations
#[ test ]
fn test_health_metrics_percentages()
{
  // All healthy
  let all_healthy = vec![
    the_module::HealthCheckResult
    {
      endpoint_url : "https://api1.com".to_string(),
      status : the_module::EndpointHealthStatus::Healthy,
      response_time_ms : 100,
      error_message : None,
      timestamp : std::time::SystemTime::now(),
    },
    the_module::HealthCheckResult
    {
      endpoint_url : "https://api2.com".to_string(),
      status : the_module::EndpointHealthStatus::Healthy,
      response_time_ms : 150,
      error_message : None,
      timestamp : std::time::SystemTime::now(),
    },
  ];

  let metrics = the_module::HealthMetrics::from_results( &all_healthy );
  assert!( ( metrics.healthy_percentage() - 100.0_f64 ).abs() < f64::EPSILON );
  assert!( ( metrics.available_percentage() - 100.0_f64 ).abs() < f64::EPSILON );

  // All unhealthy
  let all_unhealthy = vec![
    the_module::HealthCheckResult
    {
      endpoint_url : "https://api1.com".to_string(),
      status : the_module::EndpointHealthStatus::Unhealthy,
      response_time_ms : 10000,
      error_message : Some( "Failed".to_string() ),
      timestamp : std::time::SystemTime::now(),
    },
  ];

  let metrics = the_module::HealthMetrics::from_results( &all_unhealthy );
  assert!( metrics.healthy_percentage().abs() < f64::EPSILON );
  assert!( metrics.available_percentage().abs() < f64::EPSILON );

  // Mixed with degraded
  let mixed = vec![
    the_module::HealthCheckResult
    {
      endpoint_url : "https://api1.com".to_string(),
      status : the_module::EndpointHealthStatus::Degraded,
      response_time_ms : 1500,
      error_message : None,
      timestamp : std::time::SystemTime::now(),
    },
    the_module::HealthCheckResult
    {
      endpoint_url : "https://api2.com".to_string(),
      status : the_module::EndpointHealthStatus::Degraded,
      response_time_ms : 1600,
      error_message : None,
      timestamp : std::time::SystemTime::now(),
    },
  ];

  let metrics = the_module::HealthMetrics::from_results( &mixed );
  assert!( metrics.healthy_percentage().abs() < f64::EPSILON );
  assert!( ( metrics.available_percentage() - 100.0_f64 ).abs() < f64::EPSILON );
}

/// Test health check configuration serialization
#[ test ]
fn test_health_check_config_serialization()
{
  let config = the_module::HealthCheckConfig::new()
    .with_timeout_ms( 3000 )
    .with_strategy( the_module::HealthCheckStrategy::Ping );

  // Test serialization
  let json = serde_json::to_string( &config );
  assert!( json.is_ok() );

  // Test deserialization
  let deserialized : Result< the_module::HealthCheckConfig, _ > = serde_json::from_str( &json.unwrap() );
  assert!( deserialized.is_ok() );

  let deserialized_config = deserialized.unwrap();
  assert_eq!( deserialized_config.timeout_ms, 3000 );
  assert_eq!( deserialized_config.strategy, the_module::HealthCheckStrategy::Ping );
}

/// Test health check result serialization
#[ test ]
fn test_health_check_result_serialization()
{
  let result = the_module::HealthCheckResult
  {
    endpoint_url : "https://api.anthropic.com".to_string(),
    status : the_module::EndpointHealthStatus::Healthy,
    response_time_ms : 250,
    error_message : None,
    timestamp : std::time::SystemTime::now(),
  };

  // Test serialization
  let json = serde_json::to_string( &result );
  assert!( json.is_ok() );

  // Test deserialization
  let deserialized : Result< the_module::HealthCheckResult, _ > = serde_json::from_str( &json.unwrap() );
  assert!( deserialized.is_ok() );

  let deserialized_result = deserialized.unwrap();
  assert_eq!( deserialized_result.endpoint_url, "https://api.anthropic.com" );
  assert_eq!( deserialized_result.status, the_module::EndpointHealthStatus::Healthy );
  assert_eq!( deserialized_result.response_time_ms, 250 );
}

/// Test health metrics serialization
#[ test ]
fn test_health_metrics_serialization()
{
  let results = vec![
    the_module::HealthCheckResult
    {
      endpoint_url : "https://api.anthropic.com".to_string(),
      status : the_module::EndpointHealthStatus::Healthy,
      response_time_ms : 300,
      error_message : None,
      timestamp : std::time::SystemTime::now(),
    },
  ];

  let metrics = the_module::HealthMetrics::from_results( &results );

  // Test serialization
  let json = serde_json::to_string( &metrics );
  assert!( json.is_ok() );

  // Test deserialization
  let deserialized : Result< the_module::HealthMetrics, _ > = serde_json::from_str( &json.unwrap() );
  assert!( deserialized.is_ok() );

  let deserialized_metrics = deserialized.unwrap();
  assert_eq!( deserialized_metrics.total_endpoints, 1 );
  assert_eq!( deserialized_metrics.healthy_count, 1 );
}

/// Test health check result response time conversion
#[ test ]
fn test_response_time_conversion()
{
  let result = the_module::HealthCheckResult
  {
    endpoint_url : "https://api.anthropic.com".to_string(),
    status : the_module::EndpointHealthStatus::Healthy,
    response_time_ms : 1234,
    error_message : None,
    timestamp : std::time::SystemTime::now(),
  };

  let duration = result.response_time();
  assert_eq!( duration.as_millis(), 1234 );
  assert_eq!( duration.as_secs(), 1 );
}

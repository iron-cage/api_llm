//! Failover functionality tests

#[ path = "common/mod.rs" ] mod common;
#[ cfg( feature = "integration" ) ]
use common::create_integration_client;
use api_gemini::models::failover::*;
use core::time::Duration;
use std::time::SystemTime;

#[ cfg( feature = "integration" ) ]
mod integration_tests
{
  use super::*;

  #[ tokio::test ]
  async fn test_failover_configuration_basic() -> Result< (), Box< dyn std::error::Error > >
  {
    let client = create_integration_client();

    // Create failover configuration with multiple endpoints
    let failover_config = FailoverConfig::builder()
    .primary_endpoint( "https://primary.googleapis.com".to_string() )
    .backup_endpoint( "https://backup.googleapis.com".to_string() )
    .timeout( Duration::from_secs( 5 ) )
    .strategy( FailoverStrategy::Priority )
    .build()?;

    // Get failover manager for explicit failover operations
    let failover_manager = client.failover().configure( failover_config );

    // Verify configuration
    let current_config = failover_manager.current_config();
    assert_eq!( current_config.primary_endpoint, "https://primary.googleapis.com" );
    assert!( current_config.backup_endpoints.contains( &"https://backup.googleapis.com".to_string() ) );

    Ok( () )
  }

  #[ tokio::test ]
  async fn test_explicit_failover_operation() -> Result< (), Box< dyn std::error::Error > >
  {
    let client = create_integration_client();

    // Configure failover
    let failover_config = FailoverConfig::builder()
    .primary_endpoint( "https://invalid-primary.example.com".to_string() )
    .backup_endpoint( "https://generativelanguage.googleapis.com".to_string() )
    .timeout( Duration::from_secs( 1 ) )
    .build()?;

    let failover_manager = client.failover().configure( failover_config );

    // Explicit failover to backup (simplified test)
    let backup_client = failover_manager.switch_to_backup()?;

    // Verify we're using backup endpoint
    assert_eq!( backup_client.base_url(), "https://generativelanguage.googleapis.com" );

    Ok( () )
  }

  #[ tokio::test ]
  async fn test_failover_strategies() -> Result< (), Box< dyn std::error::Error > >
  {
    let client = create_integration_client();

    // Test Priority strategy configuration
    let priority_config = FailoverConfig::builder()
    .primary_endpoint( "https://primary.googleapis.com".to_string() )
    .backup_endpoint( "https://backup1.googleapis.com".to_string() )
    .backup_endpoint( "https://backup2.googleapis.com".to_string() )
    .strategy( FailoverStrategy::Priority )
    .build()?;

    let priority_manager = client.failover().configure( priority_config );

    // Just verify configuration works
    let config = priority_manager.current_config();
    assert_eq!( config.strategy, FailoverStrategy::Priority );
    assert_eq!( config.backup_endpoints.len(), 2 );

    // Test RoundRobin strategy configuration
    let roundrobin_config = FailoverConfig::builder()
    .primary_endpoint( "https://primary.googleapis.com".to_string() )
    .backup_endpoint( "https://backup1.googleapis.com".to_string() )
    .backup_endpoint( "https://backup2.googleapis.com".to_string() )
    .strategy( FailoverStrategy::RoundRobin )
    .build()?;

    let roundrobin_manager = client.failover().configure( roundrobin_config );

    // Verify configuration
    let config = roundrobin_manager.current_config();
    assert_eq!( config.strategy, FailoverStrategy::RoundRobin );

    Ok( () )
  }

  #[ tokio::test ]
  async fn test_failover_metrics() -> Result< (), Box< dyn std::error::Error > >
  {
    let client = create_integration_client();

    let failover_config = FailoverConfig::builder()
    .primary_endpoint( "https://primary.googleapis.com".to_string() )
    .backup_endpoint( "https://backup.googleapis.com".to_string() )
    .build()?;

    let failover_manager = client.failover().configure( failover_config );

    // Get failover metrics
    let metrics = failover_manager.get_metrics();
    assert_eq!( metrics.total_endpoints, 2 );
    assert_eq!( metrics.failover_count, 0 );
    // Endpoint health starts empty until health checks are performed
    assert_eq!( metrics.active_endpoint, "https://primary.googleapis.com" );

    Ok( () )
  }

  #[ tokio::test ]
  async fn test_failover_with_request_execution() -> Result< (), Box< dyn std::error::Error > >
  {
    let client = create_integration_client();

    let failover_config = FailoverConfig::builder()
    .primary_endpoint( "https://invalid-endpoint.example.com".to_string() )
    .backup_endpoint( "https://generativelanguage.googleapis.com".to_string() )
    .timeout( Duration::from_millis( 500 ) )
    .max_retries( 1 )
    .build()?;

    let failover_manager = client.failover().configure( failover_config );

    // For now, just test that the failover manager can execute operations
    // The full implementation would require more complex setup
    let backup_client = failover_manager.switch_to_backup()?;

    // Just verify the backup client was created with correct endpoint
    assert_eq!( backup_client.base_url(), "https://generativelanguage.googleapis.com" );

    // Skip actual API call for basic test
    // TODO: Implement API call test when ready

    Ok( () )
  }
}

mod unit_tests
{
  use super::*;

  #[ test ]
  fn test_failover_config_builder() -> Result< (), Box< dyn std::error::Error > >
  {
    let config = FailoverConfig::builder()
    .primary_endpoint( "https://primary.com".to_string() )
    .backup_endpoint( "https://backup1.com".to_string() )
    .backup_endpoint( "https://backup2.com".to_string() )
    .timeout( Duration::from_secs( 10 ) )
    .max_retries( 3 )
    .strategy( FailoverStrategy::RoundRobin )
    .build()?;

    assert_eq!( config.primary_endpoint, "https://primary.com" );
    assert_eq!( config.backup_endpoints.len(), 2 );
    assert_eq!( config.timeout, Duration::from_secs( 10 ) );
    assert_eq!( config.max_retries, 3 );
    assert_eq!( config.strategy, FailoverStrategy::RoundRobin );

    Ok( () )
  }

  #[ test ]
  fn test_failover_strategy_enum()
  {
    let priority = FailoverStrategy::Priority;
    let roundrobin = FailoverStrategy::RoundRobin;

    assert_ne!( priority, roundrobin );
    assert_eq!( priority, FailoverStrategy::Priority );
  }

  #[ test ]
  fn test_endpoint_health_status()
  {
    let healthy = EndpointHealth {
      endpoint: "https://test.com".to_string(),
      status: HealthStatus::Healthy,
      last_check: SystemTime::now(),
      response_time: Some( Duration::from_millis( 100 ) ),
      consecutive_failures: 0,
    };

    assert_eq!( healthy.status, HealthStatus::Healthy );
    assert_eq!( healthy.consecutive_failures, 0 );

    let unhealthy = EndpointHealth {
      endpoint: "https://test.com".to_string(),
      status: HealthStatus::Unhealthy,
      last_check: SystemTime::now(),
      response_time: None,
      consecutive_failures: 3,
    };

    assert_eq!( unhealthy.status, HealthStatus::Unhealthy );
    assert_eq!( unhealthy.consecutive_failures, 3 );
  }

  #[ test ]
  fn test_failover_config_validation()
  {
    // Test invalid configuration (no backup endpoints)
    let result = FailoverConfig::builder()
    .primary_endpoint( "https://primary.com".to_string() )
    .build();

    assert!( result.is_err() );

    // Test invalid timeout
    let result = FailoverConfig::builder()
    .primary_endpoint( "https://primary.com".to_string() )
    .backup_endpoint( "https://backup.com".to_string() )
    .timeout( Duration::from_secs( 0 ) )
    .build();

    assert!( result.is_err() );
  }
}
//! Failover tests for Anthropic API client
//!
//! These tests cover the failover pattern implementation including:
//! - Endpoint configuration and health tracking
//! - Failover strategy selection (Priority, `RoundRobin`, Random, Sticky)
//! - Context management and retry attempts
//! - Configuration validation
//! - Manager operations and endpoint selection

use super::*;
use core::time::Duration;

/// Test endpoint health status transitions
#[ test ]
fn test_endpoint_health_states()
{
  let mut endpoint = the_module::FailoverEndpoint::new(
    "test".to_string(),
    "https://api.anthropic.com".to_string(),
    100,
    Duration::from_secs( 30 )
  );

  // Initial state should be Unknown
  assert_eq!( endpoint.health, the_module::EndpointHealth::Unknown );
  assert!( !endpoint.is_available() );
  assert!( !endpoint.is_preferred() );

  // Update to Healthy
  endpoint.update_health( the_module::EndpointHealth::Healthy );
  assert_eq!( endpoint.health, the_module::EndpointHealth::Healthy );
  assert!( endpoint.is_available() );
  assert!( endpoint.is_preferred() );

  // Update to Degraded
  endpoint.update_health( the_module::EndpointHealth::Degraded );
  assert_eq!( endpoint.health, the_module::EndpointHealth::Degraded );
  assert!( endpoint.is_available() );
  assert!( !endpoint.is_preferred() );

  // Update to Unhealthy
  endpoint.update_health( the_module::EndpointHealth::Unhealthy );
  assert_eq!( endpoint.health, the_module::EndpointHealth::Unhealthy );
  assert!( !endpoint.is_available() );
  assert!( !endpoint.is_preferred() );
}

/// Test endpoint time tracking
#[ test ]
fn test_endpoint_time_tracking()
{
  let endpoint = the_module::FailoverEndpoint::new(
    "test".to_string(),
    "https://api.anthropic.com".to_string(),
    100,
    Duration::from_secs( 30 )
  );

  // Should have minimal elapsed time right after creation
  let elapsed = endpoint.time_since_check();
  assert!( elapsed < Duration::from_secs( 1 ) );

  // Sleep and check again
  std::thread::sleep( Duration::from_millis( 100 ) );
  let elapsed = endpoint.time_since_check();
  assert!( elapsed >= Duration::from_millis( 100 ) );
}

/// Test failover configuration validation
#[ test ]
fn test_failover_config_validation()
{
  // Test valid configuration
  let valid_config = the_module::FailoverConfig::new()
    .with_max_retries( 3 )
    .with_retry_delay_ms( 1000 )
    .with_max_retry_delay_ms( 30000 )
    .with_health_check_interval_ms( 10000 )
    .with_failover_timeout_ms( 5000 );

  assert!( valid_config.is_valid() );
  assert_eq!( valid_config.max_retries, 3 );
  assert_eq!( valid_config.retry_delay_ms, 1000 );
  assert_eq!( valid_config.max_retry_delay_ms, 30000 );

  // Test invalid configuration - zero retries
  let invalid_config = the_module::FailoverConfig::new()
    .with_max_retries( 0 );
  assert!( !invalid_config.is_valid() );

  // Test invalid configuration - zero retry delay
  let invalid_config = the_module::FailoverConfig::new()
    .with_retry_delay_ms( 0 );
  assert!( !invalid_config.is_valid() );

  // Test invalid configuration - max delay less than base delay
  let invalid_config = the_module::FailoverConfig::new()
    .with_retry_delay_ms( 5000 )
    .with_max_retry_delay_ms( 1000 );
  assert!( !invalid_config.is_valid() );
}

/// Test failover configuration defaults
#[ test ]
fn test_failover_config_defaults()
{
  let config = the_module::FailoverConfig::default();

  assert_eq!( config.strategy, the_module::FailoverStrategy::Priority );
  assert_eq!( config.max_retries, 3 );
  assert_eq!( config.retry_delay_ms, 1000 );
  assert_eq!( config.max_retry_delay_ms, 30000 );
  assert_eq!( config.health_check_interval_ms, 30000 );
  assert_eq!( config.failover_timeout_ms, 10000 );
  assert!( config.is_valid() );
}

/// Test failover context creation and progression
#[ test ]
fn test_failover_context_progression()
{
  let endpoint1 = the_module::FailoverEndpoint::new(
    "primary".to_string(),
    "https://api.anthropic.com".to_string(),
    100,
    Duration::from_secs( 30 )
  );

  let endpoint2 = the_module::FailoverEndpoint::new(
    "backup".to_string(),
    "https://api-backup.anthropic.com".to_string(),
    50,
    Duration::from_secs( 30 )
  );

  // Create initial context
  let context = the_module::FailoverContext::new( endpoint1.clone() );
  assert_eq!( context.attempt, 1 );
  assert_eq!( context.endpoint.id, "primary" );
  assert!( context.failed_endpoints.is_empty() );
  assert!( !context.is_exhausted( 3 ) );

  // Progress to next attempt
  let context = context.next_attempt( endpoint2.clone() );
  assert_eq!( context.attempt, 2 );
  assert_eq!( context.endpoint.id, "backup" );
  assert_eq!( context.failed_endpoints.len(), 1 );
  assert_eq!( context.failed_endpoints[ 0 ], "primary" );
  assert!( !context.is_exhausted( 3 ) );

  // Progress to third attempt
  let context = context.next_attempt( endpoint1.clone() );
  assert_eq!( context.attempt, 3 );
  assert!( !context.is_exhausted( 3 ) );

  // Fourth attempt should be exhausted
  let context = context.next_attempt( endpoint2.clone() );
  assert_eq!( context.attempt, 4 );
  assert!( context.is_exhausted( 3 ) );
}

/// Test context tracks tried endpoints
#[ test ]
fn test_context_tracks_tried_endpoints()
{
  let endpoint1 = the_module::FailoverEndpoint::new(
    "ep1".to_string(),
    "https://ep1.com".to_string(),
    100,
    Duration::from_secs( 30 )
  );

  let endpoint2 = the_module::FailoverEndpoint::new(
    "ep2".to_string(),
    "https://ep2.com".to_string(),
    50,
    Duration::from_secs( 30 )
  );

  let context = the_module::FailoverContext::new( endpoint1.clone() );

  // Should track current endpoint
  assert!( context.was_endpoint_tried( "ep1" ) );
  assert!( !context.was_endpoint_tried( "ep2" ) );

  // Progress to next
  let context = context.next_attempt( endpoint2.clone() );
  assert!( context.was_endpoint_tried( "ep1" ) );
  assert!( context.was_endpoint_tried( "ep2" ) );
}

/// Test failover manager with priority strategy
#[ test ]
fn test_failover_manager_priority_strategy()
{
  let config = the_module::FailoverConfig::new()
    .with_strategy( the_module::FailoverStrategy::Priority );

  let mut endpoints = vec![
    the_module::FailoverEndpoint::new(
      "low".to_string(),
      "https://low.com".to_string(),
      10,
      Duration::from_secs( 30 )
    ),
    the_module::FailoverEndpoint::new(
      "high".to_string(),
      "https://high.com".to_string(),
      100,
      Duration::from_secs( 30 )
    ),
    the_module::FailoverEndpoint::new(
      "medium".to_string(),
      "https://medium.com".to_string(),
      50,
      Duration::from_secs( 30 )
    ),
  ];

  // Mark all as healthy
  for endpoint in &mut endpoints
  {
    endpoint.update_health( the_module::EndpointHealth::Healthy );
  }

  let manager = the_module::FailoverManager::new( config, endpoints );

  // Should select highest priority
  let selected = manager.select_endpoint( None );
  assert!( selected.is_some() );
  assert_eq!( selected.unwrap().id, "high" );
  assert_eq!( manager.healthy_count(), 3 );
  assert_eq!( manager.available_count(), 3 );
}

/// Test failover manager with round-robin strategy
#[ test ]
fn test_failover_manager_round_robin_strategy()
{
  let config = the_module::FailoverConfig::new()
    .with_strategy( the_module::FailoverStrategy::RoundRobin );

  let mut endpoints = vec![
    the_module::FailoverEndpoint::new(
      "ep1".to_string(),
      "https://ep1.com".to_string(),
      100,
      Duration::from_secs( 30 )
    ),
    the_module::FailoverEndpoint::new(
      "ep2".to_string(),
      "https://ep2.com".to_string(),
      100,
      Duration::from_secs( 30 )
    ),
    the_module::FailoverEndpoint::new(
      "ep3".to_string(),
      "https://ep3.com".to_string(),
      100,
      Duration::from_secs( 30 )
    ),
  ];

  // Mark all as healthy
  for endpoint in &mut endpoints
  {
    endpoint.update_health( the_module::EndpointHealth::Healthy );
  }

  let manager = the_module::FailoverManager::new( config, endpoints );

  // Should cycle through endpoints
  let mut seen_endpoints = std::collections::HashSet::new();
  for _ in 0..10
  {
    let selected = manager.select_endpoint( None );
    assert!( selected.is_some() );
    seen_endpoints.insert( selected.unwrap().id );
  }

  // Should have seen all three endpoints
  assert_eq!( seen_endpoints.len(), 3 );
}

/// Test failover manager with sticky strategy
#[ test ]
fn test_failover_manager_sticky_strategy()
{
  let config = the_module::FailoverConfig::new()
    .with_strategy( the_module::FailoverStrategy::Sticky );

  let mut endpoints = vec![
    the_module::FailoverEndpoint::new(
      "ep1".to_string(),
      "https://ep1.com".to_string(),
      100,
      Duration::from_secs( 30 )
    ),
    the_module::FailoverEndpoint::new(
      "ep2".to_string(),
      "https://ep2.com".to_string(),
      50,
      Duration::from_secs( 30 )
    ),
  ];

  // Mark all as healthy
  for endpoint in &mut endpoints
  {
    endpoint.update_health( the_module::EndpointHealth::Healthy );
  }

  let manager = the_module::FailoverManager::new( config, endpoints );

  // Should stick to first preferred (healthy) endpoint
  let first_selection = manager.select_endpoint( None );
  assert!( first_selection.is_some() );
  let first_id = first_selection.unwrap().id.clone();

  // Multiple selections should return same endpoint
  for _ in 0..5
  {
    let selected = manager.select_endpoint( None );
    assert!( selected.is_some() );
    assert_eq!( selected.unwrap().id, first_id );
  }
}

/// Test failover manager filters unavailable endpoints
#[ test ]
fn test_failover_manager_filters_unavailable()
{
  let config = the_module::FailoverConfig::new()
    .with_strategy( the_module::FailoverStrategy::Priority );

  let mut endpoints = vec![
    the_module::FailoverEndpoint::new(
      "unhealthy".to_string(),
      "https://unhealthy.com".to_string(),
      100,
      Duration::from_secs( 30 )
    ),
    the_module::FailoverEndpoint::new(
      "healthy".to_string(),
      "https://healthy.com".to_string(),
      50,
      Duration::from_secs( 30 )
    ),
  ];

  // Mark first as unhealthy, second as healthy
  endpoints[ 0 ].update_health( the_module::EndpointHealth::Unhealthy );
  endpoints[ 1 ].update_health( the_module::EndpointHealth::Healthy );

  let manager = the_module::FailoverManager::new( config, endpoints );

  // Should select only healthy endpoint despite lower priority
  let selected = manager.select_endpoint( None );
  assert!( selected.is_some() );
  assert_eq!( selected.unwrap().id, "healthy" );
  assert_eq!( manager.healthy_count(), 1 );
  assert_eq!( manager.available_count(), 1 );
}

/// Test failover manager respects context
#[ test ]
fn test_failover_manager_respects_context()
{
  let config = the_module::FailoverConfig::new()
    .with_strategy( the_module::FailoverStrategy::Priority );

  let mut endpoints = vec![
    the_module::FailoverEndpoint::new(
      "primary".to_string(),
      "https://primary.com".to_string(),
      100,
      Duration::from_secs( 30 )
    ),
    the_module::FailoverEndpoint::new(
      "backup".to_string(),
      "https://backup.com".to_string(),
      50,
      Duration::from_secs( 30 )
    ),
  ];

  // Mark all as healthy
  for endpoint in &mut endpoints
  {
    endpoint.update_health( the_module::EndpointHealth::Healthy );
  }

  let manager = the_module::FailoverManager::new( config, endpoints.clone() );

  // First selection should get primary
  let first = manager.select_endpoint( None );
  assert_eq!( first.as_ref().unwrap().id, "primary" );

  // Create context with primary already tried
  let context = the_module::FailoverContext::new( endpoints[ 0 ].clone() );

  // Next selection should skip primary and get backup
  let second = manager.select_endpoint( Some( &context ) );
  assert_eq!( second.as_ref().unwrap().id, "backup" );
}

/// Test failover manager with no available endpoints
#[ test ]
fn test_failover_manager_no_available_endpoints()
{
  let config = the_module::FailoverConfig::new();

  let mut endpoints = vec![
    the_module::FailoverEndpoint::new(
      "ep1".to_string(),
      "https://ep1.com".to_string(),
      100,
      Duration::from_secs( 30 )
    ),
  ];

  // Mark as unhealthy
  endpoints[ 0 ].update_health( the_module::EndpointHealth::Unhealthy );

  let manager = the_module::FailoverManager::new( config, endpoints );

  // Should return None when no endpoints available
  let selected = manager.select_endpoint( None );
  assert!( selected.is_none() );
  assert_eq!( manager.healthy_count(), 0 );
  assert_eq!( manager.available_count(), 0 );
  assert!( !manager.has_available_endpoints() );
}

/// Test failover manager endpoint health updates
#[ test ]
fn test_failover_manager_health_updates()
{
  let config = the_module::FailoverConfig::new();

  let mut endpoints = vec![
    the_module::FailoverEndpoint::new(
      "ep1".to_string(),
      "https://ep1.com".to_string(),
      100,
      Duration::from_secs( 30 )
    ),
  ];

  endpoints[ 0 ].update_health( the_module::EndpointHealth::Healthy );

  let mut manager = the_module::FailoverManager::new( config, endpoints );

  assert_eq!( manager.healthy_count(), 1 );

  // Update to unhealthy
  manager.update_endpoint_health( "ep1", the_module::EndpointHealth::Unhealthy );
  assert_eq!( manager.healthy_count(), 0 );

  // Update back to healthy
  manager.update_endpoint_health( "ep1", the_module::EndpointHealth::Healthy );
  assert_eq!( manager.healthy_count(), 1 );
}

/// Test degraded endpoints are available but not preferred
#[ test ]
fn test_degraded_endpoints_available_not_preferred()
{
  let mut endpoint = the_module::FailoverEndpoint::new(
    "test".to_string(),
    "https://test.com".to_string(),
    100,
    Duration::from_secs( 30 )
  );

  endpoint.update_health( the_module::EndpointHealth::Degraded );

  assert!( endpoint.is_available() );
  assert!( !endpoint.is_preferred() );

  // Manager should count degraded as available but not healthy
  let config = the_module::FailoverConfig::new();
  let manager = the_module::FailoverManager::new( config, vec![ endpoint ] );

  assert_eq!( manager.available_count(), 1 );
  assert_eq!( manager.healthy_count(), 0 );
}

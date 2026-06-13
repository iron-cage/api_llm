//! Failover Tests
//!
//! Comprehensive test suite for failover functionality including:
//! - Endpoint health management
//! - Failover strategy implementations
//! - Configuration validation
//! - Manager operations and context handling
//! - Executor patterns and error handling
//! - Event notifications and performance

#[ cfg( test ) ]
mod failover_tests
{
  use api_openai::failover::*;
  use core::time::Duration;
  use tokio::time;

  // ===== ENDPOINT HEALTH TESTS =====

  #[ tokio::test ]
  async fn test_endpoint_health_creation()
  {
    let endpoint = FailoverEndpoint::new(
      "test1".to_string(),
      "https://api.test1.com".to_string(),
      100,
      Duration::from_secs( 30 ),
    );

    assert_eq!( endpoint.id, "test1" );
    assert_eq!( endpoint.url, "https://api.test1.com" );
    assert_eq!( endpoint.priority, 100 );
    assert_eq!( endpoint.timeout, Duration::from_secs( 30 ) );
    assert_eq!( endpoint.health, EndpointHealth::Unknown );
  }

  #[ tokio::test ]
  async fn test_endpoint_availability()
  {
    let mut endpoint = FailoverEndpoint::new(
      "test1".to_string(),
      "https://api.test1.com".to_string(),
      100,
      Duration::from_secs( 30 ),
    );

    // Unknown health - not available
    assert!( !endpoint.is_available() );
    assert!( !endpoint.is_preferred() );

    // Healthy - available and preferred
    endpoint.update_health( EndpointHealth::Healthy );
    assert!( endpoint.is_available() );
    assert!( endpoint.is_preferred() );

    // Degraded - available but not preferred
    endpoint.update_health( EndpointHealth::Degraded );
    assert!( endpoint.is_available() );
    assert!( !endpoint.is_preferred() );

    // Unhealthy - not available
    endpoint.update_health( EndpointHealth::Unhealthy );
    assert!( !endpoint.is_available() );
    assert!( !endpoint.is_preferred() );
  }

  #[ tokio::test ]
  async fn test_endpoint_health_update()
  {
    let mut endpoint = FailoverEndpoint::new(
      "test1".to_string(),
      "https://api.test1.com".to_string(),
      100,
      Duration::from_secs( 30 ),
    );

    let initial_time = endpoint.last_checked;
    time ::sleep( Duration::from_millis( 10 ) ).await;

    endpoint.update_health( EndpointHealth::Healthy );
    assert_eq!( endpoint.health, EndpointHealth::Healthy );
    assert!( endpoint.last_checked > initial_time );
    assert!( endpoint.time_since_check() < Duration::from_millis( 100 ) );
  }

  #[ tokio::test ]
  async fn test_endpoint_serialization()
  {
    let endpoint = FailoverEndpoint::new(
      "test1".to_string(),
      "https://api.test1.com".to_string(),
      100,
      Duration::from_secs( 30 ),
    );

    // Test serialization
    let serialized = serde_json::to_string( &endpoint ).expect( "Failed to serialize endpoint" );
    assert!( !serialized.is_empty() );

    // Test deserialization
    let deserialized : FailoverEndpoint = serde_json::from_str( &serialized )
      .expect( "Failed to deserialize endpoint" );

    assert_eq!( endpoint.id, deserialized.id );
    assert_eq!( endpoint.url, deserialized.url );
    assert_eq!( endpoint.priority, deserialized.priority );
    assert_eq!( endpoint.timeout, deserialized.timeout );
    assert_eq!( endpoint.health, deserialized.health );
  }

  // ===== FAILOVER CONFIG TESTS =====

  #[ tokio::test ]
  async fn test_failover_config_defaults()
  {
    let config = FailoverConfig::default();

    assert_eq!( config.strategy, FailoverStrategy::Priority );
    assert_eq!( config.max_retries, 3 );
    assert_eq!( config.retry_delay_ms, 1000 );
    assert_eq!( config.max_retry_delay_ms, 30000 );
    assert_eq!( config.health_check_interval_ms, 30000 );
    assert_eq!( config.failover_timeout_ms, 10000 );
  }

  #[ tokio::test ]
  async fn test_failover_config_serialization()
  {
    let config = FailoverConfig
    {
      strategy : FailoverStrategy::RoundRobin,
      max_retries : 5,
      retry_delay_ms : 500,
      max_retry_delay_ms : 60000,
      health_check_interval_ms : 15000,
      failover_timeout_ms : 20000,
    };

    // Test serialization
    let serialized = serde_json::to_string( &config ).expect( "Failed to serialize config" );
    assert!( !serialized.is_empty() );

    // Test deserialization
    let deserialized : FailoverConfig = serde_json::from_str( &serialized )
      .expect( "Failed to deserialize config" );

    assert_eq!( config.strategy, deserialized.strategy );
    assert_eq!( config.max_retries, deserialized.max_retries );
    assert_eq!( config.retry_delay_ms, deserialized.retry_delay_ms );
  }

  #[ tokio::test ]
  async fn test_failover_config_validation()
  {
    // Valid config
    let valid_config = FailoverConfig::default();
    assert!( FailoverExecutor::validate_config( &valid_config ).is_ok() );

    // Invalid : zero retries
    let invalid_config1 = FailoverConfig
    {
      max_retries : 0,
      ..Default::default()
    };
    assert!( FailoverExecutor::validate_config( &invalid_config1 ).is_err() );

    // Invalid : zero retry delay
    let invalid_config2 = FailoverConfig
    {
      retry_delay_ms : 0,
      ..Default::default()
    };
    assert!( FailoverExecutor::validate_config( &invalid_config2 ).is_err() );

    // Invalid : max delay less than base delay
    let invalid_config3 = FailoverConfig
    {
      retry_delay_ms : 1000,
      max_retry_delay_ms : 500,
      ..Default::default()
    };
    assert!( FailoverExecutor::validate_config( &invalid_config3 ).is_err() );
  }

  // ===== FAILOVER CONTEXT TESTS =====

  #[ tokio::test ]
  async fn test_failover_context_creation()
  {
    let endpoint = FailoverEndpoint::new(
      "test1".to_string(),
      "https://api.test1.com".to_string(),
      100,
      Duration::from_secs( 30 ),
    );

    let context = FailoverContext::new( endpoint );

    assert_eq!( context.attempt, 1 );
    assert_eq!( context.endpoint.id, "test1" );
    assert!( context.failed_endpoints.is_empty() );
    assert!( !context.is_exhausted( 3 ) );
  }

  #[ tokio::test ]
  async fn test_failover_context_next_attempt()
  {
    let endpoint1 = FailoverEndpoint::new(
      "test1".to_string(),
      "https://api.test1.com".to_string(),
      100,
      Duration::from_secs( 30 ),
    );

    let endpoint2 = FailoverEndpoint::new(
      "test2".to_string(),
      "https://api.test2.com".to_string(),
      90,
      Duration::from_secs( 30 ),
    );

    let context = FailoverContext::new( endpoint1 );
    let next_context = context.next_attempt( endpoint2 );

    assert_eq!( next_context.attempt, 2 );
    assert_eq!( next_context.endpoint.id, "test2" );
    assert_eq!( next_context.failed_endpoints, vec![ "test1" ] );
    assert!( next_context.was_endpoint_tried( "test1" ) );
    assert!( !next_context.was_endpoint_tried( "test3" ) );
  }

  #[ tokio::test ]
  async fn test_failover_context_exhaustion()
  {
    let endpoint = FailoverEndpoint::new(
      "test1".to_string(),
      "https://api.test1.com".to_string(),
      100,
      Duration::from_secs( 30 ),
    );

    let mut context = FailoverContext::new( endpoint );

    assert!( !context.is_exhausted( 3 ) );

    // Simulate multiple attempts
    for i in 2..=3
    {
      let next_endpoint = FailoverEndpoint::new(
        format!( "test{i}" ),
        format!( "https://api.test{i}.com" ),
        100,
        Duration::from_secs( 30 ),
      );
      context = context.next_attempt( next_endpoint );
    }

    assert!( !context.is_exhausted( 3 ) );

    // One more attempt should exhaust
    let final_endpoint = FailoverEndpoint::new(
      "test4".to_string(),
      "https://api.test4.com".to_string(),
      100,
      Duration::from_secs( 30 ),
    );
    context = context.next_attempt( final_endpoint );
    assert!( context.is_exhausted( 3 ) );
  }

  // ===== FAILOVER MANAGER TESTS =====

  #[ tokio::test ]
  async fn test_failover_manager_creation()
  {
    let config = FailoverConfig::default();
    let endpoints = vec![
      FailoverEndpoint::new(
        "test1".to_string(),
        "https://api.test1.com".to_string(),
        100,
        Duration::from_secs( 30 ),
      ),
    ];

    let manager = FailoverManager::new( config, endpoints );

    assert_eq!( manager.config().strategy, FailoverStrategy::Priority );
    assert_eq!( manager.endpoints().len(), 1 );
    assert_eq!( manager.healthy_endpoint_count(), 0 ); // Unknown health
    assert_eq!( manager.available_endpoint_count(), 0 );
  }

  #[ tokio::test ]
  async fn test_failover_manager_health_update()
  {
    let config = FailoverConfig::default();
    let endpoints = vec![
      FailoverEndpoint::new(
        "test1".to_string(),
        "https://api.test1.com".to_string(),
        100,
        Duration::from_secs( 30 ),
      ),
      FailoverEndpoint::new(
        "test2".to_string(),
        "https://api.test2.com".to_string(),
        90,
        Duration::from_secs( 30 ),
      ),
    ];

    let mut manager = FailoverManager::new( config, endpoints );

    // Initially no healthy endpoints
    assert_eq!( manager.healthy_endpoint_count(), 0 );
    assert_eq!( manager.available_endpoint_count(), 0 );

    // Update health
    manager.update_endpoint_health( "test1", EndpointHealth::Healthy );
    manager.update_endpoint_health( "test2", EndpointHealth::Degraded );

    assert_eq!( manager.healthy_endpoint_count(), 1 );
    assert_eq!( manager.available_endpoint_count(), 2 );
  }

  #[ tokio::test ]
  async fn test_priority_strategy()
  {
    let config = FailoverConfig
    {
      strategy : FailoverStrategy::Priority,
      ..Default::default()
    };

    let mut endpoints = vec![
      FailoverEndpoint::new(
        "low".to_string(),
        "https://api.low.com".to_string(),
        50,
        Duration::from_secs( 30 ),
      ),
      FailoverEndpoint::new(
        "high".to_string(),
        "https://api.high.com".to_string(),
        100,
        Duration::from_secs( 30 ),
      ),
      FailoverEndpoint::new(
        "medium".to_string(),
        "https://api.medium.com".to_string(),
        75,
        Duration::from_secs( 30 ),
      ),
    ];

    // Set all as healthy
    for endpoint in &mut endpoints
    {
      endpoint.update_health( EndpointHealth::Healthy );
    }

    let manager = FailoverManager::new( config, endpoints );

    // Should select highest priority endpoint
    let selected = manager.select_endpoint( None );
    assert!( selected.is_some() );
    assert_eq!( selected.unwrap().id, "high" );
  }

  #[ tokio::test ]
  async fn test_round_robin_strategy()
  {
    let config = FailoverConfig
    {
      strategy : FailoverStrategy::RoundRobin,
      ..Default::default()
    };

    let mut endpoints = vec![
      FailoverEndpoint::new(
        "test1".to_string(),
        "https://api.test1.com".to_string(),
        100,
        Duration::from_secs( 30 ),
      ),
      FailoverEndpoint::new(
        "test2".to_string(),
        "https://api.test2.com".to_string(),
        100,
        Duration::from_secs( 30 ),
      ),
      FailoverEndpoint::new(
        "test3".to_string(),
        "https://api.test3.com".to_string(),
        100,
        Duration::from_secs( 30 ),
      ),
    ];

    // Set all as healthy
    for endpoint in &mut endpoints
    {
      endpoint.update_health( EndpointHealth::Healthy );
    }

    let manager = FailoverManager::new( config, endpoints );

    // Should cycle through endpoints
    let mut selected_ids = Vec::new();
    for _ in 0..6
    {
      if let Some( endpoint ) = manager.select_endpoint( None )
      {
        selected_ids.push( endpoint.id );
      }
    }

    // Should see each endpoint twice in the cycle
    assert_eq!( selected_ids.len(), 6 );
    assert!( selected_ids.contains( &"test1".to_string() ) );
    assert!( selected_ids.contains( &"test2".to_string() ) );
    assert!( selected_ids.contains( &"test3".to_string() ) );
  }

  #[ tokio::test ]
  async fn test_sticky_strategy()
  {
    let config = FailoverConfig
    {
      strategy : FailoverStrategy::Sticky,
      ..Default::default()
    };

    let mut endpoints = vec![
      FailoverEndpoint::new(
        "test1".to_string(),
        "https://api.test1.com".to_string(),
        100,
        Duration::from_secs( 30 ),
      ),
      FailoverEndpoint::new(
        "test2".to_string(),
        "https://api.test2.com".to_string(),
        90,
        Duration::from_secs( 30 ),
      ),
    ];

    // First endpoint healthy, second degraded
    endpoints[ 0 ].update_health( EndpointHealth::Healthy );
    endpoints[ 1 ].update_health( EndpointHealth::Degraded );

    let manager = FailoverManager::new( config, endpoints );

    // Should always select the healthy endpoint
    for _ in 0..5
    {
      let selected = manager.select_endpoint( None );
      assert!( selected.is_some() );
      assert_eq!( selected.unwrap().id, "test1" );
    }
  }

  #[ tokio::test ]
  async fn test_no_available_endpoints()
  {
    let config = FailoverConfig::default();
    let mut endpoints = vec![
      FailoverEndpoint::new(
        "test1".to_string(),
        "https://api.test1.com".to_string(),
        100,
        Duration::from_secs( 30 ),
      ),
    ];

    // Set as unhealthy
    endpoints[ 0 ].update_health( EndpointHealth::Unhealthy );

    let manager = FailoverManager::new( config, endpoints );

    let selected = manager.select_endpoint( None );
    assert!( selected.is_none() );
  }

  #[ tokio::test ]
  async fn test_retry_delay_calculation()
  {
    let config = FailoverConfig
    {
      retry_delay_ms : 1000,
      max_retry_delay_ms : 10000,
      ..Default::default()
    };

    let manager = FailoverManager::new( config, vec![] );

    // Test exponential backoff
    assert_eq!( manager.calculate_retry_delay( 1 ), Duration::from_secs( 1 ) ); // 1000 * 2^0
    assert_eq!( manager.calculate_retry_delay( 2 ), Duration::from_secs( 2 ) ); // 1000 * 2^1
    assert_eq!( manager.calculate_retry_delay( 3 ), Duration::from_secs( 4 ) ); // 1000 * 2^2
    assert_eq!( manager.calculate_retry_delay( 4 ), Duration::from_secs( 8 ) ); // 1000 * 2^3

    // Should cap at max delay
    assert_eq!( manager.calculate_retry_delay( 5 ), Duration::from_secs( 10 ) ); // Capped
  }

  // ===== FAILOVER EXECUTOR TESTS =====

  #[ tokio::test ]
  async fn test_basic_manager_creation()
  {
    let endpoints = vec![
      ( "primary".to_string(), "https://api.primary.com".to_string(), 100 ),
      ( "secondary".to_string(), "https://api.secondary.com".to_string(), 50 ),
    ];

    let manager = FailoverExecutor::create_basic_manager( endpoints );

    assert_eq!( manager.endpoints().len(), 2 );
    assert_eq!( manager.endpoints()[ 0 ].id, "primary" );
    assert_eq!( manager.endpoints()[ 1 ].id, "secondary" );
  }

  #[ tokio::test ]
  async fn test_successful_operation()
  {
    let mut endpoints = vec![
      FailoverEndpoint::new(
        "test1".to_string(),
        "https://api.test1.com".to_string(),
        100,
        Duration::from_secs( 30 ),
      ),
    ];

    endpoints[ 0 ].update_health( EndpointHealth::Healthy );

    let manager = FailoverManager::new( FailoverConfig::default(), endpoints );

    let result = FailoverExecutor::execute_with_failover( &manager, | _ctx |
    {
      async move
      {
        Ok::< i32, &'static str >( 42 )
      }
    }).await;

    assert!( result.is_ok() );
    assert_eq!( result.unwrap(), 42 );
  }

  #[ tokio::test ]
  async fn test_no_available_endpoints_error()
  {
    let endpoints = vec![
      FailoverEndpoint::new(
        "test1".to_string(),
        "https://api.test1.com".to_string(),
        100,
        Duration::from_secs( 30 ),
      ),
    ];

    // Endpoint is unhealthy
    let manager = FailoverManager::new( FailoverConfig::default(), endpoints );

    let result = FailoverExecutor::execute_with_failover( &manager, | _ctx |
    {
      async move
      {
        Ok::< i32, &'static str >( 42 )
      }
    }).await;

    assert!( result.is_err() );
    assert!( matches!( result.unwrap_err(), FailoverError::NoAvailableEndpoints ) );
  }

  #[ tokio::test ]
  async fn test_all_endpoints_failed()
  {
    let mut endpoints = vec![
      FailoverEndpoint::new(
        "test1".to_string(),
        "https://api.test1.com".to_string(),
        100,
        Duration::from_secs( 30 ),
      ),
    ];

    endpoints[ 0 ].update_health( EndpointHealth::Healthy );

    let config = FailoverConfig
    {
      max_retries : 1, // Only one attempt
      ..Default::default()
    };

    let manager = FailoverManager::new( config, endpoints );

    let result = FailoverExecutor::execute_with_failover( &manager, | _ctx |
    {
      async move
      {
        Err::< i32, &'static str >( "Always fails" )
      }
    }).await;

    assert!( result.is_err() );
    assert!( matches!( result.unwrap_err(), FailoverError::AllEndpointsFailed( _ ) ) );
  }

  // ===== EVENT NOTIFICATION TESTS =====

  #[ tokio::test ]
  async fn test_failover_event_notification()
  {
    let ( sender, mut receiver ) = FailoverExecutor::create_failover_notifier();

    let event = FailoverEvent::HealthChanged
    {
      endpoint_id : "test1".to_string(),
      old_health : EndpointHealth::Unknown,
      new_health : EndpointHealth::Healthy,
    };

    sender.send_event( event.clone() ).expect( "Failed to send event" );

    let received_event = receiver.try_recv();
    assert!( received_event.is_some() );

    match received_event.unwrap()
    {
      FailoverEvent::HealthChanged { endpoint_id, old_health, new_health } =>
      {
        assert_eq!( endpoint_id, "test1" );
        assert_eq!( old_health, EndpointHealth::Unknown );
        assert_eq!( new_health, EndpointHealth::Healthy );
      }
      _ => panic!( "Wrong event type received" ),
    }
  }

  #[ tokio::test ]
  async fn test_failover_event_helper_methods()
  {
    let ( sender, mut receiver ) = FailoverExecutor::create_failover_notifier();

    // Test health change helper
    sender.send_health_change(
      "test1".to_string(),
      EndpointHealth::Healthy,
      EndpointHealth::Degraded,
    ).expect( "Failed to send health change" );

    let event = receiver.try_recv();
    assert!( event.is_some() );

    // Test failover started helper
    sender.send_failover_started( "test2".to_string(), 3 ).expect( "Failed to send failover started" );

    let event2 = receiver.try_recv();
    assert!( event2.is_some() );

    match event2.unwrap()
    {
      FailoverEvent::FailoverStarted { endpoint_id, attempt } =>
      {
        assert_eq!( endpoint_id, "test2" );
        assert_eq!( attempt, 3 );
      }
      _ => panic!( "Wrong event type received" ),
    }
  }

  #[ tokio::test ]
  async fn test_failover_event_async_recv()
  {
    let ( sender, mut receiver ) = FailoverExecutor::create_failover_notifier();

    // Send event after a delay
    tokio ::spawn( async move
    {
      time ::sleep( Duration::from_millis( 25 ) ).await;
      let _ = sender.send_failover_started( "async_test".to_string(), 1 );
    });

    // Receive asynchronously
    let event = receiver.recv().await;
    assert!( event.is_some() );

    match event.unwrap()
    {
      FailoverEvent::FailoverStarted { endpoint_id, attempt } =>
      {
        assert_eq!( endpoint_id, "async_test" );
        assert_eq!( attempt, 1 );
      }
      _ => panic!( "Wrong event type received" ),
    }
  }

  // ===== INTEGRATION TESTS =====

  #[ tokio::test ]
  async fn test_complete_failover_workflow()
  {
    let config = FailoverConfig
    {
      strategy : FailoverStrategy::Priority,
      max_retries : 3,
      retry_delay_ms : 10, // Short delay for testing
      ..Default::default()
    };

    let mut endpoints = vec![
      FailoverEndpoint::new(
        "primary".to_string(),
        "https://api.primary.com".to_string(),
        100,
        Duration::from_secs( 30 ),
      ),
      FailoverEndpoint::new(
        "secondary".to_string(),
        "https://api.secondary.com".to_string(),
        50,
        Duration::from_secs( 30 ),
      ),
    ];

    // Primary healthy, secondary degraded
    endpoints[ 0 ].update_health( EndpointHealth::Healthy );
    endpoints[ 1 ].update_health( EndpointHealth::Degraded );

    let manager = FailoverManager::new( config, endpoints );
    let ( event_sender, mut event_receiver ) = FailoverExecutor::create_failover_notifier();

    let attempt_count = std::sync::Arc::new( std::sync::Mutex::new( 0 ) );
    let attempt_count_clone = attempt_count.clone();

    let result = FailoverExecutor::execute_with_failover( &manager, | ctx |
    {
      let sender = event_sender.clone();
      let count = attempt_count_clone.clone();
      async move
      {
        {
          let mut count = count.lock().unwrap();
          *count += 1;
        }

        // Send notification
        let _ = sender.send_failover_started( ctx.endpoint.id.clone(), ctx.attempt );

        // Fail primary, succeed on secondary
        if ctx.endpoint.id == "primary"
        {
          Err( "Primary failed" )
        }
        else
        {
          Ok( "Success on secondary" )
        }
      }
    }).await;

    assert!( result.is_ok() );
    assert_eq!( result.unwrap(), "Success on secondary" );

    // Check that events were sent
    let event1 = event_receiver.try_recv();
    assert!( event1.is_some() );

    let event2 = event_receiver.try_recv();
    assert!( event2.is_some() );
  }

  #[ tokio::test ]
  async fn test_context_prevents_retry_same_endpoint()
  {
    let config = FailoverConfig
    {
      strategy : FailoverStrategy::Priority,
      max_retries : 5,
      retry_delay_ms : 1,
      ..Default::default()
    };

    let mut endpoints = vec![
      FailoverEndpoint::new(
        "test1".to_string(),
        "https://api.test1.com".to_string(),
        100,
        Duration::from_secs( 30 ),
      ),
      FailoverEndpoint::new(
        "test2".to_string(),
        "https://api.test2.com".to_string(),
        90,
        Duration::from_secs( 30 ),
      ),
    ];

    // Both healthy
    for endpoint in &mut endpoints
    {
      endpoint.update_health( EndpointHealth::Healthy );
    }

    let manager = FailoverManager::new( config, endpoints );

    let tried_endpoints = std::sync::Arc::new( std::sync::Mutex::new( Vec::new() ) );
    let tried_endpoints_clone = tried_endpoints.clone();

    let _result = FailoverExecutor::execute_with_failover( &manager, | ctx |
    {
      let endpoints = tried_endpoints_clone.clone();
      async move
      {
        endpoints.lock().unwrap().push( ctx.endpoint.id.clone() );
        Err::< String, &'static str >( "Always fail" )
      }
    }).await;

    let tried_endpoints = tried_endpoints.lock().unwrap();

    // Should try each endpoint only once
    assert_eq!( tried_endpoints.len(), 2 );
    assert!( tried_endpoints.contains( &"test1".to_string() ) );
    assert!( tried_endpoints.contains( &"test2".to_string() ) );

    // No duplicates
    let mut sorted_endpoints = tried_endpoints.clone();
    sorted_endpoints.sort();
    sorted_endpoints.dedup();
    assert_eq!( sorted_endpoints.len(), 2 );
  }

  #[ tokio::test ]
  async fn test_performance_endpoint_selection()
  {
    let config = FailoverConfig
    {
      strategy : FailoverStrategy::RoundRobin,
      ..Default::default()
    };

    let mut endpoints = Vec::new();
    for i in 0..100
    {
      let mut endpoint = FailoverEndpoint::new(
        format!( "endpoint_{i}" ),
        format!( "https://api.endpoint{i}.com" ),
        i,
        Duration::from_secs( 30 ),
      );
      endpoint.update_health( EndpointHealth::Healthy );
      endpoints.push( endpoint );
    }

    let manager = FailoverManager::new( config, endpoints );

    let start = std::time::Instant::now();

    // Select endpoints many times
    for _ in 0..1000
    {
      let _selected = manager.select_endpoint( None );
    }

    let duration = start.elapsed();
    assert!( duration < Duration::from_millis( 100 ) ); // Should be very fast
  }

  #[ tokio::test ]
  async fn test_concurrent_endpoint_selection()
  {
    let config = FailoverConfig
    {
      strategy : FailoverStrategy::RoundRobin,
      ..Default::default()
    };

    let mut endpoints = vec![
      FailoverEndpoint::new(
        "test1".to_string(),
        "https://api.test1.com".to_string(),
        100,
        Duration::from_secs( 30 ),
      ),
      FailoverEndpoint::new(
        "test2".to_string(),
        "https://api.test2.com".to_string(),
        90,
        Duration::from_secs( 30 ),
      ),
    ];

    for endpoint in &mut endpoints
    {
      endpoint.update_health( EndpointHealth::Healthy );
    }

    let manager = std::sync::Arc::new( FailoverManager::new( config, endpoints ) );

    let mut handles = Vec::new();

    // Create multiple concurrent selection tasks
    for _ in 0..10
    {
      let manager_clone = manager.clone();
      let handle = tokio::spawn( async move
      {
        for _ in 0..10
        {
          let _selected = manager_clone.select_endpoint( None );
        }
      });
      handles.push( handle );
    }

    // Wait for all tasks to complete
    for handle in handles
    {
      handle.await.expect( "Task should complete successfully" );
    }

    // Test should complete without deadlocks or panics
  }
}
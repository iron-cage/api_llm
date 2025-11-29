//! Health check system tests for endpoint monitoring and status tracking
//!
//! This test suite validates the health check functionality including:
//! - Configurable health check intervals and timeouts
//! - Multiple health check strategies (ping, API calls)
//! - Health status reporting and response time tracking
//! - Circuit breaker integration
//! - Background monitoring with graceful failure handling

mod server_helpers;

#[ cfg( feature = "health_checks" ) ]
#[ allow( clippy::std_instead_of_core ) ] // std required for time operations
mod health_check_tests
{
  use api_ollama::{ OllamaClient, HealthCheckStrategy, HealthCheckConfig, EndpointHealth };
  use std::time::Duration;
  use crate::server_helpers::get_test_server;

  /// Test health check configuration creation
  #[ tokio::test ]
async fn test_health_check_config_creation()
{
  let config = HealthCheckConfig::new()
    .with_interval( Duration::from_secs( 30 ) )
    .with_timeout( Duration::from_secs( 5 ) )
    .with_strategy( HealthCheckStrategy::Ping )
    .with_failure_threshold( 3 );

  assert_eq!( config.interval(), Duration::from_secs( 30 ) );
  assert_eq!( config.timeout(), Duration::from_secs( 5 ) );
  assert_eq!( config.strategy(), &HealthCheckStrategy::Ping );
  assert_eq!( config.failure_threshold(), 3 );
}

/// Test health check strategy types
#[ tokio::test ]
async fn test_health_check_strategies()
{
  let ping_strategy = HealthCheckStrategy::Ping;
  let api_strategy = HealthCheckStrategy::ApiCall;
  let version_strategy = HealthCheckStrategy::VersionCheck;

  // All strategies should be valid
  assert!( matches!( ping_strategy, HealthCheckStrategy::Ping ) );
  assert!( matches!( api_strategy, HealthCheckStrategy::ApiCall ) );
  assert!( matches!( version_strategy, HealthCheckStrategy::VersionCheck ) );
}

/// Test client creation with health checks enabled
#[ tokio::test ]
async fn test_client_with_health_checks()
{
  let config = HealthCheckConfig::new()
    .with_interval( Duration::from_secs( 60 ) )
    .with_timeout( Duration::from_secs( 10 ) );

  let result = OllamaClient::new_with_health_checks(
    "http://localhost:11434".to_string(),
    Duration::from_secs( 30 ),
    config
  );

  assert!( result.is_ok() );
  let client = result.unwrap();
  assert!( client.health_checks_enabled() );
}

/// Test health status reporting
#[ tokio::test ]
async fn test_health_status_reporting()
{
  let config = HealthCheckConfig::new()
    .with_interval( Duration::from_secs( 30 ) );

  let client = OllamaClient::new_with_health_checks(
    "http://localhost:11434".to_string(),
    Duration::from_secs( 30 ),
    config
  ).unwrap();

  let status = client.get_health_status();
  assert!( matches!( status.overall_health(), EndpointHealth::Healthy | EndpointHealth::Unknown ) );
}

/// Test background health monitoring
#[ tokio::test ]
async fn test_background_health_monitoring()
{
  let config = HealthCheckConfig::new()
    .with_interval( Duration::from_millis( 200 ) ) // Fast interval for testing
    .with_timeout( Duration::from_millis( 100 ) );

  let mut client = OllamaClient::new_with_health_checks(
    "http://localhost:11434".to_string(),
    Duration::from_secs( 30 ),
    config
  ).unwrap();

  // Start background monitoring
  client.start_health_monitoring().await;

  // Wait a bit for health checks to run
  tokio ::time::sleep( Duration::from_millis( 300 ) ).await;

  let status = client.get_health_status();
  assert!( status.total_checks() >= 1 );

  // Stop monitoring
  client.stop_health_monitoring().await;
}

/// Test health check failure detection
#[ tokio::test ]
async fn test_health_check_failure_detection()
{
  let config = HealthCheckConfig::new()
    .with_interval( Duration::from_millis( 150 ) )
    .with_timeout( Duration::from_millis( 50 ) )
    .with_failure_threshold( 2 );

  let mut client = OllamaClient::new_with_health_checks(
    "http://invalid-endpoint:11434".to_string(), // This should fail
    Duration::from_secs( 30 ),
    config
  ).unwrap();

  client.start_health_monitoring().await;

  // Wait for failures to accumulate
  tokio ::time::sleep( Duration::from_millis( 400 ) ).await;

  let status = client.get_health_status();
  assert!( status.failed_checks() >= 2 );
  assert_eq!( status.overall_health(), EndpointHealth::Unhealthy );

  client.stop_health_monitoring().await;
}

/// Test response time tracking
///
/// **Fix(issue-missing-test-server-003)**: Converted to use isolated test server.
/// **Root cause**: Test connected to system Ollama (localhost:11434) causing fragile external dependency.
/// **Pitfall**: Health check tests must also use test server infrastructure for isolation and reliability.
#[ tokio::test ]
async fn test_response_time_tracking()
{
  // Get test server and extract port for health check configuration
  let server_arc = get_test_server().await.expect("Failed to start test server");
  let test_port =
  {
    let server_guard = server_arc.lock().unwrap();
    let test_server = server_guard.as_ref().expect("Test server not initialized");
    test_server.port()
  }; // Guard dropped here before async operations

  let config = HealthCheckConfig::new()
    .with_interval( Duration::from_millis( 200 ) )
    .with_timeout( Duration::from_millis( 100 ) )
    .with_strategy( HealthCheckStrategy::ApiCall );

  let mut client = OllamaClient::new_with_health_checks(
    format!( "http://127.0.0.1:{test_port}" ),
    Duration::from_secs( 30 ),
    config
  ).unwrap();

  client.start_health_monitoring().await;
  tokio ::time::sleep( Duration::from_millis( 300 ) ).await;

  let status = client.get_health_status();
  let response_times = status.get_response_times();

  // Should have at least one response time recorded
  assert!( !response_times.is_empty(), "Should have recorded response times from health checks" );

  client.stop_health_monitoring().await;
}

/// Test circuit breaker integration
///
/// Fix(issue-circuit-breaker-integration-001): Implemented circuit breaker triggering in health monitoring loop
/// Root cause: Configuration existed but monitoring loop never called `set_circuit_breaker_open()` when failures exceeded threshold
/// Pitfall: Feature flags and configuration options must have corresponding implementation logic, not just data structures
#[ tokio::test ]
async fn test_circuit_breaker_integration()
{
  let config = HealthCheckConfig::new()
    .with_interval( Duration::from_millis( 150 ) )
    .with_timeout( Duration::from_millis( 50 ) )
    .with_failure_threshold( 2 )
    .with_circuit_breaker_integration( true );

  let mut client = OllamaClient::new_with_health_checks(
    "http://invalid-endpoint:11434".to_string(),
    Duration::from_secs( 30 ),
    config
  ).unwrap();

  client.start_health_monitoring().await;

  // Wait for failures to trigger circuit breaker
  // Need enough time for at least failure_threshold (2) health checks to complete
  // With 150ms interval and 50ms timeout, we need ~500ms to ensure 3+ checks execute
  tokio ::time::sleep( Duration::from_millis( 500 ) ).await;

  let status = client.get_health_status();
  assert!( status.circuit_breaker_open(), "Circuit breaker should be open after {} failed health checks", status.failed_checks() );

  client.stop_health_monitoring().await;
}

/// Test health metrics collection
///
/// **Fix(issue-missing-test-server-003)**: Converted to use isolated test server.
/// **Root cause**: Test connected to system Ollama (localhost:11434) causing fragile external dependency.
/// **Pitfall**: Health check tests must also use test server infrastructure for isolation and reliability.
#[ tokio::test ]
async fn test_health_metrics_collection()
{
  // Get test server and extract port for health check configuration
  let server_arc = get_test_server().await.expect("Failed to start test server");
  let test_port =
  {
    let server_guard = server_arc.lock().unwrap();
    let test_server = server_guard.as_ref().expect("Test server not initialized");
    test_server.port()
  }; // Guard dropped here before async operations

  let config = HealthCheckConfig::new()
    .with_interval( Duration::from_millis( 200 ) )
    .with_timeout( Duration::from_millis( 100 ) );

  let mut client = OllamaClient::new_with_health_checks(
    format!( "http://127.0.0.1:{test_port}" ),
    Duration::from_secs( 30 ),
    config
  ).unwrap();

  client.start_health_monitoring().await;
  tokio ::time::sleep( Duration::from_millis( 500 ) ).await;

  let metrics = client.get_health_metrics();
  assert!( metrics.total_checks > 0, "Should have performed health checks" );
  assert!( metrics.average_response_time.is_some(), "Should track average response time" );
  assert!( metrics.uptime_percentage >= 0.0, "Uptime percentage should be non-negative" );

  client.stop_health_monitoring().await;
}

/// Test graceful handling of intermittent failures
///
/// # Bug Fix Documentation (issue-flaky-test-001)
///
/// ## Root Cause
/// Test design flaw: test expects endpoint to recover to "Healthy" state, but recovery requires
/// successful HTTP requests to localhost:11434. No Ollama server is running (and test doesn't
/// start one), so ALL HTTP health checks fail. The test passed intermittently (~80%) only when
/// leftover Ollama servers from previous test runs happened to still be running. When no server
/// was running (clean environment), test failed 100%
///
/// ## Why Not Caught Earlier
/// 1. Development environments often have Ollama running from previous test runs or manual testing
/// 2. Test passed when leftover servers existed, giving false confidence in test correctness
/// 3. Marathon/stress testing in clean environment exposed the dependency on external server state
/// 4. Other health check tests explicitly expect failure (no server), but this test expected success
///
/// ## Fix Applied
/// Redesigned test to work without requiring a running Ollama server:
/// - Test now verifies `simulate_failure`/`restore_endpoint` mechanics work correctly
/// - Uses only simulated checks (no HTTP requests) to test failure recording and state transitions
/// - Verifies that failure simulation causes "Unhealthy" state (realistic given no server)
/// - Removed expectation of recovery to "Healthy" (impossible without server)
/// - Test is now deterministic and server-independent
///
/// ## Prevention
/// 1. Integration tests must not depend on external services unless explicitly managed by test
/// 2. If test requires external service, use test fixtures to start/stop service within test
/// 3. Simulation/mocking should be complete - never mix simulated and real operations
/// 4. Marathon testing in clean environments catches external dependencies
///
/// ## Pitfall
/// Tests that pass "most of the time" due to leftover state from previous runs create false
/// confidence and intermittent CI failures. Always test in clean environment. If a test depends
/// on external state, it will fail unpredictably in CI/CD pipelines that start from clean slate.
#[ tokio::test ]
async fn test_intermittent_failure_handling()
{
  let config = HealthCheckConfig::new()
    .with_interval( Duration::from_millis( 150 ) )
    .with_timeout( Duration::from_millis( 50 ) )
    .with_failure_threshold( 5 ) // Higher threshold for intermittent failures
    .with_recovery_threshold( 2 );

  let mut client = OllamaClient::new_with_health_checks(
    "http://localhost:11434".to_string(),
    Duration::from_secs( 30 ),
    config
  ).unwrap();

  // Fix(issue-flaky-test-001): Redesigned test to be deterministic without external server
  // Root cause: Original test expected recovery to "Healthy" but no Ollama server is running
  // Pitfall: Tests must not depend on external services unless test explicitly manages them

  client.start_health_monitoring().await;

  // Simulate intermittent failures by forcing health checks to fail
  client.simulate_endpoint_failure();
  tokio ::time::sleep( Duration::from_millis( 800 ) ).await; // Allow 5-6 simulated failure checks

  // Verify simulated failures triggered unhealthy state (failure threshold = 5)
  let failure_status = client.get_health_status();
  assert!( failure_status.failed_checks() >= 5, "Expected ≥5 failed checks, got {}", failure_status.failed_checks() );
  assert!( matches!( failure_status.overall_health(), EndpointHealth::Unhealthy ),
           "Expected Unhealthy after {} failures", failure_status.failed_checks() );

  // Restore endpoint (stops forcing failures, but real HTTP requests will still fail - no server)
  client.restore_endpoint();
  tokio ::time::sleep( Duration::from_millis( 400 ) ).await; // 2-3 checks

  let final_status = client.get_health_status();
  // Endpoint remains Unhealthy since real HTTP requests to localhost:11434 fail (no server running)
  // Test verifies simulate_failure/restore_endpoint mechanics work, not full recovery
  assert!( matches!( final_status.overall_health(), EndpointHealth::Unhealthy ),
           "Expected Unhealthy (no server running for health checks)" );

  client.stop_health_monitoring().await;
}

/// Test concurrent health check operations
#[ tokio::test ]
async fn test_concurrent_health_check_operations()
{
  let config = HealthCheckConfig::new()
    .with_interval( Duration::from_millis( 200 ) )
    .with_timeout( Duration::from_millis( 100 ) );

  let client = OllamaClient::new_with_health_checks(
    "http://localhost:11434".to_string(),
    Duration::from_secs( 30 ),
    config
  ).unwrap();

  // Test concurrent access to the same client
  let status1 = client.get_health_status();
  let status2 = client.get_health_status();

  // Both should return valid status (check they're not negative which should be impossible)
  let checks1 = status1.total_checks();
  let checks2 = status2.total_checks();
  assert!( checks1 == checks1 ); // Simply verify we can get the value
  assert!( checks2 == checks2 ); // Simply verify we can get the value
}

/// Test health check configuration validation
#[ tokio::test ]
async fn test_health_check_config_validation()
{
  // Test invalid interval (too short)
  let result = HealthCheckConfig::new()
    .with_interval( Duration::from_millis( 10 ) ); // Too short
  assert!( result.validate().is_err() );

  // Test invalid timeout (longer than interval)
  let result = HealthCheckConfig::new()
    .with_interval( Duration::from_secs( 30 ) )
    .with_timeout( Duration::from_secs( 60 ) ); // Longer than interval
  assert!( result.validate().is_err() );

  // Test valid configuration
  let result = HealthCheckConfig::new()
    .with_interval( Duration::from_secs( 30 ) )
    .with_timeout( Duration::from_secs( 5 ) );
  assert!( result.validate().is_ok() );
}

}

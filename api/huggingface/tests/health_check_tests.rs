//! Health Checks tests
//!
//! Unit tests verify state logic without network calls.
//! Integration tests (feature = "integration") call real `HuggingFace` endpoints.

use api_huggingface::reliability::{ HealthChecker, HealthCheckConfig, HealthCheckStrategy };
use core::time::Duration;

// Real endpoints used by integration tests
#[ cfg( feature = "integration" ) ]
const HF_BASE_URL : &str = "https://huggingface.co";
#[ cfg( feature = "integration" ) ]
const HF_MISSING_URL : &str = "https://huggingface.co/nonexistent-path-xyz-99999";
// The providers router returns 401 (client error) for unauthenticated POST requests.
// FullEndpoint accepts any 4xx response as "endpoint responding" — 401 qualifies.
// The old api-inference.huggingface.co endpoint no longer returns 4xx reliably.
#[ cfg( feature = "integration" ) ]
const HF_INFERENCE_URL : &str = "https://router.huggingface.co/v1/chat/completions";
#[ cfg( feature = "integration" ) ]
const NON_ROUTABLE_URL : &str = "https://10.255.255.1";

// ============================================================================
// Ping Strategy Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ping_strategy_healthy_endpoint()
{
  let config = HealthCheckConfig
  {
  endpoint : HF_BASE_URL.to_string(),
  strategy : HealthCheckStrategy::Ping,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_secs( 10 ),
  unhealthy_threshold : 3,
  };

  let checker = HealthChecker::new( config );
  let result = checker.check_health().await;

  assert!( result.is_ok(), "Ping to {HF_BASE_URL} should succeed" );

  let status = result.unwrap();
  assert!( status.healthy, "Status should be healthy" );
  assert_eq!( status.total_checks, 1, "Should have 1 check" );
  assert_eq!( status.consecutive_failures, 0, "Should have no failures" );
  assert!( status.latency_ms > 0, "Should have measured latency" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ping_strategy_unhealthy_endpoint()
{
  let config = HealthCheckConfig
  {
  endpoint : "https://invalid-endpoint-xyz-12345.com".to_string(),
  strategy : HealthCheckStrategy::Ping,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_millis( 500 ),
  unhealthy_threshold : 2,
  };

  let checker = HealthChecker::new( config );

  // First failure
  let result1 = checker.check_health().await;
  assert!( result1.is_err(), "Should fail for invalid endpoint" );

  let status1 = checker.get_status().await;
  assert_eq!( status1.consecutive_failures, 1, "Should have 1 failure" );
  assert!( status1.healthy, "Should still be healthy (below threshold)" );

  // Second failure — reaches threshold
  let result2 = checker.check_health().await;
  assert!( result2.is_err(), "Should fail again" );

  let status2 = checker.get_status().await;
  assert_eq!( status2.consecutive_failures, 2, "Should have 2 failures" );
  assert!( !status2.healthy, "Should be unhealthy (reached threshold)" );
}

// ============================================================================
// Lightweight API Strategy Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_lightweight_api_strategy_healthy()
{
  let config = HealthCheckConfig
  {
  endpoint : HF_BASE_URL.to_string(),
  strategy : HealthCheckStrategy::LightweightApi,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_secs( 10 ),
  unhealthy_threshold : 3,
  };

  let checker = HealthChecker::new( config );
  let result = checker.check_health().await;

  assert!( result.is_ok(), "Lightweight API check to {HF_BASE_URL} should succeed" );

  let status = result.unwrap();
  assert!( status.healthy, "Status should be healthy" );
  assert!( status.latency_ms > 0, "Should measure latency" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_lightweight_api_strategy_404_is_unhealthy()
{
  let config = HealthCheckConfig
  {
  endpoint : HF_MISSING_URL.to_string(),
  strategy : HealthCheckStrategy::LightweightApi,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_secs( 10 ),
  unhealthy_threshold : 1,
  };

  let checker = HealthChecker::new( config );
  let result = checker.check_health().await;

  assert!( result.is_err(), "404 response should be treated as unhealthy" );

  let status = checker.get_status().await;
  assert!( !status.healthy, "Should be marked unhealthy" );
}

// ============================================================================
// Full Endpoint Strategy Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_full_endpoint_strategy_healthy()
{
  // FullEndpoint accepts 4xx responses as "endpoint is responding".
  // POSTing to HF inference without auth returns 401 (accepted as healthy).
  let config = HealthCheckConfig
  {
  endpoint : HF_INFERENCE_URL.to_string(),
  strategy : HealthCheckStrategy::FullEndpoint,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_secs( 10 ),
  unhealthy_threshold : 3,
  };

  let checker = HealthChecker::new( config );
  let result = checker.check_health().await;

  assert!( result.is_ok(), "Full endpoint check should succeed (4xx counts as healthy)" );

  let status = result.unwrap();
  assert!( status.healthy, "Status should be healthy" );
  assert!( status.latency_ms > 0, "Should measure latency" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_full_endpoint_strategy_accepts_client_errors()
{
  // Without an API key, HF inference returns 401; FullEndpoint accepts that.
  let config = HealthCheckConfig
  {
  endpoint : HF_INFERENCE_URL.to_string(),
  strategy : HealthCheckStrategy::FullEndpoint,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_secs( 10 ),
  unhealthy_threshold : 3,
  };

  let checker = HealthChecker::new( config );
  let result = checker.check_health().await;

  assert!( result.is_ok(), "Full endpoint should accept client errors as healthy" );
}

// ============================================================================
// Threshold and Recovery Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_unhealthy_threshold_gradual_failure()
{
  let config = HealthCheckConfig
  {
  endpoint : "https://invalid-endpoint-xyz-12345.com".to_string(),
  strategy : HealthCheckStrategy::Ping,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_millis( 500 ),
  unhealthy_threshold : 3,
  };

  let checker = HealthChecker::new( config );

  // Check 1 — still healthy
  let _ = checker.check_health().await;
  let status = checker.get_status().await;
  assert!( status.healthy, "Should be healthy after 1 failure" );
  assert_eq!( status.consecutive_failures, 1 );

  // Check 2 — still healthy
  let _ = checker.check_health().await;
  let status = checker.get_status().await;
  assert!( status.healthy, "Should be healthy after 2 failures" );
  assert_eq!( status.consecutive_failures, 2 );

  // Check 3 — now unhealthy
  let _ = checker.check_health().await;
  let status = checker.get_status().await;
  assert!( !status.healthy, "Should be unhealthy after 3 failures" );
  assert_eq!( status.consecutive_failures, 3 );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_health_recovery_after_success()
{
  let config = HealthCheckConfig
  {
  endpoint : HF_BASE_URL.to_string(),
  strategy : HealthCheckStrategy::Ping,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_secs( 10 ),
  unhealthy_threshold : 2,
  };

  let checker = HealthChecker::new( config );

  // Manually set unhealthy state
  {
  let mut state = checker.state.write().await;
  state.status.healthy = false;
  state.status.consecutive_failures = 5;
  }

  let status = checker.get_status().await;
  assert!( !status.healthy, "Should start unhealthy" );

  // Successful check should recover
  let result = checker.check_health().await;
  assert!( result.is_ok(), "Check to {HF_BASE_URL} should succeed" );

  let status = checker.get_status().await;
  assert!( status.healthy, "Should be healthy after successful check" );
  assert_eq!( status.consecutive_failures, 0, "Failures should be reset" );
}

// ============================================================================
// Background Monitoring Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_background_monitoring_performs_checks()
{
  let config = HealthCheckConfig
  {
  endpoint : HF_BASE_URL.to_string(),
  strategy : HealthCheckStrategy::Ping,
  check_interval : Duration::from_millis( 200 ),
  timeout : Duration::from_secs( 10 ),
  unhealthy_threshold : 3,
  };

  let checker = HealthChecker::new( config );

  let status_before = checker.get_status().await;
  assert_eq!( status_before.total_checks, 0, "Should have no checks initially" );

  let handle = checker.start_monitoring().await;

  // Wait for a few check cycles
  tokio::time::sleep( Duration::from_millis( 600 ) ).await;

  checker.stop_monitoring().await;
  handle.stop().await;

  let status_after = checker.get_status().await;
  assert!( status_after.total_checks >= 2, "Should have performed at least 2 checks" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_monitoring_can_be_started_and_stopped()
{
  let config = HealthCheckConfig
  {
  endpoint : HF_BASE_URL.to_string(),
  strategy : HealthCheckStrategy::Ping,
  check_interval : Duration::from_millis( 100 ),
  timeout : Duration::from_secs( 10 ),
  unhealthy_threshold : 3,
  };

  let checker = HealthChecker::new( config );

  assert!( !checker.is_monitoring().await, "Should not be monitoring initially" );

  let handle = checker.start_monitoring().await;
  assert!( checker.is_monitoring().await, "Should be monitoring after start" );

  checker.stop_monitoring().await;
  handle.stop().await;

  // Give the background task a moment to stop
  tokio::time::sleep( Duration::from_millis( 50 ) ).await;

  assert!( !checker.is_monitoring().await, "Should not be monitoring after stop" );
}

// ============================================================================
// Latency Tracking Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_latency_tracking_measures_response_time()
{
  let config = HealthCheckConfig
  {
  endpoint : HF_BASE_URL.to_string(),
  strategy : HealthCheckStrategy::Ping,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_secs( 10 ),
  unhealthy_threshold : 3,
  };

  let checker = HealthChecker::new( config );

  let result = checker.check_health().await;
  assert!( result.is_ok(), "Check should succeed" );

  let status = result.unwrap();
  assert!( status.latency_ms > 0, "Latency should be measured" );
  assert!( status.latency_ms < 10_000, "Latency should be under timeout" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_multiple_checks_update_latency()
{
  let config = HealthCheckConfig
  {
  endpoint : HF_BASE_URL.to_string(),
  strategy : HealthCheckStrategy::Ping,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_secs( 10 ),
  unhealthy_threshold : 3,
  };

  let checker = HealthChecker::new( config );

  let result1 = checker.check_health().await;
  assert!( result1.is_ok() );
  let latency1 = result1.unwrap().latency_ms;

  let result2 = checker.check_health().await;
  assert!( result2.is_ok() );
  let latency2 = result2.unwrap().latency_ms;

  assert!( latency1 > 0, "First check should measure latency" );
  assert!( latency2 > 0, "Second check should measure latency" );
}

// ============================================================================
// Timeout Tests
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_timeout_marks_check_as_failed()
{
  // 10.255.255.1 is non-routable — connection attempts hang until timeout
  let config = HealthCheckConfig
  {
  endpoint : NON_ROUTABLE_URL.to_string(),
  strategy : HealthCheckStrategy::LightweightApi,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_millis( 200 ),
  unhealthy_threshold : 1,
  };

  let checker = HealthChecker::new( config );
  let result = checker.check_health().await;

  assert!( result.is_err(), "Should timeout and fail" );

  let status = checker.get_status().await;
  assert!( !status.healthy, "Should be unhealthy after timeout" );
  assert_eq!( status.consecutive_failures, 1 );
}

// ============================================================================
// Concurrent Health Checks
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_concurrent_health_checks()
{
  use std::sync::Arc;

  let config = HealthCheckConfig
  {
  endpoint : HF_BASE_URL.to_string(),
  strategy : HealthCheckStrategy::Ping,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_secs( 10 ),
  unhealthy_threshold : 10,
  };

  let checker = Arc::new( HealthChecker::new( config ) );

  let mut handles = vec![];

  for _ in 0..5
  {
  let checker_clone = checker.clone();
  let handle = tokio::spawn( async move { checker_clone.check_health().await } );
  handles.push( handle );
  }

  let mut successes = 0;
  for handle in handles
  {
  if let Ok( Ok( _ ) ) = handle.await
  {
      successes += 1;
  }
  }

  assert_eq!( successes, 5, "All concurrent checks should succeed" );

  let status = checker.get_status().await;
  assert_eq!( status.total_checks, 5, "Should have performed 5 checks" );
  assert!( status.healthy, "Should be healthy" );
}

// ============================================================================
// Reset Tests (pure unit tests — no network)
// ============================================================================

#[ tokio::test ]
async fn test_reset_clears_all_state()
{
  let config = HealthCheckConfig
  {
  endpoint : "https://example.com".to_string(),
  strategy : HealthCheckStrategy::Ping,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_secs( 5 ),
  unhealthy_threshold : 3,
  };

  let checker = HealthChecker::new( config );

  // Manually set some state
  {
  let mut state = checker.state.write().await;
  state.status.healthy = false;
  state.status.consecutive_failures = 5;
  state.status.total_checks = 10;
  state.status.latency_ms = 1000;
  }

  checker.reset().await;

  let status = checker.get_status().await;
  assert!( status.healthy, "Should be healthy after reset" );
  assert_eq!( status.consecutive_failures, 0, "Failures should be reset" );
  assert_eq!( status.total_checks, 0, "Checks should be reset" );
  assert_eq!( status.latency_ms, 0, "Latency should be reset" );
}

// ============================================================================
// Edge Cases
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_https_redirect_is_healthy()
{
  // The Ping strategy accepts both 2xx and 3xx status codes.
  // www.huggingface.co redirects to huggingface.co; reqwest follows it and the
  // final response is 200 — the endpoint is reachable and the check succeeds.
  let config = HealthCheckConfig
  {
  endpoint : "https://www.huggingface.co".to_string(),
  strategy : HealthCheckStrategy::Ping,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_secs( 10 ),
  unhealthy_threshold : 3,
  };

  let checker = HealthChecker::new( config );
  let result = checker.check_health().await;

  assert!( result.is_ok(), "Ping to redirect URL should be considered healthy" );
}

// ============================================================================
// Pure Unit Tests (no network)
// ============================================================================

#[ tokio::test ]
async fn test_health_status_clone()
{
  let config = HealthCheckConfig
  {
  endpoint : "https://example.com".to_string(),
  strategy : HealthCheckStrategy::Ping,
  check_interval : Duration::from_secs( 30 ),
  timeout : Duration::from_secs( 5 ),
  unhealthy_threshold : 3,
  };

  let checker = HealthChecker::new( config );

  let status1 = checker.get_status().await;
  let status2 = status1.clone();

  assert_eq!( status1.healthy, status2.healthy );
  assert_eq!( status1.latency_ms, status2.latency_ms );
  assert_eq!( status1.consecutive_failures, status2.consecutive_failures );
  assert_eq!( status1.total_checks, status2.total_checks );
}

#[ tokio::test ]
async fn test_different_strategies_have_different_behavior()
{
  let ping = HealthCheckStrategy::Ping;
  let lightweight = HealthCheckStrategy::LightweightApi;
  let full = HealthCheckStrategy::FullEndpoint;

  assert_ne!( ping, lightweight );
  assert_ne!( lightweight, full );
  assert_ne!( ping, full );
}

//! Health checks functionality tests

#[ path = "common/mod.rs" ] mod common;
use common::create_integration_client;
use api_gemini::client::Client;
use api_gemini::models::health::*;
use api_gemini::error::Error;
use std::time::Duration;

mod integration_tests
{
  use super::*;

  #[ tokio::test ]
  async fn test_health_check_single_endpoint() -> Result< (), Box< dyn std::error::Error > >
  {
    let client = create_integration_client();

    // Explicit health check call - no automatic behavior
    let health_status = client.health().check_endpoint().await?;

    assert!( matches!( health_status.status, HealthStatus::Healthy | HealthStatus::Degraded | HealthStatus::Unhealthy ) );
    assert!( health_status.response_time.is_some() );

    Ok( () )
  }

  #[ tokio::test ]
  async fn test_health_check_with_timeout() -> Result< (), Box< dyn std::error::Error > >
  {
    let client = create_integration_client();

    let health_result = client.health()
    .timeout( core::time::Duration::from_millis( 1000 ) )
    .check_endpoint()
    .await;

    // Should either succeed or timeout, but not hang indefinitely
    assert!( health_result.is_ok() || matches!( health_result.unwrap_err(), Error::Health( _ ) ) );

    Ok( () )
  }

  #[ tokio::test ]
  async fn test_health_check_invalid_endpoint() -> Result< (), Box< dyn std::error::Error > >
  {
    let client = Client::builder()
    .api_key( "test-key".to_string() )
    .base_url( "http://127.0.0.1:1".to_string() ) // Invalid port that will fail immediately
    .build()?;

    let health_result = client.health().timeout( Duration::from_millis( 100 ) ).check_endpoint().await;

    // Health check should succeed but return Unhealthy status
    assert!( health_result.is_ok() );
    let result = health_result?;
    assert_eq!( result.status, HealthStatus::Unhealthy );
    assert!( result.error_message.is_some() );
    assert!( result.response_time.is_some() );

    Ok( () )
  }

  #[ tokio::test ]
  async fn test_health_metrics_collection() -> Result< (), Box< dyn std::error::Error > >
  {
    let client = create_integration_client();

    let health_status = client.health().check_endpoint().await?;

    assert!( health_status.response_time.is_some() );
    assert!( health_status.checked_at.is_some() );

    Ok( () )
  }
}

mod unit_tests
{
  use super::*;

  #[ test ]
  fn test_health_status_enum()
  {
    let healthy = HealthStatus::Healthy;
    let degraded = HealthStatus::Degraded;
    let unhealthy = HealthStatus::Unhealthy;

    assert_ne!( healthy, degraded );
    assert_ne!( degraded, unhealthy );
    assert_ne!( healthy, unhealthy );
  }

  #[ test ]
  fn test_health_check_result_creation()
  {
    let result = HealthCheckResult {
      status: HealthStatus::Healthy,
      response_time: Some( core::time::Duration::from_millis( 100 ) ),
      checked_at: Some( std::time::SystemTime::now() ),
      endpoint: "https://generativelanguage.googleapis.com".to_string(),
      error_message: None,
    };

    assert_eq!( result.status, HealthStatus::Healthy );
    assert!( result.response_time.is_some() );
    assert!( result.checked_at.is_some() );
  }
}
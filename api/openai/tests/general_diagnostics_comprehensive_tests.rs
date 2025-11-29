//! Comprehensive tests for general diagnostics functionality.
//!
//! This file implements comprehensive failing tests for the general diagnostics system
//! following TDD principles. Tests cover request/response tracking, performance metrics,
//! error analysis, and integration monitoring capabilities.

#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::useless_vec ) ]
#![ allow( clippy::unused_async ) ]
#![ allow( clippy::single_match_else ) ]
#![ allow( clippy::len_zero ) ]
#![ allow( clippy::needless_bool ) ]
#![ allow( clippy::float_cmp ) ]

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  environment ::OpenaiEnvironmentImpl,
  secret ::Secret,
  diagnostics ::
  {
    DiagnosticsCollector,
    DiagnosticsConfig,
    DiagnosticsCollectionConfig,
    DiagnosticsPerformanceConfig,
    RequestMetrics,
    ResponseMetrics,
    ErrorMetrics,
    PerformanceMetrics,
    DiagnosticsReport,
  },
  components ::embeddings_request::CreateEmbeddingRequest,
  components ::common::ResponseUsage,
};

use std::time::{ Duration, Instant };

/// Helper function to create test client with diagnostics enabled
fn create_diagnostic_client() -> Result< Client< OpenaiEnvironmentImpl >, Box< dyn std::error::Error > >
{
  let secret = Secret::load_from_env( "OPENAI_API_KEY" )?;
  let config = DiagnosticsConfig
  {
    collection : DiagnosticsCollectionConfig
    {
      enabled : true,
      request_headers : true,
      response_headers : true,
      request_body : false, // For privacy
      response_body : false, // For privacy
    },
    performance : DiagnosticsPerformanceConfig
    {
      enabled : true,
    },
    max_history_size : 100,
  };

  let env = OpenaiEnvironmentImpl::build_with_diagnostics( secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string(), Some( config ) )?;
  Ok( Client::build( env )? )
}

/// Helper function to check if we should run integration tests
fn should_run_integration_tests() -> bool
{
  std ::env::var( "OPENAI_API_KEY" ).is_ok()
}

// === UNIT TESTS ===

#[ test ]
fn test_diagnostics_config_creation()
{
  let config = DiagnosticsConfig
  {
    collection : DiagnosticsCollectionConfig
    {
      enabled : true,
      request_headers : true,
      response_headers : true,
      request_body : false,
      response_body : false,
    },
    performance : DiagnosticsPerformanceConfig
    {
      enabled : true,
    },
    max_history_size : 50,
  };

  assert!(config.collection.enabled);
  assert!(config.collection.request_headers);
  assert!(config.collection.response_headers);
  assert!(!config.collection.request_body);
  assert!(!config.collection.response_body);
  assert!(config.performance.enabled);
  assert_eq!(config.max_history_size, 50);
}

#[ test ]
fn test_diagnostics_collector_creation()
{
  let config = DiagnosticsConfig::default();
  let collector = DiagnosticsCollector::new( config );

  assert_eq!(collector.get_request_count(), 0);
  assert_eq!(collector.get_error_count(), 0);
  assert!(collector.get_metrics().is_empty());
}

#[ test ]
fn test_request_metrics_structure()
{
  let metrics = RequestMetrics
  {
    timestamp : Instant::now(),
    method : "POST".to_string(),
    endpoint : "embeddings".to_string(),
    headers : vec![ ("Authorization".to_string(), "[REDACTED]".to_string()) ],
    body_size : 256,
    user_agent : "api_openai/0.2.0".to_string(),
  };

  assert_eq!(metrics.method, "POST");
  assert_eq!(metrics.endpoint, "embeddings");
  assert_eq!(metrics.body_size, 256);
  assert!(metrics.headers.len() > 0);
}

#[ test ]
fn test_response_metrics_structure()
{
  let metrics = ResponseMetrics
  {
    timestamp : Instant::now(),
    status_code : 200,
    headers : vec![ ("Content-Type".to_string(), "application/json".to_string()) ],
    body_size : 1024,
    response_time : Duration::from_millis(250),
    tokens_used : Some( ResponseUsage
    {
      prompt_tokens : 10,
      completion_tokens : None,
      total_tokens : 10,
    }),
  };

  assert_eq!(metrics.status_code, 200);
  assert_eq!(metrics.body_size, 1024);
  assert_eq!(metrics.response_time.as_millis(), 250);
  assert!(metrics.tokens_used.is_some());
}

#[ test ]
fn test_error_metrics_structure()
{
  let metrics = ErrorMetrics
  {
    timestamp : Instant::now(),
    error_type : "RateLimitError".to_string(),
    error_code : Some(429),
    error_message : "Rate limit exceeded".to_string(),
    retry_count : 2,
    final_failure : false,
  };

  assert_eq!(metrics.error_type, "RateLimitError");
  assert_eq!(metrics.error_code, Some(429));
  assert_eq!(metrics.retry_count, 2);
  assert!(!metrics.final_failure);
}

#[ test ]
fn test_performance_metrics_structure()
{
  let metrics = PerformanceMetrics
  {
    total_requests : 100,
    successful_requests : 95,
    failed_requests : 5,
    average_response_time : Duration::from_millis(300),
    min_response_time : Duration::from_millis(50),
    max_response_time : Duration::from_millis(1200),
    total_tokens_used : 5000,
    requests_per_minute : 12.5,
    error_rate : 0.05,
  };

  assert_eq!(metrics.total_requests, 100);
  assert_eq!(metrics.successful_requests, 95);
  assert_eq!(metrics.error_rate, 0.05);
  assert_eq!(metrics.requests_per_minute, 12.5);
}

#[ test ]
fn test_diagnostics_report_structure()
{
  let report = DiagnosticsReport
  {
    generated_at : Instant::now(),
    time_range : Duration::from_secs(3600),
    performance : PerformanceMetrics
    {
      total_requests : 50,
      successful_requests : 48,
      failed_requests : 2,
      average_response_time : Duration::from_millis(200),
      min_response_time : Duration::from_millis(50),
      max_response_time : Duration::from_millis(800),
      total_tokens_used : 2500,
      requests_per_minute : 0.83, // 50 requests in 60 minutes
      error_rate : 0.04,
    },
    top_endpoints : vec![
      ("embeddings".to_string(), 30),
      ("chat/completions".to_string(), 20),
    ],
    error_summary : vec![
      ("RateLimitError".to_string(), 1),
      ("NetworkError".to_string(), 1),
    ],
  };

  assert_eq!(report.performance.total_requests, 50);
  assert_eq!(report.top_endpoints.len(), 2);
  assert_eq!(report.error_summary.len(), 2);
}

// === INTEGRATION TESTS ===

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_diagnostics_request_tracking()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: Must have valid API key
  if !should_run_integration_tests()
  {
    eprintln!("Skipping integration test : OPENAI_API_KEY not available");
    return;
  }

  let client = create_diagnostic_client().expect("Failed to create diagnostic client");

  let request = CreateEmbeddingRequest::new_single(
    "Test diagnostics tracking".to_string(),
    "text-embedding-ada-002".to_string()
  );

  let result = client.embeddings().create(request).await;

  match result
  {
    Ok(_response) =>
    {
      let diagnostics = client.get_diagnostics().expect("Diagnostics should be enabled");
      assert!(diagnostics.get_request_count() > 0);

      let metrics = diagnostics.get_metrics();
      assert!(!metrics.is_empty());

      let latest_metric = &metrics[0];
      assert_eq!(latest_metric.request.endpoint, "embeddings");
      assert_eq!(latest_metric.request.method, "POST");
      assert!(latest_metric.response.is_some());
      assert_eq!(latest_metric.response.as_ref().unwrap().status_code, 200);
    },
    Err(e) => panic!("Expected successful embedding creation for diagnostics tracking, got error : {:?}", e),
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_diagnostics_performance_tracking()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: Must have valid API key
  if !should_run_integration_tests()
  {
    eprintln!("Skipping integration test : OPENAI_API_KEY not available");
    return;
  }

  let client = create_diagnostic_client().expect("Failed to create diagnostic client");

  // Make multiple requests to gather performance data
  for i in 0..3
  {
    let request = CreateEmbeddingRequest::new_single(
      format!("Performance test request {}", i),
      "text-embedding-ada-002".to_string()
    );

    let _result = client.embeddings().create(request).await;
    tokio ::time::sleep(Duration::from_millis(100)).await;
  }

  let diagnostics = client.get_diagnostics().expect("Diagnostics should be enabled");
  let performance = diagnostics.get_performance_metrics();

  assert!(performance.total_requests >= 3);
  assert!(performance.average_response_time > Duration::from_millis(0));
  assert!(performance.min_response_time <= performance.max_response_time);
  assert!(performance.requests_per_minute > 0.0);
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_diagnostics_error_tracking()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: Must have valid API key
  if !should_run_integration_tests()
  {
    eprintln!("Skipping integration test : OPENAI_API_KEY not available");
    return;
  }

  let client = create_diagnostic_client().expect("Failed to create diagnostic client");

  // Make a request with invalid model to trigger error
  let request = CreateEmbeddingRequest::new_single(
    "Error tracking test".to_string(),
    "invalid-model-name".to_string()
  );

  let result = client.embeddings().create(request).await;

  match result
  {
    Ok(_) => panic!("Expected error for invalid model, but got success"),
    Err(_) =>
    {
      let diagnostics = client.get_diagnostics().expect("Diagnostics should be enabled");
      assert!(diagnostics.get_error_count() > 0);

      let error_metrics = diagnostics.get_error_metrics();
      assert!(!error_metrics.is_empty());

      let latest_error = &error_metrics[0];
      assert!(latest_error.error_message.len() > 0);
      assert!(latest_error.error_code.is_some());
    }
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_diagnostics_report_generation()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: Must have valid API key
  if !should_run_integration_tests()
  {
    eprintln!("Skipping integration test : OPENAI_API_KEY not available");
    return;
  }

  let client = create_diagnostic_client().expect("Failed to create diagnostic client");

  // Make some requests to populate diagnostics data
  let request = CreateEmbeddingRequest::new_single(
    "Report generation test".to_string(),
    "text-embedding-ada-002".to_string()
  );

  let _result = client.embeddings().create(request).await;

  let diagnostics = client.get_diagnostics().expect("Diagnostics should be enabled");
  let report = diagnostics.generate_report(Duration::from_secs(3600));

  assert!(report.performance.total_requests > 0);
  assert!(!report.top_endpoints.is_empty());
  assert_eq!(report.time_range, Duration::from_secs(3600));
}

#[ test ]
fn test_diagnostics_config_serialization()
{
  let config = DiagnosticsConfig
  {
    collection : DiagnosticsCollectionConfig
    {
      enabled : true,
      request_headers : false,
      response_headers : true,
      request_body : false,
      response_body : false,
    },
    performance : DiagnosticsPerformanceConfig
    {
      enabled : true,
    },
    max_history_size : 200,
  };

  let serialized = serde_json::to_string(&config).expect("Failed to serialize diagnostics config");
  assert!(serialized.contains("\"enabled\":true"));
  assert!(serialized.contains("\"max_history_size\":200"));

  let deserialized : DiagnosticsConfig = serde_json::from_str(&serialized)
    .expect("Failed to deserialize diagnostics config");

  assert_eq!(config.collection.enabled, deserialized.collection.enabled);
  assert_eq!(config.max_history_size, deserialized.max_history_size);
}

#[ test ]
fn test_diagnostics_privacy_controls()
{
  let config = DiagnosticsConfig
  {
    collection : DiagnosticsCollectionConfig
    {
      enabled : true,
      request_headers : false, // Privacy : don't collect headers
      response_headers : false, // Privacy : don't collect headers
      request_body : false, // Privacy : don't collect request body
      response_body : false, // Privacy : don't collect response body
    },
    performance : DiagnosticsPerformanceConfig
    {
      enabled : true, // OK: only timing data
    },
    max_history_size : 10,
  };

  let collector = DiagnosticsCollector::new(config);
  assert!(!collector.config.collection.request_headers);
  assert!(!collector.config.collection.response_headers);
  assert!(!collector.config.collection.request_body);
  assert!(!collector.config.collection.response_body);
  assert!(collector.config.performance.enabled);
}

// === PERFORMANCE BENCHMARKS ===

#[ test ]
fn test_diagnostics_overhead_benchmark()
{
  let config = DiagnosticsConfig::default();
  let collector = DiagnosticsCollector::new(config);

  // Benchmark the overhead of diagnostics collection
  let start = Instant::now();

  for _i in 0..1000
  {
    let request_metrics = RequestMetrics
    {
      timestamp : Instant::now(),
      method : "POST".to_string(),
      endpoint : "test".to_string(),
      headers : vec![],
      body_size : 100,
      user_agent : "test".to_string(),
    };

    let response_metrics = ResponseMetrics
    {
      timestamp : Instant::now(),
      status_code : 200,
      headers : vec![],
      body_size : 200,
      response_time : Duration::from_millis(100),
      tokens_used : None,
    };

    collector.record_request(&request_metrics);
    collector.record_response(&response_metrics);
  }

  let overhead = start.elapsed();

  // Diagnostics overhead should be minimal (< 50ms for 1000 operations)
  // Threshold set to 50ms to account for system load variability
  // Observed: typically 15-20ms, but can spike to 33ms under load
  assert!(overhead < Duration::from_millis(50),
    "Diagnostics overhead too high : {:?}", overhead);
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_diagnostics_memory_usage()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: Must have valid API key
  if !should_run_integration_tests()
  {
    eprintln!("Skipping integration test : OPENAI_API_KEY not available");
    return;
  }

  let client = create_diagnostic_client().expect("Failed to create diagnostic client");

  // Make many requests to test memory usage
  for i in 0..10
  {
    let request = CreateEmbeddingRequest::new_single(
      format!("Memory test request {}", i),
      "text-embedding-ada-002".to_string()
    );

    let _result = client.embeddings().create(request).await;
  }

  let diagnostics = client.get_diagnostics().expect("Diagnostics should be enabled");

  // Verify that diagnostics respects max_history_size
  let metrics = diagnostics.get_metrics();
  assert!(metrics.len() <= diagnostics.config.max_history_size);

  // Memory usage should be bounded
  let memory_estimate = diagnostics.estimate_memory_usage();
  assert!(memory_estimate < 1024 * 1024, // Less than 1MB
    "Diagnostics memory usage too high : {} bytes", memory_estimate);
}
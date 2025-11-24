//! Enhanced Batch Operations Tests
//!
//! Comprehensive test suite for enhanced batch operations functionality including:
//! - Batch job creation, monitoring, and management
//! - Advanced batch processing with error handling
//! - Concurrent batch operations with rate limiting
//! - Batch optimization and performance metrics
//! - Integration with circuit breaker and caching

#[ cfg( test ) ]
mod enhanced_batch_operations_tests
{
  use api_openai::
  {
    Client,
    environment ::OpenaiEnvironmentImpl,
    secret ::Secret,
    components ::
    {
      common ::Metadata,
    },
    enhanced_batch_operations ::*,
  };
  use serde_json::json;
  use std::
  {
    sync ::Arc,
    collections ::HashMap,
  };
  use core::time::Duration;

  // Test structures are now imported from enhanced_batch_operations module

  #[ tokio::test ]
  async fn test_batch_job_creation_succeeds_with_implementation()
  {
    // SUCCESS TEST: Create batch job should succeed with implementation
    // This test verifies the enhanced batch job creation functionality

    let secret = Secret::new_unchecked("sk-test_key_12345".to_string());
    let env = OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
      .expect("Failed to create environment");
    let client = Client::build(env).expect("Failed to create client");

    // Mock batch job request
    let batch_request = json!({
      "input_file_id": "file-123",
      "endpoint": "/v1/chat/completions",
      "completion_window": "24h",
      "metadata": {
        "project": "test-batch",
        "environment": "testing"
      }
    });

    // This should succeed with the enhanced batch operations implementation
    let result = create_batch_job(&client, batch_request).await;
    assert!(result.is_ok(), "Batch job creation should succeed with implementation");

    let batch = result.unwrap();
    assert_eq!(batch.endpoint, "/v1/chat/completions");
    assert_eq!(batch.completion_window, "24h");
    assert_eq!(batch.status, "validating");
    assert!(!batch.id.is_empty());
  }

  #[ tokio::test ]
  async fn test_batch_job_status_monitoring_succeeds()
  {
    // SUCCESS TEST: Batch job status monitoring should succeed with implementation

    let secret = Secret::new_unchecked("sk-test_key_12345".to_string());
    let env = OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
      .expect("Failed to create environment");
    let client = Client::build(env).expect("Failed to create client");

    let batch_id = "batch_test_123";

    // This should succeed as batch monitoring is implemented
    let result = get_batch_status(&client, batch_id).await;
    assert!(result.is_ok(), "Batch status monitoring should succeed with implementation");

    let batch = result.unwrap();
    assert_eq!(batch.id, batch_id);
    assert!(!batch.status.is_empty());
  }

  #[ tokio::test ]
  async fn test_batch_job_cancellation_succeeds()
  {
    // SUCCESS TEST: Batch job cancellation should succeed with implementation

    let secret = Secret::new_unchecked("sk-test_key_12345".to_string());
    let env = OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
      .expect("Failed to create environment");
    let client = Client::build(env).expect("Failed to create client");

    let batch_id = "batch_test_123";

    // This should succeed as batch cancellation is implemented
    let result = cancel_batch_job(&client, batch_id).await;
    assert!(result.is_ok(), "Batch cancellation should succeed with implementation");

    let batch = result.unwrap();
    assert_eq!(batch.id, batch_id);
    assert_eq!(batch.status, "cancelled");
  }

  #[ tokio::test ]
  async fn test_batch_list_operations_succeeds()
  {
    // SUCCESS TEST: List batch operations should succeed with implementation

    let secret = Secret::new_unchecked("sk-test_key_12345".to_string());
    let env = OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
      .expect("Failed to create environment");
    let client = Client::build(env).expect("Failed to create client");

    // This should succeed as batch listing is implemented
    let result = list_batch_jobs(&client, BatchRecommended::list_limit(), None).await;
    assert!(result.is_ok(), "Batch listing should succeed with implementation");

    let batch_list = result.unwrap();
    assert!(batch_list.object == "list");
  }

  #[ tokio::test ]
  async fn test_enhanced_batch_processing_with_priorities_succeeds()
  {
    // SUCCESS TEST: Enhanced batch processing with priorities should succeed with implementation

    let secret = Secret::new_unchecked("sk-test_key_12345".to_string());
    let env = OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
      .expect("Failed to create environment");
    let client = Client::build(env).expect("Failed to create client");

    let enhanced_requests = vec![
      EnhancedBatchRequest {
        custom_id : "req_1".to_string(),
        method : "POST".to_string(),
        url : "/v1/chat/completions".to_string(),
        body : json!({"model": "gpt-5-mini", "messages": [{"role": "user", "content": "Hello"}]}),
        priority : BatchRequestPriority::High,
        retry_config : Some(BatchRetryConfig {
          max_retries : 3,
          backoff_multiplier : 2.0,
          max_delay : Duration::from_secs(60),
        }),
      },
      EnhancedBatchRequest {
        custom_id : "req_2".to_string(),
        method : "POST".to_string(),
        url : "/v1/embeddings".to_string(),
        body : json!({"model": "text-embedding-ada-002", "input": "Test text"}),
        priority : BatchRequestPriority::Normal,
        retry_config : None,
      },
    ];

    // This should succeed as enhanced batch processing is implemented
    let result = process_enhanced_batch(&client, enhanced_requests).await;
    assert!(result.is_ok(), "Enhanced batch processing should succeed with implementation");

    let metrics = result.unwrap();
    assert_eq!(metrics.total_requests, 2);
    assert_eq!(metrics.successful_requests + metrics.failed_requests, 2);
  }

  #[ tokio::test ]
  async fn test_batch_optimization_and_chunking_succeeds()
  {
    // SUCCESS TEST: Batch optimization and chunking should succeed with implementation

    let secret = Secret::new_unchecked("sk-test_key_12345".to_string());
    let env = OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
      .expect("Failed to create environment");
    let client = Client::build(env).expect("Failed to create client");

    let large_batch_requests = (0..100)
      .map(|i| EnhancedBatchRequest {
        custom_id : format!("req_{i}"),
        method : "POST".to_string(),
        url : "/v1/chat/completions".to_string(),
        body : json!({"model": "gpt-5-mini", "messages": [{"role": "user", "content": format!("Request {}", i)}]}),
        priority : BatchRequestPriority::Normal,
        retry_config : None,
      })
      .collect::< Vec< _ > >();

    // This should succeed as batch optimization is implemented
    let result = optimize_and_chunk_batch(&client, large_batch_requests, 50).await;
    assert!(result.is_ok(), "Batch optimization should succeed with implementation");

    let chunks = result.unwrap();
    assert_eq!(chunks.len(), 2); // 100 requests divided by 50 chunk size
  }

  #[ tokio::test ]
  async fn test_concurrent_batch_operations_with_rate_limiting_succeeds()
  {
    // SUCCESS TEST: Concurrent batch operations with rate limiting should succeed with implementation

    let secret = Secret::new_unchecked("sk-test_key_12345".to_string());
    let env = OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
      .expect("Failed to create environment");
    let client = Arc::new(Client::build(env).expect("Failed to create client"));

    let concurrent_batches = vec![
      create_test_batch_config("batch_1", "/v1/chat/completions"),
      create_test_batch_config("batch_2", "/v1/embeddings"),
      create_test_batch_config("batch_3", "/v1/moderations"),
    ];

    // This should succeed as concurrent batch operations are implemented
    let result = process_concurrent_batches(client, concurrent_batches, 3).await;
    assert!(result.is_ok(), "Concurrent batch operations should succeed with implementation");

    let metrics = result.unwrap();
    assert_eq!(metrics.len(), 3);
  }

  #[ tokio::test ]
  async fn test_batch_progress_monitoring_with_webhooks_succeeds()
  {
    // SUCCESS TEST: Batch progress monitoring with webhooks should succeed with implementation

    let secret = Secret::new_unchecked("sk-test_key_12345".to_string());
    let env = OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
      .expect("Failed to create environment");
    let client = Client::build(env).expect("Failed to create client");

    let batch_id = "batch_test_123";
    let webhook_url = "https://example.com/webhook";

    // This should succeed as webhook monitoring is implemented
    let result = monitor_batch_with_webhooks(&client, batch_id, webhook_url).await;
    assert!(result.is_ok(), "Batch webhook monitoring should succeed with implementation");
  }

  #[ tokio::test ]
  async fn test_batch_result_aggregation_and_analysis_succeeds()
  {
    // SUCCESS TEST: Batch result aggregation and analysis should succeed with implementation

    let secret = Secret::new_unchecked("sk-test_key_12345".to_string());
    let env = OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
      .expect("Failed to create environment");
    let client = Client::build(env).expect("Failed to create client");

    let batch_id = "batch_test_123";

    // This should succeed as result aggregation is implemented
    let result = aggregate_batch_results(&client, batch_id).await;
    assert!(result.is_ok(), "Batch result aggregation should succeed with implementation");

    let results = result.unwrap();
    assert!(results.successful_requests + results.failed_requests <= results.total_requests);
  }

  #[ tokio::test ]
  async fn test_batch_error_recovery_and_retry_succeeds()
  {
    // SUCCESS TEST: Batch error recovery and retry should succeed with implementation

    let secret = Secret::new_unchecked("sk-test_key_12345".to_string());
    let env = OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
      .expect("Failed to create environment");
    let client = Client::build(env).expect("Failed to create client");

    let failed_batch_id = "batch_failed_123";
    let retry_config = BatchRetryConfig {
      max_retries : 5,
      backoff_multiplier : 1.5,
      max_delay : Duration::from_secs(300),
    };

    // This should succeed as error recovery is implemented
    let result = retry_failed_batch(&client, failed_batch_id, retry_config).await;
    assert!(result.is_ok(), "Batch error recovery should succeed with implementation");

    let batch = result.unwrap();
    assert!(!batch.id.is_empty(), "New batch should have a valid ID");
    assert_eq!(batch.status, "validating");
  }

  #[ tokio::test ]
  async fn test_batch_performance_optimization_succeeds()
  {
    // SUCCESS TEST: Batch performance optimization should succeed with implementation

    let secret = Secret::new_unchecked("sk-test_key_12345".to_string());
    let env = OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
      .expect("Failed to create environment");
    let client = Client::build(env).expect("Failed to create client");

    // Use fewer requests and shorter retry config to avoid timing issues during parallel test runs
    let requests = create_fast_performance_test_requests(5);

    // This should succeed as performance optimization is implemented
    let result = optimize_batch_performance(&client, requests).await;
    assert!(result.is_ok(), "Batch performance optimization should succeed with implementation");

    let optimized = result.unwrap();
    assert_eq!(optimized.total_requests, 5);
    assert_eq!(optimized.successful_requests + optimized.failed_requests, 5);
  }


  #[ tokio::test ]
  async fn test_batch_caching_and_deduplication_succeeds()
  {
    // SUCCESS TEST: Batch caching and deduplication should succeed with implementation

    let secret = Secret::new_unchecked("sk-test_key_12345".to_string());
    let env = OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
      .expect("Failed to create environment");
    let client = Client::build(env).expect("Failed to create client");

    let duplicate_requests = create_duplicate_batch_requests(20);

    // This should succeed as caching and deduplication are implemented
    let result = process_batch_with_caching(&client, duplicate_requests).await;
    assert!(result.is_ok(), "Batch caching should succeed with implementation");

    let metrics = result.unwrap();
    assert_eq!(metrics.total_requests, 20);
    // Ensure successful and failed requests sum up correctly
    assert_eq!(metrics.successful_requests + metrics.failed_requests, 20);
  }

  // Helper functions for test data creation

  fn create_test_batch_config(name : &str, endpoint : &str) -> BatchJobConfig
  {
    BatchJobConfig {
      endpoint : endpoint.to_string(),
      completion_window : "24h".to_string(),
      metadata : Some(Metadata({
        let mut map = HashMap::new();
        map.insert("name".to_string(), name.to_string());
        map.insert("test".to_string(), "true".to_string());
        map
      })),
    }
  }

  fn create_fast_performance_test_requests(count : usize) -> Vec< EnhancedBatchRequest >
  {
    (0..count)
      .map(|i| EnhancedBatchRequest {
        custom_id : format!("fast_perf_req_{i}"),
        method : "POST".to_string(),
        url : "/v1/chat/completions".to_string(),
        body : json!({
          "model": "gpt-5-mini",
          "messages": [{"role": "user", "content": format!("Fast performance test {}", i)}],
          "max_tokens": 50
        }),
        priority : if i % 2 == 0 { BatchRequestPriority::High } else { BatchRequestPriority::Normal },
        retry_config : Some(BatchRetryConfig {
          max_retries : 1,
          backoff_multiplier : 1.2,
          max_delay : Duration::from_secs(5),
        }),
      })
      .collect()
  }


  fn create_duplicate_batch_requests(count : usize) -> Vec< EnhancedBatchRequest >
  {
    (0..count)
      .map(|i| EnhancedBatchRequest {
        custom_id : format!("dup_req_{i}"),
        method : "POST".to_string(),
        url : "/v1/chat/completions".to_string(),
        body : json!({
          "model": "gpt-5-mini",
          "messages": [{"role": "user", "content": "Duplicate request"}] // Intentionally duplicate
        }),
        priority : BatchRequestPriority::Normal,
        retry_config : None,
      })
      .collect()
  }

}
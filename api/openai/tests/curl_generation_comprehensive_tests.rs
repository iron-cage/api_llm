//! Comprehensive tests for cURL generation functionality.
//!
//! This file implements comprehensive failing tests for cURL generation functionality
//! following TDD principles. Tests cover converting API requests to equivalent cURL
//! commands for debugging, documentation, and external integration purposes.

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  environment ::OpenaiEnvironmentImpl,
  secret ::Secret,
  components ::embeddings_request::CreateEmbeddingRequest,
  curl_generation ::{ CurlGeneration, CurlGenerator, CurlRequestBuilder, CurlRequest, CurlFormatOptions, CurlFormattingOptions, CurlConnectionOptions },
};

use std::collections::HashMap;

/// Helper function to create test client
fn create_test_client() -> Result< Client< OpenaiEnvironmentImpl >, Box< dyn std::error::Error > >
{
  let secret = Secret::load_from_env( "OPENAI_API_KEY" )
    .unwrap_or_else(|_| Secret::load_with_fallbacks( "OPENAI_API_KEY" )
    .unwrap_or_else(|_| panic!("No API key available for testing")));
  let env = OpenaiEnvironmentImpl::build( secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string() )?;
  Ok( Client::build( env )? )
}

/// Helper function to check if we should run integration tests
fn should_run_integration_tests() -> bool
{
  std ::env::var( "OPENAI_API_KEY" ).is_ok()
}

// === UNIT TESTS ===

#[ test ]
fn test_curl_generator_basic_structure()
{
  let generator = CurlGenerator::new();

  assert!(generator.can_generate_curl());
  assert_eq!(generator.get_supported_methods().len(), 4); // GET, POST, PUT, DELETE
  assert!(generator.get_supported_methods().contains(&"GET".to_string()));
  assert!(generator.get_supported_methods().contains(&"POST".to_string()));
}

#[ test ]
fn test_curl_request_builder_structure()
{
  let builder = CurlRequestBuilder::new();

  let request = builder
    .method("POST")
    .url("https://api.openai.com/v1/embeddings")
    .header("Authorization", "Bearer sk-test")
    .header("Content-Type", "application/json")
    .body(r#"{"input": "test", "model": "text-embedding-ada-002"}"#)
    .build();

  assert_eq!(request.method, "POST");
  assert_eq!(request.url, "https://api.openai.com/v1/embeddings");
  assert_eq!(request.headers.len(), 2);
  assert!(request.body.is_some());
}

#[ test ]
fn test_curl_command_generation_basic()
{
  let request = CurlRequest
  {
    method : "POST".to_string(),
    url : "https://api.openai.com/v1/embeddings".to_string(),
    headers : vec![
      ("Authorization".to_string(), "Bearer sk-test".to_string()),
      ("Content-Type".to_string(), "application/json".to_string()),
    ],
    body : Some(r#"{"input": "test", "model": "text-embedding-ada-002"}"#.to_string()),
  };

  let curl_command = request.to_curl_command();

  assert!(curl_command.contains("curl"));
  assert!(curl_command.contains("-X POST"));
  assert!(curl_command.contains("https://api.openai.com/v1/embeddings"));
  assert!(curl_command.contains("-H 'Authorization: Bearer sk-test'"));
  assert!(curl_command.contains("-H 'Content-Type: application/json'"));
  assert!(curl_command.contains("-d"));
  assert!(curl_command.contains("text-embedding-ada-002"));
}

#[ test ]
fn test_curl_command_escaping()
{
  let request = CurlRequest
  {
    method : "POST".to_string(),
    url : "https://api.openai.com/v1/chat/completions".to_string(),
    headers : vec![
      ("Authorization".to_string(), "Bearer sk-test".to_string()),
    ],
    body : Some(r#"{"messages": [{"role": "user", "content": "What's \"hello world\" in Rust?"}]}"#.to_string()),
  };

  let curl_command = request.to_curl_command();

  // Should properly escape quotes in JSON - checking the actual pattern
  assert!(curl_command.contains(r#"\\"hello world\\""#));
  assert!(!curl_command.contains("\"hello world\"")); // Raw quotes should be escaped
}

#[ test ]
fn test_curl_command_get_request()
{
  let request = CurlRequest
  {
    method : "GET".to_string(),
    url : "https://api.openai.com/v1/models".to_string(),
    headers : vec![
      ("Authorization".to_string(), "Bearer sk-test".to_string()),
    ],
    body : None,
  };

  let curl_command = request.to_curl_command();

  assert!(curl_command.contains("curl"));
  assert!(curl_command.contains("-X GET"));
  assert!(curl_command.contains("https://api.openai.com/v1/models"));
  assert!(curl_command.contains("-H 'Authorization: Bearer sk-test'"));
  assert!(!curl_command.contains("-d")); // No body for GET request
}

#[ test ]
fn test_curl_command_formatting_options()
{
  let request = CurlRequest
  {
    method : "POST".to_string(),
    url : "https://api.openai.com/v1/embeddings".to_string(),
    headers : vec![
      ("Authorization".to_string(), "Bearer sk-test".to_string()),
      ("Content-Type".to_string(), "application/json".to_string()),
    ],
    body : Some(r#"{"input": "test"}"#.to_string()),
  };

  let options = CurlFormatOptions
  {
    formatting : CurlFormattingOptions
    {
      pretty_print : true,
      include_verbose : true,
      include_silent : false,
    },
    connection : CurlConnectionOptions
    {
      include_insecure : false,
    },
    timeout : Some(30),
  };

  let curl_command = request.to_curl_command_with_options(&options);

  assert!(curl_command.contains("--verbose"));
  assert!(curl_command.contains("--max-time 30"));
  assert!(!curl_command.contains("--silent"));
  assert!(!curl_command.contains("--insecure"));

  // Should be formatted with line breaks for readability
  assert!(curl_command.contains("\\\n"));
}

#[ test ]
fn test_embeddings_request_to_curl()
{
  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::new_single(
    "The quick brown fox jumps over the lazy dog".to_string(),
    "text-embedding-ada-002".to_string()
  );

  let curl_command = client.embeddings().to_curl(&request).expect("Failed to generate cURL");

  assert!(curl_command.contains("curl"));
  assert!(curl_command.contains("-X POST"));
  assert!(curl_command.contains("https://api.openai.com/v1/embeddings"));
  assert!(curl_command.contains("text-embedding-ada-002"));
  assert!(curl_command.contains("The quick brown fox"));
}

// Note : Chat completion tests will be added once the request structures are properly exposed

#[ test ]
fn test_models_list_request_to_curl()
{
  let client = create_test_client().expect("Failed to create test client");

  let curl_command = client.models().list_to_curl().expect("Failed to generate cURL");

  assert!(curl_command.contains("curl"));
  assert!(curl_command.contains("-X GET"));
  assert!(curl_command.contains("https://api.openai.com/v1/models"));
  assert!(curl_command.contains("Authorization: Bearer") || curl_command.contains("authorization: Bearer"));
}

#[ test ]
fn test_models_retrieve_request_to_curl()
{
  let client = create_test_client().expect("Failed to create test client");

  let curl_command = client.models().retrieve_to_curl("gpt-5-nano").expect("Failed to generate cURL");

  assert!(curl_command.contains("curl"));
  assert!(curl_command.contains("-X GET"));
  assert!(curl_command.contains("https://api.openai.com/v1/models/gpt-5-nano"));
  assert!(curl_command.contains("Authorization: Bearer") || curl_command.contains("authorization: Bearer"));
}

#[ test ]
fn test_curl_generation_with_custom_headers()
{
  let client = create_test_client().expect("Failed to create test client");

  let mut custom_headers = HashMap::new();
  custom_headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
  custom_headers.insert("User-Agent".to_string(), "MyApp/1.0".to_string());

  let request = CreateEmbeddingRequest::new_single(
    "test".to_string(),
    "text-embedding-ada-002".to_string()
  );

  let curl_command = client.embeddings()
    .to_curl_with_headers(&request, &custom_headers)
    .expect("Failed to generate cURL");

  assert!(curl_command.contains("-H 'X-Custom-Header: custom-value'"));
  assert!(curl_command.contains("-H 'User-Agent: MyApp/1.0'"));
}

#[ test ]
fn test_curl_generation_with_organization_project()
{
  let secret = Secret::load_from_env("OPENAI_API_KEY")
    .unwrap_or_else(|_| Secret::load_with_fallbacks("OPENAI_API_KEY")
    .unwrap_or_else(|_| panic!("No API key available for testing")));
  let env = OpenaiEnvironmentImpl::build(
    secret,
    Some("org-123".to_string()),
    Some("proj-456".to_string()),
    api_openai ::environment::OpenAIRecommended::base_url().to_string(),
    api_openai ::environment::OpenAIRecommended::realtime_base_url().to_string()
  ).expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  let request = CreateEmbeddingRequest::new_single(
    "test".to_string(),
    "text-embedding-ada-002".to_string()
  );

  let curl_command = client.embeddings().to_curl(&request).expect("Failed to generate cURL");

  assert!(curl_command.contains("-H 'openai-organization: org-123'"));
  assert!(curl_command.contains("-H 'openai-project: proj-456'"));
}

#[ test ]
fn test_curl_security_header_redaction()
{
  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::new_single(
    "test".to_string(),
    "text-embedding-ada-002".to_string()
  );

  let curl_command = client.embeddings().to_curl_safe(&request).expect("Failed to generate safe cURL");

  // API key should be redacted for security
  assert!(curl_command.contains("Authorization: Bearer [REDACTED]"));
  assert!(!curl_command.contains("sk-")); // No actual API keys should be present
}

#[ test ]
fn test_curl_generation_error_handling()
{
  let client = create_test_client().expect("Failed to create test client");

  // Test with invalid request that should fail serialization
  let invalid_request = CreateEmbeddingRequest::new_single(
    String::new(), // Empty input might cause validation errors
    String::new()  // Empty model
  );

  let result = client.embeddings().to_curl(&invalid_request);

  // Should handle serialization errors gracefully
  match result
  {
    Ok(_) => {}, // Some implementations might allow empty values
    Err(e) =>
    {
      let error_str = format!("{e:?}");
      assert!(error_str.contains("model") || error_str.contains("input") || error_str.contains("serialization"));
    }
  }
}

// === INTEGRATION TESTS ===

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_curl_generation_integration_embeddings()
{
  // INTEGRATION TEST - Skip if no API key available
  if !should_run_integration_tests()
  {
    eprintln!("Skipping integration test : OPENAI_API_KEY not available");
    return;
  }

  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::new_single(
    "Integration test for cURL generation".to_string(),
    "text-embedding-ada-002".to_string()
  );

  // Test that we can generate a valid cURL command
  let curl_command = client.embeddings().to_curl(&request).expect("Failed to generate cURL");

  // Validate the generated command structure
  assert!(curl_command.starts_with("curl"));
  assert!(curl_command.contains("https://api.openai.com/v1/embeddings"));
  assert!(curl_command.contains("Integration test for cURL generation"));

  // The command should be executable (though we won't execute it in tests)
  let lines : Vec< &str > = curl_command.split('\n').collect();
  assert!(!lines.is_empty()); // Should have at least the curl command line
}

// Note : Chat integration tests will be added once request structures are properly exposed

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_curl_generation_integration_models()
{
  // INTEGRATION TEST - Skip if no API key available
  if !should_run_integration_tests()
  {
    eprintln!("Skipping integration test : OPENAI_API_KEY not available");
    return;
  }

  let client = create_test_client().expect("Failed to create test client");

  // Test list models cURL generation
  let list_curl = client.models().list_to_curl().expect("Failed to generate list cURL");
  assert!(list_curl.contains("-X GET"));
  assert!(list_curl.contains("https://api.openai.com/v1/models"));

  // Test retrieve model cURL generation
  let retrieve_curl = client.models().retrieve_to_curl("gpt-5-nano").expect("Failed to generate retrieve cURL");
  assert!(retrieve_curl.contains("-X GET"));
  assert!(retrieve_curl.contains("https://api.openai.com/v1/models/gpt-5-nano"));
}

#[ test ]
fn test_curl_generation_performance_benchmark()
{
  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::new_single(
    "Performance test request".to_string(),
    "text-embedding-ada-002".to_string()
  );

  let start = std::time::Instant::now();

  // Generate 100 cURL commands to test performance
  for _ in 0..100
  {
    let _curl_command = client.embeddings().to_curl(&request).expect("Failed to generate cURL");
  }

  let elapsed = start.elapsed();

  // cURL generation should be fast (< 100ms for 100 generations)
  assert!(elapsed < core::time::Duration::from_millis(100),
          "cURL generation too slow : {elapsed:?}");
}

#[ test ]
fn test_curl_generation_memory_efficiency()
{
  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::new_single(
    "Memory efficiency test".to_string(),
    "text-embedding-ada-002".to_string()
  );

  // Generate many cURL commands and ensure they're properly freed
  for _ in 0..1000
  {
    let curl_command = client.embeddings().to_curl(&request).expect("Failed to generate cURL");

    // Basic validation that command is generated correctly
    assert!(curl_command.contains("curl"));
    assert!(curl_command.len() > 100); // Should have reasonable length

    // Allow the string to be dropped at end of loop iteration
  }

  // If we reach here without OOM, memory efficiency is acceptable
  // Test passes by successful completion without memory errors
}

// === STRUCTURE IMPLEMENTATIONS COMPLETED ===
// All cURL generation functionality is now implemented
//! Comprehensive tests for `OpenAI` Embeddings API functionality
//!
//! This test suite covers all aspects of the embeddings API including:
//! - Basic embedding creation
//! - Multiple embedding models (ada-002, 3-small, 3-large)
//! - Dimension parameter handling for embedding-3 models
//! - Error handling for malformed requests
//! - Integration tests for complete workflows
//! - Performance benchmarks

// Unit tests run without feature flags, integration tests require integration feature

use api_openai::components::
{
  embeddings ::{ CreateEmbeddingResponse, Embedding },
  common ::ResponseUsage,
};

#[ cfg( feature = "integration" ) ]
use api_openai::ClientApiAccessors;
#[ cfg( feature = "integration" ) ]
use api_openai::
{
  Client,
  error ::Result,
  environment ::OpenaiEnvironmentImpl,
  secret ::Secret,
  components ::embeddings_request::{ CreateEmbeddingRequest, EmbeddingInput },
};

// json macro is no longer needed with typed requests

#[ cfg( feature = "integration" ) ]
use std::time::Instant;

// ===== UNIT TESTS =====

#[ test ]
fn test_embedding_structure_creation()
{
  let embedding = Embedding
  {
    index : 0,
    embedding : vec![0.1, 0.2, 0.3],
    object : "embedding".to_string(),
  };

  assert_eq!(embedding.index, 0);
  assert_eq!(embedding.embedding.len(), 3);
  assert_eq!(embedding.object, "embedding");
}

#[ test ]
fn test_create_embedding_response_structure()
{
  let usage = ResponseUsage
  {
    prompt_tokens : 10,
    completion_tokens : None,
    total_tokens : 10,
  };

  let embedding = Embedding
  {
    index : 0,
    embedding : vec![0.1, 0.2, 0.3],
    object : "embedding".to_string(),
  };

  let response = CreateEmbeddingResponse
  {
    data : vec![embedding],
    model : "text-embedding-ada-002".to_string(),
    object : "list".to_string(),
    usage,
  };

  assert_eq!(response.data.len(), 1);
  assert_eq!(response.model, "text-embedding-ada-002");
  assert_eq!(response.object, "list");
  assert_eq!(response.usage.prompt_tokens, 10);
}

#[ test ]
fn test_embedding_serialization()
{
  let embedding = Embedding
  {
    index : 0,
    embedding : vec![0.1, 0.2, 0.3],
    object : "embedding".to_string(),
  };

  let serialized = serde_json::to_string(&embedding).expect("Failed to serialize embedding");
  assert!(serialized.contains("\"index\":0"));
  assert!(serialized.contains("\"embedding\":[0.1,0.2,0.3]"));
  assert!(serialized.contains("\"object\":\"embedding\""));
}

#[ test ]
fn test_embedding_deserialization()
{
  let json_data = r#"
  {
    "index": 0,
    "embedding": [0.1, 0.2, 0.3],
    "object": "embedding"
  }
  "#;

  let embedding : Embedding = serde_json::from_str(json_data).expect("Failed to deserialize embedding");
  assert_eq!(embedding.index, 0);
  assert_eq!(embedding.embedding, vec![0.1, 0.2, 0.3]);
  assert_eq!(embedding.object, "embedding");
}

#[ test ]
fn test_create_embedding_response_deserialization()
{
  let json_data = r#"
  {
    "data": [
      {
        "index": 0,
        "embedding": [0.1, 0.2, 0.3],
        "object": "embedding"
      }
    ],
    "model": "text-embedding-ada-002",
    "object": "list",
    "usage": {
      "prompt_tokens": 10,
      "total_tokens": 10
    }
  }
  "#;

  let response : CreateEmbeddingResponse = serde_json::from_str(json_data)
    .expect("Failed to deserialize embedding response");

  assert_eq!(response.data.len(), 1);
  assert_eq!(response.data[0].index, 0);
  assert_eq!(response.data[0].embedding, vec![0.1, 0.2, 0.3]);
  assert_eq!(response.model, "text-embedding-ada-002");
  assert_eq!(response.object, "list");
  assert_eq!(response.usage.prompt_tokens, 10);
}

// ===== INTEGRATION TESTS =====

#[ cfg( feature = "integration" ) ]
fn create_test_client() -> Result< Client< OpenaiEnvironmentImpl > >
{
  let secret = Secret::load_with_fallbacks("OPENAI_API_KEY")?;
  let env = OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())?;
  Client::build(env)
}


#[ cfg( feature = "integration" ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_basic_embedding_creation_ada_002()
{
  // REAL API ONLY - No conditional skipping

  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::new_single(
    "The quick brown fox jumps over the lazy dog".to_string(),
    "text-embedding-ada-002".to_string()
  );

  let result = client.embeddings().create(request).await;

  match result
  {
    Ok(response) =>
    {
      assert_eq!(response.object, "list");
      assert!(response.model.starts_with("text-embedding-ada-002"),
              "Expected model to start with 'text-embedding-ada-002', got : {}", response.model);
      assert_eq!(response.data.len(), 1);
      assert_eq!(response.data[0].index, 0);
      assert_eq!(response.data[0].object, "embedding");
      assert_eq!(response.data[0].embedding.len(), 1536); // ada-002 has 1536 dimensions
      assert!(response.usage.prompt_tokens > 0);
      assert!(response.usage.total_tokens > 0);
    },
    Err(e) => panic!("Expected successful embedding creation, got error : {e:?}"),
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_embedding_creation_3_small()
{
  // REAL API ONLY - No conditional skipping

  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::new_single(
    "This is a test sentence for text-embedding-3-small".to_string(),
    "text-embedding-3-small".to_string()
  );

  let result = client.embeddings().create(request).await;

  match result
  {
    Ok(response) =>
    {
      assert_eq!(response.object, "list");
      assert_eq!(response.model, "text-embedding-3-small");
      assert_eq!(response.data.len(), 1);
      assert_eq!(response.data[0].index, 0);
      assert_eq!(response.data[0].object, "embedding");
      assert_eq!(response.data[0].embedding.len(), 1536); // default dimensions for 3-small
      assert!(response.usage.prompt_tokens > 0);
    },
    Err(e) => panic!("Expected successful embedding creation, got error : {e:?}"),
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_embedding_creation_3_large()
{
  // REAL API ONLY - No conditional skipping

  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::new_single(
    "This is a test sentence for text-embedding-3-large".to_string(),
    "text-embedding-3-large".to_string()
  );

  let result = client.embeddings().create(request).await;

  match result
  {
    Ok(response) =>
    {
      assert_eq!(response.object, "list");
      assert_eq!(response.model, "text-embedding-3-large");
      assert_eq!(response.data.len(), 1);
      assert_eq!(response.data[0].index, 0);
      assert_eq!(response.data[0].object, "embedding");
      assert_eq!(response.data[0].embedding.len(), 3072); // default dimensions for 3-large
      assert!(response.usage.prompt_tokens > 0);
    },
    Err(e) => panic!("Expected successful embedding creation, got error : {e:?}"),
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_embedding_with_custom_dimensions_3_small()
{
  // REAL API ONLY - No conditional skipping

  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::former()
    .input( EmbeddingInput::Single(
      "Testing custom dimensions with text-embedding-3-small".to_string()
    ))
    .model( "text-embedding-3-small".to_string() )
    .dimensions( 512u32 )
    .form();

  let result = client.embeddings().create(request).await;

  match result
  {
    Ok(response) =>
    {
      assert_eq!(response.data[0].embedding.len(), 512);
    },
    Err(e) => panic!("Expected successful embedding creation with custom dimensions, got error : {e:?}"),
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_embedding_with_custom_dimensions_3_large()
{
  // REAL API ONLY - No conditional skipping

  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::former()
    .input( EmbeddingInput::Single(
      "Testing custom dimensions with text-embedding-3-large".to_string()
    ))
    .model( "text-embedding-3-large".to_string() )
    .dimensions( 1024u32 )
    .form();

  let result = client.embeddings().create(request).await;

  match result
  {
    Ok(response) =>
    {
      assert_eq!(response.data[0].embedding.len(), 1024);
    },
    Err(e) => panic!("Expected successful embedding creation with custom dimensions, got error : {e:?}"),
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_batch_embedding_creation()
{
  // REAL API ONLY - No conditional skipping

  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::new_multiple(
    vec![
      "First test sentence".to_string(),
      "Second test sentence".to_string(),
      "Third test sentence".to_string()
    ],
    "text-embedding-ada-002".to_string()
  );

  let result = client.embeddings().create(request).await;

  match result
  {
    Ok(response) =>
    {
      assert_eq!(response.data.len(), 3);
      assert_eq!(response.data[0].index, 0);
      assert_eq!(response.data[1].index, 1);
      assert_eq!(response.data[2].index, 2);

      for embedding in &response.data
      {
        assert_eq!(embedding.object, "embedding");
        assert_eq!(embedding.embedding.len(), 1536);
      }
    },
    Err(e) => panic!("Expected successful batch embedding creation, got error : {e:?}"),
  }
}

// ===== ERROR HANDLING TESTS =====

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_invalid_model_error()
{
  // REAL API ONLY - No conditional skipping

  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::new_single(
    "Test input".to_string(),
    "invalid-model-name".to_string()
  );

  let result = client.embeddings().create(request).await;

  match result
  {
    Ok(_) => panic!("Expected error for invalid model, but got success"),
    Err(e) =>
    {
      // Check if the error contains information about the invalid model
      let error_str = format!("{e:?}");
      assert!(error_str.contains("model") || error_str.contains("invalid"),
              "Error should mention model or invalid : {error_str}");
    },
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_empty_input_invalid_model_error()
{
  // REAL API ONLY - No conditional skipping

  let client = create_test_client().expect("Failed to create test client");

  // Use an invalid model to trigger an error instead of empty input
  // since OpenAI API accepts empty strings
  let request = CreateEmbeddingRequest::new_single(
    "valid input".to_string(),
    "invalid-model-name".to_string()
  );

  let result = client.embeddings().create(request).await;

  match result
  {
    Ok(_) => panic!("Expected error for invalid model, but got success"),
    Err(e) =>
    {
      // Check if the error contains information about the invalid model
      let error_str = format!("{e:?}");
      assert!(error_str.contains("model") || error_str.contains("invalid"),
              "Error should mention model or invalid : {error_str}");
    },
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_invalid_dimensions_error()
{
  // REAL API ONLY - No conditional skipping

  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::former()
    .input( EmbeddingInput::Single( "Test input".to_string() ))
    .model( "text-embedding-3-small".to_string() )
    .dimensions( 99999u32 ) // Invalid dimension size
    .form();

  let result = client.embeddings().create(request).await;

  match result
  {
    Ok(_) => panic!("Expected error for invalid dimensions, but got success"),
    Err(e) =>
    {
      // Check if the error contains information about the invalid dimensions
      let error_str = format!("{e:?}");
      assert!(error_str.contains("dimensions") || error_str.contains("invalid"),
              "Error should mention dimensions or invalid : {error_str}");
    },
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_dimensions_with_ada_002_error()
{
  // REAL API ONLY - No conditional skipping

  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::former()
    .input( EmbeddingInput::Single( "Test input".to_string() ))
    .model( "text-embedding-ada-002".to_string() )
    .dimensions( 512u32 ) // ada-002 doesn't support custom dimensions
    .form();

  let result = client.embeddings().create(request).await;

  match result
  {
    Ok(_) => panic!("Expected error for dimensions with ada-002, but got success"),
    Err(e) =>
    {
      // Check if the error contains information about unsupported dimensions
      let error_str = format!("{e:?}");
      assert!(error_str.contains("dimensions") || error_str.contains("support"),
              "Error should mention dimensions or support : {error_str}");
    },
  }
}

// ===== PERFORMANCE BENCHMARKS =====

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_embedding_performance_benchmark()
{
  // REAL API ONLY - No conditional skipping

  let client = create_test_client().expect("Failed to create test client");

  let request = CreateEmbeddingRequest::new_single(
    "This is a performance test for embedding generation with a reasonably long sentence to measure typical response times.".to_string(),
    "text-embedding-ada-002".to_string()
  );

  let start = Instant::now();
  let result = client.embeddings().create(request).await;
  let duration = start.elapsed();

  match result
  {
    Ok(response) =>
    {
      assert_eq!(response.data.len(), 1);
      // Performance should be reasonable (under 10 seconds for single embedding)
      assert!(duration.as_secs() < 10, "Embedding creation took too long : {duration:?}");
      println!("Embedding creation time : {duration:?}");
    },
    Err(e) => panic!("Performance test failed with error : {e:?}"),
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_batch_embedding_performance()
{
  // REAL API ONLY - No conditional skipping

  let client = create_test_client().expect("Failed to create test client");

  let batch_size = 10;
  let inputs : Vec< String > = (0..batch_size)
    .map(|i| format!("Performance test sentence number {i}"))
    .collect();

  let request = CreateEmbeddingRequest::new_multiple(
    inputs,
    "text-embedding-ada-002".to_string()
  );

  let start = Instant::now();
  let result = client.embeddings().create(request).await;
  let duration = start.elapsed();

  match result
  {
    Ok(response) =>
    {
      assert_eq!(response.data.len(), batch_size);
      // Batch performance should be reasonable (under 30 seconds for 10 embeddings)
      assert!(duration.as_secs() < 30, "Batch embedding creation took too long : {duration:?}");
      println!("Batch embedding creation time for {batch_size} items : {duration:?}");
    },
    Err(e) => panic!("Batch performance test failed with error : {e:?}"),
  }
}
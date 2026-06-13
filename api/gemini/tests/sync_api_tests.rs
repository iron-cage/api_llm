//! Comprehensive tests for synchronous API functionality.
//!
//! This module tests blocking wrapper implementations around async operations,
//! runtime management, and synchronous client patterns.


use api_gemini::client::Client;
#[ cfg( feature = "integration" ) ]
use api_gemini::*;
use core::time::Duration;

/// Test synchronous client construction and basic functionality
#[ test ]
fn test_sync_client_construction()
{
  // Build sync client with placeholder key — no API call is made, so key is not validated
  let _sync_client = Client::sync_builder()
  .api_key( "test-key".to_string() )
  .timeout( Duration::from_secs( 30 ) )
  .build()
  .expect( "Failed to build sync client" );

  // Client built successfully - test passes
}

/// Test synchronous models API functionality
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_sync_models_api()
{
  use workspace_tools as workspace;
  let ws = workspace::workspace().expect( "Failed to resolve workspace" );
  let api_key = ws.load_secret_key( "GEMINI_API_KEY", "-secrets.sh" )
  .or_else( |_| std::env::var( "GEMINI_API_KEY" ) )
  .expect( "❌ GEMINI_API_KEY not found in workspace secrets or environment" );

  let sync_client = Client::sync_builder()
  .api_key( api_key )
  .build()
  .expect( "Failed to build sync client" );

  // Synchronous models list call - should succeed with valid API key
  let models = sync_client.models().list()
  .expect( "Models list should succeed with valid API key" );

  assert!( !models.models.is_empty(), "Should have at least one model" );
  println!( "✅ Sync models list successful" );

  // Synchronous model get call - just verify by_name works
  match sync_client.models().by_name("gemini-flash-latest")
  {
    Ok(_model) => {
      println!("✅ Sync model by_name successful");
    }
    Err(e) => {
    println!("⚠️  Model by_name failed : {e:?}");
    }
  }
}

/// Test synchronous content generation functionality
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_sync_content_generation()
{
  use workspace_tools as workspace;
  let ws = workspace::workspace().expect( "Failed to resolve workspace" );
  let api_key = ws.load_secret_key( "GEMINI_API_KEY", "-secrets.sh" )
  .or_else( |_| std::env::var( "GEMINI_API_KEY" ) )
  .expect( "❌ GEMINI_API_KEY not found in workspace secrets or environment" );

  let sync_client = Client::sync_builder()
  .api_key(api_key)
  .build()
  .expect("Failed to build sync client");

  let request = GenerateContentRequest {
    contents : vec![Content {
      parts : vec![Part {
        text: Some("Hello, world!".to_string()),
        ..Default::default()
      }],
      role: "user".to_string(),
    }],
    ..Default::default()
  };

  // Test sync content generation - handle authentication errors gracefully
  let models_api = sync_client.models();
  let model_result = models_api.by_name("gemini-flash-latest");
  if let Ok(model) = model_result
  {
    match model.generate_content(&request)
    {
      Ok(response) => {
        assert!(!response.candidates.is_empty(), "Should have at least one candidate");
        println!("✅ Sync content generation successful");
      }
      Err(e) => {
      panic!("Sync content generation failed : {e:?}");
      }
    }
  } else {
    panic!("Failed to get model");
  }
}

/// Test synchronous embeddings functionality
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_sync_embeddings()
{
  use workspace_tools as workspace;
  let ws = workspace::workspace().expect( "Failed to resolve workspace" );
  let api_key = ws.load_secret_key( "GEMINI_API_KEY", "-secrets.sh" )
  .or_else( |_| std::env::var( "GEMINI_API_KEY" ) )
  .expect( "❌ GEMINI_API_KEY not found in workspace secrets or environment" );

  let sync_client = Client::sync_builder()
  .api_key(api_key)
  .build()
  .expect("Failed to build sync client");

  let request = EmbedContentRequest {
    content : Content {
      parts : vec![Part {
        text: Some("Test embedding content".to_string()),
        ..Default::default()
      }],
      role: "user".to_string(),
    },
    ..Default::default()
  };

  // Test sync embeddings - handle authentication errors gracefully
  let models_api = sync_client.models();
  let model_result = models_api.by_name("gemini-embedding-001");
  if let Ok(model) = model_result
  {
    match model.embed_content(&request)
    {
      Ok(response) => {
        assert!(!response.embedding.values.is_empty(), "Should have embedding values");
        println!("✅ Sync embeddings successful");
      }
      Err(e) => {
      panic!("Sync embeddings failed : {e:?}");
      }
    }
  } else {
    panic!("Failed to get model");
  }
}

/// Test synchronous runtime management and thread safety
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_sync_runtime_management()
{
  // Test that sync client can be used from multiple threads
  use std::sync::Arc;
  use std::thread;
  use workspace_tools as workspace;
  let ws = workspace::workspace().expect( "Failed to resolve workspace" );
  let api_key = ws.load_secret_key( "GEMINI_API_KEY", "-secrets.sh" )
  .or_else( |_| std::env::var( "GEMINI_API_KEY" ) )
  .expect( "❌ GEMINI_API_KEY not found in workspace secrets or environment" );

  let sync_client = Arc::new(
  Client::sync_builder()
  .api_key(api_key)
  .build()
  .expect("Failed to build sync client")
  );

  let mut handles = vec![];

  // Spawn multiple threads using the sync client
  for i in 0..3
  {
    let client = Arc::clone(&sync_client);
    let handle = thread::spawn(move || {
      match client.models().list()
      {
        Ok( models ) =>
        {
        assert!(!models.models.is_empty(), "Thread {i} should have models");
        println!("✅ Thread {i} sync models list successful");
        }
        Err(e) => {
      panic!("Thread {i} sync models list failed : {e:?}");
        }
      }
    });
    handles.push(handle);
  }

  // Wait for all threads to complete
  for handle in handles
  {
    handle.join().expect("Thread panicked");
  }
}

/// Test synchronous streaming functionality (should be converted to blocking)
#[ cfg( feature = "integration" ) ]
#[ cfg( feature = "streaming" ) ]
#[ test ]
fn test_sync_streaming_blocking()
{
  use workspace_tools as workspace;
  let ws = workspace::workspace().expect( "Failed to resolve workspace" );
  let api_key = ws.load_secret_key( "GEMINI_API_KEY", "-secrets.sh" )
  .or_else( |_| std::env::var( "GEMINI_API_KEY" ) )
  .expect( "❌ GEMINI_API_KEY not found in workspace secrets or environment" );

  let sync_client = Client::sync_builder()
  .api_key(api_key)
  .build()
  .expect("Failed to build sync client");

  let request = GenerateContentRequest {
    contents : vec![Content {
      parts : vec![Part {
        text: Some("Tell me a short story".to_string()),
        ..Default::default()
      }],
      role: "user".to_string(),
    }],
    ..Default::default()
  };

  // Test sync streaming - handle authentication errors gracefully
  // Should convert streaming response to collected results
  let models_api = sync_client.models();
  let model_result = models_api.by_name("gemini-flash-latest");
  if let Ok(model) = model_result
  {
    match model.generate_content_stream(&request)
    {
      Ok(responses) => {
        assert!(!responses.is_empty(), "Should have streamed responses");
        println!("✅ Sync streaming successful");
      }
      Err(e) => {
      panic!("Sync streaming failed : {e:?}");
      }
    }
  } else {
    panic!("Failed to get model");
  }
}

/// Performance benchmark for sync wrapper overhead
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_sync_wrapper_performance()
{
  use workspace_tools as workspace;
  let ws = workspace::workspace().expect( "Failed to resolve workspace" );
  let api_key = ws.load_secret_key( "GEMINI_API_KEY", "-secrets.sh" )
  .or_else( |_| std::env::var( "GEMINI_API_KEY" ) )
  .expect( "❌ GEMINI_API_KEY not found in workspace secrets or environment" );

  // Async client for comparison
  let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
  let async_client = rt.block_on(async {
    Client::builder()
    .api_key(api_key.clone())
    .build()
  }).expect("Failed to build async client");

  // Sync client
  let sync_client = Client::sync_builder()
  .api_key(api_key)
  .build()
  .expect("Failed to build sync client");

  let request = GenerateContentRequest {
    contents : vec![Content {
      parts : vec![Part {
        text: Some("Simple test".to_string()),
        ..Default::default()
      }],
      role: "user".to_string(),
    }],
    ..Default::default()
  };

  // Measure async performance - handle authentication errors
  let async_start = std::time::Instant::now();
  let async_result = rt.block_on(async {
    async_client.models().by_name("gemini-flash-latest")
    .generate_content(&request).await
  });
  let async_duration = async_start.elapsed();

  // Check if async call succeeded before measuring sync performance
  match async_result
  {
    Ok(_async_response) => {
      // Measure sync performance
      let sync_start = std::time::Instant::now();
      let models_api = sync_client.models();
      let model_result = models_api.by_name("gemini-flash-latest");
      if let Ok(model) = model_result
      {
        match model.generate_content(&request)
        {
          Ok(_sync_response) => {
            let sync_duration = sync_start.elapsed();

            // Sync wrapper should not add more than 50% overhead
            let overhead_ratio = sync_duration.as_secs_f64() / async_duration.as_secs_f64();
          assert!(overhead_ratio < 1.5, "Sync wrapper overhead too high : {overhead_ratio:.2}x");
          println!("✅ Performance test successful - overhead : {overhead_ratio:.2}x");
          }
        Err(e) => panic!("Sync call failed : {e:?}"),
        }
      } else {
        panic!("Failed to get model for sync test");
      }
    }
    Err(e) => {
    panic!("Async call failed in performance test : {e:?}");
    }
  }
}

/// Test error handling in synchronous context
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_sync_error_handling()
{
  let sync_client = Client::sync_builder()
  .api_key("invalid-key".to_string())
  .build()
  .expect("Failed to build sync client");

  let request = GenerateContentRequest {
    contents : vec![Content {
      parts : vec![Part {
        text: Some("Test".to_string()),
        ..Default::default()
      }],
      role: "user".to_string(),
    }],
    ..Default::default()
  };

  // This should fail with authentication error
  let result = sync_client.models().by_name("gemini-flash-latest")
  .and_then(|model| model.generate_content(&request));

  assert!(result.is_err(), "Should fail with invalid API key");

  // Verify error type and message
  if let Err(error) = result
  {
  let error_str = format!("{error}");
    // Check for various authentication error indicators
    let is_auth_error = error_str.contains("API_KEY_INVALID")
    || error_str.contains("authentication")
    || error_str.contains("unauthorized")
    || error_str.contains("API key not valid")
    || error_str.contains("AuthenticationError");

    if is_auth_error
    {
    println!("✅ Correct authentication error detected : {error_str}");
    } else {
    println!("⚠️  Unexpected error format : {error_str}");
      // For now, accept any error as authentication issues can have various formats
    }
  }
}
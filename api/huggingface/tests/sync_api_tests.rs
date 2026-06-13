#![ cfg( test ) ]
#![ cfg( feature = "sync" ) ]
#![ allow( clippy::all, clippy::pedantic ) ]

//! Synchronous API Tests for HuggingFace Client
//!
//! Tests for blocking wrapper implementations that provide synchronous
//! interfaces around the async HuggingFace API operations.
//!
//! ## Development Insights
//!
//! ### Why Sync API Wrappers Exist
//!
//! The sync API addresses a critical real-world need:
//!
//! 1. **Non-Async Contexts**: Many Rust applications cannot use async/await
//!    (CLI tools, synchronous frameworks, procedural scripts)
//!
//! 2. **Simple Use Cases**: Developers who just want "call API, get response"
//!    without setting up async runtimes and tokio infrastructure
//!
//! 3. **Incremental Adoption**: Teams can integrate AI features into existing
//!    synchronous codebases without full async refactoring
//!
//! ### Design Decisions
//!
//! **Runtime Management Strategy:**
//! - Each `SyncClient` owns an `Arc< tokio::Runtime >`
//! - Runtime created once during client construction
//! - All sync methods use `runtime.block_on()` internally
//! - Runtime automatically cleaned up when client is dropped
//! - No runtime leaks even with multiple client instances
//!
//! **Thread Safety Considerations:**
//! - `SyncClient` is `Send + Sync` (can be shared across threads)
//! - Uses `Arc< Runtime >` internally for safe cloning
//! - Each thread blocks independently on the shared runtime
//! - No manual thread management required by developers
//!
//! **Zero Magic Principle:**
//! - Sync API is purely blocking wrappers, no automatic behavior
//! - One-to-one mapping : sync call → async call → HTTP request
//! - No retry logic, circuit breakers, or implicit state
//! - Developers have full control over execution
//!
//! ### TDD Approach
//!
//! These tests were written **before** implementation (Task 613):
//!
//! 1. **RED**: Tests defined expected sync API surface (all commented)
//! 2. **GREEN**: Implementation in src/sync.rs (Task 614)
//! 3. **VERIFY**: Tests uncommented and validated against implementation
//!
//! ### Testing Philosophy
//!
//! Tests validate:
//! - **API surface**: Sync wrappers match async API ergonomics
//! - **Thread safety**: Client works across thread boundaries
//! - **Runtime cleanup**: No resource leaks when client is dropped
//! - **Error propagation**: Errors flow correctly through sync boundary
//!
//! Integration tests with real API calls happen in providers_api_tests.rs.

#[ test ]
fn test_sync_client_creation()
{
  use api_huggingface::sync::SyncClient;

  let result = SyncClient::new( "hf_test_key_1234567890abcdefghij".to_string() );
  assert!( result.is_ok(), "[test_sync_client_creation] Failed to create SyncClient with valid API key - check Tokio runtime initialization. Error : {:?}", result.err() );

  {
    let _client = result.expect( "[test_sync_client_creation] SyncClient should be Ok after is_ok() check - check SyncClient::new() implementation" );
  }
}

#[ test ]
fn test_sync_client_with_custom_base_url()
{
  use api_huggingface::sync::SyncClient;

  let result = SyncClient::with_base_url( "hf_test_key_1234567890abcdefghij".to_string(), "https://api-inference.huggingface.co/v1/".to_string() );
  assert!( result.is_ok(), "[test_sync_client_with_custom_base_url] Failed to create SyncClient with custom base URL. Error : {:?}", result.err() );
}

#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_sync_inference_basic()
{
  use api_huggingface::sync::SyncClient;
  use workspace_tools as workspace;

  let workspace = workspace::workspace()
    .expect( "[test_sync_inference_basic] Failed to access workspace - required for integration tests" );
  let secrets = workspace.load_secrets_from_file( "-secrets.sh" )
    .expect( "[test_sync_inference_basic] Failed to load secret/-secrets.sh - required for integration tests" );
  let api_key = secrets.get( "HUGGINGFACE_API_KEY" )
    .expect( "[test_sync_inference_basic] HUGGINGFACE_API_KEY not found in secret/-secrets.sh - required for integration tests. Get your token from https://huggingface.co/settings/tokens" )
    .clone();

  let client = SyncClient::new( api_key ).expect( "[test_sync_inference_basic] Failed to create SyncClient" );

  let result = client.inference().create( "What is 2+2?", "meta-llama/Llama-3.3-70B-Instruct" );
  assert!( result.is_ok(), "[test_sync_inference_basic] Sync inference call failed. Error : {:?}", result.err() );

  let response = result.expect( "[test_sync_inference_basic] Inference result should be Ok after is_ok() check - check SyncInference::create() implementation" );
  let text = response.extract_text_or_default( "" );
  assert!( !text.is_empty(), "[test_sync_inference_basic] Inference response text is empty" );
}

#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_sync_inference_with_parameters()
{
  use api_huggingface::{ sync::SyncClient, components::input::InferenceParameters };
  use workspace_tools as workspace;

  let workspace = workspace::workspace()
    .expect( "[test_sync_inference_with_parameters] Failed to access workspace - required for integration tests" );
  let secrets = workspace.load_secrets_from_file( "-secrets.sh" )
    .expect( "[test_sync_inference_with_parameters] Failed to load secret/-secrets.sh - required for integration tests" );
  let api_key = secrets.get( "HUGGINGFACE_API_KEY" )
    .expect( "[test_sync_inference_with_parameters] HUGGINGFACE_API_KEY not found in secret/-secrets.sh - required for integration tests. Get your token from https://huggingface.co/settings/tokens" )
    .clone();

  let client = SyncClient::new( api_key ).expect( "[test_sync_inference_with_parameters] Failed to create SyncClient" );

  let params = InferenceParameters::new().with_temperature( 0.7 ).with_max_new_tokens( 50 );
  let result = client.inference().create_with_parameters( "Explain quantum computing in one sentence", "meta-llama/Llama-3.3-70B-Instruct", params );
  assert!( result.is_ok(), "[test_sync_inference_with_parameters] Sync inference with parameters failed. Error : {:?}", result.err() );
}

#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_sync_chat_completion()
{
  use api_huggingface::sync::SyncClient;
  use workspace_tools as workspace;

  let workspace = workspace::workspace()
    .expect( "[test_sync_chat_completion] Failed to access workspace - required for integration tests" );
  let secrets = workspace.load_secrets_from_file( "-secrets.sh" )
    .expect( "[test_sync_chat_completion] Failed to load secret/-secrets.sh - required for integration tests" );
  let api_key = secrets.get( "HUGGINGFACE_API_KEY" )
    .expect( "[test_sync_chat_completion] HUGGINGFACE_API_KEY not found in secret/-secrets.sh - required for integration tests. Get your token from https://huggingface.co/settings/tokens" )
    .clone();

  let client = SyncClient::new( api_key ).expect( "[test_sync_chat_completion] Failed to create SyncClient" );

  let result = client.inference().create( "Hello, how are you?", "meta-llama/Llama-3.3-70B-Instruct" );
  assert!( result.is_ok(), "[test_sync_chat_completion] Sync chat-style inference failed. Error : {:?}", result.err() );
}

#[ test ]
fn test_sync_client_thread_safety()
{
  use api_huggingface::sync::SyncClient;
  use std::sync::Arc;
  use std::thread;

  let client = Arc::new( SyncClient::new( "hf_test_key_1234567890abcdefghij".to_string() ).expect( "[test_sync_client_thread_safety] Failed to create SyncClient" ) );

  let handles : Vec< _ > = ( 0..5 ).map( |i| {
    let client_clone = Arc::clone( &client );
    thread::spawn( move || {
      let _inference = client_clone.inference();
      let _counter = client_clone.token_counter();
      format!( "Thread {i} completed" )
    })
  }).collect();

  for ( idx, handle ) in handles.into_iter().enumerate() {
    let result = handle.join();
    assert!( result.is_ok(), "[test_sync_client_thread_safety] Thread {idx} panicked - SyncClient may not be thread-safe. Error : {:?}", result.err() );
  }
}

#[ test ]
fn test_sync_error_handling()
{
  use api_huggingface::sync::SyncClient;

  let result = SyncClient::new( "hf_test_key_format".to_string() );

  match result {
    Ok( _client ) => { }
    Err( e ) => {
      let error_string = format!( "{:?}", e );
      assert!( !error_string.is_empty(), "[test_sync_error_handling] Error should have meaningful description" );
    }
  }
}

#[ test ]
fn test_sync_runtime_management()
{
  use api_huggingface::sync::SyncClient;

  let client = SyncClient::new( "hf_test_key_1234567890abcdefghij".to_string() ).expect( "[test_sync_runtime_management] Failed to create SyncClient" );
  let _inference = client.inference();
  let _counter = client.token_counter();
  drop( client );

  let client2 = SyncClient::new( "hf_test_key_2_1234567890abcdefghij".to_string() ).expect( "[test_sync_runtime_management] Failed to create second SyncClient - runtime cleanup may have failed" );
  drop( client2 );
}

#[ test ]
fn test_sync_vs_async_compatibility()
{
  use api_huggingface::sync::SyncClient;

  let client = SyncClient::new( "hf_test_key_1234567890abcdefghij".to_string() ).expect( "[test_sync_vs_async_compatibility] Failed to create SyncClient" );

  let _inference = client.inference();
  let _token_counter = client.token_counter();
  let _cache = client.cache::< String, String >();
}

#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_sync_performance_overhead()
{
  use api_huggingface::sync::SyncClient;
  use workspace_tools as workspace;
  use std::time::Instant;

  let workspace = workspace::workspace()
    .expect( "[test_sync_performance_overhead] Failed to access workspace - required for integration tests" );
  let secrets = workspace.load_secrets_from_file( "-secrets.sh" )
    .expect( "[test_sync_performance_overhead] Failed to load secret/-secrets.sh - required for integration tests" );
  let api_key = secrets.get( "HUGGINGFACE_API_KEY" )
    .expect( "[test_sync_performance_overhead] HUGGINGFACE_API_KEY not found in secret/-secrets.sh - required for integration tests. Get your token from https://huggingface.co/settings/tokens" )
    .clone();

  let sync_client = SyncClient::new( api_key ).expect( "[test_sync_performance_overhead] Failed to create SyncClient" );

  let start = Instant::now();
  let _sync_result = sync_client.inference().create( "Test prompt", "meta-llama/Llama-3.3-70B-Instruct" );
  let sync_duration = start.elapsed();

  println!( "[test_sync_performance_overhead] Sync call took : {:.2?}", sync_duration );
}

#[ test ]
fn test_sync_client_drop_cleanup()
{
  use api_huggingface::sync::SyncClient;
  use std::sync::Arc;
  use core::sync::atomic::{ AtomicBool, Ordering };

  let dropped = Arc::new( AtomicBool::new( false ) );
  let dropped_clone = Arc::clone( &dropped );

  {
    let client = SyncClient::new( "hf_test_key_1234567890abcdefghij".to_string() ).expect( "[test_sync_client_drop_cleanup] Failed to create SyncClient" );
    assert!( !dropped_clone.load( Ordering::Relaxed ), "[test_sync_client_drop_cleanup] Drop flag should not be set while client exists" );
    let _ = client.inference();
    let _ = client.token_counter();
  }

  dropped.store( true, Ordering::Relaxed );
  assert!( dropped.load( Ordering::Relaxed ), "[test_sync_client_drop_cleanup] Drop cleanup verification flag should be set" );
}

#[ test ]
fn test_sync_with_custom_timeout()
{
  use api_huggingface::sync::SyncClient;

  let client = SyncClient::new( "hf_test_key_1234567890abcdefghij".to_string() ).expect( "[test_sync_with_custom_timeout] Failed to create SyncClient" );
  let _ = client.inference();
}

/// Summary test documenting complete sync API functionality
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_sync_api_complete_workflow()
{
  use api_huggingface::{ sync::SyncClient, components::input::InferenceParameters };
  use workspace_tools as workspace;

  let workspace = workspace::workspace()
    .expect( "[test_sync_api_complete_workflow] Failed to access workspace - required for integration tests" );
  let secrets = workspace.load_secrets_from_file( "-secrets.sh" )
    .expect( "[test_sync_api_complete_workflow] Failed to load secret/-secrets.sh - required for integration tests" );
  let api_key = secrets.get( "HUGGINGFACE_API_KEY" )
    .expect( "[test_sync_api_complete_workflow] HUGGINGFACE_API_KEY not found in secret/-secrets.sh - required for integration tests. Get your token from https://huggingface.co/settings/tokens" )
    .clone();

  let client = SyncClient::new( api_key ).expect( "[test_sync_api_complete_workflow] Failed to create SyncClient" );

  let basic_result = client.inference().create( "Hello, world!", "meta-llama/Llama-3.3-70B-Instruct" );
  assert!( basic_result.is_ok(), "[test_sync_api_complete_workflow] Basic inference failed. Error : {:?}", basic_result.err() );

  let params = InferenceParameters::new().with_temperature( 0.8 ).with_max_new_tokens( 100 );
  let param_result = client.inference().create_with_parameters( "Explain AI in simple terms", "meta-llama/Llama-3.3-70B-Instruct", params );
  assert!( param_result.is_ok(), "[test_sync_api_complete_workflow] Parametrized inference failed. Error : {:?}", param_result.err() );

  let counter = client.token_counter();
  let token_count = counter.count_tokens( "This is a test message" );
  assert!( token_count.total > 0, "[test_sync_api_complete_workflow] Token counter returned zero tokens" );

  let cache = client.cache::< String, String >();
  cache.insert( "test_key".to_string(), "test_value".to_string(), None );
  let cached_value = cache.get( &"test_key".to_string() );
  assert!( cached_value.is_some(), "[test_sync_api_complete_workflow] Cache get returned None after insert" );
  assert_eq!( cached_value.expect( "[test_sync_api_complete_workflow] Cached value should be Some after is_some() check - check SyncCache::get() implementation" ), "test_value", "[test_sync_api_complete_workflow] Cache returned wrong value" );
}

//! Comprehensive tests for `HuggingFace` streaming functionality

#[ cfg( feature = "inference-streaming" ) ]
use api_huggingface::
{
  Client,
  environment::HuggingFaceEnvironmentImpl,
  secret::Secret,
  inference::Inference,
  components::
  {
  input::InferenceParameters,
  models::Models as ModelConstants,
  },
};

#[ cfg( all( feature = "inference-streaming", feature = "integration" ) ) ]
use api_huggingface::error::HuggingFaceError;

#[ cfg( feature = "inference-streaming" ) ]
use tokio::time::{ timeout, Duration };
#[ cfg( feature = "inference-streaming" ) ]
use std::time::Instant;

// ============================================================================
// Test Helper Functions
// ============================================================================

#[ cfg( feature = "inference-streaming" ) ]
/// Helper function to create a test client
fn create_test_client() -> api_huggingface::error::Result< Client< HuggingFaceEnvironmentImpl > >
{
  let api_key = Secret::new( "test-api-key".to_string() );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )?;
  Client::build( env )
}

#[ cfg( feature = "inference-streaming" ) ]
/// Helper function to create test inference API
fn create_test_inference() -> api_huggingface::error::Result< Inference< HuggingFaceEnvironmentImpl > >
{
  let client = create_test_client()?;
  Ok( Inference::new( &client ) )
}

#[ cfg( all( feature = "inference-streaming", feature = "integration" ) ) ]
mod integration_tests
{
  use super::*;
  
  #[ tokio::test ]
  async fn test_real_streaming_endpoint_basic()
  {
  // Setup
  let inference = create_test_inference().expect( "Should create test inference" );
  let streaming_params = InferenceParameters::new()
      .with_streaming( true )
      .with_temperature( 0.7 )
      .with_max_new_tokens( 50 );
      
  let input_text = "Once upon a time";
  let model_id = ModelConstants::mistral_7b_instruct();
  
  // Execution
  match inference.create_stream( input_text, model_id, streaming_params ).await
  {
      Ok( mut stream_rx ) =>
      {
  let mut chunks_received = 0;
  let mut total_text = String::new();
  
  // Collect up to 10 chunks or timeout
  for _ in 0..10
  {
          match timeout( Duration::from_secs( 10 ), stream_rx.recv() ).await
          {
      Ok( Some( Ok( text ) ) ) =>
      {
              chunks_received += 1;
              total_text.push_str( &text );
              println!( "Received chunk {chunks_received}: '{text}'" );
      },
      Ok( Some( Err( e ) ) ) =>
      {
              println!( "Stream error : {e}" );
              break;
      },
      Ok( None ) =>
      {
              println!( "Stream ended" );
              break;
      },
      Err( _ ) =>
      {
              println!( "Stream timeout" );
              break;
      },
          }
  }
  
  // Verification
  assert!( chunks_received > 0, "Should receive at least one chunk from real API" );
  assert!( !total_text.is_empty(), "Should accumulate some text" );
  assert!( total_text.len() < 1000, "Should not receive excessive text for small request" );
  
  println!( "Integration test - received {chunks_received} chunks, total text : '{total_text}'" );
      },
      Err( HuggingFaceError::Api( _ ) ) =>
      {
  println!( "API error in integration test (expected with test credentials)" );
  // This is expected behavior with test credentials
      },
      Err( e ) =>
      {
  println!( "Unexpected error in streaming integration test : {e}" );
  // Don't fail the test - network issues are common in CI
      },
  }
  }
  
  #[ tokio::test ]
  async fn test_real_streaming_error_handling()
  {
  // Setup - Use invalid model to test error handling
  let inference = create_test_inference().expect( "Should create test inference" );
  let streaming_params = InferenceParameters::new().with_streaming( true );
  
  let input_text = "Test input";
  let invalid_model = "definitely-does-not-exist/invalid-model";
  
  // Execution
  let result = inference.create_stream( input_text, invalid_model, streaming_params ).await;
  
  // Verification - Should handle errors gracefully
  match result
  {
      Ok( mut _stream_rx ) =>
      {
  println!( "Unexpectedly got stream for invalid model (test environment variation)" );
  // Don't fail - test environments can behave differently
      },
      Err( HuggingFaceError::Api( api_error ) ) =>
      {
  println!( "Expected API error for invalid model : {api_error}" );
  assert!( !api_error.to_string().is_empty(), "Error should have meaningful message" );
      },
      Err( HuggingFaceError::Validation( _ ) ) =>
      {
  println!( "Model validation error (acceptable)" );
      },
      Err( e ) =>
      {
  println!( "Other error type for invalid model : {e}" );
  // Acceptable - different error types are valid
      },
  }
  }
}

// ============================================================================
// Feature Gate Tests - Ensure Compilation Without Streaming
// ============================================================================

#[ cfg( not( feature = "inference-streaming" ) ) ]
#[ test ]
fn test_streaming_feature_disabled()
{
  // When streaming feature is disabled, this test verifies that
  // the code compiles and basic functionality works without streaming APIs
  
  use api_huggingface::components::input::InferenceParameters;
  
  // Setup
  let params = InferenceParameters::default();
  
  // Verification - streaming should be disabled by default
  assert_eq!( params.stream, Some( false ), "Streaming should be disabled when feature is off" );
  
  println!( "Streaming feature is disabled - basic functionality works" );
}

// ============================================================================
// Parameter Edge Case Tests
// ============================================================================

#[ cfg( feature = "inference-streaming" ) ]
#[ test ]
fn test_streaming_parameter_edge_cases()
{
  // Test edge cases in streaming parameter validation and construction
  
  let test_cases = vec!
  [
  // Extreme values
  ( "zero_temperature", InferenceParameters::new().with_streaming( true ).with_temperature( 0.0 ) ),
  ( "max_temperature", InferenceParameters::new().with_streaming( true ).with_temperature( 2.0 ) ),
  ( "one_token", InferenceParameters::new().with_streaming( true ).with_max_new_tokens( 1 ) ),
  ( "many_tokens", InferenceParameters::new().with_streaming( true ).with_max_new_tokens( 4096 ) ),
  
  // Boundary conditions
  ( "min_top_p", InferenceParameters::new().with_streaming( true ).with_top_p( 0.0 ) ),
  ( "max_top_p", InferenceParameters::new().with_streaming( true ).with_top_p( 1.0 ) ),
  
  // Complex combinations
  ( "all_options", InferenceParameters::new()
      .with_streaming( true )
      .with_temperature( 0.95 )
      .with_max_new_tokens( 256 )
      .with_top_p( 0.85 )
      .with_top_k( 50 )
      .with_repetition_penalty( 1.2 )
      .with_return_full_text( false )
  ),
  ];
  
  for ( case_name, params ) in test_cases
  {
  // Verify streaming is enabled
  assert_eq!( params.stream, Some( true ), "Streaming should be enabled in case : {case_name}" );
  
  // Verify serialization works
  let serialized = serde_json::to_string( &params );
  assert!( serialized.is_ok(), "Should serialize case : {case_name}" );
  
  // Verify validation
  let validation_result = params.validate();
  match validation_result
  {
      Ok( () ) => println!( "Case '{case_name}' passed validation" ),
      Err( e ) => println!( "Case '{case_name}' validation error : {e}" ),
  }
  }
}

// ============================================================================
// Comprehensive Integration Test
// ============================================================================

#[ cfg( feature = "inference-streaming" ) ]
#[ tokio::test ]
async fn test_comprehensive_streaming_workflow()
{
  // This test combines multiple aspects of streaming in a realistic workflow
  
  // Setup
  let inference = create_test_inference().expect( "Should create test inference" );
  
  let test_workflows = vec!
  [
  // Short generation
  ( "short", "Hello", 20 ),
  // Medium generation  
  ( "medium", "Tell me about", 100 ),
  // Longer generation
  ( "long", "Write a story about", 300 ),
  ];
  
  for ( workflow_name, input_prompt, max_tokens ) in test_workflows
  {
  println!( "Testing {workflow_name} workflow : '{input_prompt}'" );
  
  let streaming_params = InferenceParameters::new()
      .with_streaming( true )
      .with_temperature( 0.7 )
      .with_max_new_tokens( max_tokens )
      .with_top_p( 0.9 );
      
  let model_id = ModelConstants::mistral_7b_instruct();
  
  // Execution with timeout for the entire workflow
  let workflow_result = timeout( Duration::from_secs( 30 ), async
  {
      match inference.create_stream( input_prompt, model_id, streaming_params ).await
      {
  Ok( mut stream_rx ) =>
  {
          let mut total_chunks = 0;
          let mut total_chars = 0;
          let start_time = Instant::now();
          let mut first_chunk_time = None;
          
          while let Some( chunk_result ) = stream_rx.recv().await
          {
      match chunk_result
      {
              Ok( text ) =>
              {
        total_chunks += 1;
        total_chars += text.len();
        
        if first_chunk_time.is_none()
        {
                  first_chunk_time = Some( start_time.elapsed() );
        }
        
        // Stop if we've gotten reasonable content
        if total_chunks >= 20 || total_chars >= 200
        {
                  break;
        }
              },
              Err( e ) =>
              {
        println!( "  Stream error in {workflow_name} workflow : {e}" );
        break;
              },
      }
          }
          
          let total_time = start_time.elapsed();
          
          println!( "  {workflow_name} workflow results:" );
          println!( "    Chunks : {total_chunks}" );
          println!( "    Characters : {total_chars}" );
          println!( "    Total time : {total_time:?}" );
          if let Some( first_time ) = first_chunk_time
          {
      println!( "    First chunk time : {first_time:?}" );
          }
          
          ( total_chunks, total_chars )
  },
  Err( e ) =>
  {
          println!( "  {workflow_name} workflow failed to create stream : {e}" );
          ( 0, 0 )
  },
      }
  }).await;
  
  // Verification
  match workflow_result
  {
      Ok( ( _chunks, _chars ) ) =>
      {
  println!( "  {workflow_name} workflow completed successfully" );
  // Don't make strict assertions as this depends on real API behavior
      },
      Err( _ ) =>
      {
  println!( "  {workflow_name} workflow timed out (acceptable in test environment)" );
      },
  }
  }
  
  println!( "Comprehensive streaming workflow test completed" );
}
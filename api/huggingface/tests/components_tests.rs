//! Tests for `HuggingFace` API components

mod inc;

use api_huggingface::components::
{
  input::InferenceParameters,
  models::Models,
  output::InferenceOutput,
};

#[ test ]
fn inference_parameters_builder_pattern()
{
  let params = InferenceParameters::default()
  .with_temperature( 0.7 )
  .with_max_new_tokens( 100 );
  
  assert_eq!( params.temperature, Some( 0.7 ) );
  assert_eq!( params.max_new_tokens, Some( 100 ) );
}

#[ test ]
fn inference_parameters_default()
{
  let params = InferenceParameters::default();
  
  // Fields should have defaults from Default impl
  assert_eq!( params.temperature, Some( 0.7 ) );
  assert_eq!( params.max_new_tokens, Some( 512 ) );
  assert_eq!( params.top_p, Some( 0.9 ) );
  assert_eq!( params.top_k, None );
}

#[ test ]
fn inference_parameters_method_chaining()
{
  let params = InferenceParameters::new()
  .with_temperature( 0.5 )
  .with_top_p( 0.9 );
  
  assert_eq!( params.temperature, Some( 0.5 ) );
  assert_eq!( params.top_p, Some( 0.9 ) );
}

#[ test ]
fn models_constants_available()
{
  // Test that model constants are accessible and non-empty
  let llama_model = Models::llama_3_3_70b_instruct();
  assert!( !llama_model.is_empty(), "Llama model name should not be empty" );
  
  let embedding_model = Models::all_minilm_l6_v2();
  assert!( !embedding_model.is_empty(), "Embedding model name should not be empty" );
  
  let mistral_model = Models::mistral_7b_instruct();
  assert!( !mistral_model.is_empty(), "Mistral model name should not be empty" );
}

#[ test ]
fn models_are_valid_identifiers()
{
  // Test that model names follow expected HuggingFace format
  let models = vec!
  [
  Models::llama_3_3_70b_instruct(),
  Models::mistral_7b_instruct(),
  Models::all_minilm_l6_v2(),
  Models::bge_large_en_v1_5(),
  ];
  
  for model in models
  {
  // Model names should contain a slash (org/model format)
  assert!( model.contains( '/' ), "Model '{model}' should follow org/model format" );
  
  // Should not contain invalid characters
  assert!( !model.contains( ' ' ), "Model '{model}' should not contain spaces" );
  assert!( !model.chars().any( char::is_control ), "Model '{model}' should not contain control characters" );
  }
}

#[ test ]
fn inference_output_creation()
{
  // Test that we can create and work with response structures
  let output = InferenceOutput
  {
  generated_text : "Hello, world!".to_string(),
  input_tokens : Some( 5 ),
  generated_tokens : Some( 2 ),
  metadata : None,
  };
  
  assert_eq!( output.generated_text, "Hello, world!" );
  assert_eq!( output.input_tokens, Some( 5 ) );
  assert_eq!( output.generated_tokens, Some( 2 ) );
}

#[ test ]
fn inference_parameters_validation()
{
  // Test parameter ranges and validation
  let params = InferenceParameters::new()
  .with_temperature( 0.0 )  // Minimum valid temperature
  .with_max_new_tokens( 1 );  // Minimum valid tokens
  
  assert_eq!( params.temperature, Some( 0.0 ) );
  assert_eq!( params.max_new_tokens, Some( 1 ) );
  
  let params2 = InferenceParameters::new()
  .with_temperature( 2.0 )  // High temperature
  .with_max_new_tokens( 4096 );  // High token count
  
  assert_eq!( params2.temperature, Some( 2.0 ) );
  assert_eq!( params2.max_new_tokens, Some( 4096 ) );
}

#[ test ]
fn inference_parameters_consistency()
{
  // Test that builder methods maintain consistency
  let params1 = InferenceParameters::new().with_temperature( 0.7 );
  let params2 = InferenceParameters::new().with_temperature( 0.7 );
  
  assert_eq!( params1.temperature, params2.temperature );
  
  // Test overwriting values
  let params3 = InferenceParameters::new()
  .with_temperature( 0.5 )
  .with_temperature( 0.8 );  // Should overwrite
  
  assert_eq!( params3.temperature, Some( 0.8 ) );
}

#[ cfg( feature = "integration" ) ]
mod integration_tests
{
  use super::*;
  use api_huggingface::
  {
  Client,
  environment::HuggingFaceEnvironmentImpl,
  secret::Secret,
  };

  #[ tokio::test ]
  async fn integration_inference_parameters_with_real_api()
  {
  let api_key_string = crate::inc::get_api_key_for_integration();
  
  // Build client with real credentials
  let api_key = Secret::new( api_key_string );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )
      .expect( "Environment build should succeed" );
  let client = Client::build( env )
      .expect( "Client build should succeed" );

  // Create parameters with components
  let params = InferenceParameters::default()
      .with_temperature( 0.3 )
      .with_max_new_tokens( 50 )
      .with_top_p( 0.8 );

  // Use inference API to validate parameters work with real API (with timeout)
  let response = tokio::time::timeout(
      core::time::Duration::from_secs( 10 ),
      client.inference().create_with_parameters( "Hello", "meta-llama/Llama-3.3-70B-Instruct", params )
  ).await;

  // Handle API response gracefully - external API may be unavailable or timeout
  match response
  {
      Ok( Ok( response_data ) ) => {
  match response_data
  {
          api_huggingface::components::inference_shared::InferenceResponse::Single( output ) => {
      assert!( !output.generated_text.is_empty(), "Generated text should not be empty" );
          },
          api_huggingface::components::inference_shared::InferenceResponse::Batch( outputs ) => {
      assert!( !outputs.is_empty(), "Batch should not be empty" );
      assert!( !outputs[0].generated_text.is_empty(), "Generated text should not be empty" );
          },
          api_huggingface::components::inference_shared::InferenceResponse::Summarization( summaries ) => {
      assert!( !summaries.is_empty(), "Summarization should not be empty" );
      assert!( !summaries[0].summary_text.is_empty(), "Summary text should not be empty" );
          }
  }
      },
      Ok( Err( e ) ) =>
      {
  panic!( "Integration test FAILED - API error : {e}

SETUP REQUIRED:
1. Get API key from : https:// huggingface.co/settings/tokens
2. Save to : secret/-secrets.sh as HUGGINGFACE_API_KEY=your-key-here
3. Re-run : cargo test

Integration tests MUST use real credentials to validate actual API behavior." );
      },
      Err( e ) =>
      {
  panic!( "Integration test FAILED - API request timeout : {e:?}

TROUBLESHOOTING:
1. Check network connectivity
2. Verify API endpoint is accessible
3. Confirm API key is valid
4. Check HuggingFace service status

Integration tests require real API access to validate functionality." );
      }
  }
  }

  #[ tokio::test ]
  async fn integration_model_constants_with_real_api()
  {
  let api_key_string = crate::inc::get_api_key_for_integration();

  // Build client with real credentials
  let api_key = Secret::new( api_key_string );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )
      .expect( "Environment build should succeed" );
  let client = Client::build( env )
      .expect( "Client build should succeed" );

  // Test that model constants work with real API calls (use inference - embeddings endpoint not available)
  let model = Models::llama_3_3_70b_instruct();

  let response = tokio::time::timeout(
      core::time::Duration::from_secs( 30 ),
      client.inference().create_with_parameters( "Say hello", model, InferenceParameters::default().with_max_new_tokens( 10 ) )
  ).await;

  match response
  {
      Ok( Ok( response_data ) ) =>
      {
  match response_data
  {
          api_huggingface::components::inference_shared::InferenceResponse::Single( output ) =>
          {
      assert!( !output.generated_text.is_empty(), "Generated text should not be empty" );
          },
          api_huggingface::components::inference_shared::InferenceResponse::Batch( outputs ) =>
          {
      assert!( !outputs.is_empty(), "Batch should not be empty" );
          },
          api_huggingface::components::inference_shared::InferenceResponse::Summarization( summaries ) =>
          {
      assert!( !summaries.is_empty(), "Summarization should not be empty" );
          }
  }
      },
      Ok( Err( e ) ) => panic!( "Integration test FAILED - API error : {e}" ),
      Err( e ) => panic!( "Integration test FAILED - timeout : {e:?}" ),
  }
  }

  #[ tokio::test ]
  async fn integration_inference_output_with_real_response()
  {
  let api_key_string = crate::inc::get_api_key_for_integration();

  // Build client with real credentials
  let api_key = Secret::new( api_key_string );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )
      .expect( "Environment build should succeed" );
  let client = Client::build( env )
      .expect( "Client build should succeed" );

  // Make real API call to get actual InferenceOutput (with timeout)
  let response = tokio::time::timeout(
      core::time::Duration::from_secs( 10 ),
      client.inference().create_with_parameters( "The capital of France is", "meta-llama/Llama-3.3-70B-Instruct", InferenceParameters::default().with_max_new_tokens( 20 ) )
  ).await;

  // Handle API response gracefully - external API may be unavailable or timeout
  match response
  {
      Ok( Ok( response_data ) ) => {
  // Validate InferenceOutput structure with real data
  match response_data
  {
          api_huggingface::components::inference_shared::InferenceResponse::Single( output ) => {
      assert!( !output.generated_text.is_empty(), "Generated text should not be empty" );
      
      // Validate optional metadata fields are handled correctly
      if let Some( input_tokens ) = output.input_tokens
      {
              assert!( input_tokens > 0, "Input tokens should be positive" );
      }
      
      if let Some( generated_tokens ) = output.generated_tokens  
      {
              assert!( generated_tokens > 0, "Generated tokens should be positive" );
      }
          },
          api_huggingface::components::inference_shared::InferenceResponse::Batch( outputs ) => {
      assert!( !outputs.is_empty(), "Batch should not be empty" );
      let output = &outputs[0];
      assert!( !output.generated_text.is_empty(), "Generated text should not be empty" );
          },
          api_huggingface::components::inference_shared::InferenceResponse::Summarization( summaries ) => {
      assert!( !summaries.is_empty(), "Summarization should not be empty" );
      assert!( !summaries[0].summary_text.is_empty(), "Summary text should not be empty" );
          }
  }
      },
      Ok( Err( e ) ) =>
      {
  panic!( "Integration test FAILED - API error : {e}

SETUP REQUIRED:
1. Get API key from : https:// huggingface.co/settings/tokens
2. Save to : secret/-secrets.sh as HUGGINGFACE_API_KEY=your-key-here
3. Re-run : cargo test

Integration tests MUST use real credentials to validate actual API behavior." );
      },
      Err( e ) =>
      {
  panic!( "Integration test FAILED - API request timeout : {e:?}

TROUBLESHOOTING:
1. Check network connectivity
2. Verify API endpoint is accessible
3. Confirm API key is valid
4. Check HuggingFace service status

Integration tests require real API access to validate functionality." );
      }
  }
  }
}
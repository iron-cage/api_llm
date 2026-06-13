//! Debug validation errors

mod inc;

use api_huggingface::
{
  components::input::InferenceParameters,
};

#[ test ]
fn debug_temperature_validation()
{
  let result = InferenceParameters::new()
      .with_temperature( -0.1 )
      .validate();
  assert!( result.is_err(), "Temperature -0.1 is invalid and should fail validation" );
}

#[ test ]
fn debug_model_identifier_validation()
{
  use api_huggingface::validation::validate_model_identifier;
  let result = validate_model_identifier( "" );
  assert!( result.is_err(), "Empty model identifier is invalid and should fail validation" );
}

#[ test ]
fn debug_batch_validation()
{
  use api_huggingface::validation::validate_batch_inputs;
  let large_batch : Vec< String > = ( 0..1001 ).map( | i | format!( "input_{i}" ) ).collect();
  let result = validate_batch_inputs( &large_batch );
  assert!( result.is_err(), "Batch of 1001 inputs exceeds the limit and should fail validation" );
}

#[ cfg( feature = "integration" ) ]
use api_huggingface::
{
  Client,
  environment::HuggingFaceEnvironmentImpl,
  secret::Secret,
  validation::{ validate_model_identifier, validate_batch_inputs },
};

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_validation_with_real_api_calls()
{
  let api_key_string = crate::inc::get_api_key_for_integration();
  
  // Build client with real credentials
  let api_key = Secret::new( api_key_string );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )
      .expect( "Environment build should succeed" );
  let client = Client::build( env )
      .expect( "Client build should succeed" );

  // Test validation with invalid parameters that should fail in real API
  let invalid_params = InferenceParameters::new()
      .with_temperature( -1.0 )  // Invalid temperature
      .with_max_new_tokens( 0 );  // Invalid token count

  let validation_result = invalid_params.validate();
  assert!( validation_result.is_err(), "Invalid parameters should fail validation" );

  // Test that validation errors prevent real API calls
  let api_result = client.inference()
      .create_with_parameters( "test", "microsoft/DialoGPT-medium", invalid_params )
      .await;
  
  assert!( api_result.is_err(), "Invalid parameters should cause API call to fail" );
  }

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_model_validation_with_real_api()
{
  let api_key_string = crate::inc::get_api_key_for_integration();

  // Build client with real credentials
  let api_key = Secret::new( api_key_string );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )
      .expect( "Environment build should succeed" );
  let client = Client::build( env )
      .expect( "Client build should succeed" );

  // Test model identifier validation with empty/invalid models
  assert!( validate_model_identifier( "" ).is_err(), "Empty model should be invalid" );
  assert!( validate_model_identifier( "invalid model name with spaces" ).is_err(), "Spaces should be invalid" );
  assert!( validate_model_identifier( "valid/model-name" ).is_ok(), "Valid format should pass" );

  // Test that invalid model names fail in real API calls
  let result = client.embeddings()
      .create( "test", "invalid_model_name" )
      .await;
  
  // Should fail due to invalid model (either validation or API rejection)
  assert!( result.is_err(), "Invalid model name should cause API failure : {result:?}" );
  }

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_batch_validation_with_real_api()
{
  let api_key_string = crate::inc::get_api_key_for_integration();

  // Build client with real credentials
  let api_key = Secret::new( api_key_string );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )
      .expect( "Environment build should succeed" );
  let client = Client::build( env )
      .expect( "Client build should succeed" );

  // Test batch size validation
  let large_batch : Vec< String > = ( 0..1001 ).map( | i | format!( "input_{i}" ) ).collect();
  assert!( validate_batch_inputs( &large_batch ).is_err(), "Large batch should be invalid" );

  let small_batch = vec![ "test1".to_string(), "test2".to_string() ];
  assert!( validate_batch_inputs( &small_batch ).is_ok(), "Small batch should be valid" );

  // Test that valid small batch works with real API  
  let response = client.embeddings()
      .create_batch( small_batch.clone(), "BAAI/bge-large-en-v1.5" )
      .await;

  // Handle API response gracefully - external API may be unavailable
  match response
  {
      Ok( embeddings ) => {
  match embeddings
  {
          api_huggingface::components::embeddings::EmbeddingResponse::Batch( batch_embeddings ) => {
      assert_eq!( batch_embeddings.len(), small_batch.len(), "Should get embedding for each input" );
          },
          api_huggingface::components::embeddings::EmbeddingResponse::Single( _ ) => {
      // Some APIs might return single response even for batches - that's ok for integration test
          }
  }
      },
      Err( e ) => panic!( "Integration embedding batch should succeed with valid credentials: {e}" ),
  }
}
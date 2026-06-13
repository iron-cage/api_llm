//! Comprehensive tests for `HuggingFace` Inference API functionality

use api_huggingface::
{
  Client,
  environment::HuggingFaceEnvironmentImpl,
  secret::Secret,
  components::
  {
  input::InferenceParameters,
  inference_shared::{ InferenceRequest, InferenceOptions },
  },
  error::{ HuggingFaceError, Result },
};

/// Helper function to create a test client
fn create_test_client() -> Result< Client< HuggingFaceEnvironmentImpl > >
{
  let api_key = Secret::new( "test-api-key".to_string() );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )?;
  Client::build( env )
}

/// Test inference API group creation
#[ tokio::test ]
async fn test_inference_api_creation()
{
  // Setup
  let client = create_test_client().expect( "Client creation should succeed" );
  
  // Execution
  let inference = client.inference();
  
  // Verification
  assert!( core::mem::size_of_val( &inference ) > 0, "Inference API group should be created" );
}

/// Test `InferenceRequest` construction
#[ test ]
fn test_inference_request_construction()
{
  // Setup
  let input_text = "Hello, world!";
  
  // Execution
  let request = InferenceRequest::new( input_text );
  
  // Verification
  assert_eq!( request.inputs, input_text );
  assert!( request.parameters.is_none() );
  assert!( request.options.is_none() );
}

/// Test `InferenceRequest` with parameters
#[ test ]
fn test_inference_request_with_parameters()
{
  // Setup
  let input_text = "Generate text with custom parameters";
  let parameters = InferenceParameters::new()
  .with_temperature( 0.8 )
  .with_max_new_tokens( 100 )
  .with_top_p( 0.95 );
  
  // Execution
  let request = InferenceRequest::new( input_text ).with_parameters( parameters.clone() );
  
  // Verification
  assert_eq!( request.inputs, input_text );
  assert!( request.parameters.is_some() );
  let req_params = request.parameters.as_ref().expect( "[test_inference_request_with_parameters] InferenceRequest parameters should be Some after with_parameters() call - check InferenceRequest::with_parameters() implementation" );
  assert_eq!( req_params.temperature, Some( 0.8 ) );
  assert_eq!( req_params.max_new_tokens, Some( 100 ) );
  assert_eq!( req_params.top_p, Some( 0.95 ) );
}

/// Test `InferenceRequest` with options
#[ test ]
fn test_inference_request_with_options()
{
  // Setup
  let input_text = "Generate text with options";
  let options = InferenceOptions::new().with_wait_for_model( true );
  
  // Execution
  let request = InferenceRequest::new( input_text ).with_options( options );
  
  // Verification
  assert_eq!( request.inputs, input_text );
  assert!( request.options.is_some() );
  let req_options = request.options.as_ref().expect( "[test_inference_request_with_options] InferenceRequest options should be Some after with_options() call - check InferenceRequest::with_options() implementation" );
  assert_eq!( req_options.wait_for_model, Some( true ) );
}

/// Test `InferenceParameters` validation integration
#[ test ]
fn test_inference_parameters_validation_in_request()
{
  // Setup - Create parameters with invalid values
  let invalid_params = InferenceParameters::new()
  .with_temperature( -0.5 )  // Invalid : negative temperature
  .with_max_new_tokens( 0 ); // Invalid : zero tokens
  
  // Execution & Verification
  let validation_result = invalid_params.validate();
  assert!( validation_result.is_err(), "Validation should fail for invalid parameters" );
  
  if let Err( HuggingFaceError::Validation( msg ) ) = validation_result
  {
  assert!( msg.to_lowercase().contains( "temperature" ), "Error should mention temperature" );
  assert!( msg.to_lowercase().contains( "max_new_tokens" ), "Error should mention max_new_tokens" );
  }
  else
  {
  panic!( "Expected validation error" );
  }
}

/// Test model identifier validation for inference
#[ test ]
fn test_model_identifier_validation()
{
  use api_huggingface::validation::validate_model_identifier;
  
  // Setup - Valid model identifiers
  let valid_models = vec![
  "gpt2",
  "meta-llama/Llama-2-7b-hf",
  "microsoft/DialoGPT-medium",
  "sentence-transformers/all-MiniLM-L6-v2",
  ];
  
  // Execution & Verification - Valid models
  for model in valid_models
  {
  let result = validate_model_identifier( model );
  assert!( result.is_ok(), "Model '{model}' should be valid" );
  }
  
  // Setup - Invalid model identifiers
  let invalid_models = vec![
  "",                    // Empty
  " ",                   // Whitespace only
  "model with spaces",   // Contains spaces
  "/leading-slash",      // Leading slash
  "trailing-slash/",     // Trailing slash
  "double//slash",       // Double slash
  ];
  
  // Execution & Verification - Invalid models
  for model in invalid_models
  {
  let result = validate_model_identifier( model );
  assert!( result.is_err(), "Model '{model}' should be invalid" );
  }
}

/// Test input text validation for inference
#[ test ]
fn test_input_text_validation()
{
  use api_huggingface::validation::validate_input_text;
  
  // Setup & Execution & Verification - Valid inputs
  let medium_text = "a".repeat( 1000 );
  let valid_inputs = vec![
  "Hello, world!",
  "Generate a creative story about artificial intelligence.",
  medium_text.as_str(), // Medium length text
  "Mixed content : Hello 🌍! 你好世界 مرحبا بالعالم", // Unicode content
  ];
  
  for input in valid_inputs
  {
  let result = validate_input_text( input );
  assert!( result.is_ok(), "Input should be valid : '{input}'" );
  }
  
  // Setup & Execution & Verification - Invalid inputs
  // Empty input
  let result = validate_input_text( "" );
  assert!( result.is_err(), "Empty input should be invalid" );
  
  // Excessively long input
  let long_input = "a".repeat( 60_000 );
  let result = validate_input_text( &long_input );
  assert!( result.is_err(), "Excessively long input should be invalid" );
}

/// Test `InferenceParameters` builder pattern
#[ test ]
fn test_inference_parameters_builder_pattern()
{
  // Setup
  let temperature = 0.9;
  let max_tokens = 512;
  let top_p = 0.85;
  let stop_sequences = vec![ "END".to_string(), "\n\n".to_string() ];
  
  // Execution
  let parameters = InferenceParameters::new()
  .with_temperature( temperature )
  .with_max_new_tokens( max_tokens )
  .with_top_p( top_p )
  .with_stop_sequences( stop_sequences.clone() );
  
  // Verification
  assert_eq!( parameters.temperature, Some( temperature ) );
  assert_eq!( parameters.max_new_tokens, Some( max_tokens ) );
  assert_eq!( parameters.top_p, Some( top_p ) );
  assert_eq!( parameters.stop, Some( stop_sequences ) );
  
  // Verify validation passes
  let validation_result = parameters.validate();
  assert!( validation_result.is_ok(), "Valid parameters should pass validation" );
}

/// Test `InferenceOptions` construction and validation
#[ test ]
fn test_inference_options_construction()
{
  // Setup
  let use_cache = false;
  let wait_for_model = true;
  
  // Execution
  let options = InferenceOptions::new()
  .with_use_cache( use_cache )
  .with_wait_for_model( wait_for_model );
  
  // Verification
  assert_eq!( options.use_cache, Some( use_cache ) );
  assert_eq!( options.wait_for_model, Some( wait_for_model ) );
}

/// Test inference request serialization
#[ test ]
fn test_inference_request_serialization()
{
  // Setup
  let input_text = "Serialize this request";
  let parameters = InferenceParameters::new()
  .with_temperature( 0.7 )
  .with_max_new_tokens( 256 );
  let options = InferenceOptions::new().with_wait_for_model( true );
  
  let request = InferenceRequest::new( input_text )
  .with_parameters( parameters )
  .with_options( options );
  
  // Execution
  let serialized = serde_json::to_string( &request );
  
  // Verification
  assert!( serialized.is_ok(), "Request serialization should succeed" );
  let json_str = serialized.expect( "[test_inference_request_serialization] InferenceRequest serialization failed after is_ok() check - check serde_json::to_string() implementation" );
  assert!( json_str.contains( "Serialize this request" ), "Serialized JSON should contain input text" );
  assert!( json_str.contains( "0.7" ), "Serialized JSON should contain temperature" );
  assert!( json_str.contains( "256" ), "Serialized JSON should contain max_new_tokens" );
}

/// Test error handling for authentication failures
#[ test ]
fn test_authentication_error_handling()
{
  // Setup - Create client with empty API key
  let empty_key = Secret::new( String::new() );
  let env_result = HuggingFaceEnvironmentImpl::build( empty_key, None );
  
  // Execution & Verification
  // Note : The environment build might succeed but authentication will fail on API calls
  // This tests the error handling structure
  match env_result
  {
  Ok( _env ) => 
  {
      // Environment creation succeeded, authentication errors will occur during API calls
      // This is expected behavior - we test this in integration tests
  },
  Err( e ) => 
  {
      // Environment creation failed due to invalid key
      match e
      {
  HuggingFaceError::Authentication( _ ) => {}, // Expected
  other => panic!( "Expected Authentication error, got : {other:?}" ),
      }
  }
  }
}

/// Test error message formatting
#[ test ]
fn test_error_message_formatting()
{
  // Setup - Create a validation error
  let invalid_params = InferenceParameters::new()
  .with_temperature( 3.0 ); // Invalid : too high
  
  // Execution
  let result = invalid_params.validate();
  
  // Verification
  assert!( result.is_err(), "Should produce validation error" );
  
  if let Err( error ) = result
  {
  let error_string = error.to_string();
  assert!( error_string.to_lowercase().contains( "temperature" ), "Error should mention temperature" );
  assert!( error_string.contains( '3' ), "Error should include the invalid value" );
  assert!( error_string.contains( "2.0" ), "Error should mention the valid range" );
  }
  else
  {
  panic!( "Expected error result" );
  }
}

/// Test parameter boundary conditions
#[ test ]
fn test_parameter_boundary_conditions()
{
  // Setup & Test minimum valid values
  let min_params = InferenceParameters::new()
  .with_temperature( 0.0 )
  .with_max_new_tokens( 1 )
  .with_top_p( 0.0 );
  
  let result = min_params.validate();
  assert!( result.is_ok(), "Minimum valid parameters should pass validation" );
  
  // Setup & Test maximum valid values
  let max_params = InferenceParameters::new()
  .with_temperature( 2.0 )
  .with_max_new_tokens( 8192 )
  .with_top_p( 1.0 );
  
  let result = max_params.validate();
  assert!( result.is_ok(), "Maximum valid parameters should pass validation" );
  
  // Setup & Test boundary violations
  let invalid_low = InferenceParameters::new()
  .with_temperature( -0.1 ); // Below minimum
  
  let result = invalid_low.validate();
  assert!( result.is_err(), "Below-minimum parameters should fail validation" );
  
  let invalid_high = InferenceParameters::new()
  .with_temperature( 2.1 ); // Above maximum
  
  let result = invalid_high.validate();
  assert!( result.is_err(), "Above-maximum parameters should fail validation" );
}

/// Test default parameter validation
#[ test ]
fn test_default_parameters_are_valid()
{
  // Setup
  let default_params = InferenceParameters::default();
  
  // Execution
  let result = default_params.validate();
  
  // Verification
  assert!( result.is_ok(), "Default parameters should be valid" );
  
  // Verify specific default values
  assert!( default_params.temperature.is_some(), "Default should have temperature" );
  assert!( default_params.max_new_tokens.is_some(), "Default should have max_new_tokens" );
  assert!( default_params.top_p.is_some(), "Default should have top_p" );
}

#[ cfg( feature = "inference-streaming" ) ]
mod streaming_tests
{
  use super::*;
  
  /// Test streaming parameter validation
  #[ test ]
  fn test_streaming_parameters_validation()
  {
  // Setup
  let streaming_params = InferenceParameters::new()
      .with_streaming( true )
      .with_temperature( 0.8 )
      .with_max_new_tokens( 100 );
  
  // Execution
  let result = streaming_params.validate();
  
  // Verification
  assert!( result.is_ok(), "Valid streaming parameters should pass validation" );
  assert_eq!( streaming_params.stream, Some( true ) );
  }
}

#[ cfg( feature = "integration" ) ]
mod integration_tests
{
  use super::*;
  use workspace_tools as workspace;
  
  /// Helper to get API key for integration tests - panics if not found
  fn get_api_key_for_integration() -> String
  {
  let workspace = workspace::workspace()
      .expect( "Failed to access workspace - required for integration tests" );
  
  let secrets = workspace.load_secrets_from_file( "-secrets.sh" )
      .expect( "Failed to load secret/-secrets.sh - required for integration tests" );
  
  secrets.get( "HUGGINGFACE_API_KEY" )
      .expect( "HUGGINGFACE_API_KEY not found in secret/-secrets.sh - required for integration tests. Get your token from https://huggingface.co/settings/tokens" )
      .clone()
  }

  /// Helper to create integration test environment
  fn create_integration_environment() -> HuggingFaceEnvironmentImpl
  {
  let api_key_string = get_api_key_for_integration();
  let api_key = Secret::new( api_key_string );
  HuggingFaceEnvironmentImpl::build( api_key, None )
      .expect( "Should create environment with workspace API key" )
  }
  
  /// Test real API call with inference
  #[ tokio::test ]
  async fn integration_inference_create()
  {
  // Setup - Get environment with API key (will panic if missing)
  let env = create_integration_environment();
  let client = Client::build( env )
      .expect( "Should create client" );
  
  let inference = client.inference();
  
  // Execution
  let result = inference.create( 
      "Hello, how are you?", 
      "gpt2" 
  ).await;
  
  // Verification
  match result
  {
      Ok( _response ) => 
      {
  // Basic response validation
  println!( "Integration test successful - received response" );
      },
      Err( e ) => panic!( "Integration test failed: {e}" ),
  }
  }
  
  /// Test real API call with parameters
  #[ tokio::test ]
  async fn integration_inference_create_with_parameters()
  {
  // Setup - Get environment with API key (will panic if missing)
  let env = create_integration_environment();
  let client = Client::build( env )
      .expect( "Should create client" );
  
  let inference = client.inference();
  let parameters = InferenceParameters::new()
      .with_temperature( 0.7 )
      .with_max_new_tokens( 50 );
  
  // Execution
  let result = inference.create_with_parameters( 
      "Once upon a time", 
      "gpt2",
      parameters
  ).await;
  
  // Verification
  match result
  {
      Ok( _response ) => 
      {
  println!( "Integration test with parameters successful" );
      },
      Err( e ) => panic!( "Integration test with parameters failed: {e}" ),
  }
  }
}

// Additional helper tests for completeness

/// Test that inference API follows 3-phase test pattern
#[ test ]
fn test_three_phase_pattern_example()
{
  // 📋 SETUP PHASE
  let test_input = "Test input for inference";
  let expected_input = test_input;
  
  // ⚡ EXECUTION PHASE  
  let request = InferenceRequest::new( test_input );
  
  // ✅ VERIFICATION PHASE
  assert_eq!( request.inputs, expected_input );
  assert!( request.parameters.is_none() );
  assert!( request.options.is_none() );
}

/// Test request construction with all features
#[ test ]
fn test_comprehensive_request_construction()
{
  // Setup
  let input_text = "Generate comprehensive response";
  let parameters = InferenceParameters::new()
  .with_temperature( 0.8 )
  .with_max_new_tokens( 200 )
  .with_top_p( 0.9 )
  .with_stop_sequences( vec![ "END".to_string() ] );
  let options = InferenceOptions::new()
  .with_wait_for_model( true )
  .with_use_cache( false );
  
  // Execution
  let request = InferenceRequest::new( input_text )
  .with_parameters( parameters )
  .with_options( options );
  
  // Verification
  assert_eq!( request.inputs, input_text );
  assert!( request.parameters.is_some() );
  assert!( request.options.is_some() );

  let params = request.parameters.as_ref().expect( "[test_comprehensive_inference_request] InferenceRequest parameters should be Some after with_parameters() call - check InferenceRequest::with_parameters() implementation" );
  assert_eq!( params.temperature, Some( 0.8 ) );
  assert_eq!( params.max_new_tokens, Some( 200 ) );

  let opts = request.options.as_ref().expect( "[test_comprehensive_inference_request] InferenceRequest options should be Some after with_options() call - check InferenceRequest::with_options() implementation" );
  assert_eq!( opts.wait_for_model, Some( true ) );
  assert_eq!( opts.use_cache, Some( false ) );
}
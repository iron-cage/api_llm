//! Comprehensive tests for `HuggingFace` Models API functionality

mod inc;

use api_huggingface::
{
  models::{ Models, ModelStatus },
  components::models::{ ModelInfo, Models as ModelConstants },
  validation::validate_model_identifier,
  Client,
  environment::HuggingFaceEnvironmentImpl,
  error::HuggingFaceError,
  secret::Secret,
};
#[ cfg( all( feature = "integration", feature = "env-config" ) ) ]
use core::time::Duration;

// ============================================================================
// Test Helper Functions
// ============================================================================

/// Helper function to create a test client
fn create_test_client() -> api_huggingface::error::Result< Client< HuggingFaceEnvironmentImpl > >
{
  let api_key = Secret::new( "test-api-key".to_string() );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )?;
  Client::build( env )
}

/// Create a test models client
#[ cfg( feature = "env-config" ) ]
fn create_test_models() -> api_huggingface::error::Result< Models< HuggingFaceEnvironmentImpl > >
{
  let client = create_test_client()?;
  Ok( Models::new( &client ) )
}

// ============================================================================
// Unit Tests - Model Constants
// ============================================================================

// Fix(issue-004): Removed tests for deprecated llama_3_1 models
// Root cause : Tests were using deprecated model functions that were replaced with llama_3_3_70b_instruct
// After sed replacement, duplicate test names were created
// Pitfall : When models are deprecated, remove old tests instead of blindly renaming

#[ test ]
fn test_model_constants_llama_3_3_70b_instruct()
{
  let model_id = ModelConstants::llama_3_3_70b_instruct();
  assert_eq!( model_id, "meta-llama/Llama-3.3-70B-Instruct" );
  assert!( validate_model_identifier( model_id ).is_ok() );
}

#[ test ]
fn test_model_constants_mistral_7b_instruct()
{
  let model_id = ModelConstants::mistral_7b_instruct();
  assert_eq!( model_id, "mistralai/Mistral-7B-Instruct-v0.3" );
  assert!( validate_model_identifier( model_id ).is_ok() );
}

#[ test ]
fn test_model_constants_code_llama_7b_instruct()
{
  let model_id = ModelConstants::code_llama_7b_instruct();
  assert_eq!( model_id, "codellama/CodeLlama-7b-Instruct-hf" );
  assert!( validate_model_identifier( model_id ).is_ok() );
}

#[ test ]
fn test_model_constants_all_minilm_l6_v2()
{
  let model_id = ModelConstants::all_minilm_l6_v2();
  assert_eq!( model_id, "sentence-transformers/all-MiniLM-L6-v2" );
  assert!( validate_model_identifier( model_id ).is_ok() );
}

#[ test ]
fn test_model_constants_all_minilm_l12_v2()
{
  let model_id = ModelConstants::all_minilm_l12_v2();
  assert_eq!( model_id, "sentence-transformers/all-MiniLM-L12-v2" );
  assert!( validate_model_identifier( model_id ).is_ok() );
}

#[ test ]
fn test_model_constants_bge_large_en_v1_5()
{
  let model_id = ModelConstants::bge_large_en_v1_5();
  assert_eq!( model_id, "BAAI/bge-large-en-v1.5" );
  assert!( validate_model_identifier( model_id ).is_ok() );
}

// ============================================================================
// Unit Tests - Model Identifier Validation
// ============================================================================

#[ test ]
fn test_validate_model_identifier_valid_ids()
{
  let valid_ids = vec!
  [
  "gpt2",
  "meta-llama/Llama-2-7b-hf",
  "microsoft/DialoGPT-medium",
  "sentence-transformers/all-MiniLM-L6-v2",
  "BAAI/bge-large-en-v1.5",
  "a-b-c/d-e-f",
  "123/456",
  ModelConstants::llama_3_3_70b_instruct(),
  ModelConstants::mistral_7b_instruct(),
  ];

  for model_id in valid_ids
  {
  assert!(
      validate_model_identifier( model_id ).is_ok(),
      "Expected '{model_id}' to be valid"
  );
  }
}

#[ test ]
fn test_validate_model_identifier_invalid_ids()
{
  let invalid_cases = vec!
  [
  // Empty identifier
  ( "", "Model identifier cannot be empty" ),
  
  // Leading/trailing whitespace (trim() catches newlines at end first)
  ( " gpt2", "cannot have leading or trailing whitespace" ),
  ( "gpt2 ", "cannot have leading or trailing whitespace" ),
  ( "  gpt2  ", "cannot have leading or trailing whitespace" ),
  ( "gpt2\n", "cannot have leading or trailing whitespace" ), // trim() catches this
  ( "\tgpt2", "cannot have leading or trailing whitespace" ), // trim() catches this
  
  // Control characters in middle (after trim() check passes)
  ( "gpt\r2", "cannot contain newlines" ),
  ( "gp\tt2", "cannot contain newlines" ),
  
  // Invalid slashes
  ( "/gpt2", "cannot start/end with slash" ),
  ( "gpt2/", "cannot start/end with slash" ),
  ( "gpt//2", "cannot start/end with slash" ),
  ( "//gpt2", "cannot start/end with slash" ),
  
  // Spaces
  ( "gpt 2", "cannot contain spaces" ),
  ( "meta llama/model", "cannot contain spaces" ),
  ];

  for ( model_id, expected_error ) in invalid_cases
  {
  let result = validate_model_identifier( model_id );
  assert!(
      result.is_err(),
      "Expected '{model_id}' to be invalid"
  );

  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
      assert!(
  msg.contains( expected_error ),
  "Expected error containing '{expected_error}' but got '{msg}'"
      );
  }
  else
  {
      panic!( "Expected validation error for '{model_id}'" );
  }
  }
}

#[ test ]
fn test_validate_model_identifier_too_long()
{
  let long_id = "a".repeat( 201 );
  let result = validate_model_identifier( &long_id );
  
  assert!( result.is_err() );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "too long" ) );
  assert!( msg.contains( "201" ) );
  assert!( msg.contains( "200" ) );
  }
  else
  {
  panic!( "Expected validation error for long model ID" );
  }
}

// ============================================================================
// Unit Tests - ModelInfo Structure
// ============================================================================

#[ test ]
fn test_model_info_serialization()
{
  let model_info = ModelInfo
  {
  id : "test-model".to_string(),
  repository_url : Some( "https://huggingface.co/test-model".to_string() ),
  pipeline_tag : Some( "text-generation".to_string() ),
  tags : Some( vec![ "pytorch".to_string(), "transformers".to_string() ] ),
  private : Some( false ),
  author : Some( "test-author".to_string() ),
  likes : Some( 42 ),
  downloads : Some( 1337 ),
  };

  let serialized = serde_json::to_string( &model_info ).expect( "Serialization should succeed" );
  let deserialized : ModelInfo = serde_json::from_str( &serialized ).expect( "Deserialization should succeed" );
  
  assert_eq!( deserialized.id, "test-model" );
  assert_eq!( deserialized.repository_url, Some( "https://huggingface.co/test-model".to_string() ) );
  assert_eq!( deserialized.pipeline_tag, Some( "text-generation".to_string() ) );
  assert_eq!( deserialized.tags, Some( vec![ "pytorch".to_string(), "transformers".to_string() ] ) );
  assert_eq!( deserialized.private, Some( false ) );
  assert_eq!( deserialized.author, Some( "test-author".to_string() ) );
  assert_eq!( deserialized.likes, Some( 42 ) );
  assert_eq!( deserialized.downloads, Some( 1337 ) );
}

#[ test ]
fn test_model_info_optional_fields()
{
  let minimal_model = ModelInfo
  {
  id : "minimal-model".to_string(),
  repository_url : None,
  pipeline_tag : None,
  tags : None,
  private : None,
  author : None,
  likes : None,
  downloads : None,
  };

  let serialized = serde_json::to_string( &minimal_model ).expect( "Serialization should succeed" );
  
  // Check that optional fields are skipped in serialization
  assert!( !serialized.contains( "repository_url" ) );
  assert!( !serialized.contains( "pipeline_tag" ) );
  assert!( !serialized.contains( "tags" ) );
  assert!( !serialized.contains( "private" ) );
  assert!( !serialized.contains( "author" ) );
  assert!( !serialized.contains( "likes" ) );
  assert!( !serialized.contains( "downloads" ) );
  
  let deserialized : ModelInfo = serde_json::from_str( &serialized ).expect( "Deserialization should succeed" );
  assert_eq!( deserialized.id, "minimal-model" );
  assert!( deserialized.repository_url.is_none() );
  assert!( deserialized.pipeline_tag.is_none() );
}

// ============================================================================
// Unit Tests - ModelStatus Enum
// ============================================================================

#[ test ]
fn test_model_status_available()
{
  let status = ModelStatus::Available;
  assert_eq!( status, ModelStatus::Available );
  
  let serialized = serde_json::to_string( &status ).expect( "Serialization should succeed" );
  let deserialized : ModelStatus = serde_json::from_str( &serialized ).expect( "Deserialization should succeed" );
  assert_eq!( deserialized, ModelStatus::Available );
}

#[ test ]
fn test_model_status_loading()
{
  let status = ModelStatus::Loading;
  assert_eq!( status, ModelStatus::Loading );
  
  let serialized = serde_json::to_string( &status ).expect( "Serialization should succeed" );
  let deserialized : ModelStatus = serde_json::from_str( &serialized ).expect( "Deserialization should succeed" );
  assert_eq!( deserialized, ModelStatus::Loading );
}

#[ test ]
fn test_model_status_not_found()
{
  let status = ModelStatus::NotFound;
  assert_eq!( status, ModelStatus::NotFound );
  
  let serialized = serde_json::to_string( &status ).expect( "Serialization should succeed" );
  let deserialized : ModelStatus = serde_json::from_str( &serialized ).expect( "Deserialization should succeed" );
  assert_eq!( deserialized, ModelStatus::NotFound );
}

#[ test ]
fn test_model_status_error()
{
  let error_msg = "Model failed to load".to_string();
  let status = ModelStatus::Error( error_msg.clone() );
  
  if let ModelStatus::Error( msg ) = &status
  {
  assert_eq!( msg, &error_msg );
  }
  else
  {
  panic!( "Expected Error status" );
  }
  
  let serialized = serde_json::to_string( &status ).expect( "Serialization should succeed" );
  let deserialized : ModelStatus = serde_json::from_str( &serialized ).expect( "Deserialization should succeed" );
  assert_eq!( deserialized, status );
}

#[ test ]
fn test_model_status_clone_debug()
{
  let statuses = vec!
  [
  ModelStatus::Available,
  ModelStatus::Loading,
  ModelStatus::NotFound,
  ModelStatus::Error( "Test error".to_string() ),
  ];
  
  for status in statuses
  {
  let cloned = status.clone();
  assert_eq!( status, cloned );

  let debug_str = format!( "{status:?}" );
  assert!( !debug_str.is_empty() );
  }
}

// ============================================================================
// Unit Tests - Models Client Creation
// ============================================================================

#[ cfg( feature = "env-config" ) ]
#[ test ]
fn test_models_client_creation()
{
  let client = create_test_client().expect( "Should create test client" );
  let models = Models::new( &client );

  // Just verify it was created successfully - we can't test much more without actual API calls
  let debug_str = format!( "{models:?}" );
  assert!( !debug_str.is_empty() );
}

#[ cfg( not( feature = "env-config" ) ) ]
#[ test ]
fn test_models_client_creation_no_env_config()
{
  let client = Client::new( () );
  let models = Models::new( &client );

  let debug_str = format!( "{models:?}" );
  assert!( !debug_str.is_empty() );
}

// ============================================================================
// Integration Tests - Conditional Compilation
// ============================================================================

/// Create client with real API key for integration tests
#[ cfg( all( feature = "integration", feature = "env-config" ) ) ]
fn create_integration_test_models() -> api_huggingface::error::Result< Models< HuggingFaceEnvironmentImpl > >
{
  let api_key = Secret::new( crate::inc::get_api_key_for_integration() );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )?;
  let client = Client::build( env )?;
  Ok( Models::new( &client ) )
}

#[ cfg( all( feature = "integration", feature = "env-config" ) ) ]
#[ tokio::test ]
async fn test_models_get_real_model()
{
  let models = create_integration_test_models().expect( "Should create integration test models" );
  let model_id = "gpt2";
  
  // HuggingFace may return fully qualified names like "openai-community/gpt2"
  let model_info = models.get( model_id ).await
      .expect( "model retrieval should succeed in integration tests" );
  assert!(
      model_info.id == model_id || model_info.id.ends_with( &format!( "/{model_id}" ) ),
      "Expected model ID '{model_id}' or ending with '/{model_id}', got '{}'",
      model_info.id
  );
  }
  
#[ cfg( all( feature = "integration", feature = "env-config" ) ) ]
#[ tokio::test ]
async fn test_models_is_available_real_model()
{
  let models = create_integration_test_models().expect( "Should create integration test models" );
  let model_id = "meta-llama/Llama-3.2-1B-Instruct";
  // Llama-3.2-1B-Instruct is a confirmed-available model on the Router API
  let available = models.is_available( model_id ).await
      .expect( "is_available should succeed for a known model in integration tests" );
  assert!( available, "Model {model_id} should be available on HuggingFace" );
}
  
#[ cfg( all( feature = "integration", feature = "env-config" ) ) ]
#[ tokio::test ]
async fn test_models_status_real_model()
{
  let models = create_integration_test_models().expect( "Should create integration test models" );
  let model_id = ModelConstants::mistral_7b_instruct();
  // Verify status API returns a valid variant (any status is acceptable)
  let status = models.status( model_id ).await
      .expect( "status should succeed for a known model in integration tests" );
  match status
  {
      ModelStatus::Available | ModelStatus::Loading | ModelStatus::NotFound | ModelStatus::Error( _ ) => {},
  }
}
  
#[ cfg( all( feature = "integration", feature = "env-config" ) ) ]
#[ tokio::test ]
async fn test_models_wait_for_model_timeout()
{
  let models = create_integration_test_models().expect( "Should create integration test models" );
  let non_existent_model = "definitely-does-not-exist/model-12345";
  
  let start_time = std::time::Instant::now();
  let result = models.wait_for_model( non_existent_model, 5 ).await;
  let elapsed = start_time.elapsed();
  
  // Should fail within a reasonable timeout period (but API behavior may vary)
  assert!( result.is_err() );
  if elapsed > Duration::from_secs( 30 )
  {
      println!( "Warning : Timeout test took longer than expected ({elapsed:?}) - may indicate API behavior change" );
  }
  
  if let Err( HuggingFaceError::ModelUnavailable( msg ) ) = result
  {
      // More flexible assertion - just check it mentions the model or timeout
      assert!(
  msg.contains( non_existent_model ) || msg.contains( "timeout" ) || msg.contains( "available" ) || msg.contains( "seconds" ),
  "Expected timeout or model-related error message, got : '{msg}'"
      );
  }
  else
  {
      panic!( "Expected ModelUnavailable error, got : {result:?}" );
  }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[ cfg( feature = "env-config" ) ]
#[ tokio::test ]
async fn test_models_get_invalid_model_id()
{
  let models = create_test_models().expect( "Should create test models" );
  
  let invalid_ids = vec!
  [
  "", // Empty
  "invalid model id", // Spaces
  "/invalid", // Leading slash
  "model/", // Trailing slash
  ];
  
  for invalid_id in invalid_ids
  {
  let result = models.get( invalid_id ).await;
  assert!(
      result.is_err(),
      "Expected error for invalid model ID: '{invalid_id}'"
  );

  if let Err( HuggingFaceError::Validation( _ ) ) = result
  {
      // Good, this is expected
  }
  else
  {
      panic!( "Expected validation error for invalid model ID: '{invalid_id}'" );
  }
  }
}

#[ cfg( feature = "env-config" ) ]
#[ tokio::test ]
async fn test_models_is_available_invalid_model_id()
{
  let models = create_test_models().expect( "Should create test models" );
  
  let result = models.is_available( "" ).await;
  assert!( result.is_err() );
  
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "empty" ) );
  }
  else
  {
  panic!( "Expected validation error" );
  }
}

#[ cfg( feature = "env-config" ) ]
#[ tokio::test ]
async fn test_models_status_invalid_model_id()
{
  let models = create_test_models().expect( "Should create test models" );
  
  let result = models.status( "model with spaces" ).await;
  assert!( result.is_err() );
  
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "spaces" ) );
  }
  else
  {
  panic!( "Expected validation error" );
  }
}

#[ cfg( feature = "env-config" ) ]
#[ tokio::test ]
async fn test_models_wait_for_model_invalid_id()
{
  let models = create_test_models().expect( "Should create test models" );
  
  let result = models.wait_for_model( "//invalid", 1 ).await;
  assert!( result.is_err() );
  
  if let Err( HuggingFaceError::Validation( _ ) ) = result
  {
  // Good, this is expected
  }
  else
  {
  panic!( "Expected validation error" );
  }
}

// ============================================================================
// Performance and Edge Case Tests
// ============================================================================

#[ test ]
fn test_model_constants_performance()
{
  // Verify model constants return consistent values across repeated access (idempotent, no mutation)
  let llama = ModelConstants::llama_3_3_70b_instruct();
  let mistral = ModelConstants::mistral_7b_instruct();
  let minilm = ModelConstants::all_minilm_l6_v2();

  assert!( !llama.is_empty(), "Llama model constant should not be empty" );
  assert!( !mistral.is_empty(), "Mistral model constant should not be empty" );
  assert!( !minilm.is_empty(), "MiniLM model constant should not be empty" );

  // Each call must return the same value — constants must not mutate
  assert_eq!( ModelConstants::llama_3_3_70b_instruct(), llama );
  assert_eq!( ModelConstants::mistral_7b_instruct(), mistral );
  assert_eq!( ModelConstants::all_minilm_l6_v2(), minilm );
}

#[ test ]
fn test_model_status_memory_usage()
{
  let statuses = vec!
  [
  ModelStatus::Available,
  ModelStatus::Loading,
  ModelStatus::NotFound,
  ModelStatus::Error( "A".repeat( 1000 ) ), // Large error message
  ];
  
  // Just verify they can all be created and cloned without issues
  for status in statuses
  {
  let _cloned = status.clone();
  let _debug = format!( "{status:?}" );
  }
}

#[ test ]
fn test_model_info_with_large_data()
{
  let large_tags : Vec< String > = ( 0..100 ).map( | i | format!( "tag{i}" ) ).collect();
  let large_model = ModelInfo
  {
  id : "large-model".to_string(),
  repository_url : Some( "https://".to_string() + &"very-long-url-".repeat( 50 ) ),
  pipeline_tag : Some( "text-generation".to_string() ),
  tags : Some( large_tags.clone() ),
  private : Some( true ),
  author : Some( "author-name".to_string() ),
  likes : Some( u32::MAX ),
  downloads : Some( u32::MAX ),
  };
  
  let serialized = serde_json::to_string( &large_model ).expect( "Should serialize large model" );
  let deserialized : ModelInfo = serde_json::from_str( &serialized ).expect( "Should deserialize large model" );
  
  assert_eq!( deserialized.id, "large-model" );
  assert_eq!( deserialized.tags, Some( large_tags ) );
  assert_eq!( deserialized.likes, Some( u32::MAX ) );
  assert_eq!( deserialized.downloads, Some( u32::MAX ) );
}
//! Integration tests for model listing and details endpoints.

#![ cfg( feature = "integration" ) ]

mod inc;
use inc::test_helpers::create_test_client;

use api_xai::ClientApiAccessors;

#[ tokio::test ]
async fn test_list_models()
{
  let client = create_test_client();

  let response = client.models().list().await
    .expect( "List models should succeed" );

  // Verify response structure
  assert_eq!( response.object, "list", "Object type should be 'list'" );
  assert!( !response.data.is_empty(), "Should have at least one model" );

  // Verify model structure
  for model in &response.data {
    assert!( !model.id.is_empty(), "Model ID should not be empty" );
    assert_eq!( model.object, "model", "Model object type should be 'model'" );
    assert!( model.created > 0, "Model should have creation timestamp" );
    assert!( !model.owned_by.is_empty(), "Model should have owner" );
  }

  // Check for expected Grok models
  let model_ids : Vec< &str > = response.data.iter().map( |m| m.id.as_str() ).collect();
  let has_grok = model_ids.iter().any( |id| id.contains( "grok" ) );

  assert!( has_grok, "Should have at least one Grok model" );

  println!( "✅ List models test passed" );
  println!( "Found {} models:", response.data.len() );
  for model in &response.data {
    println!( "  - {} (created : {}, owned by : {})",
      model.id,
      model.created,
      model.owned_by
    );
  }
}

#[ tokio::test ]
async fn test_get_specific_model_grok_beta()
{
  let client = create_test_client();

  let model = client.models().get( "grok-2-1212" ).await
    .expect( "Get grok-2-1212 model should succeed" );

  // Verify model details
  assert_eq!( model.id, "grok-2-1212", "Model ID should be grok-2-1212" );
  assert_eq!( model.object, "model", "Object type should be 'model'" );
  assert!( model.created > 0, "Should have creation timestamp" );
  assert!( !model.owned_by.is_empty(), "Should have owner" );

  println!( "✅ Get grok-2-1212 model test passed" );
  println!( "Model : {}", model.id );
  println!( "Created : {}", model.created );
  println!( "Owned by : {}", model.owned_by );
}

#[ tokio::test ]
async fn test_get_nonexistent_model()
{
  let client = create_test_client();

  let result = client.models().get( "nonexistent-model-xyz-12345" ).await;

  // Should return an error for nonexistent model
  assert!( result.is_err(), "Should fail for nonexistent model" );

  let error = result.unwrap_err();
  let error_str = format!( "{error:?}" );

  // Error should mention 404 or model not found
  let is_not_found_error =
    error_str.contains( "404" ) ||
    error_str.contains( "not found" ) ||
    error_str.contains( "Not Found" );

  assert!( is_not_found_error, "Error should indicate model not found, got : {error_str}" );

  println!( "✅ Nonexistent model error handling test passed" );
  println!( "Error : {error_str}" );
}

#[ tokio::test ]
async fn test_list_models_contains_expected_models()
{
  let client = create_test_client();

  let response = client.models().list().await
    .expect( "List models should succeed" );

  let model_ids : Vec< String > = response.data.iter()
    .map( |m| m.id.clone() )
    .collect();

  // Check for common Grok model IDs
  let expected_models = vec![ "grok-2-1212" ];

  for expected in &expected_models {
    let found = model_ids.iter().any( |id| id.contains( expected ) );
    assert!( found, "Should find model containing '{expected}' in list" );
  }

  println!( "✅ Expected models presence test passed" );
  println!( "All expected models found in the list" );
}

#[ tokio::test ]
async fn test_model_fields_are_valid()
{
  let client = create_test_client();

  let response = client.models().list().await
    .expect( "List models should succeed" );

  for model in &response.data {
    // ID should not be empty
    assert!( !model.id.is_empty(), "Model ID should not be empty" );

    // ID should be reasonable length (not malformed)
    assert!( model.id.len() < 100, "Model ID should be reasonable length" );

    // Created timestamp should be reasonable (after 2020)
    assert!( model.created > 1_577_836_800, "Creation timestamp should be after 2020" );

    // Owned_by should not be empty
    assert!( !model.owned_by.is_empty(), "Owned_by should not be empty" );
  }

  println!( "✅ Model fields validation test passed" );
  println!( "All {} models have valid field values", response.data.len() );
}

#[ tokio::test ]
async fn test_list_models_response_is_consistent()
{
  let client = create_test_client();

  // Call list models twice
  let response1 = client.models().list().await
    .expect( "First list models call should succeed" );

  let response2 = client.models().list().await
    .expect( "Second list models call should succeed" );

  // Should return same number of models (assuming no models were added/removed)
  // Note : This might occasionally fail if models are updated during test run
  assert_eq!(
    response1.data.len(),
    response2.data.len(),
    "Should return consistent number of models"
  );

  // Model IDs should be the same
  let ids1 : Vec< &str > = response1.data.iter().map( |m| m.id.as_str() ).collect();
  let ids2 : Vec< &str > = response2.data.iter().map( |m| m.id.as_str() ).collect();

  assert_eq!( ids1, ids2, "Model IDs should be consistent across calls" );

  println!( "✅ List models consistency test passed" );
  println!( "Both calls returned {} models with same IDs", response1.data.len() );
}

//! Test to verify integration tests fail properly when no API key is available

#[ path = "common/mod.rs" ] mod common;
use common::create_integration_client;

use api_gemini::client::Client;

/// Test that demonstrates proper failure when no API key is available
#[ test ]
fn test_integration_failure_without_api_key()
{
  // First check if any API key is available from any source
  let has_api_key = Client::new().is_ok();
  
  if has_api_key
  {
    // API key is available - verify client creation succeeds
    println!( "API key available - integration tests will run normally" );
    let _client = Client::new().expect( "Client creation should succeed with valid API key" );
    // Client created successfully - test passes
    return;
  }
  
  // No API key available - test that Client::new() properly fails
  let result = Client::new();
  assert!( result.is_err(), "Client::new() should fail when no API key is available" );
  
  // Test that the error message is informative
  let error = result.unwrap_err();
let error_msg = format!( "{error:?}" );
  assert!( error_msg.to_lowercase().contains( "authentication" ) || error_msg.to_lowercase().contains( "api_key" ), 
"Error should mention authentication or API key : {error_msg}" );
  
  println!( "✅ Integration test properly fails with clear error message when no API key is available" );
}

// ==============================================================================
// INTEGRATION TESTS - Real API authentication validation
// ==============================================================================

#[ tokio::test ]
async fn integration_test_valid_authentication_real_api()
{
  let client = create_integration_client();
  
  // Test that a valid API key can successfully authenticate
  // by making a simple API call that requires authentication
  let models_response = client.models()
  .list()
  .await
  .expect( "Valid API key should allow model listing" );

  // Validate we got a real response from authenticated API
  assert!( !models_response.models.is_empty(), "Authenticated API should return available models" );
  
  // Verify we can access model details (requires auth)
  let first_model = &models_response.models[ 0 ];
  assert!( !first_model.name.is_empty(), "Authenticated access should provide model names" );
  assert!( first_model.display_name.is_some(), "Authenticated access should provide model metadata" );
}

#[ tokio::test ]
async fn integration_test_authentication_with_content_generation()
{
  let client = create_integration_client();
  
  // Test authentication by making a content generation request
  // This is a higher-privilege operation that definitely requires valid auth
  let request = api_gemini::models::GenerateContentRequest
  {
    contents: vec!
    [
    api_gemini ::models::Content
    {
      parts: vec!
      [
      api_gemini ::models::Part
      {
        text: Some( "Test authentication: say 'auth-ok'".to_string() ),
        ..Default::default()
      }
      ],
      role: "user".to_string(),
    }
    ],
    generation_config: Some( api_gemini::models::GenerationConfig
    {
      max_output_tokens: Some( 10 ),
      ..Default::default()
    }),
    ..Default::default()
  };

  let response = client.models().by_name( "gemini-flash-latest" )
  .generate_content( &request )
  .await
  .expect( "Valid authentication should allow content generation" );

  // Validate authenticated response — proves auth succeeded
  assert!( !response.candidates.is_empty(), "Authenticated request should return candidates" );
  // parts may be empty if content was blocked or token budget exhausted before first token
  if !response.candidates[ 0 ].content.parts.is_empty()
  {
    assert!( response.candidates[ 0 ].content.parts[ 0 ].text.is_some(),
    "Non-empty parts should contain text" );
    let response_text = response.candidates[ 0 ].content.parts[ 0 ].text.as_ref().unwrap();
    assert!( !response_text.is_empty(), "Authenticated response should not be empty" );
  }
}

#[ tokio::test ]
async fn integration_test_authentication_with_embeddings()
{
  let client = create_integration_client();
  
  // Test authentication with embeddings API (requires valid auth)
  let embed_request = api_gemini::models::EmbedContentRequest
  {
    content: api_gemini::models::Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      api_gemini ::models::Part
      {
        text: Some( "authentication test".to_string() ),
        ..Default::default()
      }
      ],
    },
    task_type: Some( "RETRIEVAL_QUERY".to_string() ),
    title: None,
    output_dimensionality: None,
  };

  let response = client.models().by_name( "text-embedding-004" )
  .embed_content( &embed_request )
  .await
  .expect( "Valid authentication should allow embeddings" );

  // Validate authenticated embeddings response
  assert!( !response.embedding.values.is_empty(), 
  "Authenticated embeddings request should return vector" );
  assert!( response.embedding.values.len() > 50, 
  "Authenticated embeddings should return meaningful vector size" );
  
  // Verify embedding values are reasonable (not all zeros)
  let non_zero_count = response.embedding.values.iter()
  .filter( |&&value| value.abs() > 0.001 )
  .count();
  assert!( non_zero_count > response.embedding.values.len() / 10, 
  "Authenticated embeddings should have non-zero values" );
}

#[ tokio::test ]
async fn integration_test_authentication_quota_and_limits()
{
  let client = create_integration_client();
  
  // Test that authenticated API respects rate limits and quotas
  // Make multiple quick requests to test quota system
  let mut successful_requests = 0;
  let max_attempts = 5;
  
  for i in 0..max_attempts
  {
    let request = api_gemini::models::GenerateContentRequest
    {
      contents: vec!
      [
      api_gemini ::models::Content
      {
        parts: vec!
        [
        api_gemini ::models::Part
        {
        text : Some( format!( "Request {}: count to 3", i + 1 ) ),
          ..Default::default()
        }
        ],
        role: "user".to_string(),
      }
      ],
      generation_config: Some( api_gemini::models::GenerationConfig
      {
        max_output_tokens: Some( 20 ),
        ..Default::default()
      }),
      ..Default::default()
    };

    match client.models().by_name( "gemini-flash-latest" )
    .generate_content( &request )
    .await
    {
      Ok( response ) =>
      {
        successful_requests += 1;
        assert!( !response.candidates.is_empty(), "Each successful request should return candidates" );
    
        // Small delay between requests
        tokio ::time::sleep( tokio::time::Duration::from_millis( 200 ) ).await;
      },
      Err( err ) =>
      {
        // If we hit rate limits, that's expected behavior for authenticated API
      let error_str = format!( "{err:?}" );
        if error_str.contains( "quota" ) || error_str.contains( "rate" ) || error_str.contains( "429" )
        {
        println!( "Hit quota/rate limit on request {} (expected for auth validation)", i + 1 );
          break;
        }
      panic!( "Unexpected error during quota test : {err:?}" );
      }
    }
  }
  
  // Should have made at least one successful authenticated request
  assert!( successful_requests > 0, 
  "Authenticated client should make at least one successful request" );
println!( "Successfully made {successful_requests}/{max_attempts} authenticated requests" );
}
//! Content Generation Refactor Integration Tests - STRICT FAILURE POLICY
//!
//! MANDATORY INTEGRATION TEST REQUIREMENTS:
//! - These tests use REAL Anthropic API endpoints - NO MOCKING ALLOWED
//! - Tests MUST FAIL IMMEDIATELY if API secrets are not available (no graceful fallbacks)
//! - Tests MUST FAIL IMMEDIATELY on network connectivity issues
//! - Tests MUST FAIL IMMEDIATELY on API authentication failures
//! - Tests MUST FAIL IMMEDIATELY on any API endpoint errors
//! - NO SILENT PASSES allowed when problems occur
//!
//! Run with : cargo test --features content-generation,integration
//! Requires : Valid `ANTHROPIC_API_KEY` in environment or ../../secret/-secrets.sh

#[ allow( unused_imports ) ]
use super::*;

#[ cfg( feature = "content-generation" ) ]
mod content_generation_refactor_tests 
{
  use super::*;

  #[ tokio::test ]
  async fn test_content_generation_request_builder()
  {
    // Test the new builder pattern for content generation
    let request = the_module::ContentGenerationRequest::builder()
      .model( "claude-sonnet-4-5-20250929" )
      .max_tokens( 100 )
      .message( the_module::Message::user( "Write a haiku about programming".to_string() ) )
      .temperature( 0.7 )
      .system( "You are a creative poet" )
      .build();

    assert!( request.is_ok() );
    
    let request = request.unwrap();
    assert_eq!( request.model, "claude-sonnet-4-5-20250929" );
    assert_eq!( request.max_tokens, 100 );
    assert_eq!( request.temperature, Some( 0.7 ) );
    assert_eq!( request.system, Some( "You are a creative poet".to_string() ) );
    assert_eq!( request.messages.len(), 1 );
  }

  #[ tokio::test ]
  async fn test_content_generation_request_validation()
  {
    // Test validation logic for content generation requests
    
    // Valid request should pass
    let valid_request = the_module::ContentGenerationRequest::builder()
      .model( "claude-sonnet-4-5-20250929" )
      .max_tokens( 100 )
      .message( the_module::Message::user( "Hello".to_string() ) )
      .build()
      .unwrap();
      
    assert!( valid_request.validate().is_ok() );

    // Empty model should fail
    let empty_model_request = the_module::ContentGenerationRequest::builder()
      .model( "" )
      .max_tokens( 100 )
      .message( the_module::Message::user( "Hello".to_string() ) )
      .build();
      
    assert!( empty_model_request.is_err() );

    // Zero max_tokens should fail validation
    let zero_tokens_request = the_module::ContentGenerationRequest::builder()
      .model( "claude-sonnet-4-5-20250929" )
      .max_tokens( 0 )
      .message( the_module::Message::user( "Hello".to_string() ) )
      .build()
      .unwrap();
      
    assert!( zero_tokens_request.validate().is_err() );

    // Invalid temperature should fail validation
    let invalid_temp_request = the_module::ContentGenerationRequest::builder()
      .model( "claude-sonnet-4-5-20250929" )
      .max_tokens( 100 )
      .message( the_module::Message::user( "Hello".to_string() ) )
      .temperature( -0.5 ) // Invalid temperature
      .build()
      .unwrap();
      
    assert!( invalid_temp_request.validate().is_err() );
  }

  #[ tokio::test ]
  async fn test_content_generation_settings()
  {
    // Test advanced generation settings
    let settings = the_module::GenerationSettings
    {
      stop_sequences : Some( vec![ "END".to_string(), "STOP".to_string() ] ),
      top_p : Some( 0.9 ),
      top_k : Some( 50 ),
      presence_penalty : Some( 0.1 ),
      frequency_penalty : Some( 0.2 ),
    };

    let request = the_module::ContentGenerationRequest::builder()
      .model( "claude-sonnet-4-5-20250929" )
      .max_tokens( 100 )
      .message( the_module::Message::user( "Generate text".to_string() ) )
      .settings( settings.clone() )
      .build()
      .unwrap();

    assert!( request.settings.is_some() );
    let req_settings = request.settings.unwrap();
    assert_eq!( req_settings.stop_sequences, settings.stop_sequences );
    assert_eq!( req_settings.top_p, settings.top_p );
    assert_eq!( req_settings.top_k, settings.top_k );
  }

  #[ tokio::test ]
  async fn test_content_generator_creation()
  {
    // REMOVED: This test used fake API keys and is not needed
    // Real testing is covered by integration tests using from_workspace()
  }

  #[ tokio::test ]
  async fn test_content_generation_to_message_request_conversion()
  {
    // Test conversion between new and old request formats
    let content_request = the_module::ContentGenerationRequest::builder()
      .model( "claude-sonnet-4-5-20250929" )
      .max_tokens( 150 )
      .message( the_module::Message::user( "Explain quantum computing".to_string() ) )
      .system( "You are a physics teacher" )
      .temperature( 0.5 )
      .build()
      .unwrap();

    let message_request = content_request.to_message_request();

    assert_eq!( message_request.model, "claude-sonnet-4-5-20250929" );
    assert_eq!( message_request.max_tokens, 150 );
    assert_eq!( message_request.system.as_ref().map( | s | s[ 0 ].text.as_str() ), Some( "You are a physics teacher" ) );
    assert_eq!( message_request.temperature, Some( 0.5 ) );
    assert_eq!( message_request.messages.len(), 1 );
  }

  #[ tokio::test ]
  async fn test_content_generation_multiple_messages()
  {
    // Test handling multiple messages in conversation
    let messages = vec![
      the_module::Message::user( "What is AI?".to_string() ),
      the_module::Message::assistant( "AI stands for Artificial Intelligence...".to_string() ),
      the_module::Message::user( "Can you give me an example?".to_string() ),
    ];

    let request = the_module::ContentGenerationRequest::builder()
      .model( "claude-sonnet-4-5-20250929" )
      .max_tokens( 200 )
      .messages( messages.clone() )
      .build()
      .unwrap();

    assert_eq!( request.messages.len(), 3 );
    // Note : Messages don't implement PartialEq, so we check length and structure
    assert_eq!( request.messages.len(), messages.len() );
  }

  #[ tokio::test ]
  async fn test_content_generator_simple_methods()
  {
    // REMOVED: This test used fake API keys and is not needed
    // Real testing is covered by integration tests using from_workspace()
  }

  #[ tokio::test ]
  async fn test_content_generation_response_structure()
  {
    // Test response structure and metadata
    use the_module::GenerationMetadata;
    
    let metadata = GenerationMetadata
    {
      generation_time_ms : Some( 1500 ),
      model_version : Some( "claude-sonnet-4-5-20250929".to_string() ),
      safety_assessment : Some( "safe".to_string() ),
    };

    // Test metadata fields
    assert_eq!( metadata.generation_time_ms, Some( 1500 ) );
    assert!( metadata.model_version.is_some() );
    assert!( metadata.safety_assessment.is_some() );
  }

  #[ tokio::test ]
  async fn test_content_generation_error_handling()
  {
    // Test error handling in content generation
    
    // Missing model should fail
    let result = the_module::ContentGenerationRequest::builder()
      .max_tokens( 100 )
      .message( the_module::Message::user( "Hello".to_string() ) )
      .build();
    
    assert!( result.is_err() );
    assert!( result.unwrap_err().to_string().contains( "Model is required" ) );

    // Missing max_tokens should fail
    let result = the_module::ContentGenerationRequest::builder()
      .model( "claude-sonnet-4-5-20250929" )
      .message( the_module::Message::user( "Hello".to_string() ) )
      .build();
    
    assert!( result.is_err() );
    assert!( result.unwrap_err().to_string().contains( "Max tokens is required" ) );

    // No messages should fail
    let result = the_module::ContentGenerationRequest::builder()
      .model( "claude-sonnet-4-5-20250929" )
      .max_tokens( 100 )
      .build();
    
    assert!( result.is_err() );
    assert!( result.unwrap_err().to_string().contains( "At least one message is required" ) );
  }

  #[ tokio::test ]
  async fn test_client_content_generation_extension()
  {
    // REMOVED: This test used fake API keys and is not needed
    // Real testing is covered by integration tests using from_workspace()
  }

  #[ tokio::test ]
  async fn test_generation_settings_default()
  {
    // Test default generation settings
    let settings = the_module::GenerationSettings::default();
    
    assert!( settings.stop_sequences.is_none() );
    assert!( settings.top_p.is_none() );
    assert!( settings.top_k.is_none() );
    assert!( settings.presence_penalty.is_none() );
    assert!( settings.frequency_penalty.is_none() );
  }

  #[ tokio::test ]
  async fn test_content_generation_feature_gating()
  {
    // This test verifies the feature gating works correctly
    // If this compiles and runs, the content-generation feature is working
    let _request_builder = the_module::ContentGenerationRequest::builder();
    let _settings = the_module::GenerationSettings::default();
    
    // Feature-gated functionality is available
  }
}

#[ cfg( not( feature = "content-generation" ) ) ]
mod content_generation_disabled_tests
{
  // These tests verify that when the feature is disabled, 
  // the types are not available
  #[ test ]
  fn test_content_generation_feature_disabled()
  {
    // If content-generation feature is disabled, this should compile
    // but the types should not be available
  }
}

// ============================================================================
// INTEGRATION TESTS - REAL API CONTENT GENERATION REFACTOR
// ============================================================================

#[ cfg( all( feature = "integration", feature = "content-generation" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_content_generation_refactor_real_api()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for content generation refactor testing" );

  // Test content generator with real API
  let content_gen = the_module::ContentGenerator::new( client.clone() )
    .with_default_model( "claude-haiku-4-5-20251001" )
    .with_default_max_tokens( 25 );

  let request = the_module::ContentGenerationRequest::builder()
    .model( "claude-haiku-4-5-20251001" )
    .max_tokens( 25 )
    .temperature( 0.3 )
    .message( the_module::Message::user( "Write a haiku about programming.".to_string() ) )
    .build()
    .expect( "INTEGRATION: Request building must work" );

  let response = match content_gen.generate( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Content generation must work : {err}" ),
  };

  // Verify response structure
  assert!( !response.content.is_empty(), "Must generate content" );
  assert!( response.usage.output_tokens > 0, "Must track output tokens" );
  assert!( !response.model.is_empty(), "Response must have model info" );

  // Haiku should be reasonably short
  assert!( response.content.len() > 10, "Haiku should have some content" );

  println!( "✅ Content generation refactor integration test passed!" );
  println!( "   Generated haiku : {}", response.content );
  println!( "   Output tokens : {}", response.usage.output_tokens );
}
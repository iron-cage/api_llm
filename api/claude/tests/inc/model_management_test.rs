//! Model Management Integration Tests - STRICT FAILURE POLICY
//!
//! MANDATORY INTEGRATION TEST REQUIREMENTS:
//! - These tests use REAL Anthropic API endpoints - NO MOCKING ALLOWED
//! - Tests MUST FAIL IMMEDIATELY if API secrets are not available (no graceful fallbacks)
//! - Tests MUST FAIL IMMEDIATELY on network connectivity issues
//! - Tests MUST FAIL IMMEDIATELY on API authentication failures
//! - Tests MUST FAIL IMMEDIATELY on any API endpoint errors
//! - NO SILENT PASSES allowed when problems occur
//!
//! Run with : cargo test --features integration
//! Requires : Valid `ANTHROPIC_API_KEY` in environment or ../../secret/-secrets.sh

#[ allow( unused_imports ) ]
use super::*;

// TODO: Add model validation structure tests here when model management API is implemented
// TODO: Add model capability structure tests here when model management API is implemented
// TODO: Add model parameter validation tests here when model management API is implemented

// ============================================================================
// INTEGRATION TESTS - REAL API MODEL MANAGEMENT  
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_model_management_real_model_validation()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for model management testing" );

  // Test with known valid models
  let valid_models = vec![
    "claude-sonnet-4-5-20250929",
    "claude-3-5-haiku-20241022", 
    "claude-3-opus-20240229",
  ];

  for model_name in valid_models
  {
    let request = the_module::CreateMessageRequest
    {
      model : model_name.to_string(),
      max_tokens : 5,
      messages : vec![ the_module::Message::user( "Hi".to_string() ) ],
      system : None,
      temperature : None,
      stream : None,
      tools : None,
      tool_choice : None,
    };

    let response = match client.create_message( request ).await
    {
      Ok( response ) => response,
      Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
        panic!( "INTEGRATION: Credit balance exhausted — top up account to run tests : {}", api_err.message ),
      Err( err ) => panic!( "INTEGRATION: Valid model {model_name} must work : {err}" ),
    };

    // Verify response uses the requested model
    assert_eq!( response.model, model_name, "Response model must match request" );
    assert!( !response.id.is_empty(), "Response must have valid ID" );
    assert!( response.usage.input_tokens > 0, "Must track token usage" );
    assert!( response.usage.output_tokens > 0, "Must generate tokens" );
    
    println!( "✅ Model {model_name} validation passed" );
    println!( "   Response ID: {}", response.id );
    println!( "   Tokens : {} in, {} out", response.usage.input_tokens, response.usage.output_tokens );
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_model_management_invalid_model_handling()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for invalid model testing" );

  // Test with known invalid models
  let invalid_models = vec![
    "claude-999-invalid",
    "gpt-4", // Wrong provider
    "claude-3-5-sonnet-invalid-date",
  ];

  for model_name in invalid_models
  {
    let request = the_module::CreateMessageRequest
    {
      model : model_name.to_string(),
      max_tokens : 5,
      messages : vec![ the_module::Message::user( "Hi".to_string() ) ],
      system : None,
      temperature : None,
      stream : None,
      tools : None,
      tool_choice : None,
    };

    let result = client.create_message( request ).await;
    
    // Invalid models should return errors
    assert!( result.is_err(), "Invalid model {model_name} should return error" );
    let error = result.unwrap_err();
    let error_str = error.to_string().to_lowercase();
    assert!( 
      error_str.contains( "model" ) || error_str.contains( "invalid" ) || error_str.contains( "not found" ),
      "Error for invalid model {model_name} should mention model issue : {error}"
    );
    
    println!( "✅ Invalid model {model_name} properly rejected : {error}" );
  }
}

#[ cfg( all( feature = "integration", feature = "tools" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_model_management_capability_validation()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for capability testing" );

  // Test tool-capable model
  let tool_request = the_module::CreateMessageRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(), // Tool-capable model
    max_tokens : 50,
    messages : vec![ the_module::Message::user( "What's 5 * 7?".to_string() ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : Some( vec![ 
      the_module::ToolDefinition::simple( "calculator", "Calculate mathematical expressions" ) 
    ] ),
    tool_choice : None,
  };

  let response = match client.create_message( tool_request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Tool-capable model must handle tools : {err}" ),
  };

  // Verify the model can handle tools
  assert!( !response.id.is_empty() );
  assert!( response.usage.input_tokens > 0 );
  
  // The response might contain tool usage or text response - both are valid
  assert!( !response.content.is_empty(), "Tool-capable model must return content" );
  
  println!( "✅ Model capability validation passed!" );
  println!( "   Tool-capable model processed request successfully" );
  println!( "   Response has {} content blocks", response.content.len() );
}
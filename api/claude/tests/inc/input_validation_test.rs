//! Input Validation Tests
//!
//! Comprehensive tests for input validation framework covering request validation,
//! parameter bounds checking, and early error detection before API calls.

#[ allow( unused_imports ) ]
use super::*;

// ============================================================================
// UNIT TESTS - PARAMETER BOUNDS VALIDATION
// ============================================================================

#[ test ]
fn test_model_validation_empty()
{
  // Test that empty model name is rejected
  let request = the_module::CreateMessageRequest
  {
    model : String::new(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : None,
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let result = request.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "Model cannot be empty" ) );
}

#[ test ]
fn test_model_validation_whitespace()
{
  // Test that whitespace-only model name is rejected
  let request = the_module::CreateMessageRequest
  {
    model : "   ".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : None,
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let result = request.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "Model cannot be empty" ) );
}

#[ test ]
fn test_max_tokens_below_minimum()
{
  // Test that max_tokens below minimum is rejected
  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 0,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : None,
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let result = request.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "max_tokens must be between" ) );
}

#[ test ]
fn test_max_tokens_above_maximum()
{
  // Test that max_tokens above maximum is rejected
  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 300_000,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : None,
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let result = request.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "max_tokens must be between" ) );
}

#[ test ]
fn test_max_tokens_at_boundaries()
{
  // Test that max_tokens at valid boundaries is accepted
  let request_min = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 1,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : None,
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let request_max = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 200_000,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : None,
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  assert!( request_min.validate().is_ok() );
  assert!( request_max.validate().is_ok() );
}

#[ test ]
fn test_messages_empty()
{
  // Test that empty messages array is rejected
  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![],
    system : None,
    temperature : None,
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let result = request.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "At least one message is required" ) );
}

#[ test ]
fn test_temperature_below_minimum()
{
  // Test that temperature below minimum is rejected
  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : Some( -0.1 ),
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let result = request.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "Temperature must be between" ) );
}

#[ test ]
fn test_temperature_above_maximum()
{
  // Test that temperature above maximum is rejected
  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : Some( 1.5 ),
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let result = request.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "Temperature must be between" ) );
}

#[ test ]
fn test_temperature_at_boundaries()
{
  // Test that temperature at valid boundaries is accepted
  let request_min = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : Some( 0.0 ),
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let request_max = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : Some( 1.0 ),
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  assert!( request_min.validate().is_ok() );
  assert!( request_max.validate().is_ok() );
}

#[ test ]
fn test_valid_request()
{
  // Test that a valid request passes validation
  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 1000,
    messages : vec!
    [
      the_module::Message::user( "Hello!" ),
      the_module::Message::assistant( "Hi there!" ),
      the_module::Message::user( "How are you?" ),
    ],
    system : Some( vec![ the_module::SystemContent::text( "You are helpful" ) ] ),
    temperature : Some( 0.7 ),
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  assert!( request.validate().is_ok() );
}

// ============================================================================
// UNIT TESTS - TOOL VALIDATION
// ============================================================================

#[ test ]
#[ cfg( feature = "tools" ) ]
fn test_tool_choice_without_tools()
{
  // Test that tool_choice without tools is rejected
  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : None,
    tool_choice : Some( the_module::ToolChoice::Auto ),
  };

  let result = request.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "tool_choice specified but no tools provided" ) );
}

#[ test ]
#[ cfg( feature = "tools" ) ]
fn test_tool_choice_unknown_tool()
{
  // Test that tool_choice referencing unknown tool is rejected
  let tool = the_module::ToolDefinition
  {
    name : "calculator".to_string(),
    description : "Perform calculations".to_string(),
    input_schema : serde_json::json!( { "type" : "object" } ),
  };

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : Some( vec![ tool ] ),
    tool_choice : Some( the_module::ToolChoice::specific( "unknown_tool" ) ),
  };

  let result = request.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "tool_choice references unknown tool" ) );
}

#[ test ]
#[ cfg( feature = "tools" ) ]
fn test_empty_tools_array()
{
  // Test that empty tools array is rejected
  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : Some( vec![] ),
    tool_choice : None,
  };

  let result = request.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "tools array cannot be empty" ) );
}

#[ test ]
#[ cfg( feature = "tools" ) ]
fn test_tool_with_empty_name()
{
  // Test that tool with empty name is rejected
  let tool = the_module::ToolDefinition
  {
    name : String::new(),
    description : "A tool".to_string(),
    input_schema : serde_json::json!( { "type" : "object" } ),
  };

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : Some( vec![ tool ] ),
    tool_choice : None,
  };

  let result = request.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "tool name cannot be empty" ) );
}

#[ test ]
#[ cfg( feature = "tools" ) ]
fn test_duplicate_tool_names()
{
  // Test that duplicate tool names are rejected
  let tool1 = the_module::ToolDefinition
  {
    name : "calculator".to_string(),
    description : "First calculator".to_string(),
    input_schema : serde_json::json!( { "type" : "object" } ),
  };

  let tool2 = the_module::ToolDefinition
  {
    name : "calculator".to_string(),
    description : "Second calculator".to_string(),
    input_schema : serde_json::json!( { "type" : "object" } ),
  };

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : Some( vec![ tool1, tool2 ] ),
    tool_choice : None,
  };

  let result = request.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "duplicate tool name" ) );
}

#[ test ]
#[ cfg( feature = "tools" ) ]
fn test_tool_with_empty_description()
{
  // Test that tool with empty description is rejected
  let tool = the_module::ToolDefinition
  {
    name : "calculator".to_string(),
    description : String::new(),
    input_schema : serde_json::json!( { "type" : "object" } ),
  };

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : Some( vec![ tool ] ),
    tool_choice : None,
  };

  let result = request.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "description cannot be empty" ) );
}

#[ test ]
#[ cfg( feature = "tools" ) ]
fn test_too_many_tools()
{
  // Test that more than 64 tools is rejected
  let tools : Vec< the_module::ToolDefinition > = ( 0..65 ).map( |i|
  {
    the_module::ToolDefinition
    {
      name : format!( "tool_{i}" ),
      description : format!( "Tool number {i}" ),
      input_schema : serde_json::json!( { "type" : "object" } ),
    }
  } ).collect();

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : Some( tools ),
    tool_choice : None,
  };

  let result = request.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "maximum of 64 tools" ) );
}

#[ test ]
#[ cfg( feature = "tools" ) ]
fn test_valid_tools()
{
  // Test that valid tools pass validation
  let tool = the_module::ToolDefinition
  {
    name : "calculator".to_string(),
    description : "Perform mathematical calculations".to_string(),
    input_schema : serde_json::json!
    ({
      "type" : "object",
      "properties" :
      {
        "expression" : { "type" : "string" }
      }
    }),
  };

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Calculate 2 + 2" ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : Some( vec![ tool ] ),
    tool_choice : Some( the_module::ToolChoice::specific( "calculator" ) ),
  };

  assert!( request.validate().is_ok() );
}

// ============================================================================
// UNIT TESTS - SYSTEM CONTENT VALIDATION
// ============================================================================

#[ test ]
fn test_system_content_empty_text()
{
  // Test that empty system content text is rejected
  let content = the_module::SystemContent
  {
    r#type : "text".to_string(),
    text : String::new(),
    cache_control : None,
  };

  let result = content.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().contains( "text cannot be empty" ) );
}

#[ test ]
fn test_system_content_invalid_type()
{
  // Test that invalid system content type is rejected
  let content = the_module::SystemContent
  {
    r#type : "image".to_string(),
    text : "Some text".to_string(),
    cache_control : None,
  };

  let result = content.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().contains( "Invalid system content type" ) );
}

#[ test ]
fn test_system_content_valid()
{
  // Test that valid system content passes validation
  let content = the_module::SystemContent::text( "You are a helpful assistant" );

  assert!( content.validate().is_ok() );
}

#[ test ]
fn test_system_instructions_empty()
{
  // Test that empty system instructions are rejected
  let instructions = the_module::SystemInstructions::new();

  let result = instructions.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().contains( "cannot be empty" ) );
}

#[ test ]
fn test_system_instructions_with_invalid_content()
{
  // Test that system instructions with invalid content are rejected
  let empty_content = the_module::SystemContent
  {
    r#type : "text".to_string(),
    text : String::new(),
    cache_control : None,
  };

  let instructions = the_module::SystemInstructions::new()
    .add( empty_content );

  let result = instructions.validate();

  assert!( result.is_err() );
  assert!( result.unwrap_err().contains( "Invalid content at index" ) );
}

#[ test ]
fn test_system_instructions_valid()
{
  // Test that valid system instructions pass validation
  let instructions = the_module::SystemInstructions::new()
    .add_text( "You are a helpful assistant" )
    .add_cached_text( "Knowledge base content" )
    .add_text( "Help the user" );

  assert!( instructions.validate().is_ok() );
}

// ============================================================================
// INTEGRATION TESTS - VALIDATION BEFORE API CALLS
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_validation_prevents_invalid_requests()
{
  // Test that validation prevents invalid requests from reaching the API
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let invalid_request = the_module::CreateMessageRequest
  {
    model : String::new(), // Invalid empty model
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Hello" ) ],
    system : None,
    temperature : None,
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  // Validate before sending - should fail
  let validation_result = invalid_request.validate();
  assert!( validation_result.is_err() );

  // Attempt to send anyway - client might also validate
  let response = client.create_message( invalid_request ).await;
  assert!( response.is_err() );

  println!( "✅ Validation prevents invalid requests!" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_validation_allows_valid_requests()
{
  // Test that validation allows valid requests
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let valid_request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 50,
    messages : vec![ the_module::Message::user( "Say hello!" ) ],
    system : None,
    temperature : Some( 0.0 ),
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  // Validate before sending - should pass
  let validation_result = valid_request.validate();
  assert!( validation_result.is_ok() );

  // Send request - should work
  let response = match client.create_message( valid_request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Valid request must work : {err}" ),
  };

  assert!( !response.id.is_empty() );

  println!( "✅ Validation allows valid requests!" );
}

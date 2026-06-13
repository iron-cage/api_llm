//! Enhanced Function Calling Tests
//!
//! Tests for advanced tool choice modes : AUTO, ANY, NONE

#[ allow( unused_imports ) ]
use super::*;

// ============================================================================
// UNIT TESTS - TOOL CHOICE MODES
// ============================================================================

#[ test ]
fn test_tool_choice_auto()
{
  // Test AUTO mode - model decides when to use tools
  let choice = the_module::ToolChoice::Auto;

  assert!( choice.is_auto() );
  assert!( !choice.is_none() );
  assert!( !choice.is_specific() );
}

#[ test ]
fn test_tool_choice_none()
{
  // Test NONE mode - disable all tool calling
  let choice = the_module::ToolChoice::None;

  assert!( choice.is_none() );
  assert!( !choice.is_auto() );
  assert!( !choice.is_specific() );
}

#[ test ]
fn test_tool_choice_any()
{
  // Test ANY mode - force use of any available tool
  let choice = the_module::ToolChoice::Any;

  assert!( choice.is_any() );
  assert!( !choice.is_auto() );
  assert!( !choice.is_none() );
  assert!( !choice.is_specific() );
}

#[ test ]
fn test_tool_choice_specific()
{
  // Test specific tool choice
  let choice = the_module::ToolChoice::specific( "calculator" );

  assert!( choice.is_specific() );
  assert!( !choice.is_auto() );
  assert!( !choice.is_none() );
  assert!( !choice.is_any() );
  assert_eq!( choice.tool_name(), Some( "calculator" ) );
}

#[ test ]
fn test_tool_choice_serialization()
{
  // Test that tool choices serialize to correct API format
  let auto = the_module::ToolChoice::Auto;
  let none = the_module::ToolChoice::None;
  let any = the_module::ToolChoice::Any;
  let specific = the_module::ToolChoice::specific( "get_weather" );

  let auto_json = serde_json::to_value( &auto ).unwrap();
  let none_json = serde_json::to_value( &none ).unwrap();
  let any_json = serde_json::to_value( &any ).unwrap();
  let specific_json = serde_json::to_value( &specific ).unwrap();

  assert_eq!( auto_json[ "type" ], "auto" );
  assert_eq!( none_json[ "type" ], "none" );
  assert_eq!( any_json[ "type" ], "any" );
  assert_eq!( specific_json[ "type" ], "tool" );
  assert_eq!( specific_json[ "name" ], "get_weather" );
}

// ============================================================================
// INTEGRATION TESTS - REAL API TOOL CHOICE MODES
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_tool_choice_auto_mode()
{
  // Test AUTO mode - model decides whether to use tools
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let tool = the_module::ToolDefinition
  {
    name : "calculator".to_string(),
    description : "Perform basic arithmetic operations".to_string(),
    input_schema : serde_json::json!({
      "type": "object",
      "properties": {
        "operation": { "type": "string", "enum": ["add", "subtract", "multiply", "divide"] },
        "a": { "type": "number" },
        "b": { "type": "number" }
      },
      "required": ["operation", "a", "b"]
    }),
  };

  let request = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "What is 25 + 17?".to_string() ) ],
    system : None,
    temperature : Some( 0.0 ),
    stream : None,
    tools : Some( vec![ tool ] ),
    tool_choice : Some( the_module::ToolChoice::Auto ),
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: AUTO mode must work : {err}" ),
  };

  // With AUTO mode, model should decide to use calculator for math
  assert!( !response.id.is_empty() );

  println!( "✅ Tool choice AUTO mode test passed!" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_tool_choice_any_mode()
{
  // Test ANY mode - force use of any available tool
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let tool = the_module::ToolDefinition
  {
    name : "get_info".to_string(),
    description : "Get information about a topic".to_string(),
    input_schema : serde_json::json!({
      "type": "object",
      "properties": {
        "topic": { "type": "string" }
      },
      "required": ["topic"]
    }),
  };

  let request = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Tell me about Rust".to_string() ) ],
    system : None,
    temperature : Some( 0.0 ),
    stream : None,
    tools : Some( vec![ tool ] ),
    tool_choice : Some( the_module::ToolChoice::Any ),
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: ANY mode must work : {err}" ),
  };

  // With ANY mode, model must use at least one tool
  assert!( !response.id.is_empty() );

  // Check that model used a tool (check type field in response content)
  let has_tool_use = response.content.iter().any( |c| c.r#type == "tool_use" );
  assert!( has_tool_use, "INTEGRATION: ANY mode must force tool use" );

  println!( "✅ Tool choice ANY mode test passed!" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_tool_choice_none_mode()
{
  // Test NONE mode - disable all tool calling
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let tool = the_module::ToolDefinition
  {
    name : "calculator".to_string(),
    description : "Perform calculations".to_string(),
    input_schema : serde_json::json!({
      "type": "object",
      "properties": {
        "expression": { "type": "string" }
      },
      "required": ["expression"]
    }),
  };

  let request = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Calculate 10 + 5".to_string() ) ],
    system : None,
    temperature : Some( 0.0 ),
    stream : None,
    tools : Some( vec![ tool ] ),
    tool_choice : Some( the_module::ToolChoice::None ),
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: NONE mode must work : {err}" ),
  };

  // With NONE mode, model should not use tools
  assert!( !response.id.is_empty() );

  // Check that no tools were used (check type field in response content)
  let has_tool_use = response.content.iter().any( |c| c.r#type == "tool_use" );
  assert!( !has_tool_use, "INTEGRATION: NONE mode must prevent tool use" );

  println!( "✅ Tool choice NONE mode test passed!" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_tool_choice_mode_transitions()
{
  // Test switching between different tool choice modes
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let tool = the_module::ToolDefinition::simple( "test_tool", "A test tool" );

  // First request with AUTO
  let request1 = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    max_tokens : 50,
    messages : vec![ the_module::Message::user( "Hello".to_string() ) ],
    system : None,
    temperature : Some( 0.0 ),
    stream : None,
    tools : Some( vec![ tool.clone() ] ),
    tool_choice : Some( the_module::ToolChoice::Auto ),
  };

  let response1 = match client.create_message( request1 ).await
  {
    Ok( r ) => r,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: First request must work : {err}" ),
  };

  assert!( !response1.id.is_empty() );

  // Second request with NONE
  let request2 = the_module::CreateMessageRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    max_tokens : 50,
    messages : vec![ the_module::Message::user( "Hi again".to_string() ) ],
    system : None,
    temperature : Some( 0.0 ),
    stream : None,
    tools : Some( vec![ tool ] ),
    tool_choice : Some( the_module::ToolChoice::None ),
  };

  let response2 = match client.create_message( request2 ).await
  {
    Ok( r ) => r,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted on second request - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Second request must work : {err}" ),
  };

  assert!( !response2.id.is_empty() );
  assert_ne!( response1.id, response2.id, "Responses should have different IDs" );

  println!( "✅ Tool choice mode transitions test passed!" );
}

// ============================================================================
// UNIT TESTS - TOOL EXECUTOR AND REGISTRY
// (Migrated from src/enhanced_function_calling.rs #[cfg(test)] block)
// ============================================================================

struct RegistryTestTool
{
  name : String,
  description : String,
}

impl the_module::ToolExecutor for RegistryTestTool
{
  fn name( &self ) -> &str
  {
    &self.name
  }

  fn description( &self ) -> &str
  {
    &self.description
  }

  fn parameter_schema( &self ) -> serde_json::Value
  {
    serde_json::json!(
    {
      "type" : "object",
      "properties" :
      {
        "input" :
        {
          "type" : "string",
          "description" : "Test input"
        }
      },
      "required" : [ "input" ]
    })
  }

  fn execute( &self, params : serde_json::Value ) -> the_module::ToolResult
  {
    let input = params[ "input" ].as_str()
      .ok_or( "Missing input parameter" )?;
    Ok( format!( "Executed {} with: {}", self.name, input ) )
  }
}

#[ test ]
fn test_tool_registry_new()
{
  let registry = the_module::ToolRegistry::new();
  assert!( registry.is_empty() );
  assert_eq!( registry.len(), 0 );
}

#[ test ]
fn test_tool_registry_register()
{
  let mut registry = the_module::ToolRegistry::new();

  let tool = Box::new( RegistryTestTool
  {
    name : "test_tool".to_string(),
    description : "Test tool".to_string(),
  } );

  registry.register( tool );

  assert!( !registry.is_empty() );
  assert_eq!( registry.len(), 1 );
  assert!( registry.has_tool( "test_tool" ) );
}

#[ test ]
fn test_tool_registry_execute()
{
  let mut registry = the_module::ToolRegistry::new();

  let tool = Box::new( RegistryTestTool
  {
    name : "test_tool".to_string(),
    description : "Test tool".to_string(),
  } );

  registry.register( tool );

  let params = serde_json::json!( { "input" : "test value" } );
  let result = registry.execute( "test_tool", params );

  assert!( result.is_ok() );
  let output = result.unwrap();
  assert!( output.contains( "test_tool" ) );
  assert!( output.contains( "test value" ) );
}

#[ test ]
fn test_tool_registry_execute_not_found()
{
  let registry = the_module::ToolRegistry::new();
  let params = serde_json::json!( { "input" : "test" } );
  let result = registry.execute( "nonexistent", params );

  assert!( result.is_err() );
  let err = result.unwrap_err();
  assert!( err.contains( "not found" ) );
}

#[ test ]
fn test_tool_registry_definitions()
{
  let mut registry = the_module::ToolRegistry::new();

  let tool = Box::new( RegistryTestTool
  {
    name : "test_tool".to_string(),
    description : "Test tool description".to_string(),
  } );

  registry.register( tool );

  let definitions = registry.definitions();
  assert_eq!( definitions.len(), 1 );
  assert_eq!( definitions[ 0 ].name, "test_tool" );
  assert_eq!( definitions[ 0 ].description, "Test tool description" );
}

#[ test ]
fn test_tool_registry_tool_names()
{
  let mut registry = the_module::ToolRegistry::new();

  registry.register( Box::new( RegistryTestTool
  {
    name : "tool1".to_string(),
    description : "Tool 1".to_string(),
  } ) );

  registry.register( Box::new( RegistryTestTool
  {
    name : "tool2".to_string(),
    description : "Tool 2".to_string(),
  } ) );

  let names = registry.tool_names();
  assert_eq!( names.len(), 2 );
  assert!( names.contains( &"tool1".to_string() ) );
  assert!( names.contains( &"tool2".to_string() ) );
}

#[ test ]
fn test_create_tool_definition()
{
  let schema = serde_json::json!( { "type" : "object" } );
  let tool_def = the_module::create_tool_definition( "test", "Test tool", schema.clone() );

  assert_eq!( tool_def.name, "test" );
  assert_eq!( tool_def.description, "Test tool" );
  assert_eq!( tool_def.input_schema, schema );
}

#[ test ]
fn test_create_parameter_schema()
{
  let properties = serde_json::json!(
  {
    "location" :
    {
      "type" : "string",
      "description" : "City name"
    }
  });

  let schema = the_module::create_parameter_schema( &properties, &[ "location".to_string() ] );

  assert_eq!( schema[ "type" ], "object" );
  assert!( schema[ "properties" ][ "location" ].is_object() );
  assert_eq!( schema[ "required" ][ 0 ], "location" );
}

#[ test ]
fn test_tool_executor_default_schema()
{
  use the_module::ToolExecutor;
  struct MinimalTool;

  impl the_module::ToolExecutor for MinimalTool
  {
    fn name( &self ) -> &'static str { "minimal" }
    fn description( &self ) -> &'static str { "Minimal tool" }
    fn execute( &self, _params : serde_json::Value ) -> the_module::ToolResult
    {
      Ok( "done".to_string() )
    }
  }

  let tool = MinimalTool;
  let schema = tool.parameter_schema();

  assert_eq!( schema[ "type" ], "object" );
  assert!( schema[ "properties" ].is_object() );
}

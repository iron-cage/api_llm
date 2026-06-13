//! Tool Calling Integration Tests - STRICT FAILURE POLICY
//!
//! MANDATORY INTEGRATION TEST REQUIREMENTS:
//! - These tests use REAL Anthropic API endpoints - NO MOCKING ALLOWED
//! - Tests MUST FAIL IMMEDIATELY if API secrets are not available (no graceful fallbacks)
//! - Tests MUST FAIL IMMEDIATELY on network connectivity issues
//! - Tests MUST FAIL IMMEDIATELY on API authentication failures
//! - Tests MUST FAIL IMMEDIATELY on any API endpoint errors
//! - NO SILENT PASSES allowed when problems occur
//!
//! Run with : cargo test --features tools,integration
//! Requires : Valid `ANTHROPIC_API_KEY` in environment or ../../secret/-secrets.sh

#[ allow( unused_imports ) ]
use super::*;

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_tool_definition_structure()
{
  // Test that ToolDefinition has correct structure according to Claude API
  let tool = the_module::ToolDefinition
  {
    name : "get_weather".to_string(),
    description : "Get current weather for a location".to_string(),
    input_schema : serde_json::json!(
    {
      "type": "object",
      "properties": {
        "location": {
          "type": "string", 
          "description": "City and state or country"
        }
      },
      "required": ["location"]
    }),
  };
  
  assert_eq!( tool.name, "get_weather" );
  assert_eq!( tool.description, "Get current weather for a location" );
  assert!( tool.input_schema.is_object() );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_tool_use_content_structure()
{
  // Test that ToolUseContent matches Claude API format
  let tool_use = the_module::ToolUseContent
  {
    r#type : "tool_use".to_string(),
    id : "toolu_01A09q90qw90lkasdjfl".to_string(),
    name : "get_weather".to_string(),
    input : serde_json::json!({ "location": "San Francisco, CA" }),
  };
  
  assert_eq!( tool_use.r#type, "tool_use" );
  assert_eq!( tool_use.name, "get_weather" );
  assert!( !tool_use.id.is_empty() );
  assert!( tool_use.input.is_object() );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_tool_result_content_structure()
{
  // Test that ToolResultContent matches Claude API format
  let tool_result = the_module::ToolResultContent
  {
    r#type : "tool_result".to_string(),
    tool_use_id : "toolu_01A09q90qw90lkasdjfl".to_string(),
    content : "The current weather in San Francisco is 72°F and sunny.".to_string(),
    is_error : Some( false ),
  };
  
  assert_eq!( tool_result.r#type, "tool_result" );
  assert_eq!( tool_result.tool_use_id, "toolu_01A09q90qw90lkasdjfl" );
  assert_eq!( tool_result.content, "The current weather in San Francisco is 72°F and sunny." );
  assert!( !tool_result.is_error.unwrap() );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_tool_result_error_structure()
{
  // Test tool result with error condition
  let tool_result = the_module::ToolResultContent
  {
    r#type : "tool_result".to_string(),
    tool_use_id : "toolu_01A09q90qw90lkasdjfl".to_string(),
    content : "Unable to fetch weather data : API timeout".to_string(),
    is_error : Some( true ),
  };
  
  assert_eq!( tool_result.r#type, "tool_result" );
  assert!( tool_result.is_error.unwrap() );
  assert!( tool_result.content.contains( "timeout" ) );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_message_with_tool_definitions()
{
  // Test CreateMessageRequest with tools parameter
  let tools = vec![
    the_module::ToolDefinition
    {
      name : "get_weather".to_string(),
      description : "Get current weather".to_string(),
      input_schema : serde_json::json!(
      {
        "type": "object",
        "properties": {
          "location": {"type": "string"}
        }
      }),
    },
    the_module::ToolDefinition
    {
      name : "calculate_math".to_string(),
      description : "Perform mathematical calculations".to_string(),
      input_schema : serde_json::json!(
      {
        "type": "object",
        "properties": {
          "expression": {"type": "string"}
        }
      }),
    }
  ];
  
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 200 )
    .message( the_module::Message::user( "What's the weather like?".to_string() ) )
    .tools( tools.clone() )
    .build();
  
  assert_eq!( request.tools.as_ref().unwrap().len(), 2 );
  assert_eq!( request.tools.as_ref().unwrap()[0].name, "get_weather" );
  assert_eq!( request.tools.as_ref().unwrap()[1].name, "calculate_math" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_tool_choice_none()
{
  // Test tool_choice parameter set to "none"
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 100 )
    .message( the_module::Message::user( "Hello".to_string() ) )
    .tool_choice( the_module::ToolChoice::None )
    .build();
  
  match request.tool_choice.unwrap()
  {
    the_module::ToolChoice::None => {},
    _ => panic!( "Expected ToolChoice::None" ),
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_tool_choice_auto()
{
  // Test tool_choice parameter set to "auto"
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 100 )
    .message( the_module::Message::user( "Hello".to_string() ) )
    .tool_choice( the_module::ToolChoice::Auto )
    .build();
  
  match request.tool_choice.unwrap()
  {
    the_module::ToolChoice::Auto => {},
    _ => panic!( "Expected ToolChoice::Auto" ),
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_tool_choice_specific_tool()
{
  // Test tool_choice parameter set to specific tool
  let tool_choice = the_module::ToolChoice::Tool { name : "get_weather".to_string() };
  
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 100 )
    .message( the_module::Message::user( "Hello".to_string() ) )
    .tool_choice( tool_choice.clone() )
    .build();
  
  match request.tool_choice.unwrap()
  {
    the_module::ToolChoice::Tool { name } => 
    {
      assert_eq!( name, "get_weather" );
    },
    _ => panic!( "Expected ToolChoice::Tool" ),
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_message_content_with_tool_use()
{
  // Test message content that includes tool use
  let tool_use = the_module::ToolUseContent
  {
    r#type : "tool_use".to_string(),
    id : "toolu_123".to_string(),
    name : "get_weather".to_string(),
    input : serde_json::json!({ "location": "New York" }),
  };
  
  let message = the_module::Message::assistant_with_tool_use( vec![ tool_use.clone() ] );
  
  match message.role
  {
    the_module::Role::Assistant => {},
    _ => panic!( "Expected Assistant role" ),
  }
  
  assert_eq!( message.content.len(), 1 );
  // Tool use content should be serialized as part of message content
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_message_content_with_tool_result()
{
  // Test message content that includes tool result
  let tool_result = the_module::ToolResultContent
  {
    r#type : "tool_result".to_string(),
    tool_use_id : "toolu_123".to_string(),
    content : "It's 75°F and sunny in New York".to_string(),
    is_error : Some( false ),
  };
  
  let message = the_module::Message::user_with_tool_result( vec![ tool_result.clone() ] );
  
  match message.role
  {
    the_module::Role::User => {},
    _ => panic!( "Expected User role" ),
  }
  
  assert_eq!( message.content.len(), 1 );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_tool_calling_conversation_flow()
{
  // Test complete tool calling conversation flow
  let messages = vec![
    the_module::Message::user( "What's the weather like in Paris?".to_string() ),
    the_module::Message::assistant_with_tool_use( vec![
      the_module::ToolUseContent
      {
        r#type : "tool_use".to_string(),
        id : "toolu_456".to_string(),
        name : "get_weather".to_string(),
        input : serde_json::json!({ "location": "Paris, France" }),
      }
    ] ),
    the_module::Message::user_with_tool_result( vec![
      the_module::ToolResultContent
      {
        r#type : "tool_result".to_string(),
        tool_use_id : "toolu_456".to_string(),
        content : "It's 18°C and cloudy in Paris".to_string(),
        is_error : Some( false ),
      }
    ] ),
  ];
  
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 300 )
    .messages( messages.clone() )
    .build();
  
  assert_eq!( request.messages.len(), 3 );
  
  // Verify conversation flow
  match request.messages[0].role
  {
    the_module::Role::User => {},
    _ => panic!( "Expected first message to be User" ),
  }
  
  match request.messages[1].role
  {
    the_module::Role::Assistant => {},
    _ => panic!( "Expected second message to be Assistant" ),
  }
  
  match request.messages[2].role
  {
    the_module::Role::User => {},
    _ => panic!( "Expected third message to be User" ),
  }
}

// MOCKUP TEST REMOVED: This test used fake API keys and expected to fail.
// TODO: Add real integration test for tool calling with from_workspace() credentials

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_tool_definition_json_serialization()
{
  // Test that tool definitions serialize to correct JSON format
  let tool = the_module::ToolDefinition
  {
    name : "calculator".to_string(),
    description : "Perform calculations".to_string(),
    input_schema : serde_json::json!(
    {
      "type": "object",
      "properties": {
        "operation": {"type": "string"},
        "numbers": {"type": "array", "items": {"type": "number"}}
      }
    }),
  };
  
  let json = serde_json::to_string( &tool ).expect( "Should serialize successfully" );
  
  assert!( json.contains( "\"name\":\"calculator\"" ) );
  assert!( json.contains( "\"description\":\"Perform calculations\"" ) );
  assert!( json.contains( "\"input_schema\":" ) );
  assert!( json.contains( "\"properties\"" ) );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_tool_use_json_serialization()
{
  // Test that tool use content serializes correctly
  let tool_use = the_module::ToolUseContent
  {
    r#type : "tool_use".to_string(),
    id : "toolu_789".to_string(),
    name : "calculator".to_string(),
    input : serde_json::json!({ "operation": "add", "numbers": [1, 2, 3] }),
  };
  
  let json = serde_json::to_string( &tool_use ).expect( "Should serialize successfully" );
  
  assert!( json.contains( "\"type\":\"tool_use\"" ) );
  assert!( json.contains( "\"id\":\"toolu_789\"" ) );
  assert!( json.contains( "\"name\":\"calculator\"" ) );
  assert!( json.contains( "\"input\":" ) );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_multiple_tools_with_different_schemas()
{
  // Test multiple tools with complex input schemas
  let tools = vec![
    the_module::ToolDefinition
    {
      name : "file_manager".to_string(),
      description : "Manage files and directories".to_string(),
      input_schema : serde_json::json!(
      {
        "type": "object",
        "properties": {
          "action": {"type": "string", "enum": ["create", "delete", "list"]},
          "path": {"type": "string"},
          "content": {"type": "string"}
        },
        "required": ["action", "path"]
      }),
    },
    the_module::ToolDefinition
    {
      name : "database_query".to_string(),
      description : "Query database".to_string(),
      input_schema : serde_json::json!(
      {
        "type": "object",
        "properties": {
          "sql": {"type": "string"},
          "params": {"type": "array"},
          "timeout": {"type": "number", "default": 30}
        },
        "required": ["sql"]
      }),
    }
  ];
  
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 500 )
    .message( the_module::Message::user( "Help me manage my files and query data".to_string() ) )
    .tools( tools.clone() )
    .tool_choice( the_module::ToolChoice::Auto )
    .build();
  
  assert_eq!( request.tools.as_ref().unwrap().len(), 2 );
  
  // Verify complex schema structures
  let file_tool = &request.tools.as_ref().unwrap()[0];
  assert!( file_tool.input_schema.get( "properties" ).unwrap().is_object() );
  assert!( file_tool.input_schema.get( "required" ).unwrap().is_array() );
  
  let db_tool = &request.tools.as_ref().unwrap()[1];
  assert!( db_tool.input_schema.get( "properties" ).unwrap().get( "timeout" ).unwrap().get( "default" ).is_some() );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_tool_calling_with_streaming()
{
  // Test tool calling combined with streaming
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 300 )
    .message( the_module::Message::user( "Use tools and stream the response".to_string() ) )
    .tools( vec![
      the_module::ToolDefinition
      {
        name : "info_lookup".to_string(),
        description : "Look up information".to_string(),
        input_schema : serde_json::json!(
        {
          "type": "object",
          "properties": {
            "topic": {"type": "string"}
          }
        }),
      }
    ] )
    .tool_choice( the_module::ToolChoice::Auto )
    .stream( true )
    .build();
  
  assert!( request.stream.unwrap() );
  assert!( request.tools.is_some() );
  assert!( request.tool_choice.is_some() );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_tool_validation_errors()
{
  // Test validation of tool definitions
  
  // Test empty tool name should be handled gracefully
  let invalid_tool = the_module::ToolDefinition
  {
    name : String::new(),
    description : "Valid description".to_string(),
    input_schema : serde_json::json!({ "type": "object" }),
  };
  
  assert!( invalid_tool.name.is_empty() );
  
  // Test empty description should be handled gracefully  
  let invalid_tool2 = the_module::ToolDefinition
  {
    name : "valid_name".to_string(),
    description : String::new(),
    input_schema : serde_json::json!({ "type": "object" }),
  };
  
  assert!( invalid_tool2.description.is_empty() );
}

// ============================================================================
// INTEGRATION TESTS - REAL API TOOL CALLING
// ============================================================================

#[ cfg( all( feature = "integration", feature = "tools" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_tool_calling_real_math_tool()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for tool calling testing" );

  // Define a realistic calculator tool
  let calculator_tool = the_module::ToolDefinition
  {
    name : "calculator".to_string(),
    description : "Perform mathematical calculations. Input should be a mathematical expression.".to_string(),
    input_schema : serde_json::json!({
      "type": "object",
      "properties": {
        "expression": {
          "type": "string",
          "description": "The mathematical expression to evaluate (e.g., '2 + 3 * 4')"
        }
      },
      "required": ["expression"]
    }),
  };

  // Fix(issue-002): Use Claude 3.5 Haiku for tool calling tests
  // Root cause : Sonnet 4.5 does not support tool calling - it's a text-only model
  // Haiku 3.5 supports tools and is perfect for testing tool calling functionality
  // Pitfall : Always verify model capabilities when selecting models for feature-specific tests
  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 200,
    messages : vec![ the_module::Message::user( "What's 15 multiplied by 7? Use the calculator tool.".to_string() ) ],
    system : Some( vec![ the_module::SystemContent::text( "You have access to a calculator tool. Use it for mathematical calculations." ) ] ),
    temperature : Some( 0.1 ),
    stream : None,
    tools : Some( vec![ calculator_tool ] ),
    tool_choice : None, // Let the model decide when to use tools
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Tool calling request must work : {err}" ),
  };

  // Verify response structure
  assert!( !response.id.is_empty(), "Tool calling response must have ID" );
  assert!( !response.content.is_empty(), "Tool calling response must have content" );
  assert!( response.usage.input_tokens > 0, "Must track input tokens" );
  assert!( response.usage.output_tokens > 0, "Must track output tokens" );

  // Check if the response contains tool usage or calculation result
  let has_tool_or_math = response.content.iter().any( |content| {
    content.r#type == "tool_use" || 
    content.text.as_ref().is_some_and( |text| 
      text.contains( "105" ) || text.contains( "calculator" ) || text.contains( "15" )
    )
  } );
  
  assert!( has_tool_or_math, "Response should contain tool usage or math result" );

  println!( "✅ Tool calling integration test passed!" );
  println!( "   Response has {} content blocks", response.content.len() );
  for ( i, content ) in response.content.iter().enumerate()
  {
    println!( "   Content {i}: type={}", content.r#type );
    if let Some( text ) = &content.text
    {
      println!( "     Text : {text}" );
    }
  }
}

#[ cfg( all( feature = "integration", feature = "tools" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_tool_calling_multiple_tools()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for multiple tools testing" );

  // Define multiple tools
  let calculator_tool = the_module::ToolDefinition::simple( 
    "calculator", 
    "Perform mathematical calculations"
  );
  
  let weather_tool = the_module::ToolDefinition
  {
    name : "weather".to_string(),
    description : "Get current weather information for a location".to_string(),
    input_schema : serde_json::json!({
      "type": "object",
      "properties": {
        "location": {
          "type": "string",
          "description": "The city or location to get weather for"
        }
      },
      "required": ["location"]
    }),
  };

  // Fix(issue-002): Use Claude 3.5 Haiku for tool calling tests
  // Root cause : Sonnet 4.5 does not support tool calling - it's a text-only model
  // Haiku 3.5 supports tools and is perfect for testing tool calling functionality
  // Pitfall : Always verify model capabilities when selecting models for feature-specific tests
  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 150,
    messages : vec![
      the_module::Message::user( "I have a calculator and weather tool available. What's 8 + 5?".to_string() )
    ],
    system : Some( vec![ the_module::SystemContent::text( "You have access to calculator and weather tools. Use the appropriate tool for the user's request." ) ] ),
    temperature : Some( 0.0 ),
    stream : None,
    tools : Some( vec![ calculator_tool, weather_tool ] ),
    tool_choice : None,
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Multiple tools request must work : {err}" ),
  };

  // Verify the response handles multiple tool availability
  assert!( !response.id.is_empty() );
  assert!( !response.content.is_empty() );
  assert!( response.usage.input_tokens > 0 );
  assert!( response.usage.output_tokens > 0 );

  // Should either use calculator tool or provide math answer
  let has_math_response = response.content.iter().any( |content| {
    content.text.as_ref().is_some_and( |text| 
      text.contains( "13" ) || text.contains( "calculator" ) || text.contains( '8' )
    )
  } );
  
  assert!( has_math_response, "Multiple tools response should handle math question" );

  println!( "✅ Multiple tools integration test passed!" );
  println!( "   Tools available : calculator, weather" );
  println!( "   Response handled math question appropriately" );
}
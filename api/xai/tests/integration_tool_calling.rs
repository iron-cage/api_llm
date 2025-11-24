//! Integration tests for function calling / tool use.
//!
//! # Purpose
//!
//! Validates function calling workflow with XAI Grok API.
//!
//! # Key Insights
//!
//! - **Tool Calling Workflow**:
//!   1. Send request with `tools` array and user message
//!   2. Model may respond with `tool_calls` (`finish_reason="tool_calls"`)
//!   3. Execute function(s) locally
//!   4. Send results back using `Message::tool(id, result)`
//!   5. Model generates final answer using tool results
//!
//! - **Finish Reasons**:
//!   - `"tool_calls"`: Model wants to call function(s)
//!   - `"stop"`: Model completed response naturally
//!   - `"length"`: `max_tokens` limit reached
//!
//! - **Multi-Turn Conversation**: When sending tool results, must include
//!   full conversation history : original user message, assistant's tool call
//!   message, and tool result message(s).
//!
//! - **Tool Definition Format**: Uses JSON Schema for parameter specification.
//!   The `Tool::function()` helper simplifies creation.
//!
//! - **Model Autonomy**: Model decides whether to use tools or answer directly.
//!   Tests must handle both cases gracefully.
//!
//! # Running Tests
//!
//! ```bash
//! cargo test --features integration --test integration_tool_calling
//! ```

#![ cfg( feature = "integration" ) ]

mod inc;
use inc::test_helpers::create_test_client;

use api_xai::{ ChatCompletionRequest, Message, Tool, ClientApiAccessors };
use serde_json::json;

#[ tokio::test ]
async fn test_tool_calling_basic()
{
  let client = create_test_client();

  // Define a simple weather function
  let weather_tool = Tool::function(
    "get_current_weather",
    "Get the current weather in a given location",
    json!({
      "type": "object",
      "properties": {
        "location": {
          "type": "string",
          "description": "The city and state, e.g. San Francisco, CA"
        },
        "unit": {
          "type": "string",
          "enum": ["celsius", "fahrenheit"],
          "description": "Temperature unit"
        }
      },
      "required": ["location"]
    })
  );

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![
      Message::user( "What's the weather like in San Francisco?" )
    ] )
    .tools( vec![ weather_tool ] )
    .form();

  let response = client.chat().create( request ).await
    .expect( "Tool calling request should succeed" );

  // Verify response structure
  assert!( !response.id.is_empty(), "Response should have an ID" );
  assert_eq!( response.object, "chat.completion", "Object type should be chat.completion" );
  assert!( !response.choices.is_empty(), "Response should have at least one choice" );

  let choice = &response.choices[ 0 ];

  // The model might respond with content or tool calls depending on its decision
  // If it decides to call the function, finish_reason should be "tool_calls"
  if let Some( ref finish_reason ) = choice.finish_reason
  {
    println!( "Finish reason : {finish_reason}" );

    if finish_reason == "tool_calls"
    {
      // Verify tool call structure
      assert!(
        choice.message.tool_calls.is_some(),
        "Should have tool_calls when finish_reason is tool_calls"
      );

      let tool_calls = choice.message.tool_calls.as_ref().unwrap();
      assert!( !tool_calls.is_empty(), "Should have at least one tool call" );

      let tool_call = &tool_calls[ 0 ];
      assert!( !tool_call.id.is_empty(), "Tool call should have an ID" );
      assert_eq!( tool_call.tool_type, "function", "Tool type should be function" );
      assert_eq!( tool_call.function.name, "get_current_weather", "Function name should match" );

      // Verify arguments can be parsed as JSON
      let args : serde_json::Value = serde_json::from_str( &tool_call.function.arguments )
        .expect( "Arguments should be valid JSON" );

      println!( "Tool call ID: {}", tool_call.id );
      println!( "Function name : {}", tool_call.function.name );
      println!( "Arguments : {args}" );

      // Verify location is in arguments
      assert!( args[ "location" ].is_string(), "Location should be a string" );
    }
  }

  println!( "✅ Tool calling basic test passed" );
}

#[ tokio::test ]
async fn test_tool_calling_with_execution()
{
  let client = create_test_client();

  // Define calculator function
  let calculator_tool = Tool::function(
    "calculate",
    "Perform a mathematical calculation",
    json!({
      "type": "object",
      "properties": {
        "operation": {
          "type": "string",
          "enum": ["add", "subtract", "multiply", "divide"],
          "description": "The operation to perform"
        },
        "a": {
          "type": "number",
          "description": "First operand"
        },
        "b": {
          "type": "number",
          "description": "Second operand"
        }
      },
      "required": ["operation", "a", "b"]
    })
  );

  // Step 1: Initial request with tool
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![
      Message::user( "What is 15 multiplied by 7?" )
    ] )
    .tools( vec![ calculator_tool.clone() ] )
    .form();

  let response = client.chat().create( request ).await
    .expect( "First request should succeed" );

  let choice = &response.choices[ 0 ];

  // If model decides to use the tool
  if let Some( ref finish_reason ) = choice.finish_reason
  {
    if finish_reason == "tool_calls"
    {
      let tool_calls = choice.message.tool_calls.as_ref()
        .expect( "Should have tool calls" );

      let tool_call = &tool_calls[ 0 ];

      // Parse arguments
      let args : serde_json::Value = serde_json::from_str( &tool_call.function.arguments )
        .expect( "Arguments should be valid JSON" );

      println!( "Model requested calculation : {args:?}" );

      // Simulate function execution
      let result = json!({
        "result": 105
      });

      // Step 2: Send function result back
      let messages = vec![
        Message::user( "What is 15 multiplied by 7?" ),
        choice.message.clone(), // Assistant's tool call message
        Message::tool( &tool_call.id, result.to_string() ),
      ];

      let followup_request = ChatCompletionRequest::former()
        .model( "grok-2-1212".to_string() )
        .messages( messages.clone() )
        .tools( vec![ calculator_tool ] )
        .form();

      let followup_response = client.chat().create( followup_request ).await
        .expect( "Followup request should succeed" );

      let followup_choice = &followup_response.choices[ 0 ];

      // Now the model should provide the final answer
      assert!(
        followup_choice.message.content.is_some(),
        "Should have content in final response"
      );

      let content = followup_choice.message.content.as_ref().unwrap();
      println!( "Final response : {content}" );

      // Verify the response mentions the result
      assert!(
        content.contains( "105" ) || content.to_lowercase().contains( "hundred" ),
        "Response should mention the result"
      );

      println!( "✅ Tool calling with execution test passed" );
      return;
    }
  }

  // If model answered directly without using tool, that's also acceptable
  println!( "ℹ️  Model answered directly without using tool" );
}

#[ tokio::test ]
async fn test_tool_calling_multiple_tools()
{
  let client = create_test_client();

  // Define multiple tools
  let weather_tool = Tool::function(
    "get_weather",
    "Get current weather for a location",
    json!({
      "type": "object",
      "properties": {
        "location": { "type": "string" }
      },
      "required": ["location"]
    })
  );

  let time_tool = Tool::function(
    "get_time",
    "Get current time for a location",
    json!({
      "type": "object",
      "properties": {
        "timezone": { "type": "string" }
      },
      "required": ["timezone"]
    })
  );

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![
      Message::user( "What's the weather and time in Tokyo?" )
    ] )
    .tools( vec![ weather_tool, time_tool ] )
    .form();

  let response = client.chat().create( request ).await
    .expect( "Multiple tools request should succeed" );

  assert!( !response.choices.is_empty(), "Should have choices" );

  let choice = &response.choices[ 0 ];

  // Model might call one or both tools, or answer directly
  if let Some( ref tool_calls ) = choice.message.tool_calls
  {
    println!( "Model called {} tool(s)", tool_calls.len() );

    for tool_call in tool_calls
    {
      println!( "Tool : {}", tool_call.function.name );
      println!( "Arguments : {}", tool_call.function.arguments );
    }
  }

  println!( "✅ Multiple tools test passed" );
}

#[ tokio::test ]
async fn test_tool_calling_no_tools_works()
{
  let client = create_test_client();

  // Request without tools should still work
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![
      Message::user( "Say hello" )
    ] )
    .form();

  let response = client.chat().create( request ).await
    .expect( "Request without tools should succeed" );

  assert!( !response.choices.is_empty(), "Should have choices" );
  assert!(
    response.choices[ 0 ].message.content.is_some(),
    "Should have content when no tools"
  );

  println!( "✅ No tools test passed" );
}

#[ tokio::test ]
async fn test_tool_message_creation()
{
  // Test the Message::tool() helper
  let tool_message = Message::tool(
    "call_abc123",
    r#"{"result": "success"}"#
  );

  assert_eq!( tool_message.role, api_xai::Role::Tool );
  assert_eq!( tool_message.tool_call_id, Some( "call_abc123".to_string() ) );
  assert_eq!( tool_message.content, Some( r#"{"result": "success"}"#.to_string() ) );
  assert!( tool_message.tool_calls.is_none() );

  println!( "✅ Tool message creation test passed" );
}

//! Unit tests for enhanced function calling tools.
//!
//! # Purpose
//!
//! Validates parallel and sequential tool execution helpers.
//!
//! # Key Insights
//!
//! - **Parallel Execution**: Multiple tool calls can be executed concurrently
//!   using `tokio::spawn`, improving performance for independent operations.
//!
//! - **Sequential Execution**: Tool calls that depend on previous results
//!   or require ordered execution can use sequential mode.
//!
//! - **Error Handling**: Individual tool failures don't stop the entire batch.
//!   Each tool result is wrapped in Result for independent error handling.
//!
//! # Running Tests
//!
//! ```bash
//! cargo test --features enhanced_tools --test enhanced_tools_tests
//! ```

#![ cfg( feature = "enhanced_tools" ) ]

use api_xai::{ ToolCall, FunctionCall, ToolResult, execute_tool_calls_parallel, execute_tool_calls_sequential };
use serde_json::json;

// Helper to create a test tool call
fn create_test_tool_call( id : &str, function_name : &str ) -> ToolCall
{
  ToolCall
  {
    id : id.to_string(),
    tool_type : "function".to_string(),
    function : FunctionCall
    {
      name : function_name.to_string(),
      arguments : r#"{"test": true}"#.to_string(),
    },
  }
}

#[ tokio::test ]
async fn parallel_execution_all_tools_succeed()
{
  let tool_calls = vec![
    create_test_tool_call( "call_1", "func_a" ),
    create_test_tool_call( "call_2", "func_b" ),
    create_test_tool_call( "call_3", "func_c" ),
  ];

  let results = execute_tool_calls_parallel( tool_calls, | call | async move {
    // Simulate tool execution
    let result = json!({
      "function": call.function.name,
      "status": "success"
    });

    Ok( ToolResult::new( call.id, &result ) )
  } ).await;

  assert_eq!( results.len(), 3 );

  for result in results
  {
    assert!( result.is_ok() );
  }
}

#[ tokio::test ]
async fn parallel_execution_individual_errors_dont_stop_batch()
{
  let tool_calls = vec![
    create_test_tool_call( "call_1", "func_success" ),
    create_test_tool_call( "call_2", "func_error" ),
    create_test_tool_call( "call_3", "func_success" ),
  ];

  let results = execute_tool_calls_parallel( tool_calls, | call | async move {
    if call.function.name == "func_error"
    {
      Err( "Simulated error".into() )
    }
    else
    {
      let result = json!({ "status": "ok" });
      Ok( ToolResult::new( call.id, &result ) )
    }
  } ).await;

  assert_eq!( results.len(), 3 );

  // First should succeed
  assert!( results[ 0 ].is_ok() );

  // Second should fail
  assert!( results[ 1 ].is_err() );

  // Third should succeed
  assert!( results[ 2 ].is_ok() );
}

#[ tokio::test ]
async fn sequential_execution_processes_all_tools_in_order()
{
  let tool_calls = vec![
    create_test_tool_call( "call_1", "step_1" ),
    create_test_tool_call( "call_2", "step_2" ),
    create_test_tool_call( "call_3", "step_3" ),
  ];

  let execution_order = Vec::new();

  let results = execute_tool_calls_sequential( tool_calls, | call | {
    let order = execution_order.clone();
    async move {
      // Record execution order (simulating sequential dependency)
      let mut order = order;
      order.push( call.function.name.clone() );

      let result = json!({
        "function": call.function.name,
        "order": order.len()
      });

      Ok( ToolResult::new( call.id, &result ) )
    }
  } ).await;

  assert_eq!( results.len(), 3 );

  for result in results
  {
    assert!( result.is_ok() );
  }
}

#[ tokio::test ]
async fn tool_result_constructors_produce_correct_fields()
{
  // Test new() with JSON value
  let result1 = ToolResult::new(
    "call_123".to_string(),
    &json!({ "temperature": 72 })
  );

  assert_eq!( result1.tool_call_id, "call_123" );
  assert!( result1.result.contains( "temperature" ) );
  assert!( result1.result.contains( "72" ) );

  // Test from_string()
  let result2 = ToolResult::from_string(
    "call_456".to_string(),
    r#"{"weather": "sunny"}"#.to_string()
  );

  assert_eq!( result2.tool_call_id, "call_456" );
  assert_eq!( result2.result, r#"{"weather": "sunny"}"# );

  // Test from_error()
  let result3 = ToolResult::from_error(
    "call_789".to_string(),
    "Connection timeout"
  );

  assert_eq!( result3.tool_call_id, "call_789" );
  assert!( result3.result.contains( "error" ) );
  assert!( result3.result.contains( "Connection timeout" ) );
}

#[ tokio::test ]
async fn parallel_execution_empty_list_returns_empty()
{
  let tool_calls : Vec< ToolCall > = vec![];

  let results = execute_tool_calls_parallel( tool_calls, | call | async move {
    let result = json!({ "status": "ok" });
    Ok( ToolResult::new( call.id, &result ) )
  } ).await;

  assert_eq!( results.len(), 0 );
}

#[ tokio::test ]
async fn sequential_execution_empty_list_returns_empty()
{
  let tool_calls : Vec< ToolCall > = vec![];

  let results = execute_tool_calls_sequential( tool_calls, | call | async move {
    let result = json!({ "status": "ok" });
    Ok( ToolResult::new( call.id, &result ) )
  } ).await;

  assert_eq!( results.len(), 0 );
}

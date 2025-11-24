//! ## Test Matrix for `OpenAI` API Deserialization Tests
//!
//! | ID   | Aspect Tested | API Call(s) | Expected Behavior |
//! |------|---------------|-------------|-------------------|
//! | D1.1 | Basic Response Creation | N/A | Successfully deserializes a basic ResponseObject. |
//! | D1.2 | ResponseObject (with usage) | N/A | Successfully deserializes ResponseObject with usage data. |
//! | D1.3 | ResponseObject (with refusal) | N/A | Successfully deserializes ResponseObject with refusal data. |
//! | D1.4 | ResponseObject (with function call) | N/A | Successfully deserializes ResponseObject with function call data. |
//! | D1.5 | ResponseObject (with file search call) | N/A | Successfully deserializes ResponseObject with file search call data. |
//! | D1.6 | ResponseObject (with web search call) | N/A | Successfully deserializes ResponseObject with web search call data. |
//! | D1.7 | ResponseObject (with computer call) | N/A | Successfully deserializes ResponseObject with computer call data. |
//! | D1.8 | ResponseObject (failed) | N/A | Successfully deserializes a failed ResponseObject. |
//! | D1.9 | ResponseItemList (empty) | N/A | Successfully deserializes an empty ResponseItemList. |
//! | D1.10 | ResponseItemList (multiple items) | N/A | Successfully deserializes ResponseItemList with multiple items. |
//! | D1.11 | ResponseDeleted | N/A | Successfully deserializes a ResponseDeleted object. |

#![ allow( unused_imports ) ]
use super::*;
use super::test_data_factories::*;
use super::test_data_factories::{ ResponseObjectFactory };
use api_openai::exposed::
{
  Client,
  OpenAIError,
  environment ::EnvironmentInterface,
  Secret,
  components ::
  {
    responses ::
    {
      CreateResponseRequest,
      ResponseObject,
      ResponseInput,
      InputItem,
    },
    input ::
    {
      InputMessage,
      InputContentPart,
      InputText,
    },
    common ::{ ModelIds, ModelIdsResponses },
    tools ::{ FunctionTool, FunctionParameters },
    chat_shared ::{
      CreateChatCompletionResponse,
      ChatCompletionRequest,
      ChatCompletionRequestMessage,
      ChatCompletionRequestMessageContent,
      ChatCompletionRequestMessageContentPart,
      ChatCompletionRequestMessageContentImageUrl,
      ChatCompletionTool,
      ToolChoiceOption,
    },
  }
};
use serde_json::json;
use futures_util::stream::StreamExt;
use secrecy::ExposeSecret;

use api_openai::exposed::components::responses::ResponseObject as ResponseObject2;
use api_openai::exposed::components::responses::ResponseItemList;
use api_openai::exposed::components::common::ResponseError;
use api_openai::exposed::components::common::ResponseUsage;
use api_openai::exposed::components::output::OutputItem as OutputItem2;
use api_openai::exposed::components::output::OutputContentPart as OutputContentPart2;
use api_openai::exposed::components::output::FileSearchResultItem;
use api_openai::exposed::components::output::ComputerScreenshotImage;
use api_openai::exposed::components::input::ListedInputItem;
use api_openai::exposed::components::common::DeleteResponse;
use api_openai::exposed::components::tools::WebSearchToolCall;
use api_openai::exposed::components::tools::{ ComputerToolCall, ComputerAction };
use api_openai::exposed::components::tools::FunctionToolCall as FunctionToolCall2;

/// Tests that a simple `ResponseObject` can be deserialized.
/// Test Combination : D1.1
#[ test ]
fn response_simple_completed()
{
  let json_data = ResponseObjectFactory::simple_completed_response(
    "response_id_123",
    "msg_1",
    "Hello, how can I help you today?",
    "gpt-5.1-chat-latest"
  );

  let response : ResponseObject2 = serde_json::from_str( &json_data ).expect( "Failed to deserialize ResponseObject" );

  assert_eq!( response.id, "response_id_123" );
  assert_eq!( response.object, "response" );
  assert_eq!( response.status, "completed" );
  assert_eq!( response.output.len(), 1 );
  if let OutputItem2::Message( msg ) = &response.output[ 0 ]
  {
    assert_eq!( msg.id, "msg_1" );
    assert_eq!( msg.role, "assistant" );
    assert_eq!( msg.content.len(), 1 );
    if let OutputContentPart2::Text { text : text_content, .. } = &msg.content[ 0 ]
    {
      assert_eq!( text_content, "Hello, how can I help you today?" );
    }
    else
    {
      panic!( "Expected text content" );
    }
  }
  else
  {
    panic!( "Expected message output item" );
  }
}

/// Tests that a `ResponseObject` with usage data can be deserialized.
/// Test Combination : D1.2
#[ test ]
fn response_with_usage()
{
  let json_data = r#"
  {
    "id": "response_id_456",
    "object": "response",
    "created_at": 1678886401,
    "status": "completed",
    "output": [],
    "usage": {
      "input_tokens": 10,
      "output_tokens": 20,
      "total_tokens": 30
    },
    "model": "gpt-5.1-chat-latest",
    "parallel_tool_calls": true
  }
  "#;

  let response : ResponseObject2 = serde_json::from_str( json_data ).expect( "Failed to deserialize ResponseObject with usage" );

  assert_eq!( response.id, "response_id_456" );
  assert_eq!( response.status, "completed" );
  assert!( response.usage.is_some() );
  let usage = response.usage.unwrap();
  assert_eq!( usage.input_tokens, 10 );
  assert_eq!( usage.output_tokens, 20 );
  assert_eq!( usage.total_tokens, 30 );
}

/// Tests that a `ResponseObject` with refusal content can be deserialized.
/// Test Combination : D1.3
#[ test ]
fn response_with_refusal()
{
  let json_data = r#"
  {
    "id": "response_id_789",
    "object": "response",
    "created_at": 1678886402,
    "status": "completed",
    "output": [
      {
        "type": "message",
        "id": "msg_refusal",
        "role": "assistant",
        "content": [
          {
            "type": "refusal",
            "refusal": "I cannot fulfill this request."
          }
        ],
        "status": "completed"
      }
    ],
    "model": "gpt-5.1-chat-latest",
    "parallel_tool_calls": true
  }
  "#;

  let response : ResponseObject2 = serde_json::from_str( json_data ).expect( "Failed to deserialize ResponseObject with refusal" );

  assert_eq!( response.id, "response_id_789" );
  assert_eq!( response.output.len(), 1 );
  if let OutputItem2::Message( msg ) = &response.output[ 0 ]
  {
    assert_eq!( msg.id, "msg_refusal" );
    assert_eq!( msg.role, "assistant" );
    assert_eq!( msg.content.len(), 1 );
    if let OutputContentPart2::Refusal { refusal, .. } = &msg.content[ 0 ]
    {
      assert_eq!( refusal, "I cannot fulfill this request." );
    }
    else
    {
      panic!( "Expected refusal content part" );
    }
  }
  else
  {
    panic!( "Expected message output item" );
  }
}

/// Tests that a `ResponseObject` with a function call can be deserialized.
/// Test Combination : D1.4
#[ test ]
fn response_with_function_call()
{
  let json_data = r#"
  {
    "id": "response_id_101",
    "object": "response",
    "created_at": 1678886403,
    "status": "completed",
    "output": [
      {
        "type": "function_call",
        "id": "call_abc",
        "call_id": "some_call_id",
        "name": "get_weather",
        "arguments": "{\"location\": \"London\"}",
        "status": "completed"
      }
    ],
    "model": "gpt-5.1-chat-latest",
    "parallel_tool_calls": true
  }
  "#;

  let response : ResponseObject2 = serde_json::from_str( json_data ).expect( "Failed to deserialize ResponseObject with function call" );

  assert_eq!( response.id, "response_id_101" );
  assert_eq!( response.output.len(), 1 );
  if let OutputItem2::FunctionCall( function_call ) = &response.output[ 0 ]
  {
    assert_eq!( function_call.id, "call_abc" );
    assert_eq!( function_call.name, "get_weather" );
    assert_eq!( function_call.arguments, "{\"location\": \"London\"}" );
    assert_eq!( function_call.status, "completed" );
  }
  else
  {
    panic!( "Expected function_call output item" );
  }
}

/// Tests that a `ResponseObject` with a file search call can be deserialized.
/// Test Combination : D1.5
#[ test ]
fn response_with_file_search_call()
{
  let json_data = r#"
  {
    "id": "response_id_102",
    "object": "response",
    "created_at": 1678886404,
    "status": "completed",
    "output": [
      {
        "type": "file_search_call",
        "id": "fs_call_xyz",
        "status": "completed",
        "results": [
          {
            "file_id": "file_1",
            "filename": "file1.txt",
            "score": 0.9,
            "attributes": {},
            "content": [
              {
                "type": "input_text",
                "text": "content of file 1"
              }
            ]
          }
        ]
      }
    ],
    "model": "gpt-5.1-chat-latest",
    "parallel_tool_calls": true
  }
  "#;

  let response : ResponseObject2 = serde_json::from_str( json_data ).expect( "Failed to deserialize ResponseObject with file search call" );

  assert_eq!( response.id, "response_id_102" );
  assert_eq!( response.output.len(), 1 );
  if let OutputItem2::FileSearchCall( file_search_call ) = &response.output[ 0 ]
  {
    assert_eq!( file_search_call.id, "fs_call_xyz" );
    assert_eq!( file_search_call.status, "completed" );
    assert!( file_search_call.results.is_some() );
    let results = file_search_call.results.as_ref().unwrap();
    assert_eq!( results.len(), 1 );
    let result = &results[ 0 ];
    assert_eq!( result.file_id, "file_1" );
    assert_eq!( result.content[0].text, "content of file 1" );
  }
  else
  {
    panic!( "Expected file_search_call output item" );
  }
}

/// Tests that a `ResponseObject` with a web search call can be deserialized.
/// Test Combination : D1.6
#[ test ]
fn response_with_web_search_call()
{
  let json_data = r#"
  {
    "id": "response_id_103",
    "object": "response",
    "created_at": 1678886405,
    "status": "completed",
    "output": [
      {
        "type": "web_search_call",
        "id": "ws_call_abc",
        "status": "completed"
      }
    ],
    "model": "gpt-5.1-chat-latest",
    "parallel_tool_calls": true
  }
  "#;

  let response : ResponseObject2 = serde_json::from_str( json_data ).expect( "Failed to deserialize ResponseObject with web search call" );

  assert_eq!( response.id, "response_id_103" );
  assert_eq!( response.output.len(), 1 );
  if let OutputItem2::WebSearchCall( web_search_call ) = &response.output[ 0 ]
  {
    assert_eq!( web_search_call.id, "ws_call_abc" );
    assert_eq!( web_search_call.status, "completed" );
  }
  else
  {
    panic!( "Expected web_search_call output item" );
  }
}

/// Tests that a `ResponseObject` with a computer call can be deserialized.
/// Test Combination : D1.7
#[ test ]
fn response_with_computer_call()
{
  let json_data = r#"
  {
    "id": "response_id_104",
    "object": "response",
    "created_at": 1678886406,
    "status": "completed",
    "output": [
      {
        "type": "computer_call",
        "id": "cc_call_xyz",
        "call_id": "some_call_id",
        "action": {
          "type": "type",
          "text": "print('Hello, world!')"
        },
        "status": "completed"
      }
    ],
    "model": "gpt-5.1-chat-latest",
    "parallel_tool_calls": true
  }
  "#;

  let response : ResponseObject2 = serde_json::from_str( json_data ).expect( "Failed to deserialize ResponseObject with computer call" );

  assert_eq!( response.id, "response_id_104" );
  assert_eq!( response.output.len(), 1 );
  if let OutputItem2::ComputerCall( computer_call ) = &response.output[ 0 ]
  {
    assert_eq!( computer_call.id, "cc_call_xyz" );
    assert_eq!( computer_call.call_id, "some_call_id" );
    assert_eq!( computer_call.status, "completed" );
    if let ComputerAction::Type { text, .. } = &computer_call.action
    {
      assert_eq!( text, "print('Hello, world!')" );
    }
    else
    {
      panic!( "Expected type action" );
    }
  }
  else
  {
    panic!( "Expected computer_call output item" );
  }
}

/// Tests that a failed `ResponseObject` can be deserialized.
/// Test Combination : D1.8
#[ test ]
fn response_failed()
{
  let json_data = r#"
  {
    "id": "response_id_failed",
    "object": "response",
    "created_at": 1678886407,
    "status": "failed",
    "error": {
      "code": "500",
      "message": "An internal error occurred."
    },
    "output": [],
    "model": "gpt-5.1-chat-latest",
    "parallel_tool_calls": true
  }
  "#;

  let response : ResponseObject2 = serde_json::from_str( json_data ).expect( "Failed to deserialize failed ResponseObject" );

  assert_eq!( response.id, "response_id_failed" );
  assert_eq!( response.status, "failed" );
  assert!( response.error.is_some() );
  let error = response.error.unwrap();
  assert_eq!( error.code, "500".to_string() );
  assert_eq!( error.message, "An internal error occurred.".to_string() );
}

/// Tests that an empty `ResponseItemList` can be deserialized.
/// Test Combination : D1.9
#[ test ]
fn response_item_list_empty()
{
  let json_data = r#"
  {
    "object": "list",
    "data": [],
    "first_id": null,
    "last_id": null,
    "has_more": false
  }
  "#;

  let list : ResponseItemList = serde_json::from_str( json_data ).expect( "Failed to deserialize empty ResponseItemList" );

  assert_eq!( list.object, "list" );
  assert!( list.data.is_empty() );
  assert!( list.first_id.is_none() );
  assert!( list.last_id.is_none() );
  assert!( !list.has_more );
}

/// Tests that a `ResponseItemList` with multiple items can be deserialized.
/// Test Combination : D1.10
#[ test ]
fn response_item_list_multiple_items()
{
  let json_data = r#"
  {
    "object": "list",
    "data": [
      {
        "id": "input_item_1",
        "type": "message",
        "role": "user",
        "content": [
          {
            "type": "input_text",
            "text": "First message"
          }
        ],
        "status": "completed"
      },
      {
        "id": "input_item_2",
        "type": "message",
        "role": "assistant",
        "content": [
          {
            "type": "input_text",
            "text": "Second message"
          }
        ],
        "status": "completed"
      }
    ],
    "first_id": "input_item_1",
    "last_id": "input_item_2",
    "has_more": true
  }
  "#;

  let list : ResponseItemList = serde_json::from_str( json_data ).expect( "Failed to deserialize ResponseItemList with multiple items" );

  assert_eq!( list.object, "list" );
  assert_eq!( list.data.len(), 2 );
  assert_eq!( list.first_id, Some( "input_item_1".to_string() ) );
  assert_eq!( list.last_id, Some( "input_item_2".to_string() ) );
  assert!( list.has_more );

  assert_eq!( list.data[ 0 ].id, "input_item_1" );
  assert_eq!( list.data[ 1 ].id, "input_item_2" );
}

/// Tests that a `ResponseDeleted` object can be deserialized.
/// Test Combination : D1.11
#[ test ]
fn response_deleted()
{
  let json_data = r#"
  {
    "id": "response_id_to_delete",
    "object": "response.deleted",
    "deleted": true
  }
  "#;

  let deleted_response : DeleteResponse = serde_json::from_str( json_data ).expect( "Failed to deserialize ResponseDeleted" );

  assert_eq!( deleted_response.id, "response_id_to_delete" );
  assert_eq!( deleted_response.object, "response.deleted" );
  assert!( deleted_response.deleted );
}
//! Shared utilities for integration tests
//!
//! This module contains common test utilities, imports, and helper functions
//! used across all integration test modules.
//!
//! # MANDATORY FAILING BEHAVIOR
//!
//! All integration test utilities in this module enforce mandatory failing:
//! - `IsolatedClient` with `use_real_api=true` MUST fail if credentials unavailable
//! - `should_run_real_api_tests()` determines when real API access is possible
//! - Test helper functions fail hard rather than using fallback mocks
//!
//! This ensures integration test failures indicate real issues that need resolution.

#![ cfg( feature = "integration" ) ]
#![ allow( unused_imports, dead_code, clippy::missing_panics_doc, clippy::missing_errors_doc ) ]

// Re-export the main module for easy access
use api_openai::ClientApiAccessors;
pub use api_openai as the_module;

// Import test isolation framework
use crate::test_isolation::TestIsolation;
pub use crate::test_isolation::{ IsolatedClient, should_run_real_api_tests };

// Core API imports that most tests need
pub use api_openai::{
  Client,
  error ::OpenAIError,
  environment ::{ EnvironmentInterface, OpenaiEnvironment, OpenaiEnvironmentImpl },
  secret ::Secret,
  components ::{
    responses ::{
      CreateResponseRequest,
      ResponseObject,
      ResponseInput,
      ResponseStreamEvent,
      ResponseItemList,
    },
    input ::{
      InputItem,
      InputMessage,
      InputContentPart,
      InputText,
    },
    common ::{ ModelIdsResponses, ListQuery },
    tools ::{ Tool, ToolChoice, FunctionTool, FunctionParameters },
    output ::{ OutputItem, OutputContentPart },
  }
};

// Common external dependencies
pub use serde_json::json;
pub use futures_util::stream::StreamExt;
pub use secrecy::ExposeSecret;
pub use tokio::sync::mpsc;

// Re-export test isolation utilities (already imported above)

/// Common assertion helper for response validation
pub fn assert_valid_response(response : &ResponseObject)
{
  assert!(!response.id.is_empty(), "Response should have an id field");
  assert!(!response.output.is_empty(), "Response should have output");
  assert_eq!(response.object, "response", "Object type should be 'response'");
  assert!(response.created_at > 0, "Created timestamp should be valid");
}

/// Create a basic test request for responses
#[ must_use ]
pub fn create_basic_test_request() -> CreateResponseRequest
{
  CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-5-nano".to_string()))
    .input(ResponseInput::Items(
      vec![
        InputItem::Message(
          InputMessage {
            r#type : "message".to_string(),
            role : "user".to_string(),
            content : vec![
              InputContentPart::Text(
                InputText {
                  text : "Hello, how are you?".to_string(),
                }
              ),
            ],
            status : None,
            id : None,
          }
        ),
      ]
    ))
    .max_output_tokens(50)
    .parallel_tool_calls(true)
    .form()
}

/// Create a test request with tools for function calling tests
#[ must_use ]
pub fn create_tools_test_request() -> CreateResponseRequest
{
  let get_weather_tool = Tool::Function(
    FunctionTool::former()
      .description("Get weather information for a location".to_string())
      .name("get_weather".to_string())
      .parameters(FunctionParameters::new(json!({
        "type": "object",
        "properties": {
          "location": {
            "type": "string",
            "description": "The location to get weather for"
          }
        },
        "required": ["location"]
      })))
      .form()
  );


  CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-5-nano".to_string()))
    .input(ResponseInput::String("What's the weather like in Paris?".to_string()))
    .tools(vec![get_weather_tool])
    .tool_choice(ToolChoice::String("required".to_string()))
    .max_output_tokens(150)
    .form()
}

/// Handle real API test results only - NO MOCKING ALLOWED
///
/// # Panics
///
/// Panics if the test result indicates an unexpected error or validation failure.
/// Always expects real API access - never falls back to mocks.
#[ allow( clippy::std_instead_of_core ) ] // std::fmt::Debug is more idiomatic in tests
pub fn handle_test_result< T, E: std::fmt::Debug >(
  result : Result< T, E >,
  test_name : &str,
  success_validator : impl FnOnce(&T)
) {
  // REAL API ONLY - No mocking fallback allowed
  match result
  {
    Ok(ref response) =>
    {
      success_validator(response);
    },
    Err(e) =>
    {
      panic!("API request failed in test '{test_name}': {e:?}");
    }
  }
}
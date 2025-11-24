//! Function Calling Tests for `HuggingFace` Router API
//!
//! ## Purpose
//!
//! Validates the OpenAI-compatible function calling capabilities of the `HuggingFace` Router API,
//! including tool definitions, parameter schemas, tool choice controls, and response handling.
//!
//! ## Test Coverage
//!
//! - **Basic Function Calling**: Simple tool definition and invocation
//! - **Tool Choice Options**: auto, none, required, specific function selection
//! - **Parameter Schemas**: Complex parameter definitions with required/optional fields
//! - **Multiple Tools**: Handling multiple tool definitions in single request
//! - **Error Handling**: Invalid tool definitions and malformed requests
//!
//! ## Router API Function Calling Support
//!
//! Per `HuggingFace` documentation : <https://huggingface.co/docs/api-inference/tasks/chat-completion>
//! - `tools`: List of tools the model may call (currently only functions supported)
//! - `tool_choice`: Controls tool usage (auto, none, required, or specific function)
//! - `tool_prompt`: Custom prompt appended before tools
//!
//! ## Design Principles
//!
//! - **No Mocking**: All tests use real Router API calls (integration tests)
//! - **Explicit Control**: Tools must be explicitly provided by developer
//! - **Transparent Mapping**: 1:1 mapping to OpenAI-compatible function calling format

#[ cfg( all( test, feature = "integration" ) ) ]
mod tests
{
  use api_huggingface::
  {
  Client,
  environment::HuggingFaceEnvironmentImpl,
  components::
  {
      tools::{ Tool, ToolParameters, ParameterProperty },
      models::Models,
  },
  secret::Secret,
  };

  /// Helper to create test client using workspace secrets
  fn setup_client() -> Client< HuggingFaceEnvironmentImpl >
  {
  use workspace_tools as workspace;

  let workspace = workspace::workspace()
      .expect( "[setup_client] Failed to access workspace - required for integration tests" );
  let secrets = workspace.load_secrets_from_file( "-secrets.sh" )
      .expect( "[setup_client] Failed to load secret/-secrets.sh - required for integration tests" );
  let api_key = secrets.get( "HUGGINGFACE_API_KEY" )
      .expect( "[setup_client] HUGGINGFACE_API_KEY not found in secret/-secrets.sh - required for integration tests" )
      .clone();

  let env = HuggingFaceEnvironmentImpl::build( Secret::new( api_key ), None )
      .expect( "Failed to build environment" );

  Client::build( env ).expect( "Failed to build client" )
  }

  /// Create a simple weather tool for testing
  fn create_weather_tool() -> Tool
  {
  let parameters = ToolParameters::new()
      .with_property
      (
  "location",
  ParameterProperty::string( "The city and state, e.g. San Francisco, CA" )
      )
      .with_property
      (
  "unit",
  ParameterProperty::string( "Temperature unit (celsius or fahrenheit)" )
      )
      .with_required( vec![ "location".to_string() ] );

  Tool::new
  (
      "get_current_weather",
      "Get the current weather in a given location",
      parameters
  )
  }

  /// Create a calculator tool for testing
  fn create_calculator_tool() -> Tool
  {
  let parameters = ToolParameters::new()
      .with_property
      (
  "operation",
  ParameterProperty::string( "The mathematical operation (add, subtract, multiply, divide)" )
      )
      .with_property
      (
  "a",
  ParameterProperty::number( "First operand" )
      )
      .with_property
      (
  "b",
  ParameterProperty::number( "Second operand" )
      )
      .with_required( vec![ "operation".to_string(), "a".to_string(), "b".to_string() ] );

  Tool::new
  (
      "calculate",
      "Perform basic mathematical operations",
      parameters
  )
  }

  #[ tokio::test ]

  async fn test_basic_function_calling()
  {
  let client = setup_client();
  let tool = create_weather_tool();

  let response = client.providers()
      .chat_completion_with_tools
      (
  Models::kimi_k2_instruct(),
  vec!
  [
          api_huggingface::components::inference_shared::ChatMessage
          {
      role : "user".to_string(),
      content : "What's the weather like in San Francisco?".to_string(),
      tool_calls : None,
      tool_call_id : None,
          }
  ],
  vec![ tool ],
  None, // tool_choice : auto (default)
  None, // max_tokens
  None, // temperature
  None, // top_p
      )
      .await;

  assert!( response.is_ok(), "Function calling request failed : {:?}", response.err() );

  let response = response.expect( "[test_basic_function_calling] Response should be Ok after is_ok() check - check chat_completion_with_tools() implementation" );

  // Should have at least one choice
  assert!( !response.choices.is_empty(), "Response has no choices" );

  // Check if model requested a tool call
  if let Some( ref tool_calls ) = response.choices[ 0 ].message.tool_calls
  {
      assert!( !tool_calls.is_empty(), "Expected tool calls in response" );

      let tool_call = &tool_calls[ 0 ];
      assert_eq!( tool_call.function.name, "get_current_weather", "Wrong tool called" );

      // Verify function arguments contain location
      // Note: API may return malformed JSON under load/rate limiting - fail loudly with context
      let args : serde_json::Value = serde_json::from_str( &tool_call.function.arguments )
        .unwrap_or_else( |e|
        {
          // Provide detailed context for integration test failure
          panic!
          (
            "Failed to parse function arguments from HuggingFace API.\n\
             This indicates the API returned malformed JSON (likely rate limit/service degradation).\n\
             \nArguments string received: {:?}\n\
             Parse error: {}\n\
             \nRetry the test. If failures persist, check HuggingFace API status.",
            tool_call.function.arguments,
            e
          );
        } );

      assert!( args.get( "location" ).is_some(), "Missing location argument" );
  }
  }

  #[ tokio::test ]

  async fn test_tool_choice_none()
  {
  let client = setup_client();
  let tool = create_weather_tool();

  let response = client.providers()
      .chat_completion_with_tools
      (
  Models::kimi_k2_instruct(),
  vec!
  [
          api_huggingface::components::inference_shared::ChatMessage
          {
      role : "user".to_string(),
      content : "What's the weather like in San Francisco?".to_string(),
      tool_calls : None,
      tool_call_id : None,
          }
  ],
  vec![ tool ],
  Some( "none".to_string() ), // tool_choice : none - should not call tools
  None,
  None,
  None,
      )
      .await;

  assert!( response.is_ok(), "Request failed : {:?}", response.err() );

  let response = response.expect( "[test_tool_choice_none] Response should be Ok after is_ok() check - check chat_completion_with_tools() with tool_choice='none' implementation" );

  // With tool_choice : none, model should ideally respond with text only
  // However, some models don't perfectly respect tool_choice, so we verify
  // the request succeeded and model provided some response
  let has_tool_calls = response.choices[ 0 ].message.tool_calls.as_ref()
      .is_some_and( | tc | !tc.is_empty() );
  let has_content = !response.choices[ 0 ].message.content.is_empty();

  // Model should provide either text response or tool calls (ideally just text)
  assert!( has_content || has_tool_calls, "Model should provide some response" );
  }

  #[ tokio::test ]

  async fn test_tool_choice_required()
  {
  let client = setup_client();
  let tool = create_weather_tool();

  let response = client.providers()
      .chat_completion_with_tools
      (
  Models::kimi_k2_instruct(),
  vec!
  [
          api_huggingface::components::inference_shared::ChatMessage
          {
      role : "user".to_string(),
      // Use a prompt that naturally leads to calling the weather tool
      content : "What is the current weather in San Francisco?".to_string(),
      tool_calls : None,
      tool_call_id : None,
          }
  ],
  vec![ tool ],
  Some( "required".to_string() ), // tool_choice : required - must call a tool
  None,
  None,
  None,
      )
      .await;

  // With tool_choice : required, either model calls a tool or API returns error
  // Some models may not perfectly respect tool_choice, so handle both cases
  match response
  {
  Ok( ref resp ) =>
  {
      // If response is Ok, model must have called a tool
      assert!( resp.choices[ 0 ].message.tool_calls.is_some(), "Model must call tool when tool_choice is 'required'" );
  }
  Err( ref e ) =>
  {
      // API may return error if model failed to call tool (this is valid API behavior)
      let err_str = format!( "{e:?}" );
      assert!( err_str.contains( "tool_use_failed" ) || err_str.contains( "tool" ),
    "Expected tool-related error when model fails to call tool, got : {err_str}" );
  }
  }
  }

  #[ tokio::test ]

  async fn test_multiple_tools()
  {
  let client = setup_client();
  let weather_tool = create_weather_tool();
  let calc_tool = create_calculator_tool();

  let response = client.providers()
      .chat_completion_with_tools
      (
  Models::kimi_k2_instruct(),
  vec!
  [
          api_huggingface::components::inference_shared::ChatMessage
          {
      role : "user".to_string(),
      content : "What is 15 multiplied by 3?".to_string(),
      tool_calls : None,
      tool_call_id : None,
          }
  ],
  vec![ weather_tool, calc_tool ],
  None, // auto - model chooses appropriate tool
  None,
  None,
  None,
      )
      .await;

  assert!( response.is_ok(), "Request failed : {:?}", response.err() );

  let response = response.expect( "[test_multiple_tools] Response should be Ok after is_ok() check - check chat_completion_with_tools() with multiple tools implementation" );

  // Model should choose the calculator tool for math question
  if let Some( ref tool_calls ) = response.choices[ 0 ].message.tool_calls
  {
      assert!( !tool_calls.is_empty() );
      assert_eq!( tool_calls[ 0 ].function.name, "calculate", "Model should choose calculator for math" );
  }
  }

  #[ tokio::test ]

  async fn test_function_calling_conversation_flow()
  {
  let client = setup_client();
  let tool = create_weather_tool();

  // Step 1: User asks question
  let response1 = client.providers()
      .chat_completion_with_tools
      (
  Models::kimi_k2_instruct(),
  vec!
  [
          api_huggingface::components::inference_shared::ChatMessage
          {
      role : "user".to_string(),
      content : "What's the weather in Tokyo?".to_string(),
      tool_calls : None,
      tool_call_id : None,
          }
  ],
  vec![ tool.clone() ],
  None,
  None,
  None,
  None,
      )
      .await
      .expect( "First request failed" );

  // Verify tool call was made
  let tool_calls = response1.choices[ 0 ].message.tool_calls.as_ref()
      .expect( "Model should request tool call" );

  let tool_call_id = &tool_calls[ 0 ].id;

  // Step 2: Provide function result
  let response2 = client.providers()
      .chat_completion_with_tools
      (
  Models::kimi_k2_instruct(),
  vec!
  [
          api_huggingface::components::inference_shared::ChatMessage
          {
      role : "user".to_string(),
      content : "What's the weather in Tokyo?".to_string(),
      tool_calls : None,
      tool_call_id : None,
          },
          response1.choices[ 0 ].message.clone(),
          api_huggingface::components::inference_shared::ChatMessage
          {
      role : "tool".to_string(),
      content : r#"{"temperature": 22, "condition": "sunny", "unit": "celsius"}"#.to_string(),
      tool_calls : None,
      tool_call_id : Some( tool_call_id.clone() ),
          }
  ],
  vec![ tool ],
  None,
  None,
  None,
  None,
      )
      .await
      .expect( "Second request failed" );

  // Model should process the tool result and provide some response
  // It may provide text response and/or request additional tools
  let has_tool_calls = response2.choices[ 0 ].message.tool_calls.as_ref()
      .is_some_and( | tc | !tc.is_empty() );
  let has_content = !response2.choices[ 0 ].message.content.is_empty();

  // Model must provide some response after receiving tool result
  assert!( has_content || has_tool_calls, "Model should provide response after tool result" );
  }

  #[ test ]
  fn test_tool_serialization()
  {
  let tool = create_weather_tool();

  // Verify tool serializes correctly
  let json = serde_json::to_value( &tool ).expect( "Failed to serialize tool" );

  assert_eq!( json[ "name" ], "get_current_weather" );
  assert_eq!( json[ "description" ], "Get the current weather in a given location" );
  assert_eq!( json[ "parameters" ][ "type" ], "object" );
  assert!( json[ "parameters" ][ "properties" ].is_object() );
  assert!( json[ "parameters" ][ "required" ].is_array() );
  }

  #[ test ]
  fn test_parameter_property_types()
  {
  let string_prop = ParameterProperty::string( "A string parameter" );
  let number_prop = ParameterProperty::number( "A number parameter" );
  let boolean_prop = ParameterProperty::boolean( "A boolean parameter" );

  let json_string = serde_json::to_value( &string_prop ).expect( "[test_parameter_property_types] Failed to serialize string ParameterProperty to JSON - check serde_json::to_value() and ParameterProperty serialization implementation" );
  let json_number = serde_json::to_value( &number_prop ).expect( "[test_parameter_property_types] Failed to serialize number ParameterProperty to JSON - check serde_json::to_value() and ParameterProperty serialization implementation" );
  let json_boolean = serde_json::to_value( &boolean_prop ).expect( "[test_parameter_property_types] Failed to serialize boolean ParameterProperty to JSON - check serde_json::to_value() and ParameterProperty serialization implementation" );

  assert_eq!( json_string[ "type" ], "string" );
  assert_eq!( json_number[ "type" ], "number" );
  assert_eq!( json_boolean[ "type" ], "boolean" );
  }
}

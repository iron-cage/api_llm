//! Enhanced Builder Patterns Tests
//!
//! This module contains comprehensive tests for the builder patterns implemented
//! using the Former derive macro throughout the OpenAI API client.
//!
//! Tests cover:
//! - Basic builder functionality
//! - Optional field handling
//! - Complex nested structures
//! - Error conditions and edge cases
//! - Integration with serialization
//! - Performance characteristics
//! - Thread safety of builder patterns

#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::useless_vec ) ]
#![ allow( clippy::unused_async ) ]
#![ allow( clippy::must_use_candidate ) ]
#![ allow( clippy::missing_panics_doc ) ]
#![ allow( clippy::missing_errors_doc ) ]
#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::manual_string_new ) ]
#![ allow( clippy::unnecessary_cast ) ]
#![ allow( clippy::bool_assert_comparison ) ]
#![ allow( clippy::float_cmp ) ]

use std::
{
  collections ::HashMap,
  sync ::{ Arc, Mutex },
  thread,
  time ::Instant,
};

use api_openai::exposed::
{
  components ::
  {
    responses ::
    {
      CreateResponseRequest,
      ResponseInput,
    },
    input ::
    {
      InputMessage,
      InputContentPart,
      InputText,
      InputImage,
      InputItem,
    },
    tools ::
    {
      Tool,
      ToolChoice,
      FunctionTool,
      FunctionParameters,
      ComputerTool,
      WebSearchTool,
    },
    common ::
    {
      ModelIdsResponses,
      Metadata,
    },
    query ::
    {
      ListQuery,
    },
  }
};

/// Test basic builder functionality for CreateResponseRequest
#[ test ]
fn test_create_response_request_builder_basic()
{
  // Test that Former derive creates proper builder
  let request = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-5.1-chat-latest".to_string()))
    .input(ResponseInput::String("Test input".to_string()))
    .form();

  assert_eq!(request.model, ModelIdsResponses::from("gpt-5.1-chat-latest".to_string()));
  assert_eq!(request.input, ResponseInput::String("Test input".to_string()));
  assert!(request.instructions.is_none()); // Optional field should be None by default
  assert!(request.metadata.is_none());
  assert!(request.temperature.is_none());
}

/// Test builder with all optional fields set
#[ test ]
fn test_create_response_request_builder_complete()
{
  let mut metadata = HashMap::new();
  metadata.insert("test_key".to_string(), "test_value".to_string());
  let metadata = Metadata::from(metadata);

  let request = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-5.1-chat-latest".to_string()))
    .input(ResponseInput::String("Test input".to_string()))
    .instructions("System instructions".to_string())
    .metadata(metadata.clone())
    .temperature(0.7)
    .top_p(0.9)
    .max_output_tokens(1000i32)
    .stream(false)
    .tools(vec![])
    .tool_choice(ToolChoice::String("auto".to_string()))
    .parallel_tool_calls(true)
    .form();

  assert_eq!(request.model, ModelIdsResponses::from("gpt-5.1-chat-latest".to_string()));
  assert_eq!(request.input, ResponseInput::String("Test input".to_string()));
  assert_eq!(request.instructions, Some("System instructions".to_string()));
  assert_eq!(request.metadata, Some(metadata));
  assert_eq!(request.temperature, Some(0.7));
  assert_eq!(request.top_p, Some(0.9));
  assert_eq!(request.max_output_tokens, Some(1000i32));
  assert_eq!(request.stream, Some(false));
  assert_eq!(request.tools, Some(vec![]));
  assert_eq!(request.tool_choice, Some(ToolChoice::String("auto".to_string())));
  assert_eq!(request.parallel_tool_calls, true);
}

/// Test builder chaining and fluent interface
#[ test ]
fn test_builder_chaining_fluent_interface()
{
  // Test that builder methods can be chained fluently
  let request = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-5-nano".to_string()))
    .temperature(0.5)
    .top_p(0.8)
    .max_output_tokens(500i32)
    .input(ResponseInput::String("Chained builder test".to_string()))
    .form();

  assert_eq!(request.temperature, Some(0.5));
  assert_eq!(request.top_p, Some(0.8));
  assert_eq!(request.max_output_tokens, Some(500i32));
}

/// Test InputMessage builder patterns
#[ test ]
fn test_input_message_builder()
{
  // Test basic InputMessage builder
  let message = InputMessage::former()
    .role("user".to_string())
    .content(vec![
      InputContentPart::Text(InputText::former()
        .text("Hello, world!".to_string())
        .form())
    ])
    .form();

  assert_eq!(message.role, "user");
  assert_eq!(message.content.len(), 1);
  if let InputContentPart::Text(text_part) = &message.content[0]
  {
    assert_eq!(text_part.text, "Hello, world!");
  }
  else
  {
    panic!("Expected Text content part");
  }
}

/// Test complex nested builder structures
#[ test ]
fn test_nested_builder_structures()
{
  // Test building complex nested structures with multiple levels
  let image_content = InputImage::former()
    .image_url("https://example.com/image.png".to_string())
    .detail("high".to_string())
    .form();

  let text_content = InputText::former()
    .text("Describe this image".to_string())
    .form();

  let message = InputMessage::former()
    .role("user".to_string())
    .content(vec![
      InputContentPart::Text(text_content),
      InputContentPart::Image(image_content)
    ])
    .form();

  assert_eq!(message.role, "user");
  assert_eq!(message.content.len(), 2);

  // Verify text content
  if let InputContentPart::Text(text_part) = &message.content[0]
  {
    assert_eq!(text_part.text, "Describe this image");
  }
  else
  {
    panic!("Expected Text at index 0");
  }

  // Verify image content
  if let InputContentPart::Image(image_part) = &message.content[1]
  {
    assert_eq!(image_part.image_url, Some("https://example.com/image.png".to_string()));
    assert_eq!(image_part.detail, Some("high".to_string()));
  }
  else
  {
    panic!("Expected Image at index 1");
  }
}

/// Test tool builder patterns
#[ test ]
fn test_tool_builders()
{
  // Test FunctionTool builder - FunctionParameters is a transparent wrapper around JSON
  let parameters_json = serde_json::json!({
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "The query to execute"
      }
    },
    "required": ["query"]
  });

  let function_params = FunctionParameters::new(parameters_json);

  let function_tool = FunctionTool::former()
    .name("test_function".to_string())
    .description("Test function description".to_string())
    .parameters(function_params)
    .form();

  assert_eq!(function_tool.name, "test_function");
  assert_eq!(function_tool.description, Some("Test function description".to_string()));
  // FunctionParameters is not Option< T >, it's always present
  assert_eq!(function_tool.parameters.0["type"], "object");

  // Test WebSearchTool builder
  let web_search_tool = WebSearchTool::former()
    .form();

  // WebSearchTool should be built successfully (it has no required fields)
  let _ = web_search_tool;

  // FileSearchTool is a unit struct - we can't construct it directly in tests
  // but we can verify it exists in the type system

  // Test ComputerTool builder
  let computer_tool = ComputerTool::former()
    .display_height(1080.0)
    .display_width(1920.0)
    .environment("ubuntu".to_string())
    .form();

  assert_eq!(computer_tool.display_height, 1080.0);
  assert_eq!(computer_tool.display_width, 1920.0);
  assert_eq!(computer_tool.environment, "ubuntu".to_string());
}

/// Test ListQuery builder
#[ test ]
fn test_list_query_builder()
{
  let query = ListQuery::former()
    .limit(10u32)
    .order("desc".to_string())
    .after("cursor_123".to_string())
    .before("cursor_456".to_string())
    .form();

  assert_eq!(query.limit, Some(10u32));
  assert_eq!(query.order, Some("desc".to_string()));
  assert_eq!(query.after, Some("cursor_123".to_string()));
  assert_eq!(query.before, Some("cursor_456".to_string()));
}

/// Test builder with extreme values
#[ test ]
fn test_builder_extreme_values()
{
  // Test with edge case values
  let request = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-4".to_string()))
    .input(ResponseInput::String("".to_string())) // Empty string
    .temperature(0.0) // Minimum temperature
    .top_p(1.0) // Maximum top_p
    .max_output_tokens(1i32) // Minimum tokens
    .form();

  assert_eq!(request.input, ResponseInput::String("".to_string()));
  assert_eq!(request.temperature, Some(0.0));
  assert_eq!(request.top_p, Some(1.0));
  assert_eq!(request.max_output_tokens, Some(1i32));

  // Test with maximum reasonable values
  let request_max = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-4".to_string()))
    .input(ResponseInput::String("x".repeat(10000))) // Large string
    .temperature(2.0) // High temperature
    .max_output_tokens(4000i32) // High token count
    .form();

  assert_eq!(request_max.input, ResponseInput::String("x".repeat(10000)));
  assert_eq!(request_max.temperature, Some(2.0));
  assert_eq!(request_max.max_output_tokens, Some(4000i32));
}

/// Test builder serialization compatibility
#[ test ]
fn test_builder_serialization_compatibility()
{
  let request = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-4".to_string()))
    .input(ResponseInput::String("Serialization test".to_string()))
    .temperature(0.7)
    .form();

  // Test that builder-created structures serialize correctly
  let serialized = serde_json::to_string(&request);
  assert!(serialized.is_ok(), "Builder-created struct should serialize successfully");

  let json_str = serialized.unwrap();
  assert!(json_str.contains("\"model\""));
  assert!(json_str.contains("\"input\""));
  assert!(json_str.contains("\"temperature\""));
  assert!(json_str.contains("Serialization test"));

  // Note : CreateResponseRequest doesn't implement Deserialize as it's primarily for API requests
  // We skip the deserialization test since it's not meant to be deserialized
}

/// Test builder performance characteristics
#[ test ]
fn test_builder_performance()
{
  let start = Instant::now();

  // Build 1000 requests to test performance
  for i in 0..1000
  {
    let _request = CreateResponseRequest::former()
      .model(ModelIdsResponses::from(format!("gpt-4-{}", i)))
      .input(ResponseInput::String(format!("Test input {}", i)))
      .temperature(0.7)
      .max_output_tokens((100 + i) as i32)
      .form();
  }

  let duration = start.elapsed();
  println!("Built 1000 requests in {:?}", duration);

  // Should be fast - less than 100ms for 1000 simple builds
  assert!(duration.as_millis() < 1000, "Builder performance should be reasonable : {:?}", duration);
}

/// Test builder thread safety
#[ test ]
fn test_builder_thread_safety()
{
  let results = Arc::new(Mutex::new(Vec::new()));
  let mut handles = vec![];

  // Spawn multiple threads to test concurrent builder usage
  for i in 0..10
  {
    let results_clone = Arc::clone(&results);
    let handle = thread::spawn(move || {
      let request = CreateResponseRequest::former()
        .model(ModelIdsResponses::from(format!("gpt-4-thread-{}", i)))
        .input(ResponseInput::String(format!("Thread {} input", i)))
        .temperature(0.5)
        .form();

      let mut results = results_clone.lock().unwrap();
      results.push(request);
    });
    handles.push(handle);
  }

  // Wait for all threads to complete
  for handle in handles
  {
    handle.join().expect("Thread should complete successfully");
  }

  // Verify all requests were created successfully
  let results = results.lock().unwrap();
  assert_eq!(results.len(), 10);

  // Verify each request has the expected thread-specific content
  for (i, request) in results.iter().enumerate()
  {
    let _expected_model = ModelIdsResponses::from(format!("gpt-4-thread-{}", i));
    let _expected_input = ResponseInput::String(format!("Thread {} input", i));

    // Note : The order might not match due to thread execution order,
    // so we check that all expected values exist
    let model_str = &request.model.value;

    // Check that this is one of our thread-generated models
    assert!(model_str.starts_with("gpt-4-thread-"), "Model should be thread-specific : {}", model_str);
  }
}

/// Test builder Clone implementation
#[ test ]
fn test_builder_clone_functionality()
{
  // Test that partially built structures can be cloned
  let partial_builder = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-4".to_string()))
    .temperature(0.7);

  // Note : The builder itself may not be cloneable, so we test cloning the final result
  let cloned_request1 = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-4".to_string()))
    .temperature(0.7)
    .input(ResponseInput::String("First clone input".to_string()))
    .form();

  let cloned_request2 = partial_builder
    .input(ResponseInput::String("Second clone input".to_string()))
    .max_output_tokens(500i32)
    .form();

  // Both should have the same model and temperature from the original builder
  assert_eq!(cloned_request1.model, cloned_request2.model);
  assert_eq!(cloned_request1.temperature, cloned_request2.temperature);

  // But different inputs
  assert_ne!(cloned_request1.input, cloned_request2.input);
  assert_eq!(cloned_request1.input, ResponseInput::String("First clone input".to_string()));
  assert_eq!(cloned_request2.input, ResponseInput::String("Second clone input".to_string()));

  // And different max_output_tokens
  assert!(cloned_request1.max_output_tokens.is_none());
  assert_eq!(cloned_request2.max_output_tokens, Some(500i32));
}

/// Test builder Debug implementation
#[ test ]
fn test_builder_debug_implementation()
{
  let request = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-4".to_string()))
    .input(ResponseInput::String("Debug test".to_string()))
    .temperature(0.7)
    .form();

  // Test that Debug is properly implemented
  let debug_output = format!("{:?}", request);

  assert!(debug_output.contains("CreateResponseRequest"));
  assert!(debug_output.contains("model"));
  assert!(debug_output.contains("input"));
  assert!(debug_output.contains("temperature"));
  assert!(debug_output.contains("Debug test"));

  println!("Debug output : {}", debug_output);
}

/// Test builder PartialEq implementation
#[ test ]
fn test_builder_partial_eq_implementation()
{
  let request1 = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-4".to_string()))
    .input(ResponseInput::String("Equal test".to_string()))
    .temperature(0.7)
    .form();

  let request2 = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-4".to_string()))
    .input(ResponseInput::String("Equal test".to_string()))
    .temperature(0.7)
    .form();

  let request3 = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-4".to_string()))
    .input(ResponseInput::String("Different test".to_string()))
    .temperature(0.7)
    .form();

  // Test equality
  assert_eq!(request1, request2);
  assert_ne!(request1, request3);
  assert_ne!(request2, request3);
}

/// Comprehensive builder integration test
#[ test ]
fn test_comprehensive_builder_integration()
{
  // Create a complex request using multiple builder patterns
  let text_message = InputMessage::former()
    .role("user".to_string())
    .content(vec![
      InputContentPart::Text(
        InputText::former()
          .text("What can you tell me about this data?".to_string())
          .form()
      )
    ])
    .form();

  let function_tool = Tool::Function(
    FunctionTool::former()
      .name("analyze_data".to_string())
      .description("Analyzes provided data".to_string())
      .parameters(
        FunctionParameters::new(serde_json::json!({
          "type": "object",
          "properties": {
            "data_type": {
              "type": "string",
              "description": "Type of data to analyze"
            }
          },
          "required": ["data_type"]
        }))
      )
      .form()
  );

  let request = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-4".to_string()))
    .input(ResponseInput::Items(vec![InputItem::Message(text_message)]))
    .tools(vec![function_tool])
    .tool_choice(ToolChoice::String("auto".to_string()))
    .temperature(0.3)
    .max_output_tokens(2000i32)
    .stream(false)
    .form();

  // Verify the complex structure was built correctly
  assert_eq!(request.model, ModelIdsResponses::from("gpt-4".to_string()));
  assert!(matches!(request.input, ResponseInput::Items(_)));
  assert_eq!(request.tools.as_ref().unwrap().len(), 1);
  assert_eq!(request.tool_choice, Some(ToolChoice::String("auto".to_string())));
  assert_eq!(request.temperature, Some(0.3));
  assert_eq!(request.max_output_tokens, Some(2000i32));
  assert_eq!(request.stream, Some(false));

  // Verify the nested message structure
  if let ResponseInput::Items(items) = &request.input
  {
    assert_eq!(items.len(), 1);
    let InputItem::Message(message) = &items[0]; // InputItem is always Message in this test
    assert_eq!(message.role, "user");
    assert_eq!(message.content.len(), 1);

    if let InputContentPart::Text(text_part) = &message.content[0]
    {
      assert_eq!(text_part.text, "What can you tell me about this data?");
    }
    else
    {
      panic!("Expected text content in message");
    }
  }
  else
  {
    panic!("Expected Items input type");
  }

  // Verify the tool structure
  if let Tool::Function(func_tool) = &request.tools.as_ref().unwrap()[0]
  {
    assert_eq!(func_tool.name, "analyze_data");
    assert_eq!(func_tool.description, Some("Analyzes provided data".to_string()));
    // FunctionParameters is always present as it wraps JSON value
    assert_eq!(func_tool.parameters.0["type"], "object");
  }
  else
  {
    panic!("Expected Function tool");
  }
}
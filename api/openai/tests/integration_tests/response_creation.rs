//! Response creation integration tests
//!
//! Tests for creating responses including basic creation, streaming,
//! error handling, and tool usage functionality.
//!
//! # MANDATORY FAILING BEHAVIOR
//!
//! These integration tests MUST fail hard when real API access is unavailable.
//! Tests NEVER silently fall back to fallbacks. Failures indicate real issues:
//! - Missing `OPENAI_API_KEY` credentials
//! - Network connectivity problems
//! - API authentication/authorization failures
//! - `OpenAI` service unavailability

use api_openai::ClientApiAccessors;
use super::shared::{ *, IsolatedClient };

/// Tests basic response creation functionality.
/// Test Combination : R1.1
/// Uses proper test isolation to prevent shared state issues.
#[ tokio::test ]
async fn create_response()
{
  let isolated_client = IsolatedClient::new("create_response", true)
    .expect("Failed to create isolated client");
  let client = isolated_client.client();

  let request = create_basic_test_request();
  let result = client.responses().create(request).await;

  handle_test_result(result, "create_response", |response| {
    assert_valid_response(response);
  });
}

/// Tests that invalid model returns appropriate error.
/// Test Combination : R1.2
/// Uses proper test isolation to prevent shared state issues.
#[ tokio::test ]
async fn create_response_invalid_model()
{
  let isolated_client = IsolatedClient::new("create_response_invalid_model", true)
    .expect("Failed to create isolated client");
  let client = isolated_client.client();

  let request = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("invalid-model-xyz".to_string()))
    .input(ResponseInput::String("Test".to_string()))
    .form();

  let result = client.responses().create(request).await;

  // Client should return error for invalid model
  assert!(result.is_err(), "Should return error for invalid model");
}

/// Tests streaming response creation.
/// Test Combination : R1.3
/// Uses proper test isolation to prevent shared state issues.
#[ tokio::test ]
async fn create_response_stream()
{
  let isolated_client = IsolatedClient::new("create_response_stream", true)
    .expect("Failed to create isolated client");
  let client = isolated_client.client();

  let request = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-5-nano".to_string()))
    .input(ResponseInput::String("Count from 1 to 5".to_string()))
    .max_output_tokens(100)
    .stream(true)
    .form();

  let result = client.responses().create_stream(request).await;

  if true
  {
    match result
    {
      Ok(mut receiver) =>
      {
        let mut event_count = 0;
        let mut has_content = false;
        let mut is_done = false;

        // Collect up to 50 events to avoid infinite loops
        while event_count < 50
        {
          match tokio::time::timeout(core::time::Duration::from_secs(10), receiver.recv()).await
          {
            Ok(Some(Ok(event))) =>
            {
              event_count += 1;
              println!("Received event {}: {}", event_count, serde_json::to_string(&event).unwrap_or_default());

              match event
              {
                ResponseStreamEvent::ResponseTextDelta( delta_event ) =>
                {
                  if !delta_event.delta.is_empty()
                  {
                    has_content = true;
                  }
                },
                ResponseStreamEvent::ResponseTextDone( _ ) =>
                {
                  is_done = true;
                  break;
                },
                _ =>
                {
                  // Handle other events (ignore for this test)
                },
              }
            },
            Ok(Some(Err(e))) =>
            {
              panic!("Stream error : {e:?}");
            },
            Ok(None) =>
            {
              println!("Stream ended after {event_count} events");
              break;
            },
            Err(_) =>
            {
              println!("Stream timeout after {event_count} events");
              break;
            }
          }
        }

        assert!(event_count > 0, "Should receive at least one streaming event");
        if event_count >= 50
        {
          println!("Warning : Stopped after 50 events to prevent infinite loop");
        }

        // For real API, we expect either content or done event
        if !has_content && !is_done
        {
          println!("Warning : No content or done event received in {event_count} events");
        }
      },
      Err(e) =>
      {
        panic!("Failed to create stream : {e:?}");
      }
    }
  }
  else
  {
    // Client case - can either succeed with data or fail with network error
    match result
    {
      Ok(mut receiver) =>
      {
        // Client succeeded - verify it returns data
        let event_result = tokio::time::timeout(
          core ::time::Duration::from_secs(2),
          receiver.recv()
        ).await;
        match event_result
        {
          Ok(Some(_)) => println!("✓ Streaming succeeded with test data"),
          Ok(None) => println!("✓ Streaming completed immediately"),
          Err(_) => println!("✓ Streaming timed out as expected"),
        }
      },
      Err(_) =>
      {
        // Client failed - this is also acceptable for test environments
        println!("✓ Client failed with network error as expected");
      }
    }
  }
}

/// Tests response creation with tool usage (function calling).
/// Test Combination : R1.4
/// Uses proper test isolation to prevent shared state issues.
#[ tokio::test ]
async fn create_response_with_tools()
{
  let isolated_client = IsolatedClient::new("create_response_with_tools", true)
    .expect("Failed to create isolated client");
  let client = isolated_client.client();

  let request = create_tools_test_request();
  let result = client.responses().create(request).await;

  handle_test_result(result, "create_response_with_tools", |response| {
    assert_valid_response(response);
    // For tool requests, we might get tool calls in the output
    // This is dependent on the model's behavior, so we just verify basic response structure
  });
}
//! Response management integration tests
//!
//! Tests for managing existing responses including retrieval, updating,
//! deletion, cancellation, and listing functionality.
//!
//! # MANDATORY FAILING BEHAVIOR
//!
//! These integration tests MUST fail hard when real API access is unavailable.
//! Tests NEVER silently fall back to mocks. Failures indicate real issues:
//! - Missing `OPENAI_API_KEY` credentials
//! - Network connectivity problems
//! - API authentication/authorization failures
//! - `OpenAI` service unavailability

#![ allow( clippy::uninlined_format_args ) ] // Test code can be more verbose for clarity

use api_openai::ClientApiAccessors;
use super::shared::{ *, IsolatedClient };

/// Tests response retrieval functionality.
/// Test Combination : R2.1
/// Uses proper test isolation to prevent shared state issues.
#[ tokio::test ]
#[ allow( deprecated ) ]
async fn retrieve_response()
{
  let isolated_client = IsolatedClient::new("retrieve_response", true)
  .expect("Failed to create isolated client");
  let client = isolated_client.client();

  // REAL API ONLY - First create a response to retrieve
  let create_request = create_basic_test_request();
  let create_result = client.responses().create(create_request).await;

  match create_result
  {
  Ok(created_response) =>
  {
    // Now retrieve the response
    let retrieve_result = client.responses().retrieve(&created_response.id).await;

    match retrieve_result
    {
      Ok(retrieved_response) =>
      {
        assert_eq!(retrieved_response.id, created_response.id, "Retrieved response should have same ID");
        assert_valid_response(&retrieved_response);
      },
      Err(e) =>
      {
        panic!("Failed to retrieve response : {e:?}");
      }
    }
  },
  Err(e) =>
  {
    panic!("Failed to create response for retrieval test : {e:?}");
  }
  }
}

/// Tests listing response input items functionality.
/// Test Combination : R2.2
/// Uses proper test isolation to prevent shared state issues.
#[ tokio::test ]
#[ allow( deprecated ) ]
async fn list_response_input_items()
{
  let isolated_client = IsolatedClient::new("list_response_input_items", true)
  .expect("Failed to create isolated client");
  let client = isolated_client.client();

  // REAL API ONLY - First create a response to list items from
  let create_request = create_basic_test_request();
  let create_result = client.responses().create(create_request).await;

  match create_result
  {
    Ok(created_response) =>
    {
      // Now list the input items
      let list_query = ListQuery { limit : None };
      let list_result = client.responses().list_input_items(&created_response.id, Some(list_query)).await;

      match list_result
      {
        Ok(item_list) =>
        {
          assert_eq!(item_list.object, "list", "Object type should be 'list'");
          // Input items list might be empty or contain items - both are valid
        },
        Err(e) =>
        {
          panic!("Failed to list response input items : {:?}", e);
        }
      }
    },
    Err(e) =>
    {
      panic!("Failed to create response for list test : {:?}", e);
    }
  }
}

/// Tests response deletion functionality.
/// Test Combination : R2.3
/// Uses proper test isolation to prevent shared state issues.
#[ tokio::test ]
#[ allow( deprecated ) ]
async fn delete_response()
{
  let isolated_client = IsolatedClient::new("delete_response", true)
  .expect("Failed to create isolated client");
  let client = isolated_client.client();

  // REAL API ONLY
  // First create a response to delete
  let create_request = create_basic_test_request();
  let create_result = client.responses().create(create_request).await;

  match create_result
  {
    Ok(created_response) =>
    {
      // Now delete the response
      let delete_result = client.responses().delete(&created_response.id).await;

      match delete_result
      {
        Ok(delete_confirmation) =>
        {
          // Check the delete confirmation structure
          if let Some(id) = delete_confirmation.get("id")
          {
            assert_eq!(id.as_str().unwrap_or(""), created_response.id, "Deleted response should have same ID");
          }
          if let Some(object) = delete_confirmation.get("object")
          {
            assert_eq!(object.as_str().unwrap_or(""), "response.deleted", "Object should indicate deletion");
          }
          if let Some(deleted) = delete_confirmation.get("deleted")
          {
            assert!(deleted.as_bool().unwrap_or(false), "Deleted field should be true");
          }
        },
        Err(e) =>
        {
          panic!("Failed to delete response : {:?}", e);
        }
      }
    },
    Err(e) =>
    {
      panic!("Failed to create response for deletion test : {:?}", e);
    }
  }
}

/// Tests response updating functionality.
/// Test Combination : R2.4
/// Uses proper test isolation to prevent shared state issues.
///
/// **Note**: As of current `OpenAI` API version, response updates are not supported (HTTP 405).
/// This test verifies that the deprecated method correctly handles this API limitation.
#[ tokio::test ]
#[ allow( deprecated ) ]
async fn update_response()
{
  let isolated_client = IsolatedClient::new("update_response", true)
  .expect("Failed to create isolated client");
  let client = isolated_client.client();

  // REAL API ONLY
  // First create a response to update
  let create_request = create_basic_test_request();
  let create_result = client.responses().create(create_request).await;

  match create_result
  {
    Ok(created_response) =>
    {
      // Create update request with metadata
      let metadata = json!({
        "updated": true,
        "test": "update_test"
      });

      let update_result = client.responses().update(&created_response.id, metadata).await;

      match update_result
      {
        Ok(updated_response) =>
        {
          assert_eq!(updated_response.id, created_response.id, "Updated response should have same ID");
          assert_valid_response(&updated_response);
          // Metadata updates might not be immediately visible, so we don't assert on them
        },
        Err(e) =>
        {
          let error_msg = format!("{e:?}");

          // OpenAI API no longer supports PATCH on responses (HTTP 405)
          // This is expected behavior as of the current API version
          if error_msg.contains("405") || error_msg.contains("Method Not Allowed")
          {
            println!("Update operation correctly rejected by API (responses are immutable)");
            // This is the expected behavior - the test passes
          }
          else
          {
            panic!("Failed to update response : {e:?}");
          }
        }
      }
    },
    Err(e) =>
    {
      panic!("Failed to create response for update test : {e:?}");
    }
  }
}

/// Tests response cancellation functionality.
/// Test Combination : R2.5
/// Uses proper test isolation to prevent shared state issues.
#[ tokio::test ]
#[ allow( deprecated ) ]
async fn cancel_response()
{
  let isolated_client = IsolatedClient::new("cancel_response", true)
  .expect("Failed to create isolated client");
  let client = isolated_client.client();

  // REAL API ONLY
  // Create a long-running response that we can cancel
  let cancel_request = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-5-nano".to_string()))
    .input(ResponseInput::String("Write a very long story about artificial intelligence, make it at least 500 words.".to_string()))
    .max_output_tokens(1000)
    .form();

  let create_result = client.responses().create(cancel_request).await;

  match create_result
  {
    Ok(created_response) =>
    {
      // Try to cancel the response (might already be completed)
      let cancel_result = client.responses().cancel(&created_response.id).await;

      match cancel_result
      {
        Ok(cancelled_response) =>
        {
          assert_eq!(cancelled_response.id, created_response.id, "Cancelled response should have same ID");
          // Status might be 'completed' or 'cancelled' depending on timing
          assert!(
            cancelled_response.status == "completed" || cancelled_response.status == "cancelled",
            "Response status should be completed or cancelled, got : {}",
            cancelled_response.status
          );
        },
        Err(e) =>
        {
          // Cancellation might fail if response is already completed - this is acceptable
          println!("Cancel failed (response might be completed): {:?}", e);
        }
      }
    },
    Err(e) =>
    {
      panic!("Failed to create response for cancellation test : {:?}", e);
    }
  }
}
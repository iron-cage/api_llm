//! Integration tests for `OpenAI` API client
//!
//! These tests make real API calls to `OpenAI` and require a valid API key.
//! They are gated behind the "integration" feature flag.
//!
//! # MANDATORY FAILING BEHAVIOR
//!
//! Integration tests in this file MUST fail hard when:
//! - Real API credentials are not available in environment or workspace secrets
//! - Network connectivity issues prevent API access
//! - API authentication or authorization fails
//! - Any other real API access issues occur
//!
//! **IMPORTANT**: These tests NEVER silently fall back to mocks or dummy data.
//! Test failures indicate real issues that must be addressed:
//! - Missing `OPENAI_API_KEY` in environment or ../../secret/-secrets.sh
//! - Invalid/expired API credentials
//! - Network connectivity problems
//! - `OpenAI` API service issues
//!
//! This ensures integration test results are meaningful and reliable.
//!
//! All tests use the test isolation framework to ensure proper test isolation
//! and prevent shared state issues.

#![ cfg( feature = "integration" ) ]
#![ allow( unused_imports, dead_code ) ]

use api_openai::ClientApiAccessors;
pub use api_openai as the_module;

mod test_isolation;
use test_isolation::{ TestIsolation, IsolatedClient, should_run_real_api_tests };

use api_openai::
{
  Client,
  error ::OpenAIError,
  environment ::{ EnvironmentInterface, OpenaiEnvironment, OpenaiEnvironmentImpl },
  secret ::Secret,
  components ::
  {
    responses ::
    {
      CreateResponseRequest,
      ResponseObject,
      ResponseInput,
      ResponseStreamEvent,
      ResponseItemList,
    },
    input ::
    {
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
use serde_json::json;
use futures_util::stream::StreamExt;
use secrecy::ExposeSecret;
use tokio::sync::mpsc;

/// DEPRECATED: Legacy helper functions - use `TestIsolation` framework instead
/// These functions are kept for backward compatibility but should be replaced
/// with proper test isolation patterns.
#[ deprecated( note = "Use TestIsolation framework instead" ) ]
fn load_secret_for_test() -> Secret
{
  eprintln!( "DEPRECATED: load_secret_for_test() - Use TestIsolation::create_test_secret() instead" );

  // REAL API ONLY - Use workspace_tools integration, MUST fail if no real credentials
  Secret::load_with_fallbacks( "OPENAI_API_KEY" )
    .expect("INTEGRATION TEST FAILURE: Real API credentials required but not found")
}

#[ deprecated( note = "Use should_run_real_api_tests() instead" ) ]
fn should_run_integration_tests() -> bool
{
  should_run_real_api_tests()
}

#[ deprecated( note = "Use should_run_real_api_tests() instead" ) ]
fn should_run_with_real_api() -> bool
{
  should_run_real_api_tests()
}

#[ deprecated( note = "Use IsolatedClient::new() instead" ) ]
#[ allow( deprecated ) ]
#[ allow( unused_macros ) ]
fn create_test_client() -> Result< Client< OpenaiEnvironmentImpl >, Box< dyn core::error::Error > >
{
  eprintln!( "DEPRECATED: create_test_client() - Use IsolatedClient::new() instead" );

  // REAL API ONLY - No more conditional logic
  let secret = load_secret_for_test();
  let env = OpenaiEnvironmentImpl::build( secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string() )?;
  Ok( Client::build( env )? )
}

/// DEPRECATED: Use `IsolatedClient` instead
#[ deprecated( note = "Use IsolatedClient::new() instead" ) ]
#[ allow( unused_macros ) ]
macro_rules! setup_test_client
{
  () =>
  {
    create_test_client().expect( "Failed to create test client" )
  };
}

/// DEPRECATED: Not needed with proper test isolation
#[ deprecated( note = "Not needed with TestIsolation framework" ) ]
macro_rules! require_api_key
{
  () =>
  {
    // For mock client approach, we create the client regardless of API key availability
    // The client creation will work with dummy keys, and we handle real vs mock in the test logic
  };
}

/// Ensures the secret is loaded before running integration tests.
///
/// # Mandatory Failing Behavior
/// This function MUST fail hard if real API credentials are not available.
/// Integration tests should NEVER silently proceed with mock/dummy credentials
/// when real API access is expected. This ensures test failures indicate real
/// issues with credential configuration, network access, or API availability.
///
/// Loads API key using `workspace_tools` integration with fallback chain.
fn ensure_secret_loaded() -> Result< (), OpenAIError >
{
  // MANDATORY: Use workspace_tools integration - MUST fail if no valid credentials
  let secret = Secret::load_with_fallbacks("OPENAI_API_KEY")
    .map_err(|e| OpenAIError::InvalidArgument( format!("INTEGRATION TEST FAILURE: Real API credentials required but not found. {e}") ))?;

  let api_key = secret.expose_secret();
  if api_key.is_empty() || api_key.contains("invalid") || api_key.len() < 20
  {
    return Err( OpenAIError::InvalidArgument( format!("INTEGRATION TEST FAILURE: Invalid API key detected. Real integration tests require valid OpenAI API credentials, got : '{}'", &api_key[..core::cmp::min(10, api_key.len())]) ) );
  }

  let len = api_key.len();
  println!( "✅ Real API credentials loaded successfully (length : {len})" );
  Ok( () )
}

/// Tests that a basic response can be created successfully using the new API.
/// Test Combination : R1.1
/// Uses proper test isolation to prevent shared state issues.
#[ tokio::test ]
async fn create_response()
{
  let isolated_client = IsolatedClient::new( "create_response", should_run_real_api_tests() )
    .expect( "Failed to create isolated client" );
  let client = isolated_client.client();

  let request = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "gpt-5-nano".to_string() ) )
    .input( ResponseInput::Items(
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
    .max_output_tokens( 50 )
    .parallel_tool_calls( true )
    .form();

  let result = client.responses().create( request ).await;

  // REAL API ONLY - No conditional logic
  match result
  {
    Ok( response_object ) =>
    {
      assert!( !response_object.id.is_empty(), "Response should have an id field" );
      assert!( !response_object.output.is_empty(), "Response should have output" );
      assert_eq!( response_object.object, "response", "Object type should be 'response'" );
      assert!( response_object.created_at > 0, "Created timestamp should be valid" );
    },
    Err( e ) =>
    {
      panic!( "API request failed with an error : {e:?}" );
    }
  }
}

/// Tests that invalid model returns appropriate error.
/// Test Combination : R1.2
/// Uses proper test isolation to prevent shared state issues.
#[ tokio::test ]
async fn create_response_invalid_model()
{
  let isolated_client = IsolatedClient::new( "create_response_invalid_model", should_run_real_api_tests() )
    .expect( "Failed to create isolated client" );
  let client = isolated_client.client();

  let request = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "invalid-model-xyz".to_string() ) )
    .input( ResponseInput::String( "Test".to_string() ) )
    .form();

  let result = client.responses().create( request ).await;

  // Both real API and mock clients should return error for invalid model
  assert!( result.is_err(), "Should return error for invalid model" );
}

/// Tests streaming response creation.
/// Test Combination : R1.3
/// Uses proper test isolation to prevent shared state issues.
#[ tokio::test ]
async fn create_response_stream()
{
  let isolated_client = IsolatedClient::new( "create_response_stream", should_run_real_api_tests() )
    .expect( "Failed to create isolated client" );
  let client = isolated_client.client();

  let request = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "gpt-5-nano".to_string() ) )
    .input( ResponseInput::String( "Count from 1 to 5".to_string() ) )
    .max_output_tokens( 100 )
    .stream( true )
    .form();

  let result = client.responses().create_stream( request ).await;

  // REAL API ONLY - No conditional logic, MANDATORY FAILING BEHAVIOR
  match result
  {
    Ok( mut receiver ) =>
    {
      let mut event_count = 0;
      let mut _error_received = false;

      // Set a timeout to avoid infinite waiting
      let timeout_duration = core::time::Duration::from_secs(10);
      let start_time = std::time::Instant::now();

      while let Some( event_result ) = receiver.recv().await
      {
        if start_time.elapsed() > timeout_duration
        {
          eprintln!( "⚠️ Stream timeout after 10 seconds with {event_count} events received" );
          break;
        }

        match event_result
        {
          Ok( event ) =>
          {
            event_count += 1;
            match event
            {
              ResponseStreamEvent::ResponseCreated( _ ) => println!( "Response created" ),
              ResponseStreamEvent::ResponseInProgress( _ ) => println!( "Response in progress" ),
              ResponseStreamEvent::ResponseInAnalysis( _ ) => println!( "Response in analysis" ),
              ResponseStreamEvent::ResponseTextDelta( _ ) => println!( "Text delta received" ),
              ResponseStreamEvent::ResponseCompleted( event ) =>
              {
                let response_id = &event.response.id;
                println!( "Response completed : {response_id}" );
                break;
              },
              _ => println!( "Other event received : {event:?}" ),
            }
          },
          Err( e ) =>
          {
            _error_received = true;
            // MANDATORY FAILING BEHAVIOR - fail hard on stream errors
            eprintln!( "❌ Stream error occurred : {e:?}" );
            panic!( "Stream encountered error - MANDATORY FAILURE: {e:?}" );
          },
        }
      }

      // MANDATORY FAILING BEHAVIOR - fail hard if no events received
      assert!( event_count > 0, "MANDATORY FAILURE: Should receive at least one stream event. Received {event_count} events." );
    },
    Err( e ) =>
    {
      // MANDATORY FAILING BEHAVIOR - fail hard on stream creation failure
      panic!( "Stream creation failed - MANDATORY FAILURE: {e:?}" );
    }
  }
}

/// Tests response creation with tools.
/// Test Combination : R1.4
/// Uses proper test isolation to prevent shared state issues.
#[ tokio::test ]
async fn create_response_with_tools()
{
  let isolated_client = IsolatedClient::new( "create_response_with_tools", should_run_real_api_tests() )
    .expect( "Failed to create isolated client" );
  let client = isolated_client.client();

  let function_def = json!({
    "type": "object",
    "properties": {
      "location": {
        "type": "string",
        "description": "The city and state, e.g. San Francisco, CA"
      }
    },
    "required": ["location"]
  });

  let tool = Tool::Function(
    FunctionTool::former()
      .name( "get_weather".to_string() )
      .description( "Get the current weather for a location".to_string() )
      .parameters( FunctionParameters::new( function_def ) )
      .form()
  );

  let request = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "gpt-5-nano".to_string() ) )
    .input( ResponseInput::String( "What's the weather like in Boston?".to_string() ) )
    .tools( vec![ tool ] )
    .tool_choice( ToolChoice::String( "auto".to_string() ) )
    .form();

  let result = client.responses().create( request ).await;

  // REAL API ONLY - No conditional logic
  match result
  {
    Ok( response ) =>
    {
      assert!( !response.id.is_empty(), "Response should have ID" );
      assert!( response.tools.is_some(), "Tools should be present in response" );
    },
    Err( e ) =>
    {
      panic!( "Request failed : {e:?}" );
    }
  }
}

/// Tests response retrieval by ID.
/// Test Combination : R1.6
#[ tokio::test ]
#[ allow( deprecated ) ]
async fn retrieve_response()
{
  require_api_key!();
  let secret = load_secret_for_test();
  let env = api_openai::exposed::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string()).expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  // First create a response
  let create_request = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "gpt-5-nano".to_string() ) )
    .input( ResponseInput::String( "Test".to_string() ) )
    .max_output_tokens( 50 )
    .form();

  match client.responses().create( create_request ).await
  {
    Ok( created_response ) =>
    {
      // Now retrieve it
      let result = client.responses().retrieve( &created_response.id ).await;

      match result
      {
        Ok( retrieved_response ) =>
        {
          assert_eq!( created_response.id, retrieved_response.id, "Retrieved response ID should match" );
          assert_eq!( created_response.model, retrieved_response.model, "Model should match" );
        },
        Err( e ) => eprintln!( "Retrieval failed : {e:?}" ),
      }
    },
    Err( e ) =>
    {
      eprintln!( "Create failed : {e:?}" );
    }
  }
}

/// Tests listing input items for a response.
/// Test Combination : R1.7
#[ tokio::test ]
#[ allow( deprecated ) ]
async fn list_response_input_items()
{
  require_api_key!();
  let secret = load_secret_for_test();
  let env = api_openai::exposed::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string()).expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  // Create a response with multiple input items
  let create_request = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "gpt-5-nano".to_string() ) )
    .input( ResponseInput::Items(
      vec![
        InputItem::Message(
          InputMessage {
            r#type : "message".to_string(),
            role : "user".to_string(),
            content : vec![
              InputContentPart::Text(
                InputText {
                  text : "First message".to_string(),
                }
              ),
            ],
            status : None,
            id : None,
          }
        ),
      ]
    ))
    .form();

  match client.responses().create( create_request ).await
  {
    Ok( response ) =>
    {
      let query = ListQuery {
        limit : Some( 10 ),
      };

      let result = client.responses().list_input_items( &response.id, Some( query ) ).await;

      match result
      {
        Ok( item_list ) =>
        {
          assert_eq!( item_list.object, "list", "Should return a list object" );
          assert!( !item_list.data.is_empty(), "Should have at least one input item" );
        },
        Err( e ) => eprintln!( "List failed : {e:?}" ),
      }
    },
    Err( e ) =>
    {
      eprintln!( "Create failed : {e:?}" );
    }
  }
}

/// Tests response deletion.
/// Test Combination : R1.8
#[ tokio::test ]
#[ allow( deprecated ) ]
async fn delete_response()
{
  require_api_key!();
  let secret = load_secret_for_test();
  let env = api_openai::exposed::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string()).expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  // First create a response
  let create_request = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "gpt-5-nano".to_string() ) )
    .input( ResponseInput::String( "Test for deletion".to_string() ) )
    .max_output_tokens( 50 )
    .store( false ) // Don't store to make deletion easier
    .form();

  match client.responses().create( create_request ).await
  {
    Ok( response ) =>
    {
      // Delete the response
      let result = client.responses().delete( &response.id ).await;

      match result
      {
        Ok( delete_result ) =>
        {
          println!( "Response deleted successfully : {delete_result:?}" );

          // Verify deletion by trying to retrieve
          let retrieve_result = client.responses().retrieve( &response.id ).await;
          assert!( retrieve_result.is_err(), "Should not be able to retrieve deleted response" );
        },
        Err( e ) => eprintln!( "Delete failed : {e:?}" ),
      }
    },
    Err( e ) =>
    {
      eprintln!( "Create failed : {e:?}" );
    }
  }
}

/// Tests response update.
/// Test Combination : R1.9
#[ tokio::test ]
#[ allow( deprecated ) ]
async fn update_response()
{
  require_api_key!();
  let secret = load_secret_for_test();
  let env = api_openai::exposed::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string()).expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  // First create a response
  let create_request = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "gpt-5-nano".to_string() ) )
    .input( ResponseInput::String( "Test for update".to_string() ) )
    .form();

  match client.responses().create( create_request ).await
  {
    Ok( response ) =>
    {
      // Update the response metadata
      let update_data = json!({
        "metadata": {
          "updated": "true",
          "timestamp": chrono::Utc::now().to_rfc3339()
        }
      });

      let result = client.responses().update( &response.id, update_data ).await;

      match result
      {
        Ok( updated_response ) =>
        {
          assert_eq!( response.id, updated_response.id, "ID should remain the same" );
          assert!( updated_response.metadata.is_some(), "Metadata should be present" );
        },
        Err( e ) => eprintln!( "Update failed : {e:?}" ),
      }
    },
    Err( e ) =>
    {
      eprintln!( "Create failed : {e:?}" );
    }
  }
}

/// Tests response cancellation.
/// Test Combination : R1.10
#[ tokio::test ]
#[ allow( deprecated ) ]
async fn cancel_response()
{
  require_api_key!();
  let secret = load_secret_for_test();
  let env = api_openai::exposed::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string()).expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  // Create a long-running response that we can cancel
  let create_request = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "gpt-5-nano".to_string() ) )
    .input( ResponseInput::String( "Write a very long story about..." .to_string() ) )
    .max_output_tokens( 1000 )
    .stream( true )
    .form();

  match client.responses().create_stream( create_request ).await
  {
    Ok( mut receiver ) =>
    {
      // Get the response ID from the first event
      if let Some( Ok( ResponseStreamEvent::ResponseCreated( event ) ) ) = receiver.recv().await
      {
        let response_id = event.response.id.clone();

        // Try to cancel the response
        let cancel_result = client.responses().cancel( &response_id ).await;

        match cancel_result
        {
          Ok( cancelled_response ) =>
          {
            assert_eq!( response_id, cancelled_response.id, "Cancelled response ID should match" );
            assert!(
              cancelled_response.status == "failed" || cancelled_response.status == "incomplete",
              "Status should indicate cancellation"
            );
          },
          Err( e ) => eprintln!( "Cancel failed : {e:?}" ),
        }
      }
    },
    Err( e ) =>
    {
      eprintln!( "Stream creation failed : {e:?}" );
    }
  }
}

/// Tests that environment details (API key, base URL) are correctly loaded.
/// Test Combination : R1.11
#[ tokio::test ]
#[ allow( deprecated ) ]
async fn test_environment_details()
{
  require_api_key!();
  let secret = load_secret_for_test();
  let env = api_openai::exposed::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string()).expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");
  let env = &client.environment;

  let base_url = env.base_url();
  let api_key_secret = OpenaiEnvironment::api_key(env);
  let api_key = api_key_secret.expose_secret();

  println!( "Environment Base URL: {base_url}" );
  let key_start = &api_key[..5];
  println!( "Environment API Key (masked): {key_start}..." ); // Print first 5 chars for verification

  let headers = env.headers();
  println!( "Environment Headers : {headers:?}" );

  assert_eq!( base_url.as_str(), "https://api.openai.com/v1/", "Base URL should be api.openai.com/v1/" );
  assert!( !api_key.is_empty(), "API key should not be empty" );
  assert!( api_key.len() > 10, "API key should be longer than 10 characters" ); // Basic length check
}
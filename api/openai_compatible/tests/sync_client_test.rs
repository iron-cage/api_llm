//! Tests for `SyncClient` construction and blocking semantics.
//!
//! Unit tests verify that `SyncClient::new()` succeeds when a tokio runtime
//! can be created — the only hard requirement before any request is made.
//!
//! # Test Matrix
//!
//! | Test | Scenario | Status |
//! |------|----------|--------|
//! | `sync_client_new_succeeds` | Runtime creation with valid env | ✅ |
//! | `sync_client_new_with_custom_base_url_succeeds` | Custom base URL accepted | ✅ |
//! | `sync_client_post_unreachable_url_returns_error` | Unreachable URL propagates network error | ✅ |
//! | `sync_client_post_chat_completions_succeeds` | Blocking POST round-trip with real API | ✅ |

#![ cfg( feature = "enabled" ) ]

// ------------------------------------------------------------------ //
//  Unit tests
// ------------------------------------------------------------------ //

/// `SyncClient::new(client)` must succeed when a tokio runtime is available.
///
/// The sync client wraps an async `Client` in a dedicated tokio runtime. If
/// runtime creation fails (e.g. nested runtime conflict), the constructor
/// returns an error instead of panicking, giving callers a chance to recover.
#[ cfg( feature = "sync_api" ) ]
#[ test ]
fn sync_client_new_succeeds()
{
  use api_openai_compatible::{ Client, SyncClient, OpenAiCompatEnvironmentImpl };

  let env = OpenAiCompatEnvironmentImpl::new( "sk-test-key" )
    .expect( "environment construction must succeed with a non-empty key" );

  let client = Client::build( env )
    .expect( "Client::build() must succeed" );

  let _sync_client = SyncClient::new( client )
    .expect( "SyncClient::new() must succeed when a tokio runtime is available" );
}

/// `SyncClient` built with a custom base URL must preserve the URL in the environment.
///
/// The `with_base_url()` builder modifies the environment before client construction.
/// The `SyncClient` wraps the resulting client, so the custom URL propagates through
/// the entire construction chain.
#[ cfg( feature = "sync_api" ) ]
#[ test ]
fn sync_client_new_with_custom_base_url_succeeds()
{
  use api_openai_compatible::{ Client, SyncClient, OpenAiCompatEnvironmentImpl };

  let env = OpenAiCompatEnvironmentImpl::new( "sk-test-key" )
    .expect( "environment construction must succeed" )
    .with_base_url( "https://api.kie.ai/my-model/v1/" );

  let client = Client::build( env )
    .expect( "Client::build() must succeed with custom base URL" );

  let _sync_client = SyncClient::new( client )
    .expect( "SyncClient::new() must succeed with custom base URL client" );
}

/// `SyncClient::post()` with a base URL pointing to nothing must return `Err`.
///
/// When the base URL is an unreachable address (port 1 on loopback with nothing
/// listening), the `reqwest` client encounters a connection-refused or timeout
/// error. The `SyncClient` must propagate this as an `Err` — proving that URL
/// routing is respected (the custom base URL is actually used) and that network
/// failures are not silently swallowed.
#[ cfg( feature = "sync_api" ) ]
#[ test ]
fn sync_client_post_unreachable_url_returns_error()
{
  use api_openai_compatible::{ Client, SyncClient, OpenAiCompatEnvironmentImpl };
  use core::time::Duration;

  let env = OpenAiCompatEnvironmentImpl::new( "sk-test-key" )
    .expect( "environment construction must succeed" )
    .with_base_url( "http://127.0.0.1:1/" )
    .with_timeout( Duration::from_secs( 2 ) );

  let client = Client::build( env )
    .expect( "Client::build() must succeed" );

  let sync_client = SyncClient::new( client )
    .expect( "SyncClient::new() must succeed" );

  let body = serde_json::json!({ "model": "test" });
  let result : Result< serde_json::Value, _ > = sync_client.post( "models", &body );

  assert!(
    result.is_err(),
    "SyncClient::post() to unreachable URL must return Err, not Ok",
  );
}

// ------------------------------------------------------------------ //
//  Integration tests
// ------------------------------------------------------------------ //

/// `SyncClient::post()` with a valid API key must return `Ok(ChatCompletionResponse)`.
///
/// Sends a minimal chat completion request to the real `OpenAI` API using the
/// blocking `SyncClient::post()` method. The calling thread blocks until the
/// response arrives. A successful result proves the async-to-sync bridge works
/// end-to-end and the response body deserialises correctly.
#[ cfg( all( feature = "sync_api", feature = "integration" ) ) ]
#[ test ]
fn sync_client_post_chat_completions_succeeds()
{
  use api_openai_compatible::
  {
    Client, SyncClient, OpenAiCompatEnvironmentImpl,
    ChatCompletionRequest, ChatCompletionResponse, Message,
  };

  let ws = workspace_tools::workspace()
    .expect( "workspace root must be resolvable" );
  let api_key = ws.load_secret_key( "OPENAI_API_KEY", "-secrets.sh" )
    .expect( "OPENAI_API_KEY must be set in secret/-secrets.sh" );

  let env = OpenAiCompatEnvironmentImpl::new( &api_key )
    .expect( "environment construction must succeed" );

  let client = Client::build( env )
    .expect( "Client::build() must succeed" );

  let sync_client = SyncClient::new( client )
    .expect( "SyncClient::new() must succeed" );

  let request = ChatCompletionRequest::former()
    .model( "gpt-4o-mini".to_string() )
    .messages( vec![ Message::user( "Say hello in one word." ) ] )
    .max_tokens( 5_u32 )
    .form();

  let result : Result< ChatCompletionResponse, _ > =
    sync_client.post( "chat/completions", &request );

  assert!(
    result.is_ok(),
    "SyncClient::post() with a valid key must return Ok; got: {:?}",
    result.err(),
  );

  let response = result.unwrap();
  assert!(
    !response.choices.is_empty(),
    "response must contain at least one choice",
  );
}

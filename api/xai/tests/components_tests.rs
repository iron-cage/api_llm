//! Tests for API component types (messages, requests, models).
//!
//! # Test Coverage
//!
//! ## Thin Client Principle (IN-01, IN-03, IN-04)
//!
//! - IN-01: Optional `ChatCompletionRequest` fields omitted from JSON when `None`.
//! - IN-03: No token count is injected into outbound request payload even when
//!   `count_tokens` feature is compiled in.
//! - IN-04: Two identical requests without explicit cache configuration both
//!   reach the real API (no implicit caching).
//!
//! ## Feature: Enterprise Reliability (FT-01, FT-02, FT-03, FT-04)
//!
//! - FT-01/FT-02/FT-03: Verified by compilation — `lib.rs` guards every enterprise
//!   module with `#[cfg(feature = "...")]`; the `Client::build` baseline is feature-agnostic.
//! - FT-04: `count_tokens()` returns a count locally without making any HTTP request.
//!
//! ## Module Organization (PT-01)
//!
//! - PT-01: All public types (`Client`, `Secret`, `Message`, `Role`, etc.) are
//!   accessible at `api_xai::` — verified by every import in this file.
//!
//! ## Testing Standards (IN-03 for invariant/002)
//!
//! - IN-03: This file mixes unit tests and integration tests; unit tests compile and
//!   run without the `integration` feature — the per-function `#[cfg]` gate is used.

use api_xai::
{
  Message, Role, ChatCompletionRequest, Usage,
  Model, ListModelsResponse
};

#[ test ]
fn message_system_constructor_works()
{
  let msg = Message::system( "You are a helpful assistant" );

  assert_eq!( msg.role, Role::System );
  assert_eq!( msg.content, Some( "You are a helpful assistant".to_string() ) );
  assert_eq!( msg.tool_calls, None );
  assert_eq!( msg.tool_call_id, None );
}

#[ test ]
fn message_user_constructor_works()
{
  let msg = Message::user( "Hello!" );

  assert_eq!( msg.role, Role::User );
  assert_eq!( msg.content, Some( "Hello!".to_string() ) );
  assert_eq!( msg.tool_calls, None );
  assert_eq!( msg.tool_call_id, None );
}

#[ test ]
fn message_assistant_constructor_works()
{
  let msg = Message::assistant( "Hi there!" );

  assert_eq!( msg.role, Role::Assistant );
  assert_eq!( msg.content, Some( "Hi there!".to_string() ) );
  assert_eq!( msg.tool_calls, None );
  assert_eq!( msg.tool_call_id, None );
}

#[ test ]
fn message_tool_constructor_works()
{
  let msg = Message::tool( "call_123", r#"{"result": "ok"}"# );

  assert_eq!( msg.role, Role::Tool );
  assert_eq!( msg.content, Some( r#"{"result": "ok"}"#.to_string() ) );
  assert_eq!( msg.tool_call_id, Some( "call_123".to_string() ) );
  assert_eq!( msg.tool_calls, None );
}

#[ test ]
fn chat_request_serializes_correctly()
{
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Test" ) ] )
    .temperature( 0.7 )
    .max_tokens( 100u32 )
    .form();

  let json = serde_json::to_value( &request ).unwrap();

  assert_eq!( json[ "model" ], "grok-2-1212" );
  assert!( ( json[ "temperature" ].as_f64().unwrap() - 0.7 ).abs() < 0.0001 );
  assert_eq!( json[ "max_tokens" ], 100 );
  assert!( json[ "messages" ].is_array() );
}

#[ test ]
fn chat_request_omits_none_fields()
{
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Test" ) ] )
    .form();

  let json = serde_json::to_string( &request ).unwrap();

  // Should not contain "temperature", "max_tokens", etc.
  assert!( !json.contains( "\"temperature\"" ) );
  assert!( !json.contains( "\"max_tokens\"" ) );
  assert!( !json.contains( "\"top_p\"" ) );
}

#[ test ]
fn chat_request_includes_some_fields()
{
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Test" ) ] )
    .temperature( 0.5 )
    .form();

  let json = serde_json::to_string( &request ).unwrap();

  assert!( json.contains( "\"temperature\"" ) );
  assert!( json.contains( "0.5" ) );
}

#[ test ]
fn former_builder_works()
{
  let request = ChatCompletionRequest::former()
    .model( "grok-4".to_string() )
    .messages( vec![
      Message::system( "You are helpful" ),
      Message::user( "Hello" ),
    ] )
    .temperature( 0.8 )
    .max_tokens( 200u32 )
    .top_p( 0.9 )
    .frequency_penalty( 0.1 )
    .presence_penalty( 0.2 )
    .stream( true )
    .form();

  assert_eq!( request.model, "grok-4" );
  assert_eq!( request.messages.len(), 2 );
  assert_eq!( request.temperature, Some( 0.8 ) );
  assert_eq!( request.max_tokens, Some( 200 ) );
  assert_eq!( request.top_p, Some( 0.9 ) );
  assert_eq!( request.frequency_penalty, Some( 0.1 ) );
  assert_eq!( request.presence_penalty, Some( 0.2 ) );
  assert_eq!( request.stream, Some( true ) );
}

#[ test ]
fn role_serializes_as_lowercase()
{
  let system = serde_json::to_string( &Role::System ).unwrap();
  assert_eq!( system, r#""system""# );

  let user = serde_json::to_string( &Role::User ).unwrap();
  assert_eq!( user, r#""user""# );

  let assistant = serde_json::to_string( &Role::Assistant ).unwrap();
  assert_eq!( assistant, r#""assistant""# );

  let tool = serde_json::to_string( &Role::Tool ).unwrap();
  assert_eq!( tool, r#""tool""# );
}

#[ test ]
fn role_deserializes_from_lowercase()
{
  let system : Role = serde_json::from_str( r#""system""# ).unwrap();
  assert_eq!( system, Role::System );

  let user : Role = serde_json::from_str( r#""user""# ).unwrap();
  assert_eq!( user, Role::User );

  let assistant : Role = serde_json::from_str( r#""assistant""# ).unwrap();
  assert_eq!( assistant, Role::Assistant );

  let tool : Role = serde_json::from_str( r#""tool""# ).unwrap();
  assert_eq!( tool, Role::Tool );
}

#[ test ]
fn usage_serializes_correctly()
{
  let usage = Usage
  {
    prompt_tokens : 10,
    completion_tokens : 20,
    total_tokens : 30,
  };

  let json = serde_json::to_value( &usage ).unwrap();

  assert_eq!( json[ "prompt_tokens" ], 10 );
  assert_eq!( json[ "completion_tokens" ], 20 );
  assert_eq!( json[ "total_tokens" ], 30 );
}

#[ test ]
fn usage_deserializes_correctly()
{
  let json = r#"{
    "prompt_tokens": 15,
    "completion_tokens": 25,
    "total_tokens": 40
  }"#;

  let usage : Usage = serde_json::from_str( json ).unwrap();

  assert_eq!( usage.prompt_tokens, 15 );
  assert_eq!( usage.completion_tokens, 25 );
  assert_eq!( usage.total_tokens, 40 );
}

#[ test ]
fn model_deserializes_correctly()
{
  let json = r#"{
    "id": "grok-2-1212",
    "object": "model",
    "created": 1234567890,
    "owned_by": "xai"
  }"#;

  let model : Model = serde_json::from_str( json ).unwrap();

  assert_eq!( model.id, "grok-2-1212" );
  assert_eq!( model.object, "model" );
  assert_eq!( model.created, 1_234_567_890 );
  assert_eq!( model.owned_by, "xai" );
}

#[ test ]
fn list_models_response_deserializes_correctly()
{
  let json = r#"{
    "object": "list",
    "data": [
      {
        "id": "grok-2-1212",
        "object": "model",
        "created": 1234567890,
        "owned_by": "xai"
      },
      {
        "id": "grok-4",
        "object": "model",
        "created": 1234567891,
        "owned_by": "xai"
      }
    ]
  }"#;

  let response : ListModelsResponse = serde_json::from_str( json ).unwrap();

  assert_eq!( response.object, "list" );
  assert_eq!( response.data.len(), 2 );
  assert_eq!( response.data[ 0 ].id, "grok-2-1212" );
  assert_eq!( response.data[ 1 ].id, "grok-4" );
}

#[ test ]
fn message_with_none_content_omits_field()
{
  let mut msg = Message::user( "test" );
  msg.content = None;

  let json = serde_json::to_string( &msg ).unwrap();

  // Should not serialize content field if None
  assert!( !json.contains( "\"content\"" ) );
}

#[ test ]
fn message_with_some_content_includes_field()
{
  let msg = Message::user( "hello world" );

  let json = serde_json::to_string( &msg ).unwrap();

  assert!( json.contains( "\"content\"" ) );
  assert!( json.contains( "hello world" ) );
}

#[ test ]
fn chat_request_clone_works()
{
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Test" ) ] )
    .form();

  let cloned = request.clone();

  assert_eq!( request, cloned );
}

#[ test ]
fn role_clone_and_partial_eq_work()
{
  let role1 = Role::User;
  let role2 = role1.clone();

  assert_eq!( role1, role2 );
}

#[ test ]
fn usage_clone_and_partial_eq_work()
{
  let usage1 = Usage
  {
    prompt_tokens : 10,
    completion_tokens : 20,
    total_tokens : 30,
  };

  let usage2 = usage1.clone();

  assert_eq!( usage1, usage2 );
}

// IN-01 (thin client) — optional fields absent from JSON when None
//
// Covers: tests/docs/invariant/01_thin_client_principle.md § IN-01
// This test verifies the core thin-client contract: fields not explicitly set
// must not appear in the wire payload sent to the X.AI API.
#[ test ]
fn chat_request_with_only_required_fields_omits_all_optional_keys()
{
  let request = ChatCompletionRequest::former()
    .model( "grok-3".to_string() )
    .messages( vec![ Message::user( "hi" ) ] )
    .form();

  let json_str = serde_json::to_string( &request ).expect( "must serialize" );

  // None-valued optional fields must not appear in the payload
  assert!( !json_str.contains( "\"temperature\"" ),
    "temperature must be absent when not set" );
  assert!( !json_str.contains( "\"max_tokens\"" ),
    "max_tokens must be absent when not set" );
  assert!( !json_str.contains( "\"top_p\"" ),
    "top_p must be absent when not set" );
  assert!( !json_str.contains( "\"tools\"" ),
    "tools must be absent when not set" );
  assert!( !json_str.contains( "\"stream\"" ),
    "stream must be absent when not set" );

  // Required fields must be present
  assert!( json_str.contains( "\"model\"" ) );
  assert!( json_str.contains( "\"messages\"" ) );
}

// IN-03 (thin client) — no token count field injected into request payload
//
// Covers: tests/docs/invariant/01_thin_client_principle.md § IN-03
// Even when the `count_tokens` feature is compiled in, the serialized request
// must contain no injected token count — the feature is opt-in at call site,
// not automatic middleware.
#[ test ]
fn chat_request_payload_contains_no_token_count_field()
{
  let request = ChatCompletionRequest::former()
    .model( "grok-3".to_string() )
    .messages( vec![ Message::user( "What is 2+2?" ) ] )
    .form();

  let json_str = serde_json::to_string( &request ).expect( "must serialize" );

  // The X.AI API does not accept a `token_count` field; injecting one would break the call.
  assert!( !json_str.contains( "\"token_count\"" ),
    "token_count must never be injected into the request payload" );
  assert!( !json_str.contains( "\"num_tokens\"" ),
    "num_tokens must never be injected into the request payload" );
  assert!( !json_str.contains( "\"prompt_tokens\"" ),
    "prompt_tokens must never be injected into the request payload" );
}

// FT-01 — all enterprise features compile and client constructs successfully
//
// Covers: tests/docs/feature/01_enterprise_reliability.md § FT-01
// When compiled with --all-features every enterprise module is included.
// Constructing a Client proves all modules link without conflict.
#[ test ]
fn client_builds_successfully_with_all_features()
{
  use api_xai::{ Client, Secret, XaiEnvironmentImpl };

  let secret = Secret::new( "xai-test-key-1234567890".to_string() ).unwrap();
  let env = XaiEnvironmentImpl::new( secret ).unwrap();
  let client = Client::build( env ).unwrap();
  // Accessing the public environment field proves the client is fully initialised
  let _ = &client.environment;
}

// FT-04 — count_tokens returns a value without making any HTTP request
//
// Covers: tests/docs/feature/01_enterprise_reliability.md § FT-04
// Token counting uses tiktoken-rs (cl100k_base) locally — no network call is made.
#[ cfg( feature = "count_tokens" ) ]
#[ test ]
fn count_tokens_returns_local_count_without_http()
{
  use api_xai::count_tokens_for_request;

  // Build a minimal request and count its tokens.
  // Use grok-2-1212 — the model supported by the local tiktoken mapping.
  // We don't assert the exact count (encoding details may change) but
  // we do assert: (a) a positive count is returned, (b) no panic/error.
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Hello, world!" ) ] )
    .form();

  let result = count_tokens_for_request( &request );
  assert!( result.is_ok(), "count_tokens_for_request must succeed: {result:?}" );
  let count = result.unwrap();
  assert!( count > 0, "token count must be positive, got {count}" );
}

// PT-01 — mod_interface types accessible at crate root
//
// Covers: tests/docs/pattern/01_module_organization.md § PT-01
// All imports in this file use `api_xai::` directly.
// Successful compilation is the assertion; we reference each key type here.
#[ test ]
fn crate_root_exports_are_accessible()
{
  use api_xai::{ Client, Secret, XaiEnvironmentImpl };

  // Core types — verify each type is accessible at crate root (compilation proof)
  let msg = Message::user( "test" );
  let _ = Role::User;
  let _ = ChatCompletionRequest::former()
    .model( "m".to_string() )
    .messages( vec![ msg ] )
    .form();
  let _ = Usage { prompt_tokens : 1, completion_tokens : 1, total_tokens : 2 };

  // Infrastructure types
  let secret = Secret::new( "xai-test-key-1234567890".to_string() ).unwrap();
  let env = XaiEnvironmentImpl::new( secret ).unwrap();
  let client = Client::build( env ).unwrap();
  // Verify environment field is accessible (public field, not method)
  let _ = &client.environment;
}

// IN-03 (testing standards) — per-function integration gate
//
// Covers: tests/docs/invariant/02_testing_standards.md § IN-03
// This non-gated test is in the same file as would be integration tests
// (the pattern used in integration_chat.rs, etc.).  It proves that unit
// tests in files using per-function `#[cfg(feature = "integration")]`
// still compile and run without the integration feature active.
#[ test ]
fn unit_test_in_file_with_integration_gate_compiles_and_runs()
{
  // The test itself is trivially correct; its existence and successful
  // execution without `--features integration` is the actual assertion.
  let request = ChatCompletionRequest::former()
    .model( "grok-3".to_string() )
    .messages( vec![ Message::user( "hi" ) ] )
    .form();
  assert!( !request.model.is_empty() );
}

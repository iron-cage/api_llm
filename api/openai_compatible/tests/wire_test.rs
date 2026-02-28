//! Tests for OpenAI-compatible wire types: serde round-trips and JSON shape contracts.
//!
//! These tests are the authoritative specification for the serialised wire format.
//! No live HTTP is made — only `serde_json` serialisation / deserialisation.
//! The implementation in Phase 2 must satisfy them without modifying this file.

#![ cfg( feature = "enabled" ) ]

use api_openai_compatible::{ Message, Role, ChatCompletionRequest, ChatCompletionResponse };

// ------------------------------------------------------------------ //

/// `Message::user()` must serialise with `role = "user"` and the supplied content.
///
/// The field order and presence in the JSON output matters: absent optional
/// fields (`tool_calls`, `tool_call_id`) must NOT appear as `null` — they
/// must be omitted entirely to avoid confusing API servers that reject
/// unrecognised fields or explicit `null` values.
#[ test ]
fn message_user_constructor_serializes_correctly()
{
  let msg = Message::user( "What is the capital of France?" );
  let json = serde_json::to_string( &msg ).expect( "Message must be serializable" );

  let parsed : serde_json::Value =
    serde_json::from_str( &json ).expect( "serialised output must be valid JSON" );

  assert_eq!(
    parsed[ "role" ],
    "user",
    "role field must be the lowercase string \"user\"",
  );
  assert_eq!(
    parsed[ "content" ],
    "What is the capital of France?",
    "content field must match the constructor argument",
  );
  assert!(
    parsed.get( "tool_calls" ).is_none() || parsed[ "tool_calls" ].is_null(),
    "absent optional field tool_calls must be omitted or null",
  );
  // Verify exact omission (not null) by checking the raw JSON string.
  assert!(
    !json.contains( "\"tool_calls\"" ),
    "tool_calls must be omitted entirely from JSON when None",
  );
  assert!(
    !json.contains( "\"tool_call_id\"" ),
    "tool_call_id must be omitted entirely from JSON when None",
  );
}

// ------------------------------------------------------------------ //

/// `Message::system()` must serialise with `role = "system"`.
///
/// System messages set instructions for the model. Incorrect role serialisation
/// causes the model to treat instructions as user input.
#[ test ]
fn message_system_constructor_serializes_correctly()
{
  let msg  = Message::system( "You are a helpful assistant." );
  let json = serde_json::to_string( &msg ).expect( "Message must be serializable" );

  let parsed : serde_json::Value =
    serde_json::from_str( &json ).expect( "serialised output must be valid JSON" );

  assert_eq!(
    parsed[ "role" ],
    "system",
    "role field must be the lowercase string \"system\"",
  );
  assert_eq!(
    parsed[ "content" ],
    "You are a helpful assistant.",
  );
}

// ------------------------------------------------------------------ //

/// `ChatCompletionRequest` with only required fields set must NOT include
/// `null` for absent optional fields.
///
/// Some API endpoints reject requests that contain `null` for unknown or
/// optional fields (notably several OpenAI-compatible third-party providers).
/// The `skip_serializing_if = "Option::is_none"` attribute enforces this.
#[ test ]
fn chat_completion_request_omits_none_fields()
{
  let req = ChatCompletionRequest::former()
    .model( "gpt-4o".to_string() )
    .messages( vec![ Message::user( "Hello!" ) ] )
    .form();

  let json = serde_json::to_string( &req ).expect( "ChatCompletionRequest must be serializable" );

  // Required fields must be present.
  assert!( json.contains( "\"model\"" ),    "model field must be present" );
  assert!( json.contains( "\"messages\"" ), "messages field must be present" );

  // Optional fields must be absent when not set.
  assert!( !json.contains( "\"temperature\"" ),      "temperature must be omitted when None" );
  assert!( !json.contains( "\"max_tokens\"" ),        "max_tokens must be omitted when None" );
  assert!( !json.contains( "\"top_p\"" ),             "top_p must be omitted when None" );
  assert!( !json.contains( "\"frequency_penalty\"" ), "frequency_penalty must be omitted when None" );
  assert!( !json.contains( "\"presence_penalty\"" ),  "presence_penalty must be omitted when None" );
  assert!( !json.contains( "\"stream\"" ),            "stream must be omitted when None" );
  assert!( !json.contains( "\"tools\"" ),             "tools must be omitted when None" );
}

// ------------------------------------------------------------------ //

/// `ChatCompletionResponse` must deserialise from a representative API fixture.
///
/// The JSON fixture mirrors a real response shape. Deserialisation failures
/// here mean the wire types no longer match the API contract and would break
/// every caller in production.
#[ test ]
fn chat_completion_response_deserializes_from_fixture()
{
  let fixture = r#"{
    "id": "chatcmpl-abc123xyz",
    "object": "chat.completion",
    "created": 1717000000,
    "model": "gpt-4o",
    "choices": [
      {
        "index": 0,
        "message": {
          "role": "assistant",
          "content": "Paris is the capital of France."
        },
        "finish_reason": "stop"
      }
    ],
    "usage": {
      "prompt_tokens": 14,
      "completion_tokens": 8,
      "total_tokens": 22
    }
  }"#;

  let resp : ChatCompletionResponse =
    serde_json::from_str( fixture ).expect( "fixture must deserialise into ChatCompletionResponse" );

  assert_eq!( resp.id, "chatcmpl-abc123xyz" );
  assert_eq!( resp.model, "gpt-4o" );
  assert_eq!( resp.usage.total_tokens, 22 );
  assert_eq!(
    resp.choices[ 0 ].message.content.as_deref(),
    Some( "Paris is the capital of France." ),
    "assistant message content must round-trip correctly",
  );
  assert_eq!(
    resp.choices[ 0 ].finish_reason.as_deref(),
    Some( "stop" ),
  );
}

// ------------------------------------------------------------------ //

/// `Role` variants must serialise to their lowercase string equivalents and
/// deserialise back to the original variant (full serde round-trip).
///
/// The wire protocol requires lowercase role strings. A mismatch causes the
/// API server to reject the message or misclassify the participant.
#[ test ]
fn role_round_trips_through_serde()
{
  let roles = [
    ( Role::System,    "\"system\""    ),
    ( Role::User,      "\"user\""      ),
    ( Role::Assistant, "\"assistant\"" ),
    ( Role::Tool,      "\"tool\""      ),
  ];

  for ( role, expected_json ) in roles
  {
    let serialised = serde_json::to_string( &role )
      .unwrap_or_else( |e| panic!( "Role::{role:?} must be serializable: {e}" ) );

    assert_eq!(
      serialised,
      expected_json,
      "Role::{role:?} must serialise to {expected_json}",
    );

    let deserialised : Role = serde_json::from_str( &serialised )
      .unwrap_or_else( |e| panic!( "Role::{role:?} must be deserializable from {serialised}: {e}" ) );

    assert_eq!(
      deserialised,
      role,
      "Role::{role:?} must survive a serde round-trip",
    );
  }
}

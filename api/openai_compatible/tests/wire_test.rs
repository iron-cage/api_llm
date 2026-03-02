//! Tests for OpenAI-compatible wire types: serde round-trips and JSON shape contracts.
//!
//! These tests are the authoritative specification for the serialised wire format.
//! No live HTTP is made — only `serde_json` serialisation / deserialisation.
//! The implementation in Phase 2 must satisfy them without modifying this file.
//!
//! # Test Matrix
//!
//! | Test | Category | Validates |
//! |------|----------|-----------|
//! | message_user_constructor_serializes_correctly | Message | role="user", content, optional omission |
//! | message_system_constructor_serializes_correctly | Message | role="system" |
//! | message_assistant_constructor_serializes_correctly | Message | role="assistant", no tool_call_id |
//! | message_tool_constructor_serializes_correctly | Message | role="tool", tool_call_id present |
//! | message_with_tool_calls_serializes_correctly | Message | tool_calls field populated |
//! | tool_call_round_trips_through_serde | ToolCall | "type" key (not "tool_type") |
//! | chat_completion_request_omits_none_fields | Request | skip_serializing_if=None coverage |
//! | chat_completion_request_stream_set_to_false_appears_in_json | Request | Some(false) is serialised |
//! | chat_completion_response_deserializes_from_fixture | Response | canonical fixture |
//! | response_tolerates_extra_unknown_json_fields | Response | deny_unknown_fields absent |
//! | response_with_multiple_choices_deserializes | Response | n > 1 choices |
//! | role_round_trips_through_serde | Role | all 4 variants |
//! | streaming_chunk_deserializes_from_fixture | Streaming | ChatCompletionChunk fixture |
//! | streaming_delta_none_fields_omitted_from_json | Streaming | Delta{} → "{}" |

#![ cfg( feature = "enabled" ) ]

use api_openai_compatible::
{
  Message, Role, ToolCall, FunctionCall,
  ChatCompletionRequest, ChatCompletionResponse,
};

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

/// `Message::assistant()` must serialise with `role = "assistant"` and no `tool_call_id`.
///
/// Assistant messages are model-generated responses. The `tool_call_id` field
/// is exclusive to tool-result messages and must be absent here to avoid
/// confusing API servers that validate message shapes strictly.
#[ test ]
fn message_assistant_constructor_serializes_correctly()
{
  let msg = Message::assistant( "Paris is the capital of France." );
  let json = serde_json::to_string( &msg ).expect( "Message must be serializable" );

  let parsed : serde_json::Value =
    serde_json::from_str( &json ).expect( "serialised output must be valid JSON" );

  assert_eq!(
    parsed[ "role" ],
    "assistant",
    "role field must be the lowercase string \"assistant\"",
  );
  assert_eq!(
    parsed[ "content" ],
    "Paris is the capital of France.",
    "content field must match the constructor argument",
  );
  assert!(
    !json.contains( "\"tool_call_id\"" ),
    "tool_call_id must be omitted from assistant messages",
  );
  assert!(
    !json.contains( "\"tool_calls\"" ),
    "tool_calls must be omitted when not set on an assistant message",
  );
}

// ------------------------------------------------------------------ //

/// `Message::tool()` must serialise with `role = "tool"` and a `tool_call_id` field.
///
/// Tool-result messages carry the output of a function call back to the model.
/// The `tool_call_id` links the result to the specific `ToolCall` that triggered
/// the invocation. Missing this field causes the API to reject the message.
#[ test ]
fn message_tool_constructor_serializes_correctly()
{
  let msg = Message::tool( "call_abc123", r#"{"temperature":22}"# );
  let json = serde_json::to_string( &msg ).expect( "Message must be serializable" );

  let parsed : serde_json::Value =
    serde_json::from_str( &json ).expect( "serialised output must be valid JSON" );

  assert_eq!(
    parsed[ "role" ],
    "tool",
    "role field must be the lowercase string \"tool\"",
  );
  assert_eq!(
    parsed[ "tool_call_id" ],
    "call_abc123",
    "tool_call_id must match the constructor argument",
  );
  assert_eq!(
    parsed[ "content" ],
    r#"{"temperature":22}"#,
    "content must carry the serialised tool result",
  );
  assert!(
    !json.contains( "\"tool_calls\"" ),
    "tool_calls must be omitted from tool-result messages",
  );
}

// ------------------------------------------------------------------ //

/// An assistant `Message` with `tool_calls` populated must serialise the full array.
///
/// When the model decides to invoke functions it sets `tool_calls` on the
/// assistant message. This field must survive a JSON round-trip so that callers
/// can dispatch the requested function and attach the result in a follow-up
/// tool-result message.
#[ test ]
fn message_with_tool_calls_serializes_correctly()
{
  let call = ToolCall
  {
    id        : "call_xyz789".to_string(),
    tool_type : "function".to_string(),
    function  : FunctionCall
    {
      name      : "get_weather".to_string(),
      arguments : r#"{"location":"Paris"}"#.to_string(),
    },
  };

  let msg = Message
  {
    role         : Role::Assistant,
    content      : None,
    tool_calls   : Some( vec![ call ] ),
    tool_call_id : None,
  };

  let json = serde_json::to_string( &msg ).expect( "Message must be serializable" );

  let parsed : serde_json::Value =
    serde_json::from_str( &json ).expect( "serialised output must be valid JSON" );

  assert_eq!( parsed[ "role" ], "assistant" );

  let calls = parsed[ "tool_calls" ].as_array()
    .expect( "tool_calls must be a JSON array when populated" );

  assert_eq!( calls.len(), 1, "exactly one tool call must be serialised" );
  assert_eq!( calls[ 0 ][ "id" ], "call_xyz789" );
  assert_eq!( calls[ 0 ][ "function" ][ "name" ], "get_weather" );

  // content is None — must be omitted, not null
  assert!(
    !json.contains( "\"content\"" ),
    "content must be omitted when None on a tool-calling assistant message",
  );
  assert!(
    !json.contains( "\"tool_call_id\"" ),
    "tool_call_id must be omitted from assistant messages",
  );
}

// ------------------------------------------------------------------ //

/// `ToolCall` must serialise the `tool_type` field as `"type"` in JSON.
///
/// The `OpenAI` wire protocol uses `"type"` as the JSON key, but `type` is a
/// reserved keyword in Rust. The `#[serde(rename = "type")]` attribute bridges
/// this mismatch. A missing rename causes serialised JSON to contain
/// `"tool_type"` instead of `"type"`, which the API rejects.
#[ test ]
fn tool_call_round_trips_through_serde()
{
  let call = ToolCall
  {
    id        : "call_roundtrip".to_string(),
    tool_type : "function".to_string(),
    function  : FunctionCall
    {
      name      : "ping".to_string(),
      arguments : "{}".to_string(),
    },
  };

  let json = serde_json::to_string( &call ).expect( "ToolCall must be serializable" );

  // The Rust field `tool_type` must appear as `"type"` on the wire.
  assert!(
    json.contains( "\"type\"" ),
    "ToolCall.tool_type must be serialised as JSON key \"type\"; got: {json}",
  );
  assert!(
    !json.contains( "\"tool_type\"" ),
    "JSON must NOT contain \"tool_type\" — the rename attribute must be applied; got: {json}",
  );

  // Full round-trip: deserialise back and compare.
  let round_tripped : ToolCall =
    serde_json::from_str( &json ).expect( "ToolCall must be deserializable from its own output" );

  assert_eq!( round_tripped.id,              "call_roundtrip" );
  assert_eq!( round_tripped.tool_type,       "function" );
  assert_eq!( round_tripped.function.name,   "ping" );
  assert_eq!( round_tripped.function.arguments, "{}" );
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

/// `ChatCompletionRequest` with `stream` set to `false` must include the field in JSON.
///
/// The `stream` field uses `skip_serializing_if = "Option::is_none"`, so only
/// `None` is omitted. `Some(false)` must be serialised as `"stream":false` so
/// that callers who need to explicitly disable streaming can do so. Omitting
/// `Some(false)` would silently treat it the same as `None`, breaking callers
/// that require the field present.
#[ test ]
fn chat_completion_request_stream_set_to_false_appears_in_json()
{
  let req = ChatCompletionRequest::former()
    .model( "gpt-4o".to_string() )
    .messages( vec![ Message::user( "Hello!" ) ] )
    .stream( false )
    .form();

  let json = serde_json::to_string( &req ).expect( "ChatCompletionRequest must be serializable" );

  assert!(
    json.contains( "\"stream\"" ),
    "stream field must appear in JSON when set to Some(false); got: {json}",
  );

  let parsed : serde_json::Value =
    serde_json::from_str( &json ).expect( "serialised output must be valid JSON" );

  assert_eq!(
    parsed[ "stream" ],
    false,
    "stream must serialise as the boolean false (not null or missing)",
  );
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

/// `ChatCompletionResponse` must tolerate extra unknown JSON fields.
///
/// Real API responses often include additional provider-specific fields
/// (e.g. `"system_fingerprint"`, `"x_groq"`). Without `#[serde(deny_unknown_fields)]`
/// being absent (the default), serde ignores unknown keys silently. If this test
/// fails, an accidental `deny_unknown_fields` annotation was added and would
/// break production deserialization for any provider that adds custom fields.
#[ test ]
fn response_tolerates_extra_unknown_json_fields()
{
  let fixture = r#"{
    "id": "chatcmpl-extra",
    "object": "chat.completion",
    "created": 1717000001,
    "model": "gpt-4o",
    "system_fingerprint": "fp_abc123",
    "x_provider_data": { "latency_ms": 42 },
    "choices": [
      {
        "index": 0,
        "message": {
          "role": "assistant",
          "content": "Hello!",
          "unknown_future_field": true
        },
        "finish_reason": "stop",
        "logprobs": null
      }
    ],
    "usage": {
      "prompt_tokens": 5,
      "completion_tokens": 3,
      "total_tokens": 8,
      "completion_tokens_details": { "reasoning_tokens": 0 }
    }
  }"#;

  let resp : ChatCompletionResponse =
    serde_json::from_str( fixture )
    .expect( "ChatCompletionResponse must deserialise even with unknown extra fields" );

  assert_eq!( resp.id, "chatcmpl-extra" );
  assert_eq!(
    resp.choices[ 0 ].message.content.as_deref(),
    Some( "Hello!" ),
  );
}

// ------------------------------------------------------------------ //

/// `ChatCompletionResponse` with multiple choices must deserialise all of them.
///
/// The `n` parameter on requests controls how many completions are returned.
/// The type must handle `n > 1` correctly; a hardcoded single-element
/// assumption would silently drop alternatives and corrupt multi-candidate
/// response processing.
#[ test ]
fn response_with_multiple_choices_deserializes()
{
  let fixture = r#"{
    "id": "chatcmpl-multi",
    "object": "chat.completion",
    "created": 1717000002,
    "model": "gpt-4o",
    "choices": [
      {
        "index": 0,
        "message": { "role": "assistant", "content": "Answer A" },
        "finish_reason": "stop"
      },
      {
        "index": 1,
        "message": { "role": "assistant", "content": "Answer B" },
        "finish_reason": "stop"
      }
    ],
    "usage": {
      "prompt_tokens": 10,
      "completion_tokens": 6,
      "total_tokens": 16
    }
  }"#;

  let resp : ChatCompletionResponse =
    serde_json::from_str( fixture ).expect( "multi-choice fixture must deserialise" );

  assert_eq!( resp.choices.len(), 2, "both choices must be deserialised" );
  assert_eq!( resp.choices[ 0 ].index, 0 );
  assert_eq!( resp.choices[ 1 ].index, 1 );
  assert_eq!(
    resp.choices[ 0 ].message.content.as_deref(),
    Some( "Answer A" ),
  );
  assert_eq!(
    resp.choices[ 1 ].message.content.as_deref(),
    Some( "Answer B" ),
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
  let roles =
  [
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

// ------------------------------------------------------------------ //
// Streaming wire types (requires `streaming` feature)
// ------------------------------------------------------------------ //

/// `ChatCompletionChunk` must deserialise from a representative SSE payload fixture.
///
/// SSE payloads arrive as raw JSON strings after the `data: ` prefix is stripped.
/// If the wire type no longer matches the API-delivered shape, every streaming
/// consumer silently produces empty responses.
#[ cfg( feature = "streaming" ) ]
#[ test ]
fn streaming_chunk_deserializes_from_fixture()
{
  use api_openai_compatible::ChatCompletionChunk;

  // Represents a mid-stream chunk carrying a content delta.
  let fixture = r#"{
    "id": "chatcmpl-stream-abc",
    "object": "chat.completion.chunk",
    "created": 1717000010,
    "model": "gpt-4o",
    "choices": [
      {
        "index": 0,
        "delta": { "content": " world" },
        "finish_reason": null
      }
    ]
  }"#;

  let chunk : ChatCompletionChunk =
    serde_json::from_str( fixture ).expect( "streaming fixture must deserialise" );

  assert_eq!( chunk.id, "chatcmpl-stream-abc" );
  assert_eq!( chunk.object, "chat.completion.chunk" );
  assert_eq!( chunk.choices.len(), 1 );
  assert_eq!(
    chunk.choices[ 0 ].delta.content.as_deref(),
    Some( " world" ),
    "delta content must be extracted correctly from the streaming chunk",
  );
  assert!(
    chunk.choices[ 0 ].finish_reason.is_none(),
    "finish_reason must be None for intermediate chunks",
  );
}

// ------------------------------------------------------------------ //

/// `Delta` with all fields `None` must serialise to an empty JSON object `{}`.
///
/// The final chunk from some providers sends an empty delta to signal stream end
/// before the `[DONE]` marker. Serialising `None` fields as `null` instead of
/// omitting them would break providers that validate the delta shape strictly.
#[ cfg( feature = "streaming" ) ]
#[ test ]
fn streaming_delta_none_fields_omitted_from_json()
{
  use api_openai_compatible::Delta;

  let delta = Delta::default();
  let json = serde_json::to_string( &delta ).expect( "Delta must be serializable" );

  assert_eq!(
    json,
    "{}",
    "Delta with all None fields must serialise to an empty object; got: {json}",
  );
}

// ------------------------------------------------------------------ //

/// `Message` must survive a full serde round-trip: serialise → deserialise → equal original.
///
/// Verifying only serialisation is insufficient. A rename attribute that serialises
/// correctly but deserialises to the wrong field (e.g. `role` vs `content`) would pass
/// the one-way tests above but silently corrupt reconstructed messages. The round-trip
/// catches such asymmetric serde bugs.
#[ test ]
fn message_round_trips_through_serde()
{
  let original = Message::tool( "call_rt_001", r#"{"value":42}"# );
  let json = serde_json::to_string( &original ).expect( "Message must be serializable" );
  let restored : Message =
    serde_json::from_str( &json ).expect( "Message must be deserializable from its own JSON" );

  assert_eq!( restored, original, "Message must survive a serde round-trip unchanged" );
}

// ------------------------------------------------------------------ //

/// `Message::user("")` with empty content must serialise with `content = ""`.
///
/// Empty user messages are valid from a wire-protocol standpoint. Rejecting them
/// at construction time would break callers that construct messages dynamically and
/// may produce empty strings. The serialised JSON must have a `content` key (not
/// omit it) because the content was explicitly set (to `Some("")`).
#[ test ]
fn message_user_empty_content_serializes_correctly()
{
  let msg = Message::user( "" );
  let json = serde_json::to_string( &msg ).expect( "Message::user(\"\") must be serializable" );

  let parsed : serde_json::Value =
    serde_json::from_str( &json ).expect( "serialised output must be valid JSON" );

  assert_eq!( parsed[ "role" ], "user", "role must be \"user\" for empty-content user message" );
  assert_eq!( parsed[ "content" ], "", "content key must be present and equal empty string" );
  assert!(
    !json.contains( "\"tool_calls\"" ),
    "tool_calls must be omitted entirely when None",
  );
  assert!(
    !json.contains( "\"tool_call_id\"" ),
    "tool_call_id must be omitted entirely when None",
  );
}

// ------------------------------------------------------------------ //

/// `FunctionCall` with an empty `arguments` string must round-trip correctly.
///
/// Some providers send `arguments: ""` (empty) for functions with no parameters.
/// An empty string is structurally distinct from `"{}"` — both are valid but carry
/// different semantics. The round-trip must preserve the exact empty string.
#[ test ]
fn function_call_empty_arguments_round_trips()
{
  let call = ToolCall
  {
    id        : "call_empty_args".to_string(),
    tool_type : "function".to_string(),
    function  : FunctionCall
    {
      name      : "no_params_fn".to_string(),
      arguments : String::new(),
    },
  };

  let json = serde_json::to_string( &call ).expect( "ToolCall with empty arguments must be serializable" );
  let restored : ToolCall =
    serde_json::from_str( &json ).expect( "ToolCall must be deserializable from its own JSON" );

  assert_eq!(
    restored.function.arguments,
    "",
    "Empty arguments string must survive serde round-trip unchanged",
  );
}

// ------------------------------------------------------------------ //

/// `FunctionCall.arguments` must be preserved as a raw string, not re-parsed as JSON.
///
/// The wire protocol transmits arguments as a JSON-encoded string. The caller is
/// responsible for parsing the arguments. If the library ever changed `arguments`
/// from `String` to `serde_json::Value`, complex nested JSON would be re-interpreted,
/// changing its representation. The raw string must survive unchanged.
#[ test ]
fn function_call_complex_arguments_preserved_as_string()
{
  let complex_args = r#"{"location":"New York","units":"metric","extra":{"key":"val"}}"#;
  let call = ToolCall
  {
    id        : "call_complex".to_string(),
    tool_type : "function".to_string(),
    function  : FunctionCall
    {
      name      : "get_weather".to_string(),
      arguments : complex_args.to_string(),
    },
  };

  let json = serde_json::to_string( &call ).expect( "ToolCall must be serializable" );
  let restored : ToolCall =
    serde_json::from_str( &json ).expect( "ToolCall must be deserializable" );

  assert_eq!(
    restored.function.arguments,
    complex_args,
    "Complex JSON arguments must be preserved exactly as a string, not re-parsed",
  );
}

// ------------------------------------------------------------------ //

/// `ChatCompletionRequest` with `stream = true` must include `"stream": true` in JSON.
///
/// Explicitly setting `stream = Some(true)` is distinct from `None` (no preference).
/// Providers that default to non-streaming would return a blocking response if
/// `stream` is absent, whereas `stream: true` triggers Server-Sent Events mode.
#[ test ]
fn chat_completion_request_stream_true_appears_in_json()
{
  let req = ChatCompletionRequest::former()
    .model( "gpt-4o".to_string() )
    .messages( vec![ Message::user( "Hello!" ) ] )
    .stream( true )
    .form();

  let json = serde_json::to_string( &req ).expect( "ChatCompletionRequest must be serializable" );

  assert!(
    json.contains( "\"stream\"" ),
    "stream field must appear in JSON when set to Some(true); got: {json}",
  );

  let parsed : serde_json::Value =
    serde_json::from_str( &json ).expect( "serialised output must be valid JSON" );

  assert_eq!(
    parsed[ "stream" ],
    true,
    "stream must serialise as the boolean true; got: {json}",
  );
}

// ------------------------------------------------------------------ //

/// `ChatCompletionRequest` with `temperature` set must include the field in JSON.
///
/// Optional sampling parameters like `temperature` must appear in the serialised
/// request when set to `Some(_)`. Omitting them (even when set) would silently
/// fall back to the provider's default, changing model behaviour unexpectedly.
#[ test ]
fn chat_completion_request_temperature_appears_when_set()
{
  let req = ChatCompletionRequest::former()
    .model( "gpt-4o".to_string() )
    .messages( vec![ Message::user( "Hi" ) ] )
    .temperature( 0.7_f32 )
    .form();

  let json = serde_json::to_string( &req ).expect( "ChatCompletionRequest must be serializable" );

  assert!(
    json.contains( "\"temperature\"" ),
    "temperature field must appear in JSON when set; got: {json}",
  );

  let parsed : serde_json::Value =
    serde_json::from_str( &json ).expect( "serialised output must be valid JSON" );

  // Use approximate comparison: JSON float round-trip may not be exact.
  let t = parsed[ "temperature" ].as_f64()
    .expect( "temperature must be a JSON number" );
  assert!(
    ( t - 0.7 ).abs() < 0.01,
    "temperature must round-trip to approximately 0.7; got: {t}",
  );
}

// ------------------------------------------------------------------ //

/// `ChatCompletionRequest` with `max_tokens` set must include the field in JSON.
///
/// `max_tokens` is the primary token budget control. If `Some(n)` were silently
/// treated as absent, the model would generate up to its maximum context window,
/// incurring unexpected cost and latency.
#[ test ]
fn chat_completion_request_max_tokens_appears_when_set()
{
  let req = ChatCompletionRequest::former()
    .model( "gpt-4o".to_string() )
    .messages( vec![ Message::user( "Hi" ) ] )
    .max_tokens( 512_u32 )
    .form();

  let json = serde_json::to_string( &req ).expect( "ChatCompletionRequest must be serializable" );

  assert!(
    json.contains( "\"max_tokens\"" ),
    "max_tokens field must appear in JSON when set; got: {json}",
  );

  let parsed : serde_json::Value =
    serde_json::from_str( &json ).expect( "serialised output must be valid JSON" );

  assert_eq!(
    parsed[ "max_tokens" ],
    512,
    "max_tokens must serialise as the integer 512",
  );
}

// ------------------------------------------------------------------ //

/// `ChatCompletionRequest` with `tools` set must serialise the `Tool` array with `"type"` key.
///
/// The `Tool` struct uses `#[serde(rename = "type")]` for `tool_type`, just like `ToolCall`.
/// A missing rename would produce `"tool_type"` on the wire, which the API rejects.
/// This test validates both the array presence and the inner rename attribute together.
#[ test ]
fn chat_completion_request_tools_array_serializes_with_type_key()
{
  use api_openai_compatible::Tool;

  let tool = Tool::function(
    "get_weather",
    "Get current weather",
    serde_json::json!({ "type": "object", "properties": {} }),
  );

  let req = ChatCompletionRequest::former()
    .model( "gpt-4o".to_string() )
    .messages( vec![ Message::user( "What is the weather?" ) ] )
    .tools( vec![ tool ] )
    .form();

  let json = serde_json::to_string( &req ).expect( "ChatCompletionRequest must be serializable" );

  assert!(
    json.contains( "\"tools\"" ),
    "tools field must appear in JSON when set; got: {json}",
  );

  let parsed : serde_json::Value =
    serde_json::from_str( &json ).expect( "serialised output must be valid JSON" );

  let tools = parsed[ "tools" ].as_array()
    .expect( "tools must be a JSON array" );

  assert_eq!( tools.len(), 1, "exactly one tool must appear in the array" );
  assert_eq!(
    tools[ 0 ][ "type" ],
    "function",
    "Tool.tool_type must serialise as JSON key \"type\" with value \"function\"",
  );
  assert!(
    !json.contains( "\"tool_type\"" ),
    "JSON must NOT contain \"tool_type\" — the serde rename must be applied; got: {json}",
  );
  assert_eq!(
    tools[ 0 ][ "function" ][ "name" ],
    "get_weather",
    "function name must be present and correct",
  );
  assert_eq!(
    tools[ 0 ][ "function" ][ "description" ],
    "Get current weather",
    "function description must be present and correct",
  );
}

// ------------------------------------------------------------------ //

/// `ChatCompletionResponse` with `finish_reason = "tool_calls"` must deserialise correctly.
///
/// When the model requests function calling it stops generation with `finish_reason:
/// "tool_calls"`. Callers branch on this value to dispatch functions instead of
/// displaying content. Mishandling this reason causes function calls to be silently
/// ignored or treated as regular text completions.
#[ test ]
fn response_finish_reason_tool_calls_deserializes()
{
  let fixture = r#"{
    "id": "chatcmpl-toolcall",
    "object": "chat.completion",
    "created": 1717000100,
    "model": "gpt-4o",
    "choices": [
      {
        "index": 0,
        "message": {
          "role": "assistant",
          "content": null,
          "tool_calls": [
            {
              "id": "call_tc001",
              "type": "function",
              "function": {
                "name": "get_weather",
                "arguments": "{\"location\":\"Paris\"}"
              }
            }
          ]
        },
        "finish_reason": "tool_calls"
      }
    ],
    "usage": {
      "prompt_tokens": 30,
      "completion_tokens": 20,
      "total_tokens": 50
    }
  }"#;

  let resp : ChatCompletionResponse =
    serde_json::from_str( fixture )
    .expect( "fixture with finish_reason=tool_calls must deserialise" );

  assert_eq!(
    resp.choices[ 0 ].finish_reason.as_deref(),
    Some( "tool_calls" ),
    "finish_reason must be \"tool_calls\" for tool-call responses",
  );

  let tool_calls = resp.choices[ 0 ].message.tool_calls.as_ref()
    .expect( "tool_calls must be present in the assistant message" );

  assert_eq!( tool_calls.len(), 1, "exactly one tool call must be deserialised" );
  assert_eq!( tool_calls[ 0 ].id, "call_tc001" );
  assert_eq!( tool_calls[ 0 ].function.name, "get_weather" );
  assert_eq!(
    tool_calls[ 0 ].function.arguments,
    r#"{"location":"Paris"}"#,
    "arguments must be preserved as raw JSON string",
  );
}

// ------------------------------------------------------------------ //

/// `ChatCompletionResponse` where the assistant message itself contains `tool_calls`
/// must deserialise all nested fields correctly.
///
/// The `Message.tool_calls` field (Option<Vec<ToolCall>>) is only populated in
/// responses when the model is performing function calling. A mistake in the
/// nested `#[serde(rename = "type")]` on `ToolCall` would deserialise to `None`
/// silently, making the tool calls invisible to the caller.
#[ test ]
fn response_assistant_message_with_tool_calls_deserializes()
{
  let fixture = r#"{
    "id": "chatcmpl-nested-tc",
    "object": "chat.completion",
    "created": 1717000200,
    "model": "gpt-4o",
    "choices": [
      {
        "index": 0,
        "message": {
          "role": "assistant",
          "tool_calls": [
            {
              "id": "call_nested_001",
              "type": "function",
              "function": {
                "name": "send_email",
                "arguments": "{\"to\":\"alice@example.com\",\"subject\":\"Hello\"}"
              }
            },
            {
              "id": "call_nested_002",
              "type": "function",
              "function": {
                "name": "log_event",
                "arguments": "{\"event\":\"greeting\"}"
              }
            }
          ]
        },
        "finish_reason": "tool_calls"
      }
    ],
    "usage": {
      "prompt_tokens": 25,
      "completion_tokens": 40,
      "total_tokens": 65
    }
  }"#;

  let resp : ChatCompletionResponse =
    serde_json::from_str( fixture )
    .expect( "response with assistant tool_calls must deserialise" );

  let msg = &resp.choices[ 0 ].message;
  assert_eq!( msg.content, None, "content must be None when tool_calls are present" );

  let tool_calls = msg.tool_calls.as_ref()
    .expect( "tool_calls must be present when assistant is calling functions" );

  assert_eq!( tool_calls.len(), 2, "both tool calls must be deserialised" );
  assert_eq!( tool_calls[ 0 ].id, "call_nested_001" );
  assert_eq!( tool_calls[ 0 ].function.name, "send_email" );
  assert_eq!( tool_calls[ 1 ].id, "call_nested_002" );
  assert_eq!( tool_calls[ 1 ].function.name, "log_event" );
}

// ------------------------------------------------------------------ //

/// First streaming chunk (role-bearing) must deserialise correctly.
///
/// OpenAI-compatible streaming sends the `role` in the very first delta only.
/// All subsequent deltas have no `role`. If the first chunk is deserialized
/// without the role, streaming consumers cannot classify the participant,
/// losing the context needed to reconstruct the full assistant message.
#[ cfg( feature = "streaming" ) ]
#[ test ]
fn streaming_chunk_first_with_role_deserializes()
{
  use api_openai_compatible::{ ChatCompletionChunk, Role };

  let fixture = r#"{
    "id": "chatcmpl-stream-first",
    "object": "chat.completion.chunk",
    "created": 1717000020,
    "model": "gpt-4o",
    "choices": [
      {
        "index": 0,
        "delta": { "role": "assistant", "content": "" },
        "finish_reason": null
      }
    ]
  }"#;

  let chunk : ChatCompletionChunk =
    serde_json::from_str( fixture ).expect( "first chunk with role must deserialise" );

  assert_eq!(
    chunk.choices[ 0 ].delta.role,
    Some( Role::Assistant ),
    "first streaming chunk must carry role=assistant in the delta",
  );
  assert!(
    chunk.choices[ 0 ].finish_reason.is_none(),
    "finish_reason must be None for the first chunk",
  );
}

// ------------------------------------------------------------------ //

/// Final streaming chunk (`finish_reason` present) must deserialise correctly.
///
/// The last chunk from a streaming response carries `finish_reason: "stop"` (or
/// another stop cause) with an empty delta. If `finish_reason` were silently dropped
/// or treated as an error, streaming consumers could not detect stream completion
/// and would poll indefinitely or truncate the response prematurely.
#[ cfg( feature = "streaming" ) ]
#[ test ]
fn streaming_chunk_last_with_finish_reason_deserializes()
{
  use api_openai_compatible::ChatCompletionChunk;

  let fixture = r#"{
    "id": "chatcmpl-stream-last",
    "object": "chat.completion.chunk",
    "created": 1717000030,
    "model": "gpt-4o",
    "choices": [
      {
        "index": 0,
        "delta": {},
        "finish_reason": "stop"
      }
    ]
  }"#;

  let chunk : ChatCompletionChunk =
    serde_json::from_str( fixture ).expect( "final chunk with finish_reason must deserialise" );

  assert_eq!(
    chunk.choices[ 0 ].finish_reason.as_deref(),
    Some( "stop" ),
    "finish_reason must be \"stop\" in the final streaming chunk",
  );
  assert!(
    chunk.choices[ 0 ].delta.content.is_none(),
    "delta content must be None in the final empty chunk",
  );
  assert!(
    chunk.choices[ 0 ].delta.role.is_none(),
    "delta role must be None in the final empty chunk",
  );
}

// ------------------------------------------------------------------ //

/// `ChatCompletionChunk` must survive a full serde round-trip.
///
/// Streaming consumers may need to re-serialise chunks for debugging or logging.
/// A rename attribute that serialises correctly but deserialises wrong would only
/// be caught by a round-trip test, not by a one-way deserialisation test.
#[ cfg( feature = "streaming" ) ]
#[ test ]
fn streaming_chunk_round_trips_through_serde()
{
  use api_openai_compatible::{ ChatCompletionChunk, ChunkChoice, Delta, Role };

  let original = ChatCompletionChunk
  {
    id      : "chatcmpl-rt".to_string(),
    object  : "chat.completion.chunk".to_string(),
    created : 1_717_000_050,
    model   : "gpt-4o".to_string(),
    choices : vec!
    [
      ChunkChoice
      {
        index         : 0,
        delta         : Delta
        {
          role       : Some( Role::Assistant ),
          content    : Some( " Hello".to_string() ),
          tool_calls : None,
        },
        finish_reason : None,
      },
    ],
  };

  let json = serde_json::to_string( &original ).expect( "ChatCompletionChunk must be serializable" );
  let restored : ChatCompletionChunk =
    serde_json::from_str( &json )
    .expect( "ChatCompletionChunk must be deserializable from its own JSON" );

  assert_eq!( restored, original, "ChatCompletionChunk must survive serde round-trip unchanged" );
}

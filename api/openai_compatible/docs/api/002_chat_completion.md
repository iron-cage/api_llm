# API: Chat Completion

### Scope

- **Purpose**: Define the wire contract for the `POST chat/completions` endpoint as implemented in `api_openai_compatible`.
- **Responsibility**: Documents the Chat Completion API contract — wire types, field requirements, and tool calling.
- **In Scope**: Request serialization, response deserialization, optional field omission rules, tool-calling message shapes.
- **Out of Scope**: SSE streaming response format (see `docs/feature/001_streaming.md`), provider-specific extensions, enterprise retry logic.

### Endpoint

| Field | Value |
|-------|-------|
| Method | `POST` |
| Path | `chat/completions` (appended to `base_url` from environment) |
| Content-Type | `application/json` |
| Authorization | `Bearer <api_key>` |

### Request Wire Type: `ChatCompletionRequest`

All `Option<_>` fields are omitted from the serialized JSON when `None` (`#[serde(skip_serializing_if = "Option::is_none")]`).

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `model` | `String` | Yes | Provider model identifier, e.g. `"gpt-4o"`, `"grok-2-1212"` |
| `messages` | `Vec<Message>` | Yes | Ordered conversation history |
| `temperature` | `Option<f32>` | No | Sampling temperature `[0.0, 2.0]` |
| `max_tokens` | `Option<u32>` | No | Maximum tokens to generate |
| `top_p` | `Option<f32>` | No | Nucleus sampling threshold `[0.0, 1.0]` |
| `frequency_penalty` | `Option<f32>` | No | Frequency penalty `[0.0, 2.0]` |
| `presence_penalty` | `Option<f32>` | No | Presence penalty `[0.0, 2.0]` |
| `stream` | `Option<bool>` | No | `true` activates SSE streaming (requires `streaming` feature) |
| `tools` | `Option<Vec<Tool>>` | No | Function tool definitions for tool calling |

### Message Wire Type: `Message`

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `role` | `Role` | Yes | `system`, `user`, `assistant`, or `tool` |
| `content` | `Option<String>` | No | Text content; omitted when `None` |
| `tool_calls` | `Option<Vec<ToolCall>>` | No | Assistant role only; omitted when `None` |
| `tool_call_id` | `Option<String>` | No | Tool role only; correlates to `ToolCall::id` |

### Role Serialization

| Variant | Wire Value |
|---------|-----------|
| `Role::System` | `"system"` |
| `Role::User` | `"user"` |
| `Role::Assistant` | `"assistant"` |
| `Role::Tool` | `"tool"` |

### Response Wire Type: `ChatCompletionResponse`

| Field | Type | Notes |
|-------|------|-------|
| `id` | `String` | Opaque completion identifier, e.g. `"chatcmpl-abc123"` |
| `object` | `String` | Always `"chat.completion"` |
| `created` | `u64` | Unix timestamp |
| `model` | `String` | Model that generated the completion |
| `choices` | `Vec<Choice>` | One per `n` (default: 1) |
| `usage` | `Usage` | Token usage: `prompt_tokens`, `completion_tokens`, `total_tokens` |

### Choice Wire Type: `Choice`

| Field | Type | Notes |
|-------|------|-------|
| `index` | `u32` | Zero-based |
| `message` | `Message` | The generated message |
| `finish_reason` | `Option<String>` | `"stop"`, `"length"`, `"tool_calls"`, or `None` |

### Tool Calling

Tool definitions use `Tool::function(name, description, parameters)` convenience constructor. The `function` field carries `Function { name, description, parameters: serde_json::Value }`. Tool invocations in responses appear as `ToolCall { id, tool_type: "function", function: FunctionCall { name, arguments: String } }` where `arguments` is a JSON-encoded string (not a parsed `Value`).

### Behavioral Constraints

- All optional request fields absent from the JSON when `None` — the server uses its own defaults.
- `ChatCompletionRequest` derives `Former` for ergonomic builder-pattern construction.
- `Tool` and `Function` derive `Former` for builder-pattern construction.
- The `arguments` field in `FunctionCall` is a raw JSON string, not a `serde_json::Value` — callers must re-parse it.

### Sources

| File | Relationship |
|------|--------------|
| `src/components/chat.rs` | Defines all request/response types |
| `src/client.rs` | `Client::post("chat/completions", body)` invocation pattern |

### Tests

| File | Relationship |
|------|--------------|
| `tests/wire_test.rs` | Serde round-trips for all wire types; optional-field omission; tool call shapes |
| `tests/client_test.rs` | Integration: POST chat/completions error path with real HTTP |

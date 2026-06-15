# API: Chat Completion

### Scope

- **Purpose**: Define the wire contract for the `POST chat/completions` endpoint as implemented in `api_openai_compatible`.
- **Responsibility**: Documents the Chat Completion API contract — wire types, field requirements, and tool calling.
- **In Scope**: Request serialization, response deserialization, optional field omission rules, tool-calling message shapes.
- **Out of Scope**: SSE streaming response format (see `docs/feature/001_streaming.md`), provider-specific extensions, enterprise retry logic.

### Abstract

The chat completions endpoint accepts a serialized request body — specifying the model, conversation history, and optional generation parameters — and returns a completion containing the generated message, token usage, and finish reason. Tool calling is supported: callers supply function definitions in the request and the response may include tool invocation payloads to execute.

### Operations

| Field | Value |
|-------|-------|
| Method | POST |
| Path | `chat/completions` (appended to `base_url` from environment) |
| Content-Type | `application/json` |
| Authorization | Bearer token from environment |

### Request Wire Type: `ChatCompletionRequest`

Optional fields are omitted from the serialized JSON when absent.

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `model` | string | Yes | Provider model identifier, e.g. `"gpt-4o"`, `"grok-2-1212"` |
| `messages` | message list | Yes | Ordered conversation history |
| `temperature` | float, optional | No | Sampling temperature `[0.0, 2.0]` |
| `max_tokens` | integer, optional | No | Maximum tokens to generate |
| `top_p` | float, optional | No | Nucleus sampling threshold `[0.0, 1.0]` |
| `frequency_penalty` | float, optional | No | Frequency penalty `[0.0, 2.0]` |
| `presence_penalty` | float, optional | No | Presence penalty `[0.0, 2.0]` |
| `stream` | boolean, optional | No | `true` activates SSE streaming (requires `streaming` feature) |
| `tools` | tool list, optional | No | Function tool definitions for tool calling |

### Message Wire Type: `Message`

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `role` | role enum | Yes | `system`, `user`, `assistant`, or `tool` |
| `content` | string, optional | No | Text content; absent when not set |
| `tool_calls` | tool call list, optional | No | Assistant role only; absent when not set |
| `tool_call_id` | string, optional | No | Tool role only; correlates to the originating tool call |

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
| `id` | string | Opaque completion identifier, e.g. `"chatcmpl-abc123"` |
| `object` | string | Always `"chat.completion"` |
| `created` | integer | Unix timestamp |
| `model` | string | Model that generated the completion |
| `choices` | choice list | One per `n` (default: 1) |
| `usage` | usage object | Token usage: `prompt_tokens`, `completion_tokens`, `total_tokens` |

### Choice Wire Type: `Choice`

| Field | Type | Notes |
|-------|------|-------|
| `index` | integer | Zero-based |
| `message` | message object | The generated message |
| `finish_reason` | string, optional | `"stop"`, `"length"`, `"tool_calls"`, or absent when incomplete |

### Tool Calling

Tool definitions include a name, description, and parameter schema (arbitrary JSON object). Tool invocations in responses include a tool call ID, function name, and a JSON-encoded arguments string. Callers must re-parse the arguments string to obtain structured parameters.

### Behavioral Constraints

- Optional request fields are absent from the JSON when not set — the server uses its own defaults.
- The `arguments` field in a tool invocation is a raw JSON string — callers must re-parse it.

### Error Handling

Non-2xx HTTP responses produce an `Api` error containing the response body. Network failures produce `Network` or `Timeout` errors. Malformed response JSON produces `Deserialise`. All error variants are defined in `OpenAiCompatError` (see `docs/api/001_endpoint_coverage.md`).

### Compatibility Guarantees

Request and response shapes follow the OpenAI chat completions schema. New optional request fields are additive changes. Changes to required fields or response structure require a major version bump.

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

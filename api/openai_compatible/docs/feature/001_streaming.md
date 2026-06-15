# Feature: Streaming

### Scope

- **Purpose**: Define the Server-Sent Events streaming behavior for chat completions in `api_openai_compatible`.
- **Responsibility**: Documents the Streaming feature — activation requirements, wire types, and behavioral constraints.
- **In Scope**: `ChatCompletionChunk`, `ChunkChoice`, `Delta` wire types; the `stream` field in `ChatCompletionRequest`; SSE framing and parsing.
- **Out of Scope**: Non-streaming chat completion (see `docs/api/002_chat_completion.md`), sync streaming wrappers, WebSocket streaming.

### Design

When the `streaming` Cargo feature is enabled, callers may set `stream: Some(true)` in `ChatCompletionRequest` to receive the server response as a sequence of Server-Sent Events. Each SSE line delivers a `ChatCompletionChunk` — a partial update to the assistant message being assembled. All chunk wire types are defined in `src/components/streaming.rs` and are zero-overhead when the `streaming` feature is disabled.

### Activation

| Requirement | Detail |
|-------------|--------|
| Cargo feature | `streaming` — activates `ChatCompletionChunk`, `ChunkChoice`, `Delta` types |
| Request field | `ChatCompletionRequest::stream: Some(true)` |
| Default | `full` feature enables `streaming` |

### Wire Types

| Type | Role |
|------|------|
| `ChatCompletionChunk` | One SSE frame: `id`, `object`, `created`, `model`, `choices` |
| `ChunkChoice` | One delta choice within a chunk: `index`, `delta`, `finish_reason` |
| `Delta` | Incremental content update: optional `role`, `content`, `tool_calls` |

### Behavioral Constraints

- `Delta::role` is present only in the first chunk of a response; absent in all subsequent chunks.
- `Delta::content` accumulates partial text; callers concatenate across chunks.
- `ChunkChoice::finish_reason` is `None` in all intermediate chunks; set only in the final chunk.
- An empty delta is a valid, meaningful value.
- Optional fields (`role`, `content`, `tool_calls`) are absent from serialized output when unset.

### Sources

| File | Relationship |
|------|--------------|
| `src/components/streaming.rs` | Defines `ChatCompletionChunk`, `ChunkChoice`, `Delta` |
| `src/components/chat.rs` | Defines `ChatCompletionRequest::stream` field |
| `src/lib.rs` | `streaming` feature gate declaration |

### Tests

| File | Relationship |
|------|--------------|
| `tests/wire_test.rs` | 5 streaming-gated tests: chunk serialization, delta round-trips, finish_reason logic |

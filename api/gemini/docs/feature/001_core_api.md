# Feature: Core API

### Scope

- **Purpose**: Document the core Gemini API endpoints exposed by the thin client — content generation, embeddings, model management, streaming, multimodal support, and function calling.
- **Responsibility**: Documents the Core API feature — endpoint table, streaming protocol, content capabilities, and error handling contract.
- **In Scope**: All methods under `src/models/api/` — content generation, embeddings, models; streaming in `src/models/api/content_generation/`.
- **Out of Scope**: Enterprise reliability features (see feature/002); experimental stub APIs awaiting Gemini endpoint availability.

### Design

Core API features are always-on when the `enabled` feature flag is active. They expose Gemini endpoints transparently with no client-side intelligence — request types map 1:1 to API JSON structures, response types map 1:1 to API JSON responses.

Authentication uses the API key as a query parameter (`?key=...`) rather than a Bearer header — this is specific to the Gemini v1beta protocol and distinguishes it from other LLM provider crates in this workspace.

### Endpoints

| Endpoint | Method | Path | Feature Gate |
|----------|--------|------|-------------|
| List Models | `Client::models().list()` | `GET /v1beta/models` | `enabled` |
| Get Model | `Client::models().by_name(id).get()` | `GET /v1beta/models/{model}` | `enabled` |
| Generate Content | `Client::models().by_name(id).generate_content()` | `POST /v1beta/models/{model}:generateContent` | `enabled` |
| Stream Generate Content | `Client::models().by_name(id).stream_generate_content()` | `POST /v1beta/models/{model}:streamGenerateContent` | `streaming` |
| Embed Content | `Client::models().by_name(id).embed_content()` | `POST /v1beta/models/{model}:embedContent` | `enabled` |
| Batch Embed Contents | `Client::models().by_name(id).batch_embed_contents()` | `POST /v1beta/models/{model}:batchEmbedContents` | `enabled` |
| Count Tokens | `Client::models().by_name(id).count_tokens()` | `POST /v1beta/models/{model}:countTokens` | `enabled` |

### Streaming Protocol

The `:streamGenerateContent` endpoint uses a **JSON array format** — not SSE, not NDJSON.

The complete HTTP response body is buffered, then parsed as `Vec<GenerateContentResponse>`, then yielded as async stream elements via `async-stream`. This trades first-chunk latency (must wait for complete response) for simplicity. Gemini responses are typically small and fast, making buffering acceptable.

See `docs/investigations/001_streaming_format.md` for the investigation history.

### Content Capabilities

| Capability | Request Type | Feature Gate |
|------------|-------------|-------------|
| Text generation | `Part { text: Some("...") }` | `enabled` |
| Image analysis (multimodal) | `Part { inline_data: Some(Blob { mime_type, data }) }` | `enabled` |
| Function calling | `tools` field in `GenerateContentRequest` | `enabled` |
| Safety settings | `safety_settings` field in `GenerateContentRequest` | `enabled` |
| System instructions | `system_instruction` field in `GenerateContentRequest` | `enabled` |
| Code execution | `code_execution` tool in `tools` field | `enabled` |
| Search grounding | `google_search_retrieval` in `tools` field | `enabled` |

### Error Handling

The `Error` enum covers all failure categories:

| Variant | Trigger |
|---------|---------|
| `ApiError(String)` | Non-2xx HTTP response from Gemini API |
| `AuthenticationError(String)` | Invalid or missing API key |
| `NetworkError(String)` | Connection, timeout, or transport failure |
| `SerializationError(String)` | JSON serialization/deserialization failure |
| `RequestBuilding(String)` | Invalid request construction |
| `InvalidArgument(String)` | Caller-provided invalid parameter value |

### Sources

| File | Relationship |
|------|--------------|
| `src/models/api/content_generation/api_impl.rs` | `generate_content()`, `stream_generate_content()`, `process_streaming_response()` |
| `src/models/api/embeddings.rs` | `embed_content()`, `batch_embed_contents()` |
| `src/models/api/models.rs` | `list()`, `get()` — model listing and detail retrieval |
| `src/client/core.rs` | `Client` struct — constructors and HTTP infrastructure |
| `src/error.rs` | `Error` enum — all error variants |

### Tests

| File | Relationship |
|------|--------------|
| `tests/integration_tests.rs` | Core endpoint integration tests — list models, generate content, embeddings |
| `tests/count_tokens_tests.rs` | Count tokens endpoint tests |
| `tests/code_execution_tests.rs` | Code execution capability tests |
| `tests/system_instructions_tests.rs` | System instructions configuration tests |

### Protocols

| File | Relationship |
|------|-------------|
| `protocol/001_streaming_format.md` | JSON array streaming protocol used by stream_generate_content |

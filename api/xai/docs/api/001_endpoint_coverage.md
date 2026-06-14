# API: Endpoint Coverage

### Scope

- **Purpose**: Define required X.AI Grok API endpoint coverage and the feature-gating policy for all optional capabilities in `api_xai`.
- **Responsibility**: Documents the X.AI Grok API endpoint coverage — required endpoints, feature-gate policy, and error handling contract.
- **In Scope**: All client methods in `src/client.rs` and optional feature modules wrapping API endpoints.
- **Out of Scope**: OpenAI endpoints not supported by X.AI (vision, audio, embeddings, fine-tuning, assistants, files, image generation).

### Abstract

The `api_xai` crate provides a thin HTTP client covering all X.AI Grok API endpoints. The API is OpenAI-compatible at the wire level — same REST patterns and request/response schemas as OpenAI's API. Each method maps one-to-one to an X.AI API path. Optional capabilities are independently gated by Cargo feature flags and require explicit opt-in.

### Operations

| Endpoint | Method | Path | Feature Gate |
|----------|--------|------|--------------|
| Chat completions | `client.chat().create()` | `POST /v1/chat/completions` | always-on |
| Streaming chat | `client.chat().create_stream()` | `POST /v1/chat/completions` (SSE) | `streaming` |
| List models | `client.models().list()` | `GET /v1/models` | always-on |
| Get model | `client.models().get(id)` | `GET /v1/models/{id}` | always-on |

Feature-gating policy: `enabled` is the master switch for `Client` and all core types. `full` activates all features. `integration` enables integration test compilation. `default = ["full"]` for ease of use; downstream crates that need minimal builds disable defaults and select specific flags.

### Error Handling

All API methods return `Result<T, error_tools::Error>`. Error variants cover authentication failures (401), rate limits (429), network timeouts, and serialization failures. The `error_tools` crate is the exclusive error infrastructure — no mixing with `anyhow` or `thiserror`.

### Compatibility Guarantees

X.AI Grok API is OpenAI-compatible; request/response formats follow OpenAI chat completions schema. Model names differ from OpenAI: primary Grok models are `grok-3`, `grok-2-1212`, and `grok-beta`. The following OpenAI features are not available in X.AI API and are out of scope: vision/image inputs, audio processing, embeddings, fine-tuning, Assistants API, file uploads, image generation.

### Sources

| File | Relationship |
|------|--------------|
| `src/client.rs` | Main client — chat completion and model listing methods |
| `src/chat.rs` | Chat completion request/response types |
| `src/models.rs` | Model listing and retrieval types |
| `src/client_api_accessors.rs` | `ClientApiAccessors<E>` trait — `chat()` and `models()` accessors |

### Tests

| File | Relationship |
|------|--------------|
| `tests/integration_chat.rs` | Chat completion endpoint integration tests |
| `tests/integration_models.rs` | Model listing and retrieval endpoint tests |
| `tests/integration_streaming.rs` | Streaming chat endpoint integration tests |
| `tests/integration_tool_calling.rs` | Tool calling chat completion tests |

# API: Endpoint Coverage

### Scope

- **Purpose**: Define required OpenAI API endpoint coverage and the feature-gating policy for all optional capabilities in `api_openai`.
- **Responsibility**: Documents the OpenAI API endpoint coverage — required endpoints, feature-gate policy, and error handling contract.
- **In Scope**: All client methods and optional feature modules wrapping API endpoints.
- **Out of Scope**: Provider-specific extensions not in OpenAI's published API surface.

### Abstract

The `api_openai` crate provides a thin HTTP client covering OpenAI API endpoints. Each method maps one-to-one to an OpenAI API path. Optional capabilities are independently gated by Cargo feature flags and require explicit opt-in. The crate is the largest in the `api_llm` workspace (108 source files) and covers the full OpenAI API surface.

### Operations

| Endpoint Group | Feature Gate | Notes |
|---------------|-------------|-------|
| Chat completions (`/v1/chat/completions`) | always-on | Primary conversational AI interface |
| Streaming chat | `streaming` | SSE via eventsource-stream |
| Responses API | always-on | Create, retrieve, update, cancel, delete |
| Embeddings (`/v1/embeddings`) | always-on | Text-to-vector conversion |
| Models (`/v1/models`) | always-on | Model listing and retrieval |
| Images (`/v1/images`) | always-on | Generation, editing, variations |
| Audio (`/v1/audio`) | always-on | Speech-to-text and text-to-speech |
| Files (`/v1/files`) | always-on | File upload and management |
| Fine-tuning (`/v1/fine_tuning`) | always-on | Model fine-tuning operations |
| Moderations (`/v1/moderations`) | always-on | Content moderation |
| Assistants (`/v1/assistants`) | always-on | Assistant lifecycle management |
| Realtime WebSocket | `websocket` | Bidirectional real-time API |

Feature-gating policy: `enabled` is the master switch. `full` activates all features. `integration` enables integration test compilation. `default = ["full"]` for ease of use.

### Error Handling

All API methods return `Result<T, error_tools::Error>`. Error variants cover authentication failures (401), rate limits (429), server errors (5xx), network timeouts, and serialization failures. The `error_tools` crate is the exclusive error infrastructure.

### Compatibility Guarantees

The endpoint surface matches the OpenAI REST API at the version tested and documented in this instance. Additions to the OpenAI API surface are added as MINOR crate releases. Removals or breaking changes to the public crate interface require a MAJOR crate release.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | Top-level module declarations — all endpoint modules registered here |
| `src/components/` | Shared request/response types for all API endpoints |
| `src/realtime/` | WebSocket realtime endpoint (gated on `websocket` feature) |

### Tests

| File | Relationship |
|------|--------------|
| `tests/integration.rs` | Core endpoint integration tests — chat completions, models, assistants |
| `tests/integration_reorganized.rs` | Extended endpoint group integration tests |

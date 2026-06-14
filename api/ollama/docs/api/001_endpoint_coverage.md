# API: Endpoint Coverage

### Scope

- **Purpose**: Define required Ollama API endpoint coverage and the feature-gating policy for all optional capabilities in `api_ollama`.
- **Responsibility**: Documents the Ollama API endpoint coverage — required endpoints, feature-gate policy, and error handling contract.
- **In Scope**: All client methods and optional feature modules wrapping API endpoints.
- **Out of Scope**: Provider-specific extensions not in Ollama's published API surface.

### Abstract

The `api_ollama` crate provides a thin HTTP client covering Ollama's local LLM runtime API. Each method maps one-to-one to an Ollama API path. Optional capabilities are independently gated by Cargo feature flags and require explicit opt-in.

### Operations

| Endpoint Group | Feature Gate | Notes |
|---------------|-------------|-------|
| Chat completion (`/api/chat`) | always-on | Multi-turn conversational interface |
| Text generation (`/api/generate`) | always-on | Single-prompt completion |
| Model listing (`/api/tags`) | always-on | List available local models |
| Model details (`/api/show`) | always-on | Retrieve model metadata |
| Streaming chat/generation | `streaming` | NDJSON streaming responses |
| Sync API wrappers | `sync_api` | Blocking wrappers over async methods |

Feature-gating policy: `enabled` is the master switch. `full` activates all features. `integration` enables integration test compilation. `default = ["full"]` for ease of use.

### Error Handling

All API methods return `Result<T, error_tools::Error>`. Error variants cover connection failures (server not running), HTTP errors, response parsing failures, and streaming errors. The `error_tools` crate is the exclusive error infrastructure.

### Compatibility Guarantees

The endpoint surface matches the Ollama REST API at the version tested and documented in this instance. Additions to the Ollama API surface are added as MINOR crate releases. Removals or breaking changes require a MAJOR crate release.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | Top-level module declarations — all endpoint modules registered here |
| `src/client.rs` / `src/client/` | Main client implementation with all endpoint methods |

### Tests

| File | Relationship |
|------|--------------|
| `tests/core_functionality_tests.rs` | Core endpoint tests — chat, generate, model listing |
| `tests/core_client_api_tests.rs` | Client method integration tests |
| `tests/api_comprehensive_tests.rs` | Comprehensive endpoint coverage tests |

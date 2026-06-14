# API: Endpoint Coverage

### Scope

- **Purpose**: Define required Anthropic API endpoint coverage and the feature-gating policy for all optional capabilities in `api_claude`.
- **Responsibility**: Documents the Claude API endpoint coverage — required endpoints, feature-gate policy, and error handling contract.
- **In Scope**: All client methods in `src/client/implementation.rs` and optional feature modules that wrap API endpoints.
- **Out of Scope**: Workspace-level and ecosystem-level APIs outside Anthropic's published offering; internal helper utilities that do not map to API calls.

### Abstract

The `api_claude` crate provides a thin HTTP client covering all currently-available Anthropic API endpoints. Each method maps one-to-one to an Anthropic API path. Optional capabilities (streaming, tools, vision, batch processing) are independently gated by Cargo feature flags and must be explicitly enabled by the caller — no automatic activation occurs. The `embeddings` endpoint is reserved for future compatibility; Anthropic does not currently expose it.

### Operations

| Endpoint | Method | Path | Feature Gate |
|----------|--------|------|--------------|
| Create message | `Client::create_message()` | `POST /v1/messages` | always-on |
| Count tokens | `Client::count_message_tokens()` | `POST /v1/messages/count_tokens` | `count-tokens` |
| Stream message | `Client::create_message_stream()` | `POST /v1/messages` (SSE) | `streaming` |
| Create batch | `Client::create_messages_batch()` | `POST /v1/messages/batches` | `batch-processing` |
| Retrieve batch | `Client::retrieve_batch()` | `GET /v1/messages/batches/{id}` | `batch-processing` |
| List batches | `Client::list_batches()` | `GET /v1/messages/batches` | `batch-processing` |
| Cancel batch | `Client::cancel_batch()` | `DELETE /v1/messages/batches/{id}` | `batch-processing` |
| Create embedding | `Client::create_embedding()` | — | `embeddings` (stub — not available) |

Feature-gating policy: `enabled` is the master switch for all core types and the client struct. `full` activates all features. `integration` enables integration test compilation. `default = ["full"]` for ease of use; downstream crates that need minimal builds disable defaults and select specific flags.

### Error Handling

All API methods return `AnthropicResult<T>`. The `AnthropicError` type covers: authentication failures (401), rate limits (429), server errors (5xx), network timeouts, and serialization failures. The `embeddings` stub methods return `AnthropicError::NotImplemented` with an explanatory message when called. Enterprise feature errors (circuit open, quota exceeded) surface as `AnthropicError` variants when the caller has opted into those features.

### Compatibility Guarantees

All requests carry the `anthropic-version: 2023-06-01` header (constant `ANTHROPIC_API_VERSION`); this version string is part of the API contract and will only change on a breaking Anthropic API revision. Methods marked always-on in the Operations table are stable and will not be removed without a crate major version bump. Feature-gated methods are stable within their feature's enabled state. The `embeddings` stub API surface (`create_embedding`, `create_embeddings_batch`) is reserved and will return `NotImplemented` until Anthropic publishes the endpoint — its signature may change when the endpoint becomes available.

### Sources

| File | Relationship |
|------|--------------|
| `src/client/implementation.rs` | All core endpoint implementations — `create_message()`, `count_message_tokens()`, batch methods |
| `src/client/types.rs` | Request and response type definitions for all endpoints |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/api/01_endpoint_coverage.md` | Behavioral spec — 12 scenarios verifying all core and feature-gated endpoints are callable and correctly gated |
| `tests/inc/mod.rs` | Aggregates integration tests that verify each endpoint is accessible and returns expected types |

# API: Reference

### Scope

- **Purpose**: Document the complete public API contract for `api_huggingface` — operations, endpoint mapping, error conditions, and versioning guarantees.
- **Responsibility**: Complete public API contract — method signatures, endpoints, error types, and parameters.
- **In Scope**: Client construction, all API operation groups, error conditions, compatibility guarantees.
- **Out of Scope**: Internal implementation details, source-level comments, Cargo feature configuration.

### Abstract

`api_huggingface` provides access to HuggingFace Inference API, Router API, and model management endpoints through a single generic client type parameterized over an environment. All operations map directly to one HuggingFace API endpoint with no implicit behaviors. Enterprise features are accessible only when their corresponding Cargo feature flag is enabled. Authentication is provided by a secret loaded explicitly from an environment variable or workspace secrets file.

### Operations

**Client construction** — initialize a client from a configured environment; load API credentials from an environment variable or workspace secrets file; configure base URL and timeout.

**Text generation** — submit a prompt with a model identifier to the Inference API and receive generated text; supply optional inference parameters (temperature, max tokens, top-p, top-k, repetition penalty); receive output as a token stream (requires `inference-streaming`).

**Chat completion** — submit a conversation history with role-based messages to the Router API and receive an assistant reply; optionally specialize the request for math problems.

**Embedding generation** — convert text to a dense vector; generate vectors for a batch of texts (requires `embeddings-batch`); compute cosine similarity between two texts (requires `embeddings-similarity`); configure normalization and truncation.

**Model management** — retrieve model metadata; check model availability; poll until a model becomes available within a specified timeout.

**Vision** (requires `vision`) — classify an image and receive confidence scores; detect objects with bounding boxes; generate a caption from an image.

**Audio** (requires `audio`) — transcribe speech to text; synthesize text to speech; classify audio content; transform audio to audio.

### Error Handling

All operations return a `Result`. Callers must handle errors from the following conditions:

- Invalid request or parameter validation failure
- Authentication failure — missing, expired, or unauthorized API key
- Request rate exceeded or quota limit reached
- Model unavailable — loading, cold start, or not found
- HTTP transport failure — connection error, timeout, or DNS failure
- Response deserialization failure — invalid or unexpected response format

Error variant additions in future versions are non-breaking; callers should use a catch-all arm.

### Compatibility Guarantees

All public method signatures follow semantic versioning. Breaking changes require a major version increment. Feature flag additions are non-breaking; removals or renames are breaking changes.

### Features

| File | Relationship |
|------|--------------|
| `feature/001_enterprise_reliability.md` | Enterprise features usable in conjunction with all operations |

### Invariants

| File | Relationship |
|------|--------------|
| `invariant/001_thin_client_principle.md` | Enforces one-endpoint-per-method mapping — no implicit behaviors |
| `invariant/002_testing_standards.md` | Enforces real API testing for all methods documented here |

### Patterns

| File | Relationship |
|------|--------------|
| `pattern/001_module_organization.md` | Module structure exposing all API accessors via `mod_interface!` |

### Sources

| File | Relationship |
|------|--------------|
| `src/client.rs` | Client type and all API accessor methods |
| `src/components/input.rs` | Inference parameter type |
| `src/components/models.rs` | Named model constant functions |
| `src/embeddings.rs` | Embedding generation operations |
| `src/error.rs` | Error type and all error conditions |
| `src/inference.rs` | Text generation operations |
| `src/models.rs` | Model management operations |
| `src/providers.rs` | Router API chat completion operations |
| `src/secret.rs` | Credential loading |

### Tests

| File | Relationship |
|------|--------------|
| `tests/client_tests.rs` | Client initialization and configuration |
| `tests/docs/api/01_reference.md` | GWT spec scenarios for this doc instance |
| `tests/embeddings_tests.rs` | Embedding generation operations |
| `tests/inference_tests.rs` | Text generation operations |
| `tests/models_tests.rs` | Model management operations |
| `tests/providers_api_tests.rs` | Router API chat completion operations |

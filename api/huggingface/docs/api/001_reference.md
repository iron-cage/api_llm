# API: Client Interface

### Scope

- **Purpose**: Document the complete public API contract for `api_huggingface` — method signatures, endpoint mapping, error types, and parameter contracts.
- **Responsibility**: All contributors; new public methods or changed signatures require updating this document before merge.
- **In Scope**: Client method signatures, endpoint mapping to HuggingFace Inference and Router APIs, request/response type contracts, error enum variants, parameter constraints.
- **Out of Scope**: Internal implementation details, source-level comments, Cargo feature configuration, example usage code.

### Abstract

The `api_huggingface` client provides access to HuggingFace Inference API and Router API endpoints through a generic `Client<E>` type parameterized over an environment `E`. All methods map directly to one HuggingFace API endpoint with no implicit behaviors. Enterprise features (circuit breaker, rate limiting, failover, caching, etc.) are accessible only when their corresponding Cargo feature is enabled. Authentication is provided via `Secret` loaded explicitly from an environment variable or workspace secrets file.

### Client Construction

- `Client::build(env)` — construct from an environment, returns `Result<Client<E>>`
- `Secret::load_from_env(key)` — load API key from an environment variable
- `HuggingFaceEnvironmentImpl::build(secret, base_url)` — build environment from secret and optional URL override

### Inference Operations

Maps to HuggingFace Inference API endpoint `/models/{model_id}`.

- `client.inference().create(prompt, model)` — single-turn text generation
- `client.inference().create_with_parameters(prompt, model, params)` — generation with inference parameters (requires `inference-parameters` feature)
- `client.inference().create_stream(prompt, model)` — streaming text generation (requires `inference-streaming` feature)
- `InferenceParameters` — controls temperature, max tokens, top-p, top-k, repetition penalty

### Providers Operations

Maps to HuggingFace Inference Providers API endpoint `/v1/chat/completions` (Pro plan models).

- `client.providers().simple_chat(model, message)` — single-message chat completion
- `client.providers().math_completion(model, question)` — math-specialized completion
- `client.providers().chat_completion(model, messages, max_tokens, temperature, top_p)` — full chat completion with role-based messages
- `ChatMessage` — role-based message with `role: String` and `content: String`

### Embeddings Operations

Maps to HuggingFace Feature Extraction API.

- `client.embeddings().create(text, model)` — generate single embedding vector
- `client.embeddings().create_batch(texts, model)` — generate multiple embedding vectors (requires `embeddings-batch` feature)
- `client.embeddings().similarity(text1, text2, model)` — cosine similarity between two texts (requires `embeddings-similarity` feature)
- `EmbeddingOptions` — normalize and truncate configuration flags

### Model Management Operations

Maps to HuggingFace Models API.

- `client.models().get(model)` — retrieve model metadata
- `client.models().is_available(model)` — check model readiness
- `client.models().status(model)` — get current model status string
- `client.models().wait_for_model(model, timeout)` — poll until model available or timeout

### Vision Operations

Maps to HuggingFace Vision API endpoints (requires `vision` feature).

- `client.vision().classify(image, model)` — image classification with confidence scores
- `client.vision().detect(image, model)` — object detection with bounding boxes
- `client.vision().caption(image, model)` — image-to-text caption generation

### Audio Operations

Maps to HuggingFace Audio API endpoints (requires `audio` feature).

- `client.audio().transcribe(audio, model)` — automatic speech recognition
- `client.audio().synthesize(text, model)` — text-to-speech generation
- `client.audio().classify(audio, model)` — audio classification
- `client.audio().transform(audio, model)` — audio-to-audio transformation

### Error Types

`HuggingFaceError` variants:

- `Api(msg)` — invalid requests, model errors, parameter validation failures
- `Authentication(msg)` — invalid API key, expired tokens, permission errors
- `RateLimit(msg)` — request rate exceeded, quota limits
- `ModelUnavailable(msg)` — model loading, cold start, not found
- `Http(msg)` — connection errors, HTTP transport failures, timeouts, DNS failures
- `Serialization(msg)` — JSON parsing errors, invalid response formats

### Model Constants

`Models` struct provides associated functions for commonly used model identifiers:

- Text generation: `Models::llama_3_1_8b_instruct()`, `Models::mistral_7b_instruct()`, `Models::kimi_k2_instruct()`
- Embeddings: `Models::all_minilm_l6_v2()`, `Models::bge_large_en_v1_5()`
- Returns model identifier strings compatible with all API operations above

### Compatibility Guarantees

All public method signatures follow semantic versioning. Breaking changes require a major version increment. Error variant additions are non-breaking (callers should use catch-all arms). Feature flag additions are non-breaking; removals or renames are breaking changes.

### Invariants

| File | Relationship |
|------|--------------|
| `invariant/001_thin_client_principle.md` | Enforces one-endpoint-per-method mapping — no implicit behaviors |
| `invariant/002_testing_standards.md` | Enforces real API testing for all methods documented here |

### Patterns

| File | Relationship |
|------|--------------|
| `pattern/001_module_organization.md` | Module structure exposing all API accessors via `mod_interface!` |

### Features

| File | Relationship |
|------|--------------|
| `feature/001_enterprise_reliability.md` | Enterprise features usable in conjunction with all operations above |

### Sources

| File | Relationship |
|------|--------------|
| `src/secret.rs` | `Secret::load_from_env()` — credential loading |
| `src/error.rs` | `HuggingFaceError` enum — all variants listed in Error Types |
| `src/client.rs` | `Client<E>` struct — all accessor methods |
| `src/inference.rs` | `Inference<E>` — text generation methods |
| `src/providers.rs` | `Providers<E>` — Router API chat completion methods |
| `src/embeddings.rs` | `Embeddings<E>` — embedding generation methods |
| `src/models.rs` | `Models<E>` — model management methods |
| `src/components/models.rs` | `Models` struct — model constant functions |
| `src/components/input.rs` | `InferenceParameters` — inference parameter type |

### Tests

| File | Relationship |
|------|--------------|
| `tests/inference_tests.rs` | Integration tests for Inference operations |
| `tests/providers_api_tests.rs` | Unit tests for Providers API types |
| `tests/embeddings_tests.rs` | Integration tests for Embeddings operations |
| `tests/models_tests.rs` | Integration tests for Model Management |
| `tests/client_tests.rs` | Client initialization and configuration tests |
| `tests/docs/api/01_reference.md` | GWT spec scenarios for this doc instance |

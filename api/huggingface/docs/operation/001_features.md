# Operation: Feature Flag Management

### Scope

- **Purpose**: Govern how Cargo feature flags are selected, combined, and verified for `api_huggingface` builds and deployments.
- **Responsibility**: All contributors; feature flag additions or removals require updating this document and the `feature/` doc instance before merge.
- **In Scope**: Feature selection procedures, combination rules, verification steps, feature tier classification.
- **Out of Scope**: Source-level implementation details, API method signatures, testing methodology.

### Trigger

Adding, removing, or changing any Cargo feature flag in `Cargo.toml`, or selecting features for a new integration of `api_huggingface`.

### Prerequisites

- Cargo workspace with `api_huggingface` dependency
- Understanding of which capability tier is needed (Tier 1 core or Tier 2 enterprise)
- HuggingFace API key for builds that include `integration`

### Steps

1. Identify required capability tier: Tier 1 (core APIs) or Tier 2 (enterprise features).
2. Select the minimum feature set: use `enabled` for types-only, `basic` for core API access, `full` for all features.
3. Add optional capability features as needed (see Feature Reference below).
4. Verify `integration` feature is present if running integration tests.
5. Build with `RUSTFLAGS="-D warnings" cargo build --features <selected>` and confirm zero warnings.
6. Run `cargo nextest run --features <selected>` to verify tests pass for the selected feature combination.

### Expected Outcome

Build succeeds with zero warnings. All tests for the selected feature set pass. No integration test runs without real API credentials.

### Rollback Procedure

Remove the newly added feature flag from `Cargo.toml`. Revert any capability-specific code guarded by the removed feature. Re-run build and tests to confirm clean state.

### Feature Reference

#### Convenience Bundles

| Feature | Includes | Use Case |
|---------|----------|----------|
| `default` | `full` (alias) | Development and CI with all features |
| `full` | `basic` + all enterprise + `integration` | Full capability build |
| `basic` | `inference` + `embeddings` + `models` + `env-config` | Core APIs without enterprise features |
| `enabled` | core serialization deps only | Types and serde only, no HTTP |

#### Tier 1 — Core API Features

| Feature | Description |
|---------|-------------|
| `inference` | Text generation via `/models/{model_id}` |
| `embeddings` | Embedding generation and feature extraction |
| `models` | Model metadata and availability queries |
| `vision` | Image classification, detection, captioning |
| `audio` | ASR, TTS, audio classification, audio-to-audio |
| `inference-streaming` | Server-sent event streaming for text generation |
| `inference-parameters` | `InferenceParameters` for temperature, top-p, etc. |
| `inference-retry` | Explicit retry logic with exponential backoff |
| `streaming-control` | Pause, resume, cancel streaming operations |
| `embeddings-similarity` | Cosine similarity between embedding vectors |
| `embeddings-batch` | Batch embedding generation |
| `model-constants` | `Models` struct with named model constants |
| `env-config` | `HuggingFaceEnvironmentImpl` environment builder |
| `sync` | Blocking wrappers around all async operations |
| `logging` | `tracing` integration for request/response logging |

#### Tier 2 — Enterprise Reliability Features

| Feature | Description |
|---------|-------------|
| `reliability` | Base reliability module (required by all enterprise features) |
| `circuit-breaker` | Failure detection with automatic open/close state |
| `rate-limiting` | Token bucket rate limiter per second/minute/hour |
| `failover` | Multi-endpoint failover with Priority, RoundRobin, Random, Sticky strategies |
| `health-checks` | Background endpoint health monitoring |
| `performance-metrics` | Request latency, throughput, and error rate tracking |
| `caching` | LRU cache with TTL and eviction statistics |
| `token-counting` | Token estimation with multiple counting strategies |
| `dynamic-config` | Runtime configuration updates with watcher callbacks |

#### Testing Features

| Feature | Description |
|---------|-------------|
| `integration` | Enables real-API integration tests (requires `HUGGINGFACE_API_KEY`) |
| `integration-tests` | Internal alias used by `integration` — do not use directly |

### Invariants

| File | Relationship |
|------|--------------|
| `invariant/001_thin_client_principle.md` | Enterprise features must be opt-in via feature flags — never automatic |
| `invariant/002_testing_standards.md` | `integration` feature required for all real-API tests |

### Sources

| File | Relationship |
|------|--------------|
| `Cargo.toml` | Canonical feature flag definitions — authoritative for all feature names above |
| `src/lib.rs` | Feature-gated `pub mod` declarations matching the feature table |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/operation/01_features.md` | GWT spec scenarios for this doc instance |

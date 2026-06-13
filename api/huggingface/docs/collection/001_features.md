# Collection: Features

### Scope

- **Purpose**: Enumerate all Cargo feature flags for `api_huggingface`, their grouping by convenience bundle or capability tier, and their stated capabilities.
- **Responsibility**: This collection/ instance — the authoritative feature flag catalog for `api_huggingface`.
- **In Scope**: All Cargo feature flags defined in `Cargo.toml`, grouped by tier; convenience bundle descriptions.
- **Out of Scope**: Procedural steps for selecting features, source-level implementation, API method signatures.

### Items

**Convenience Bundles**

| Feature | Includes | Use Case |
|---------|----------|----------|
| `default` | `full` (alias) | Development and CI with all features |
| `full` | `basic` + all enterprise + `integration` | Full capability build |
| `basic` | `inference` + `embeddings` + `models` + `env-config` | Core APIs without enterprise features |
| `enabled` | core serialization dependencies only | Types and serde only — no HTTP |

**Tier 1 — Core API Features**

| Feature | Description |
|---------|-------------|
| `inference` | Text generation via the Inference API |
| `embeddings` | Embedding generation and feature extraction |
| `models` | Model metadata and availability queries |
| `vision` | Image classification, detection, and captioning |
| `audio` | ASR, TTS, audio classification, and audio-to-audio |
| `inference-streaming` | Server-sent event streaming for text generation |
| `inference-parameters` | Inference parameter control (temperature, top-p, etc.) |
| `inference-retry` | Explicit retry logic with exponential backoff |
| `streaming-control` | Pause, resume, and cancel streaming operations |
| `embeddings-similarity` | Cosine similarity between embedding vectors |
| `embeddings-batch` | Batch embedding generation |
| `model-constants` | Named model constant identifiers |
| `env-config` | Environment builder for client construction |
| `sync` | Blocking wrappers around all async operations |
| `logging` | `tracing` integration for request and response logging |

**Tier 2 — Enterprise Reliability Features**

| Feature | Description |
|---------|-------------|
| `reliability` | Base reliability module — required by all enterprise features |
| `circuit-breaker` | Failure detection with automatic open/close state |
| `rate-limiting` | Token bucket rate limiter per second, minute, and hour |
| `failover` | Multi-endpoint failover with configurable strategy |
| `health-checks` | Background endpoint health monitoring |
| `performance-metrics` | Request latency, throughput, and error rate tracking |
| `caching` | LRU cache with TTL and eviction statistics |
| `token-counting` | Token estimation with multiple counting strategies |
| `dynamic-config` | Runtime configuration updates with watcher callbacks |

**Testing Features**

| Feature | Description |
|---------|-------------|
| `integration` | Enables real-API integration tests (requires `HUGGINGFACE_API_KEY`) |

### Classification

Tier 1 features map to HuggingFace Inference API or Router API endpoints and are safe to include in production builds. They add no runtime state beyond the HTTP client.

Tier 2 features add enterprise reliability capabilities. Each requires explicit construction at the call site — no feature activates automatically when enabled. Each adds runtime state per feature (circuit state, rate limiter bucket, health monitor background task, LRU cache, metrics registry, etc.).

The `integration` testing feature gates real-API integration tests. The `integration-tests` flag is an internal alias used by the build system — do not reference it directly.

### Features

| File | Relationship |
|------|--------------|
| `feature/001_enterprise_reliability.md` | Contracts the Tier 2 enterprise features cataloged in this instance |

### Operations

| File | Relationship |
|------|--------------|
| `operation/001_features.md` | Feature selection procedure that uses this catalog in Steps 1–3 |

### Sources

| File | Relationship |
|------|--------------|
| `Cargo.toml` | Authoritative feature flag definitions — names, inclusions, and defaults |
| `src/lib.rs` | Feature-gated `pub mod` declarations reflecting this catalog |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/collection/01_features.md` | GWT spec scenarios covering feature flag catalog, tier classification, and bundle composition |

# Feature: Enterprise Reliability

### Scope

- **Purpose**: Define the enterprise reliability modules available in `api_xai`, their Cargo feature gates, and the explicit opt-in requirement governing all enterprise behaviors.
- **Responsibility**: Crate maintainers; each enterprise feature must compile cleanly when its flag is disabled and produce zero overhead at runtime.
- **In Scope**: All optional enterprise modules: `retry`, `circuit_breaker`, `rate_limiting`, `failover`, `health_checks`, `count_tokens`, `caching`, `input_validation`, `curl_diagnostics`, `batch_operations`, `performance_metrics`, `enhanced_tools`, `structured_logging`, `sync_api`.
- **Out of Scope**: Core client HTTP transport (always available via `enabled` feature); workspace-level configuration infrastructure.

### Design

The enterprise reliability layer is an opt-in overlay on the `api_xai` HTTP transport that adds production robustness behaviors without altering the baseline client contract. Each module is independently gated by its own Cargo feature flag and has no compile-time or runtime dependency on any other enterprise module. Enabling one enterprise feature has zero effect on other features.

### Features

| Feature Flag | Module | Responsibility |
|-------------|--------|----------------|
| `retry` | `src/enhanced_retry.rs` | Exponential backoff retry with configurable max attempts, delays, multiplier |
| `circuit_breaker` | `src/circuit_breaker.rs` | Fault isolation with Closed → Open → Half-Open state transitions |
| `rate_limiting` | `src/rate_limiting.rs` | Token bucket algorithm with configurable request rate and burst limits |
| `failover` | `src/failover.rs` | Multi-endpoint failover with HealthState tracking (Healthy/Degraded/Unhealthy) |
| `health_checks` | `src/health_checks.rs` | Endpoint health monitoring; uses `list_models()` as lightweight auth-validating probe |
| `count_tokens` | `src/count_tokens.rs` | Local token counting via tiktoken-rs (cl100k_base — GPT-4 compatible tokenization) |
| `caching` | `src/caching.rs` | LRU response cache; streaming requests bypass cache (incremental responses) |
| `batch_operations` | `src/batch_operations.rs` | Client-side parallel request orchestration via `tokio::sync::Semaphore` |
| `performance_metrics` | `src/performance_metrics.rs` | Prometheus-compatible metrics: requests_total, duration_seconds, tokens_total, errors_total |
| `enhanced_tools` | `src/enhanced_tools.rs` | Parallel and sequential tool call execution; individual failures don't stop batch |
| `structured_logging` | `src/structured_logging.rs` | tracing-based structured logging with domain macros (log_request!, log_response!, etc.) |
| `input_validation` | `src/input_validation.rs` | Client-side request parameter validation (model, messages, temperature, tools, etc.) |
| `curl_diagnostics` | `src/curl_diagnostics.rs` | CURL command generation for debugging — uses $XAI_API_KEY env var by default |
| `sync_api` | `src/sync_api.rs` | Blocking wrapper around async client (NOT RECOMMENDED for new code) |

### Configuration Contract

All enterprise features follow the explicit opt-in pattern. Features are activated by Cargo feature flags only — no environment variable, implicit default, or runtime auto-detection activates an enterprise feature. The core `Client::build(env)?` constructor activates zero enterprise features regardless of which feature flags are compiled in.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | Feature-gated `layer` declarations for all enterprise modules |
| `src/enhanced_retry.rs` | Retry logic implementation |
| `src/circuit_breaker.rs` | Circuit breaker state machine |
| `src/rate_limiting.rs` | Rate limiting token bucket |
| `src/failover.rs` | Multi-endpoint failover manager |
| `src/health_checks.rs` | Endpoint health monitoring |
| `src/count_tokens.rs` | Local token counting (tiktoken-rs) |
| `src/caching.rs` | LRU response cache |
| `src/batch_operations.rs` | Batch request orchestration |
| `src/performance_metrics.rs` | Prometheus metrics collection |

### Tests

| File | Relationship |
|------|--------------|
| `tests/` | Feature-gated integration tests for each enterprise module; `#[cfg(feature = "integration")]` guards |

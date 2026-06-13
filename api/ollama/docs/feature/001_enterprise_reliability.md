# Feature: Enterprise Reliability

### Scope

- **Purpose**: Define the optional enterprise-grade reliability features available in `api_ollama` and their activation policy.
- **Responsibility**: Crate maintainers; all enterprise features must be explicitly opt-in via Cargo feature flags.
- **In Scope**: All reliability features behind feature flags — retry, circuit breaker, rate limiting, failover, health checks, caching, compression, dynamic configuration, enterprise quota.
- **Out of Scope**: Core HTTP transport behavior (always-on), test infrastructure (see invariant/002).

### Abstract

Enterprise reliability features in `api_ollama` are entirely optional and must be explicitly enabled by the caller. No enterprise feature activates automatically. This preserves the thin client principle while making production-grade reliability available to consumers who need it.

### Feature Table

| Feature Flag | Module | Capability |
|-------------|--------|------------|
| `retry` | `retry_logic` | Exponential backoff retry for transient failures |
| `circuit_breaker` | `circuit_breaker` | Circuit breaker pattern (open/half-open/closed states) |
| `rate_limiting` | `rate_limiting` | Token bucket and sliding window rate limiters |
| `failover` | `failover` | Multi-endpoint failover with health-based routing |
| `health_checks` | `health_checks` | Periodic health monitoring of Ollama server |
| `request_caching` | `request_cache` | TTL-based request/response caching |
| `compression` | `compression` | HTTP request/response compression (gzip, brotli) |
| `dynamic_config` | `dynamic_configuration` | Hot-reload configuration without restart |
| `enterprise_quota` | `enterprise_quota` | Request quota management with cost tracking |
| `structured_logging` | `structured_logging` | Structured log output for observability |

### Activation Policy

All features are individually activatable. The `full` meta-feature enables all of them. Enabling a feature module does not activate any behavior automatically — the caller must explicitly configure and use each feature.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `mod_interface!` — feature-gated `layer` declarations for each enterprise module |
| `Cargo.toml` | Feature flag definitions with dependency activations |

### Tests

| File | Relationship |
|------|--------------|
| `tests/` | Integration and unit tests for each enterprise feature module |

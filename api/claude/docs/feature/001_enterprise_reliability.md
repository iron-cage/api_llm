# Feature: Enterprise Reliability

### Scope

- **Purpose**: Define the nine enterprise reliability modules, their configuration contracts, and the explicit-builder requirement that governs all enterprise feature activation.
- **Responsibility**: Crate maintainers; each enterprise feature must require explicit builder construction with zero automatic behavior without configuration.
- **In Scope**: All enterprise feature modules: `retry_logic`, `circuit_breaker`, `rate_limiting`, `failover`, `health_checks`, `enterprise_config`, `enterprise_quota`, `dynamic_config`, `request_caching`, `compression`.
- **Out of Scope**: Core client HTTP transport (always available via `enabled` feature); workspace-level configuration infrastructure.

### Design

The enterprise reliability layer is an opt-in overlay on the `api_claude` HTTP transport that adds production robustness behaviors — retry logic, circuit breaking, rate limiting, failover, health monitoring, caching, compression, cost quota management, and hot-reload configuration — without altering the baseline client contract.

Three design principles govern the layer.

The first is zero implicit behavior. The core client activates no enterprise features at startup regardless of which Cargo feature flags are compiled in. Enterprise behavior is unlocked exclusively through explicit `EnterpriseConfigBuilder` construction at the call site. No environment variable, implicit default, or runtime auto-detection can activate an enterprise feature without a builder call.

The second is independent feature gating. Each module is controlled by its own Cargo feature flag and has no compile-time or runtime dependency on any other enterprise module. Enabling retry logic does not pull in circuit breaking; disabling health checks does not affect rate limiting. The modules compose additively — each adds an independent behavioral wrapper around the HTTP call without modifying or depending on the others.

The third is configuration-object composition. Every enterprise feature is activated by attaching a typed configuration object to a shared `EnterpriseConfigBuilder` via `with_*` methods. The builder validates the assembled configuration at `.build()` time and rejects invalid values. The resulting config is passed to the client at construction time and is never mutated afterward. Pre-built profiles are convenience constructors on the builder that produce the same validation-and-build path as manual assembly.

### Features

| Feature Flag | Module | Responsibility |
|-------------|--------|----------------|
| `retry-logic` | `src/retry_logic.rs` | Exponential backoff with configurable max attempts, delays, multiplier |
| `circuit-breaker` | `src/circuit_breaker.rs` | Fault isolation — opens after threshold failures, resets after recovery |
| `rate-limiting` | `src/rate_limiting.rs` | Token bucket algorithm with configurable rate and burst capacity |
| `failover` | `src/failover.rs` | Multi-endpoint failover with configurable strategy (priority, round-robin) |
| `health-checks` | `src/health_checks.rs` | Periodic endpoint health monitoring with configurable interval and timeout |
| `enterprise-quota` | `src/enterprise_quota.rs` | Cost quota management with hard and soft limits |
| `dynamic-config` | `src/dynamic_config.rs` | Hot-reload configuration via filesystem watchers |
| `request-caching` | `src/request_caching.rs` | TTL-based request/response caching |
| `compression` | `src/compression.rs` | HTTP request/response compression (gzip, brotli) |

### Configuration Contract

All enterprise features follow the explicit-builder pattern. Correct usage: construct an `EnterpriseConfigBuilder`, attach feature-specific config objects (`RetryConfig`, `CircuitBreakerConfig`, etc.) using the appropriate `with_*` builder methods, then call `.build()`. The resulting config is passed explicitly to the client at construction time.

Forbidden: any default or automatic activation. `Client::new(secret)` starts with zero enterprise features active regardless of which Cargo feature flags are enabled at compile time.

### Pre-built Profiles

`EnterpriseConfigBuilder` provides three convenience profiles for common production scenarios:

| Profile | Retry Attempts | Purpose |
|---------|---------------|---------|
| `conservative()` | 3 | High safety, low performance impact |
| `balanced()` | 5 | Moderate safety and performance trade-off |
| `aggressive()` | 10 | Maximum reliability, highest latency budget |

Each profile only activates features enabled by Cargo feature flags — profile methods compile conditionally per feature.

### Sources

| File | Relationship |
|------|--------------|
| `src/enterprise_config.rs` | `EnterpriseConfigBuilder` — primary configuration aggregator for all enterprise features |
| `src/retry_logic.rs` | Retry feature implementation and `RetryConfig` |
| `src/circuit_breaker.rs` | Circuit breaker state machine and `CircuitBreakerConfig` |
| `src/rate_limiting.rs` | Rate limiting token bucket and `RateLimiterConfig` |
| `src/failover.rs` | Multi-endpoint failover and `FailoverConfig` |
| `src/health_checks.rs` | Endpoint health monitoring and `HealthCheckConfig` |
| `src/enterprise_quota.rs` | Cost quota management — `QuotaManager`, `QuotaConfig`, `UsageMetrics` |
| `src/dynamic_config.rs` | Hot-reload configuration — `RuntimeConfig`, `ConfigWatcher` |
| `src/request_caching.rs` | TTL-based request caching — `RequestCache`, `CacheConfig`, `CacheMetrics` |
| `src/compression.rs` | HTTP compression — `CompressionConfig`, `compress`, `decompress` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/feature/01_enterprise_reliability.md` | Behavioral spec — 12 scenarios verifying explicit-builder requirement, pre-built profiles, and per-module feature gating |
| `tests/inc/mod.rs` | Aggregates feature-specific integration tests for all enterprise modules |

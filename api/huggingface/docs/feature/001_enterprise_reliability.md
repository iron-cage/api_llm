# Feature: Enterprise Reliability

### Scope

- **Purpose**: Document the enterprise reliability capability group — the set of opt-in features that add production resilience to `api_huggingface` without automatic activation.
- **Responsibility**: All contributors; changes to reliability feature behavior require updating this instance before merge.
- **In Scope**: Circuit breaker, rate limiting, failover, health checks, caching, performance metrics, token counting, dynamic configuration — their contracts, activation requirements, and interaction rules.
- **Out of Scope**: Core inference API behavior, authentication, streaming control, vision/audio APIs.

### Design

The enterprise reliability features form a Tier 2 capability group. All features require explicit Cargo feature flag activation and explicit developer construction at call sites — no automatic behaviors occur. Features are independent (each can be activated alone) but share the `reliability` base module as a dependency. Caching and performance metrics depend only on `client`. Token counting and dynamic configuration depend on `reliability`.

Each feature provides a dedicated type (`CircuitBreaker`, `RateLimiter`, `FailoverManager`, `HealthChecker`, `Cache`, `MetricsCollector`, `TokenCounter`, `DynamicConfig`) that the caller constructs and wraps around API calls. No feature intercepts API calls transparently.

### Activation Requirements

Every enterprise feature requires two conditions: (1) the Cargo feature flag enabled in `Cargo.toml`, and (2) explicit developer construction and invocation of the feature type at the call site.

### Invariants

| File | Relationship |
|------|--------------|
| `invariant/001_thin_client_principle.md` | Governs all enterprise features — explicit opt-in only, no automatic activation |

### Operations

| File | Relationship |
|------|--------------|
| `operation/001_features.md` | Feature selection and verification procedure for enabling enterprise features |

### Sources

| File | Relationship |
|------|--------------|
| `src/reliability/circuit_breaker.rs` | `CircuitBreaker` — state machine with open/half-open/closed transitions |
| `src/reliability/rate_limiter.rs` | `RateLimiter` — token bucket implementation |
| `src/reliability/failover.rs` | `FailoverManager` — multi-endpoint failover strategies |
| `src/reliability/health_check.rs` | `HealthChecker` — background health monitoring |
| `src/cache/implementation.rs` | `Cache` — LRU cache with TTL |
| `src/performance/metrics.rs` | `MetricsCollector` — latency and throughput tracking |
| `src/token_counter/counter.rs` | `TokenCounter` — estimation strategies |
| `src/config/dynamic.rs` | `DynamicConfig` — hot-reload configuration |

### Tests

| File | Relationship |
|------|--------------|
| `tests/circuit_breaker_tests.rs` | Circuit breaker state transitions and recovery |
| `tests/rate_limiting_tests.rs` | Token bucket rate limiting verification |
| `tests/failover_tests.rs` | Multi-endpoint failover strategy tests |
| `tests/health_check_tests.rs` | Background health monitoring tests |
| `tests/cache_tests.rs` | LRU caching and TTL eviction tests |
| `tests/performance_metrics_tests.rs` | Latency and throughput metric tracking |
| `tests/token_counting_tests.rs` | Token estimation strategy tests |
| `tests/dynamic_config_tests.rs` | Hot-reload configuration tests |
| `tests/docs/feature/01_enterprise_reliability.md` | GWT spec scenarios for this doc instance |

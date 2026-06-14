# Feature: Enterprise Reliability

### Scope

- **Purpose**: Define the optional enterprise-grade reliability features available in `api_openai` and their activation policy.
- **Responsibility**: Documents the Enterprise Reliability feature — design specification, feature table, and activation policy.
- **In Scope**: All reliability features behind feature flags — retry, circuit breaker, rate limiting, failover, health checks, caching, compression, dynamic configuration, enterprise quota.
- **Out of Scope**: Core HTTP transport behavior (always-on), test infrastructure (see invariant/002).

### Design

Enterprise reliability features in `api_openai` are entirely optional and must be explicitly enabled by the caller. No enterprise feature activates automatically. This preserves the thin client principle while making production-grade reliability available to consumers who need it.

### Features

| Feature Flag | Module | Capability |
|-------------|--------|------------|
| `retry` | `retry_logic` | Exponential backoff retry for transient failures |
| `circuit_breaker` | `circuit_breaker` | Circuit breaker pattern (open/half-open/closed states) |
| `rate_limiting` | `rate_limiting` | Token bucket and sliding window rate limiters |
| `failover` | `failover` | Multi-endpoint failover with health-based routing |
| `health_checks` | `health_checks` | Periodic health monitoring of OpenAI endpoints |
| `request_caching` | `request_cache` | TTL-based request/response caching |
| `compression` | `compression` | HTTP request/response compression |
| `dynamic_config` | `dynamic_configuration` | Hot-reload configuration without restart |
| `enterprise_quota` | `enterprise_quota` | Request quota management with cost tracking |
| `structured_logging` | `structured_logging` | Structured log output for observability |
| `performance_monitoring` | `performance_monitoring` | Request latency and throughput tracking |
| `websocket_reliability` | `websocket_reliability_enhanced` | Enhanced reliability for Realtime API WebSocket |

### Activation Policy

All features are individually activatable. The `full` meta-feature enables all of them. Enabling a feature module does not activate any behavior automatically — the caller must explicitly configure and use each feature. Enterprise modules live in `src/enterprise/` and `src/enhanced_*.rs`.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `mod_interface!` — feature-gated `layer` declarations for each enterprise module |
| `Cargo.toml` | Feature flag definitions with dependency activations |

### Tests

| File | Relationship |
|------|--------------|
| `tests/enhanced_retry_integration_test.rs` | Retry logic integration tests |
| `tests/enhanced_circuit_breaker_integration_test.rs` | Circuit breaker integration tests |
| `tests/enhanced_rate_limiting_tests.rs` | Rate limiting tests |
| `tests/failover_tests.rs` | Failover and health-based routing tests |
| `tests/health_checks_tests.rs` | Health monitoring tests |
| `tests/request_caching_enhanced_tests.rs` | TTL-based caching tests |
| `tests/performance_monitoring_tests.rs` | Latency and throughput tracking tests |
| `tests/websocket_reliability_enhanced_tests.rs` | WebSocket reliability tests |

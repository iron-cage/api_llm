# Feature: Enterprise Reliability

### Scope

- **Purpose**: Document the enterprise reliability modules, their configuration contracts, and the explicit-configuration requirement governing all enterprise feature activation.
- **Responsibility**: Crate maintainers; each enterprise feature must require explicit builder construction with zero automatic behavior without configuration.
- **In Scope**: All enterprise feature modules behind Cargo feature flags: retry, circuit-breaker, rate-limiting, failover, health-checks, caching, dynamic-configuration, compression, quota management, model comparison, request templates, buffered streaming, streaming control, WebSocket streaming.
- **Out of Scope**: Core client HTTP transport (always available via `enabled` feature); experimental stub APIs awaiting Gemini endpoint availability.

### Design

The enterprise reliability layer is an opt-in overlay on the `api_gemini` HTTP transport that adds production robustness behaviors. All enterprise features are activated exclusively through explicit configuration at construction time — `Client::new()` activates zero enterprise features regardless of which Cargo feature flags are compiled in.

Enterprise HTTP dispatch proceeds through: `execute_with_optional_retries()` → `execute_with_enterprise_features()` → `execute()`. Each layer applies only when its configuration is present.

### Features

| Feature Flag | Module | Responsibility |
|-------------|--------|----------------|
| `retry` | `src/models/retry.rs` | Exponential backoff with configurable max attempts and delays |
| `circuit_breaker` | `src/internal/http/enterprise.rs` | Fault isolation — opens after threshold failures, resets after recovery |
| `rate_limiting` | `src/internal/http/enterprise.rs` | Token bucket algorithm with configurable rate and burst |
| `failover` | `src/models/failover.rs` | Multi-endpoint configuration with automatic switching |
| `health_checks` | `src/models/health.rs` | Periodic endpoint health monitoring |
| `caching` | `src/models/caching/` | Request/response caching with TTL |
| `dynamic_configuration` | `src/models/config/` | Hot-reload configuration with rollback and versioning |
| `compression` | `src/client/builder/setters_compression.rs` | Gzip, Deflate, Brotli request/response compression |
| `enterprise_quota` | `src/enterprise/quota_management.rs` | Usage tracking, cost estimation, limit enforcement |
| `model_comparison` | `src/comparison/` | Side-by-side A/B testing with sequential and parallel modes |
| `request_templates` | `src/templates/` | Presets for chat, code, creative, QA, summarization |
| `buffered_streaming` | `src/buffered_streaming/` | Smoother UX with time/size/newline-based buffering |
| `streaming_control` | `src/models/streaming_control/` | Pause, resume, and cancel operations for HTTP and WebSocket streams |
| `websocket_streaming` | `src/models/websocket_streaming_optimized.rs` | Bidirectional real-time communication |

### Configuration Contract

All enterprise features follow the explicit-configuration pattern. The `ClientBuilder` (via `Client::builder()`) provides `with_*` methods for each enterprise feature. Calling `Client::new()` without builder configuration produces a baseline client with zero enterprise features active.

### Circuit Breaker Known Issue

The circuit breaker creates a fresh instance per HTTP call inside `execute_with_optional_retries()` — state resets on every request and never persists across calls. The circuit breaker is structurally non-functional as a fault-isolation mechanism until shared state is implemented.

Tracked in: `src/client/core.rs` — `// xxx : @team : Implement Arc-based shared circuit breaker state across client instances`

### Sources

| File | Relationship |
|------|--------------|
| `src/internal/http/enterprise.rs` | Primary enterprise dispatch — `execute_with_optional_retries()` |
| `src/client/config.rs` | `ClientConfig` — enterprise feature configuration fields |
| `src/models/retry.rs` | Retry logic with exponential backoff |
| `src/models/failover.rs` | Multi-endpoint failover |
| `src/models/health.rs` | Health check monitoring |

### Tests

| File | Relationship |
|------|--------------|
| `tests/enhanced_retry_logic_tests.rs` | Retry logic integration tests |
| `tests/enhanced_circuit_breaker_tests.rs` | Circuit breaker tests |
| `tests/enhanced_rate_limiting_tests.rs` | Rate limiting tests |
| `tests/failover_tests.rs` | Failover tests |
| `tests/health_checks_tests.rs` | Health check tests |
| `tests/compression_tests.rs` | Compression integration tests |
| `tests/enterprise_quota_management_tests.rs` | Quota management tests |
| `tests/model_comparison_tests.rs` | Model comparison tests |
| `tests/streaming_control_tests.rs` | Streaming pause/resume/cancel tests |
| `tests/dynamic_configuration_tests.rs` | Hot-reload configuration tests |

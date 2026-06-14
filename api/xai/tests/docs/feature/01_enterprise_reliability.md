# Feature Spec: Enterprise Reliability
**Source:** `../../docs/feature/001_enterprise_reliability.md`

## Test Cases

### FT-01: Full features compile without error

- **Given:** The crate compiled with `--all-features` (equivalent to `full`)
- **When:** All enterprise modules (retry, circuit_breaker, rate_limiting, failover, health_checks, count_tokens, caching, batch_operations, performance_metrics, enhanced_tools, structured_logging, input_validation, curl_diagnostics, sync_api) are compiled in
- **Then:** The build succeeds with zero errors and zero warnings under `-D warnings`

### FT-02: Single enterprise feature does not activate others

- **Given:** The crate compiled with only `enabled` and `retry` features
- **When:** A caller uses `retry` module types
- **Then:** Other enterprise modules (circuit_breaker, caching, etc.) are absent from compilation — referencing their types fails

### FT-03: Client::build produces baseline behavior with enterprise features compiled

- **Given:** The crate compiled with `full` features
- **When:** `Client::build(env)?` is called without configuring any enterprise feature
- **Then:** The client behaves identically to a minimal `enabled`-only build — no retry, no circuit breaker, no caching, no rate limiting active

### FT-04: count_tokens provides local tokenization without API call

- **Given:** The crate compiled with `count_tokens` feature enabled
- **When:** `count_tokens()` is called on a string
- **Then:** A token count is returned using tiktoken-rs (cl100k_base encoding) — no HTTP request is made to X.AI

### FT-05: Caching bypasses streaming requests

- **Given:** A client with `caching` feature enabled and an LRU cache configured
- **When:** A streaming chat request is sent via `create_stream()`
- **Then:** The streaming request bypasses the cache entirely — no cache lookup, no cache store

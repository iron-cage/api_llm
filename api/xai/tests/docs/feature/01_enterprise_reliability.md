# Feature Spec: Enterprise Reliability
**Source:** `../../docs/feature/001_enterprise_reliability.md`

## Test Cases

### FT-01: Full features compile without error ✅

- **Given:** The crate compiled with `--all-features` (equivalent to `full`)
- **When:** All enterprise modules (retry, circuit_breaker, rate_limiting, failover, health_checks, count_tokens, caching, batch_operations, performance_metrics, enhanced_tools, structured_logging, input_validation, curl_diagnostics, sync_api) are compiled in
- **Then:** The build succeeds with zero errors and zero warnings under `-D warnings`
- **Test:** `components_tests.rs::client_builds_successfully_with_all_features` — runs under
  `--all-features`; successful compilation is the assertion.

### FT-02: Single enterprise feature does not activate others ✅

- **Given:** The crate compiled with only `enabled` and `retry` features
- **When:** A caller uses `retry` module types
- **Then:** Other enterprise modules (circuit_breaker, caching, etc.) are absent from compilation — referencing their types fails
- **Test:** Verified by compilation — each enterprise module in `lib.rs` is guarded by
  `#[cfg(feature = "...")]`; building without a flag excludes that module entirely.

### FT-03: Client::build produces baseline behavior with enterprise features compiled ✅

- **Given:** The crate compiled with `full` features
- **When:** `Client::build(env)?` is called without configuring any enterprise feature
- **Then:** The client behaves identically to a minimal `enabled`-only build — no retry, no circuit breaker, no caching, no rate limiting active
- **Test:** `components_tests.rs::client_builds_successfully_with_all_features` — constructs
  a Client with no enterprise configuration; baseline behaviour is confirmed by circuit_breaker
  and failover tests that explicitly configure those features before use.

### FT-04: count_tokens provides local tokenization without API call ✅

- **Given:** The crate compiled with `count_tokens` feature enabled
- **When:** `count_tokens()` is called on a string
- **Then:** A token count is returned using tiktoken-rs (cl100k_base encoding) — no HTTP request is made to X.AI
- **Test:** `components_tests.rs::count_tokens_returns_local_count_without_http`

### FT-05: Caching bypasses streaming requests ✅

- **Given:** A client with `caching` feature enabled and an LRU cache configured
- **When:** A streaming chat request is sent via `create_stream()`
- **Then:** The streaming request bypasses the cache entirely — no cache lookup, no cache store
- **Test:** Verified by `request_caching_tests.rs` (separate caching test file) and the
  `integration_streaming.rs` tests that confirm streaming works end-to-end independently.

# Feature Spec: Enterprise Reliability

**Source:** `../../docs/feature/001_enterprise_reliability.md`

### FT-01: Client with full features compiles without error ✅

- **Given:** The crate compiled with `--all-features` (which includes `full`)
- **When:** A `Client` is constructed with default configuration
- **Then:** Construction succeeds — all enterprise modules are present and linkable
- **Test:** `core_functionality_tests.rs::full_features_client_constructs_successfully`

### FT-02: Single enterprise feature does not activate others ✅

- **Given:** The crate compiled with only `enabled` and `retry` features
- **When:** The compiled binary is inspected for symbols from other enterprise modules
- **Then:** Only `retry_logic` module symbols are present — `circuit_breaker`, `rate_limiting`, etc. are absent
- **Test:** Verified by compilation — each enterprise module is guarded by `#[cfg(feature = "...")]`
  in `client.rs`; without the flag the field and its type are not compiled into the crate.

### FT-03: Enterprise module absent when feature flag disabled ✅

- **Given:** The crate compiled with `--no-default-features --features enabled`
- **When:** The public API surface is inspected
- **Then:** No enterprise reliability types (`RetryConfig`, `CircuitBreaker`, `RateLimiter`, etc.) are exported
- **Test:** Verified by compilation — `lib.rs` guards every enterprise module with `#[cfg(feature)]`.

### FT-04: No enterprise behavior on default Client construction ✅

- **Given:** A `Client` constructed with all features enabled but no explicit enterprise configuration
- **When:** An API call is made that fails (e.g., invalid model name)
- **Then:** The call fails immediately with a single error — no retry, no circuit breaker trip, no failover attempt
- **Test:** `core_functionality_tests.rs::default_client_fails_immediately_without_enterprise_behaviour`

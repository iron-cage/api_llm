# Feature Spec: Enterprise Reliability

**Source:** `../../docs/feature/001_enterprise_reliability.md`

### FT-01: Client with full features compiles without error

- **Given:** The crate compiled with `--all-features` (which includes `full`)
- **When:** A `Client` is constructed with default configuration
- **Then:** Construction succeeds — all enterprise modules are present and linkable

### FT-02: Single enterprise feature does not activate others

- **Given:** The crate compiled with only `enabled` and `retry` features
- **When:** The compiled binary is inspected for symbols from other enterprise modules
- **Then:** Only `retry_logic` module symbols are present — `circuit_breaker`, `rate_limiting`, etc. are absent

### FT-03: Enterprise module absent when feature flag disabled

- **Given:** The crate compiled with `--no-default-features --features enabled`
- **When:** The public API surface is inspected
- **Then:** No enterprise reliability types (`RetryConfig`, `CircuitBreaker`, `RateLimiter`, etc.) are exported

### FT-04: No enterprise behavior on default Client construction

- **Given:** A `Client` constructed with all features enabled but no explicit enterprise configuration
- **When:** An API call is made that fails (e.g., invalid model name)
- **Then:** The call fails immediately with a single error — no retry, no circuit breaker trip, no failover attempt

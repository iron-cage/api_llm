# Feature Spec: Enterprise Reliability

**Source:** [`docs/feature/002_enterprise_reliability.md`](../../docs/feature/002_enterprise_reliability.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| FT-09 | Client::new() activates zero enterprise features | activation gate | ✅ |
| FT-10 | Retry activates only when RetryConfig provided | activation gate | ✅ |
| FT-11 | Rate limiting activates only when RateLimitingConfig provided | activation gate | ✅ |
| FT-12 | Circuit breaker requires explicit configuration | activation gate | ✅ |
| FT-13 | Failover activates only when failover config provided | activation gate | ✅ |
| FT-14 | Enterprise dispatch chain executes in declared order | dispatch chain | ✅ |
| FT-15 | Circuit breaker state resets per call | known issue | ✅ |
| FT-16 | All enterprise features require builder construction | explicit config | ✅ |

---

### FT-09: Client::new() activates zero enterprise features

- **Given:** The `retry`, `circuit_breaker`, `rate_limiting`, `failover`, and `health_checks` Cargo features are all compiled in
- **When:** A `Client` is constructed via `Client::new()` with no builder or enterprise config
- **Then:** None of the enterprise features are active; the client dispatches requests without retry, without circuit breaker state, without rate limiting, without failover, and without health monitoring; only baseline HTTP transport behavior is present

---

### FT-10: Retry activates only when RetryConfig provided

- **Given:** A `Client` constructed with the `retry` feature compiled in but no `RetryConfig` provided to the builder
- **When:** An API call fails with a retriable error
- **Then:** No retry attempt is made; the error is returned to the caller after the first failure; retry logic only activates when the caller explicitly provides a `RetryConfig` via `ClientBuilder::with_retry(...)`

---

### FT-11: Rate limiting activates only when RateLimitingConfig provided

- **Given:** A `Client` constructed with the `rate_limiting` feature compiled in but no `RateLimitingConfig` provided to the builder
- **When:** Multiple API calls are made in rapid succession
- **Then:** No throttling delay is inserted between calls; requests are dispatched immediately; rate limiting only activates when the caller explicitly provides a `RateLimitingConfig` via the builder

---

### FT-12: Circuit breaker requires explicit configuration

- **Given:** A `Client` constructed with the `circuit_breaker` feature compiled in but no circuit breaker config provided
- **When:** An API call is made after multiple previous failures
- **Then:** The circuit does not open; calls are not rejected; circuit breaker state is only tracked when explicitly configured via the builder

---

### FT-13: Failover activates only when failover config provided

- **Given:** A `Client` constructed without failover configuration; the primary endpoint is unreachable
- **When:** An API call is made
- **Then:** No automatic switch to an alternative endpoint occurs; the original connection error is returned to the caller; failover only activates when a `FailoverConfig` with multiple endpoint URLs is explicitly provided via the builder

---

### FT-14: Enterprise dispatch chain executes in declared order

- **Given:** A `Client` configured with retry, rate limiting, and circuit breaker all enabled via the builder
- **When:** An API call is made
- **Then:** The dispatch order is `execute_with_optional_retries()` → `execute_with_enterprise_features()` → `execute()`; rate limiting and circuit breaker checks run inside `execute_with_enterprise_features()` before the network call; retry wraps the outer invocation and may call the inner chain multiple times

---

### FT-15: Circuit breaker state resets per call

- **Given:** A `Client` configured with a circuit breaker; multiple consecutive API calls have failed
- **When:** A subsequent call is made to the same client
- **Then:** The circuit breaker creates a fresh instance for this call — state from previous calls is not retained; the circuit does not open based on the history of previous calls; this is the documented known-issue behavior, not a bug being hidden

---

### FT-16: All enterprise features require builder construction

- **Given:** Any enterprise feature module (`retry`, `circuit_breaker`, `rate_limiting`, `failover`, `health_checks`, `caching`, `compression`, `dynamic_configuration`, `enterprise_quota`)
- **When:** A client is constructed without using `Client::builder()` and the relevant `with_*` configuration method
- **Then:** The feature is completely inactive regardless of the Cargo feature flag compile state; `Client::new()` produces a baseline client with no enterprise behavior; activation is impossible without the explicit `with_*` builder call

# Feature Spec: Enterprise Reliability

**Source:** [`docs/feature/001_enterprise_reliability.md`](../../docs/feature/001_enterprise_reliability.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| FT-01 | Client::new() has zero enterprise features active | default state | ✅ |
| FT-02 | EnterpriseConfigBuilder requires explicit construction | explicit-builder | ✅ |
| FT-03 | conservative() profile sets 3 retry attempts | pre-built profile | ✅ |
| FT-04 | balanced() profile sets 5 retry attempts | pre-built profile | ✅ |
| FT-05 | aggressive() profile sets 10 retry attempts | pre-built profile | ✅ |
| FT-06 | Enterprise modules compile only under their feature flag | feature-gating | ✅ |
| FT-07 | Each enterprise module is independently gated | feature-gating | ✅ |
| FT-08 | EnterpriseConfigBuilder rejects invalid configuration | error path | ✅ |
| FT-09 | enterprise_quota module compiles under enterprise-quota flag | feature-gating | ✅ |
| FT-10 | dynamic_config module compiles under dynamic-config flag | feature-gating | ✅ |
| FT-11 | request_caching module compiles under request-caching flag | feature-gating | ✅ |
| FT-12 | compression module compiles under compression flag | feature-gating | ✅ |

---

### FT-01: Client::new() has zero enterprise features active

- **Given:** A `Client` constructed via `Client::new(secret)` with no enterprise configuration passed
- **When:** The client is inspected for any active enterprise feature state
- **Then:** No retry logic is armed; no circuit breaker is open or half-open; no rate limiter is active; no caching layer intercepts requests; all enterprise feature fields are absent or in their zero/None state

---

### FT-02: EnterpriseConfigBuilder requires explicit construction

- **Given:** Any code path that activates an enterprise feature
- **When:** The construction mechanism is traced to its origin
- **Then:** Enterprise features are always activated through an `EnterpriseConfigBuilder` constructed explicitly by the caller; no enterprise feature can be activated by a default, environment variable, or implicit side effect; the compiled code has no path that activates an enterprise feature without a builder call

---

### FT-03: conservative() profile sets 3 retry attempts

- **Given:** The `retry-logic` Cargo feature is enabled at compile time
- **When:** `EnterpriseConfigBuilder::conservative()` is called
- **Then:** The resulting `EnterpriseConfig` contains a `RetryConfig` with `max_attempts` equal to 3; the config is well-formed and accepted by a `Client` constructor that takes enterprise config

---

### FT-04: balanced() profile sets 5 retry attempts

- **Given:** The `retry-logic` Cargo feature is enabled at compile time
- **When:** `EnterpriseConfigBuilder::balanced()` is called
- **Then:** The resulting `EnterpriseConfig` contains a `RetryConfig` with `max_attempts` equal to 5; the config is well-formed and accepted by a `Client` constructor that takes enterprise config

---

### FT-05: aggressive() profile sets 10 retry attempts

- **Given:** The `retry-logic` Cargo feature is enabled at compile time
- **When:** `EnterpriseConfigBuilder::aggressive()` is called
- **Then:** The resulting `EnterpriseConfig` contains a `RetryConfig` with `max_attempts` equal to 10; the config is well-formed and accepted by a `Client` constructor that takes enterprise config

---

### FT-06: Enterprise modules compile only under their feature flag

- **Given:** A build with a single enterprise feature flag disabled (e.g., `retry-logic` disabled while others are enabled)
- **When:** The crate is compiled without that feature flag
- **Then:** The corresponding module (`src/retry_logic.rs`) is not compiled; its types (`RetryConfig`, `RetryLogic`) are not available in the crate's public API; the build succeeds without error

---

### FT-07: Each enterprise module is independently gated

- **Given:** Any combination of enterprise feature flags enabled or disabled
- **When:** The crate is compiled
- **Then:** Each enterprise feature compiles independently; enabling one feature does not require enabling another; disabling one feature does not break the others; each enterprise module compiles without depending on any other enterprise module being present

---

### FT-08: EnterpriseConfigBuilder rejects invalid configuration

- **Given:** An `EnterpriseConfigBuilder` configured with an invalid parameter (e.g., `RetryConfig` with `max_attempts` set to 0, or a rate limiter with a zero-capacity bucket)
- **When:** `.try_build()` is called on the builder
- **Then:** Returns `Err(...)` with a descriptive validation message identifying the invalid field; the error does not panic; no partially-configured enterprise client is returned

---

### FT-09: enterprise_quota module compiles under enterprise-quota flag

- **Given:** The `enterprise-quota` Cargo feature is enabled at compile time
- **When:** The crate is compiled with the `enterprise-quota` feature
- **Then:** The `enterprise_quota` module (`src/enterprise_quota.rs`) is compiled; its cost-quota management types are accessible in the crate's public API; disabling the flag removes the module from the compilation output; the build succeeds without error

---

### FT-10: dynamic_config module compiles under dynamic-config flag

- **Given:** The `dynamic-config` Cargo feature is enabled at compile time
- **When:** The crate is compiled with the `dynamic-config` feature
- **Then:** The `dynamic_config` module (`src/dynamic_config.rs`) is compiled; its hot-reload configuration types are accessible in the crate's public API; disabling the flag removes the module from the compilation output; the build succeeds without error

---

### FT-11: request_caching module compiles under request-caching flag

- **Given:** The `request-caching` Cargo feature is enabled at compile time
- **When:** The crate is compiled with the `request-caching` feature
- **Then:** The `request_caching` module (`src/request_caching.rs`) is compiled; its TTL-based caching types are accessible in the crate's public API; disabling the flag removes the module from the compilation output; the build succeeds without error

---

### FT-12: compression module compiles under compression flag

- **Given:** The `compression` Cargo feature is enabled at compile time
- **When:** The crate is compiled with the `compression` feature
- **Then:** The `compression` module (`src/compression.rs`) is compiled; its HTTP compression types (gzip/brotli) are accessible in the crate's public API; disabling the flag removes the module from the compilation output; the build succeeds without error

# Feature Spec: Enterprise Reliability

Spec scenarios for `docs/feature/001_enterprise_reliability.md`. Verifies that enterprise reliability features are opt-in, explicit, and independent.

### FE-01: Enterprise feature is absent without its feature flag

- **Given:** `api_huggingface` compiled without the `circuit-breaker` feature flag
- **When:** user code attempts to construct a `CircuitBreaker`
- **Then:** compilation fails — the `CircuitBreaker` type is not present in the compiled output; no circuit breaking occurs automatically

### FE-02: Enterprise feature requires explicit developer construction

- **Given:** `api_huggingface` compiled with the `circuit-breaker` feature flag enabled
- **When:** `client.inference().create(prompt, model)` is called without constructing a `CircuitBreaker`
- **Then:** the API call proceeds without any circuit breaking logic; the `CircuitBreaker` is silent unless the caller explicitly wraps calls with it

### FE-03: Rate limiter only throttles when explicitly invoked

- **Given:** `api_huggingface` compiled with the `rate-limiting` feature and a `RateLimiter` constructed
- **When:** the caller does NOT call `rate_limiter.acquire().await` before an inference request
- **Then:** the request proceeds immediately without throttling; the `RateLimiter` has zero effect unless explicitly called

### FE-04: Enterprise features are independent — enabling one does not activate others

- **Given:** `api_huggingface` compiled with only the `failover` feature enabled (not `circuit-breaker`, `rate-limiting`, `health-checks`)
- **When:** the failover manager is used and multiple inference requests are made
- **Then:** only failover logic is active; no circuit breaking, rate limiting, or health monitoring occurs; those modules are not compiled in

### FE-05: caching, performance-metrics, and token-counting do not depend on the reliability module

- **Given:** `Cargo.toml` feature dependency declarations for all eight enterprise reliability features
- **When:** the deps lists for `caching`, `performance-metrics`, and `token-counting` are inspected
- **Then:** each of `caching`, `performance-metrics`, and `token-counting` depends only on `"client"` — none lists `"reliability"`; meanwhile `circuit-breaker`, `rate-limiting`, `failover`, `health-checks`, and `dynamic-config` each list `"reliability"` as a dep

### FE-06: Enterprise feature modules do not share global static state

- **Given:** `api_huggingface` compiled with `circuit-breaker`, `rate-limiting`, and `caching` features enabled
- **When:** `src/reliability/circuit_breaker.rs`, `src/reliability/rate_limiter.rs`, and `src/cache/implementation.rs` are inspected for cross-module static references
- **Then:** no `static` or `lazy_static!` declaration is shared between those three modules; each module manages its own internal state through its own struct fields with no global singletons

---

### FT-01: circuit-breaker feature — CircuitBreaker returns error when open, no panic

- **Given:** `api_huggingface` compiled with `circuit-breaker` feature enabled; a `CircuitBreaker` constructed with `CircuitBreakerConfig { failure_threshold: 1, success_threshold: 2, timeout: Duration::from_secs(60) }`
- **When:** one failing operation trips the breaker to `Open` state; a second operation is attempted through the open circuit
- **Then:** the second attempt immediately returns `Err(CircuitBreakerError::CircuitOpen)` without executing the wrapped closure; no panic occurs; the `CircuitState` is `Open`

### FT-02: rate-limiting feature gates `RateLimiter` through the `reliability` dependency

- **Given:** `Cargo.toml` feature definitions for `api_huggingface`; `lib.rs` module gating declarations
- **When:** the `rate-limiting` entry in `[features]` is inspected; `lib.rs` is inspected for `pub mod reliability`
- **Then:** `rate-limiting` lists `"reliability"` as its sole dependency; `lib.rs` gates `pub mod reliability` with `#[cfg(feature = "reliability")]`; compiling with `rate-limiting` enabled makes `api_huggingface::reliability::RateLimiter` accessible

### FT-03: failover feature gates `FailoverManager` through the `reliability` dependency

- **Given:** `Cargo.toml` feature definitions for `api_huggingface`; `lib.rs` module gating declarations
- **When:** the `failover` entry in `[features]` is inspected
- **Then:** `failover` lists `"reliability"` as its sole dependency; `lib.rs` gates `pub mod reliability` with `#[cfg(feature = "reliability")]`; compiling with `failover` enabled makes `api_huggingface::reliability::FailoverManager` accessible

### FT-04: health-checks feature gates `HealthChecker` through the `reliability` dependency

- **Given:** `Cargo.toml` feature definitions for `api_huggingface`; `lib.rs` module gating declarations
- **When:** the `health-checks` entry in `[features]` is inspected
- **Then:** `health-checks` lists `"reliability"` as its sole dependency; `lib.rs` gates `pub mod reliability` with `#[cfg(feature = "reliability")]`; compiling with `health-checks` enabled makes `api_huggingface::reliability::HealthChecker` accessible

### FT-05: caching feature gates `cache` module through `client`, not `reliability`

- **Given:** `Cargo.toml` feature definitions and `lib.rs` for `api_huggingface`
- **When:** the `caching` entry in `[features]` is inspected; `lib.rs` is inspected for `pub mod cache`
- **Then:** `caching` lists `"client"` as its sole dependency — not `"reliability"`; `lib.rs` gates `pub mod cache` with `#[cfg(feature = "client")]`; compiling with only `caching` enabled makes `api_huggingface::cache` accessible without requiring the reliability module

### FT-06: all enterprise features compile together under the full feature set without conflict

- **Given:** `api_huggingface` compiled with the `full` feature bundle, which enables all enterprise flags simultaneously: `circuit-breaker`, `rate-limiting`, `failover`, `health-checks`, `dynamic-config`, `caching`, `performance-metrics`, `token-counting`
- **When:** `cargo check --all-features` is executed against the crate
- **Then:** compilation exits with zero errors and zero warnings; all enterprise types — `api_huggingface::reliability::CircuitBreaker`, `RateLimiter`, `FailoverManager`, `HealthChecker` (via `reliability` module) and `api_huggingface::cache` (via `client` feature) — are simultaneously accessible with no name conflicts or missing-module errors

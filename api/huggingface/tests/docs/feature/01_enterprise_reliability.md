# Feature Spec: Enterprise Reliability

Spec scenarios for `docs/feature/001_enterprise_reliability.md`. Verifies that enterprise reliability features are opt-in, explicit, and independent.

## FE-01: Enterprise feature is absent without its feature flag

**Given:** `api_huggingface` compiled without the `circuit-breaker` feature flag
**When:** user code attempts to construct a `CircuitBreaker`
**Then:** compilation fails — the `CircuitBreaker` type is not present in the compiled output; no circuit breaking occurs automatically

## FE-02: Enterprise feature requires explicit developer construction

**Given:** `api_huggingface` compiled with the `circuit-breaker` feature flag enabled
**When:** `client.inference().create(prompt, model)` is called without constructing a `CircuitBreaker`
**Then:** the API call proceeds without any circuit breaking logic; the `CircuitBreaker` is silent unless the caller explicitly wraps calls with it

## FE-03: Rate limiter only throttles when explicitly invoked

**Given:** `api_huggingface` compiled with the `rate-limiting` feature and a `RateLimiter` constructed
**When:** the caller does NOT call `rate_limiter.acquire().await` before an inference request
**Then:** the request proceeds immediately without throttling; the `RateLimiter` has zero effect unless explicitly called

## FE-04: Enterprise features are independent — enabling one does not activate others

**Given:** `api_huggingface` compiled with only the `failover` feature enabled (not `circuit-breaker`, `rate-limiting`, `health-checks`)
**When:** the failover manager is used and multiple inference requests are made
**Then:** only failover logic is active; no circuit breaking, rate limiting, or health monitoring occurs; those modules are not compiled in

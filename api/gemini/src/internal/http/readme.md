# Internal HTTP Layer

HTTP request execution, reliability primitives, and caching.

## Responsibility Table

| filename | Responsibility |
|----------|---------------|
| mod.rs | HTTP client, request dispatch, and response handling |
| retry.rs | Exponential backoff retry logic |
| circuit_breaker.rs | Circuit breaker failure threshold management |
| rate_limiter.rs | Token bucket and sliding window rate limiting |
| cache.rs | LRU request cache with TTL expiry |
| compression.rs | Gzip, deflate, and brotli compression support |
| enterprise.rs | Enterprise feature coordination and entry points |

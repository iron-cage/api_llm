# Client Builder

Builder pattern for fluent client configuration.

## Responsibility Table

| filename | Responsibility |
|----------|---------------|
| mod.rs | ClientBuilder struct and build() implementation |
| presets.rs | Predefined configuration presets |
| setters_core.rs | Core field setters (api_key, base_url) |
| setters_retry.rs | Retry policy configuration setters |
| setters_circuit_breaker.rs | Circuit breaker threshold configuration setters |
| setters_rate_limiting.rs | Rate limiting configuration setters |
| setters_caching.rs | Request cache configuration setters |
| setters_compression.rs | HTTP compression configuration setters |

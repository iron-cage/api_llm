# src/

Source modules for `api_claude`. All modules declared in `lib.rs` via `mod_interface` with cargo feature gates.

## Responsibility Table

| File / Dir | Responsibility |
|------------|----------------|
| lib.rs | Crate root; feature-gated module declarations |
| client/ | Client struct, method impls, and API types |
| client.rs | Re-exports client module publicly |
| error/ | Core and enhanced error types |
| error.rs | Re-exports error module publicly |
| general_diagnostics/ | Diagnostics reporting and extended metrics |
| general_diagnostics.rs | Re-exports general diagnostics publicly |
| messages/ | Message content and tool type definitions |
| messages.rs | Re-exports messages module publicly |
| model_management/ | Model listing, detail, and analytics types |
| model_management.rs | Re-exports model management publicly |
| streaming/ | SSE streaming response types and client impl |
| streaming.rs | Re-exports streaming module publicly |
| authentication.rs | API key validation and advanced auth patterns |
| batch.rs | Batch Messages API request and response types |
| buffered_streaming.rs | Streaming buffer accumulator for smooth delivery |
| circuit_breaker.rs | Circuit breaker state machine for failure isolation |
| compression.rs | HTTP request and response compression |
| content_generation.rs | Content generation convenience API layer |
| curl_diagnostics.rs | Curl command generation for request diagnostics |
| dynamic_config.rs | Hot-reload dynamic configuration support |
| embeddings.rs | Text embedding API request and response types |
| enhanced_function_calling.rs | Type-safe tool calling with schema validation |
| enterprise_config.rs | Enterprise configuration builder and defaults |
| enterprise_quota.rs | Cost quota tracking and per-model enforcement |
| environment.rs | HTTP client setup and secret validation |
| failover.rs | Multi-endpoint failover strategy and switching |
| health_checks.rs | Periodic health check scheduling and monitoring |
| input_validation.rs | Request input boundary and constraint validation |
| model_comparison.rs | A/B testing and model comparison utilities |
| rate_limiting.rs | Token bucket and sliding window rate limiting |
| request_caching.rs | Request-level caching with TTL expiry |
| request_templates.rs | Reusable request templates for common patterns |
| retry_logic.rs | Exponential backoff retry with configurable limits |
| secret.rs | API credential loading from env and workspace |
| streaming_control.rs | Streaming pause, resume, and cancel flow control |
| sync_api.rs | Synchronous blocking wrapper over async API |

# tests/inc/

Integration and unit test modules for `api_claude`. All modules are declared in `mod.rs` and compiled as part of the single `tests` binary.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| mod.rs | Test module aggregator and shared imports |
| authentication_test.rs | Authentication feature tests |
| batch_messages_test.rs | Batch message creation API tests |
| buffered_streaming_test.rs | Buffered stream configuration and chunk behavior tests |
| circuit_breaker_test.rs | Circuit breaker state machine tests |
| comprehensive_integration_test.rs | End-to-end multi-feature integration tests |
| compression_test.rs | HTTP compression feature tests |
| content_generation_test.rs | Content generation API tests |
| content_generation_refactor_test.rs | Content generation post-refactor regression tests |
| core_client_test.rs | Client construction and configuration tests |
| curl_diagnostics_test.rs | Curl diagnostics feature tests |
| dynamic_config_test.rs | Dynamic configuration hot-reload tests |
| enhanced_function_calling_test.rs | Enhanced function calling integration tests |
| enhanced_model_details_test.rs | Enhanced model detail retrieval tests |
| enhanced_retry_logic_test.rs | Retry logic with enhanced error handling tests |
| enterprise_configuration_test.rs | EnterpriseConfig builder and validation tests |
| enterprise_quota_test.rs | Enterprise quota management tests |
| error_handling_integration_test.rs | Error handling cross-feature integration tests |
| error_handling_test.rs | Error type construction and conversion tests |
| example_model_validation_test.rs | Example file model name validation tests |
| examples_validation_test.rs | Example file compilation and structure tests |
| fallback_behavior_integration_test.rs | Feature-disabled fallback behavior tests |
| failover_test.rs | Failover strategy selection and switching tests |
| general_diagnostics_test.rs | General diagnostics reporting tests |
| health_checks_test.rs | Health check scheduling and result tests |
| input_validation_test.rs | Request input validation boundary tests |
| messages_api_test.rs | Messages API request/response type tests |
| model_management_test.rs | Model listing and detail retrieval tests |
| performance_test.rs | Performance measurement and threshold tests |
| performance_monitoring_test.rs | Performance monitoring telemetry tests |
| prompt_caching_tests.rs | Prompt caching control and cache token tests |
| rate_limiting_test.rs | Rate limiter token bucket behavior tests |
| request_caching_test.rs | Request cache hit/miss and TTL tests |
| request_templates_test.rs | Pre-configured request template builder tests |
| retry_logic_test.rs | Retry backoff and attempt limit tests |
| simple_integration_test.rs | Minimal real-API integration smoke tests |
| spec_verification_integration_test.rs | Specification compliance verification tests |
| streaming_test.rs | SSE streaming response parsing tests |
| streaming_control_test.rs | Streaming flow control tests |
| structured_logging_test.rs | Structured log output format tests |
| sync_api_test.rs | Synchronous API wrapper tests |
| sync_cached_content_test.rs | Sync API with prompt caching tests |
| sync_streaming_test.rs | Synchronous streaming wrapper tests |
| system_instructions_test.rs | System instruction content type tests |
| token_counting_test.rs | Token counting API request/response tests |
| token_validation_test.rs | Token count validation boundary tests |
| tool_calling_test.rs | Tool definition and choice type tests |
| vision_support_test.rs | Image content type and vision API tests |
| endpoint_coverage_test.rs | Endpoint coverage spec test functions (AP-01..AP-12) |
| enterprise_reliability_test.rs | Enterprise reliability spec test functions (FT-01..FT-08) |
| module_organization_test.rs | Module organization spec test functions (PT-01..PT-06) |
| operation_test_specs.rs | Operation secret-loading spec test functions (OP-01..OP-15) |
| testing_standards_test.rs | Testing standards spec test functions (TS-01..TS-06) |
| thin_client_principle_test.rs | Thin client principle spec test functions (TC-01..TC-06) |

## Known Pitfalls

**Bulk attribute replacement creates duplicates.** When replacing `#[ ignore = "..." ]` with `#[ cfg( feature = "integration" ) ]` en masse (e.g., via `perl -i -pe`), any test that already carried an existing `#[ cfg( feature = "integration" ) ]` gate ends up with two consecutive identical attributes. Rust compiles this without error, but it is dead weight. Always grep for consecutive duplicate `#[ cfg( feature = "integration" ) ]` lines after any bulk attribute replacement and dedup them.

**Gate ordering matters for readability.** The canonical order is `#[ cfg( feature = "integration" ) ]` first, then `#[ tokio::test ]`, then `async fn`. Reversing puts the test macro before the compile condition, which compiles identically but diverges from Rust convention.

**`integration` is in `default` features.** `default = ["full"]` and `full` includes `integration`, so `cargo nextest run` (no flags) runs ALL tests including integration tests. To run only unit tests without an API key, pass `--no-default-features --features <all-except-integration>`. The `w3 .test l::3` command uses `--all-features` which also includes integration — a valid API key is required for a fully green run.

**Clippy rejects `if { if { panic! } }`.** A double-nested conditional where the inner body is only a `panic!()` triggers `clippy::only_used_in_recursion` / "only a `panic!` in `if`-then statement". Use `assert!( !condition, "message", args )` inside the outer `if let` instead — semantically identical, clippy-clean.

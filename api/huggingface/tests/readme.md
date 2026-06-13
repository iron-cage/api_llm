# Tests

Automated tests for the `api_huggingface` crate. All integration tests use real HuggingFace API credentials — no mocking permitted.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `audio_tests.rs` | Audio API (ASR, TTS, classification, transformation) tests |
| `cache_tests.rs` | LRU caching with TTL and statistics tests |
| `chatbot_example_test.rs` | Conversational chatbot end-to-end workflow |
| `circuit_breaker_tests.rs` | Circuit breaker failure detection and recovery tests |
| `client_tests.rs` | Client initialization and configuration tests |
| `components_tests.rs` | Shared component type serialization tests |
| `content_generator_example_test.rs` | Content generation end-to-end workflow |
| `curl_diagnostics_tests.rs` | CURL diagnostics command generation tests |
| `debug_validation.rs` | Debugging utilities for test validation |
| `document_search_example_test.rs` | Semantic search end-to-end workflow |
| `dynamic_config_tests.rs` | Runtime dynamic configuration tests |
| `embeddings_tests.rs` | Embeddings API and similarity calculation tests |
| `error_handling_tests.rs` | Error type and recovery path tests |
| `example_compilation_test.rs` | Validates that example code compiles correctly |
| `example_error_handling_test.rs` | Error handling patterns from example code |
| `failover_tests.rs` | Multi-endpoint failover strategy tests |
| `function_calling_tests.rs` | Function/tool calling with tool definitions tests |
| `health_check_tests.rs` | Background endpoint health monitoring tests |
| `inference_tests.rs` | Text generation inference API tests |
| `models_tests.rs` | Model management (get, status, availability) tests |
| `performance_metrics_tests.rs` | Request latency and throughput tracking tests |
| `providers_api_integration_test.rs` | Real-endpoint provider integration tests |
| `providers_api_tests.rs` | Pro plan providers API unit tests |
| `qa_system_example_test.rs` | Question-answering system end-to-end workflow |
| `rate_limiting_tests.rs` | Token bucket rate limiting tests |
| `retry_tests.rs` | Explicit retry logic tests |
| `streaming_control_tests.rs` | Streaming pause, resume, and cancel tests |
| `streaming_tests.rs` | Streaming response handling tests |
| `sync_api_tests.rs` | Synchronous API wrapper tests |
| `sync_cache_tests.rs` | Synchronous caching tests |
| `sync_streaming_tests.rs` | Synchronous streaming tests |
| `sync_token_counting_tests.rs` | Synchronous token counting tests |
| `token_counting_tests.rs` | Token estimation strategy tests |
| `validation_tests.rs` | Input validation and constraint tests |
| `vision_tests.rs` | Vision API (classification, detection, captioning) tests |
| `inc/` | Shared test helper modules (not test binaries) |
| `docs/` | GWT spec scenarios for all doc entity instances |

### Running Tests

```bash
# Run all tests (includes integration tests — requires API key)
cargo nextest run --all-features

# Run without integration tests (no API key needed)
cargo nextest run --no-default-features --features enabled

# Run a single test file
cargo nextest run --test client_tests --all-features
```

### Test Requirements

Integration tests require a valid HuggingFace API key. Tests panic immediately if credentials are missing — no silent skips.

```bash
# Load from workspace secrets file
echo 'export HUGGINGFACE_API_KEY=hf_...' >> ../../secret/-huggingface.sh

# Or set environment variable directly
export HUGGINGFACE_API_KEY=hf_...
```

### Test Policy

- All integration tests are gated with `#[cfg(feature = "integration")]`
- The `integration` feature is included in `default` — `cargo nextest run --all-features` runs everything
- No mocking: all integration tests call real HuggingFace endpoints
- Loud failure: tests `panic!` on missing credentials, never silently pass
- See `docs/invariant/002_testing_standards.md` for the authoritative invariant

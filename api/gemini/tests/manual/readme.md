# Test Plan: api_gemini

**Package:** api_gemini  
**Version:** 0.2.0  
**Date:** 2025-08-23  
**Status:** Baseline  

---

## Master Code Unit Manifest

### Publicly Exposed Code Units

| Module | Unit Type | Name | Location | Status |
|--------|-----------|------|----------|--------|
| client | struct | Client | src/client.rs:35 | ✅ Implemented |
| client | struct | ClientBuilder | src/client.rs:44 | ✅ Implemented |
| client | struct | ModelsApi | src/client.rs:296 | ✅ Implemented |
| models | struct | Model | src/models/mod.rs:8 | ✅ Implemented |
| models | struct | ListModelsResponse | src/models/mod.rs:53 | ✅ Implemented |
| models | struct | GenerateContentRequest | src/models/mod.rs:66 | ✅ Implemented |
| models | struct | GenerateContentResponse | src/models/mod.rs:91 | ✅ Implemented |
| models | struct | GenerationConfig | src/models/mod.rs:108 | ✅ Implemented |
| models | struct | SafetySetting | src/models/mod.rs:138 | ✅ Implemented |
| models | struct | Tool | src/models/mod.rs:149 | ✅ Implemented |
| models | struct | FunctionDeclaration | src/models/mod.rs:163 | ✅ Implemented |
| models | struct | CodeExecution | src/models/mod.rs:178 | ✅ Implemented |
| models | struct | PromptFeedback | src/models/mod.rs:186 | ✅ Implemented |
| models | struct | UsageMetadata | src/models/mod.rs:200 | ✅ Implemented |
| models | struct | EmbedContentRequest | src/models/mod.rs:222 | ✅ Implemented |
| models | struct | EmbedContentResponse | src/models/mod.rs:243 | ✅ Implemented |
| models | struct | ContentEmbedding | src/models/mod.rs:252 | ✅ Implemented |
| models | struct | BatchEmbedContentsRequest | src/models/mod.rs:261 | ✅ Implemented |
| models | struct | BatchEmbedContentsResponse | src/models/mod.rs:269 | ✅ Implemented |
| models | struct | Content | src/models/mod.rs:279 | ✅ Implemented |
| models | struct | Part | src/models/mod.rs:290 | ✅ Implemented |
| models | struct | Blob | src/models/mod.rs:312 | ✅ Implemented |
| models | struct | FunctionCall | src/models/mod.rs:323 | ✅ Implemented |
| models | struct | FunctionResponse | src/models/mod.rs:334 | ✅ Implemented |
| models | struct | Candidate | src/models/mod.rs:345 | ✅ Implemented |
| models | struct | SafetyRating | src/models/mod.rs:374 | ✅ Implemented |
| models | struct | CitationMetadata | src/models/mod.rs:389 | ✅ Implemented |
| models | struct | CitationSource | src/models/mod.rs:398 | ✅ Implemented |
| models::api | struct | ModelApi | src/models/api.rs:70 | ✅ Implemented |
| error | enum | Error | src/error/mod.rs:8 | ✅ Implemented |
| error | struct | ApiErrorResponse | src/error/mod.rs:100 | ✅ Implemented |
| error | struct | ApiErrorDetails | src/error/mod.rs:109 | ✅ Implemented |
| internal::http | function | execute | src/internal/http.rs:6 | ✅ Implemented |

### Internal Implementation Units

| Module | Unit Type | Name | Description | Risk Level |
|--------|-----------|------|-------------|------------|
| internal::http | function | execute | Core HTTP execution logic | High |
| client | method | load_api_key_from_secret_file | Secret management | High |
| client | method | send_get_request | HTTP GET wrapper | Medium |
| client | method | send_post_request | HTTP POST wrapper | Medium |
| client | method | serialize_request_body | JSON serialization | Medium |
| client | method | deserialize_response | JSON deserialization | Medium |
| client | method | add_api_key_to_url | URL parameter handling | Medium |
| client | method | handle_response_error | Error response parsing | High |

---

## Master Requirement Manifest

### Feature Requirements

| ID | Category | Requirement | Status | Implementation |
|----|----------|-------------|--------|----------------|
| FR-1 | Client Architecture | Async/Sync client structs as main entry points | 🟡 Partial | Only async client implemented |
| FR-2 | Authentication | API key from secrets/env with workspace_tools | ✅ Complete | client.rs:149-182 |
| FR-3 | Builder Pattern | Builder for both clients with retry/timeout config | 🟡 Partial | Only basic builder implemented |
| FR-4-8 | Retry Logic | Built-in exponential backoff retry mechanism | ❌ Missing | No retry implementation found |
| FR-9-11 | Streaming | SSE streaming support with retry logic | ❌ Missing | No streaming implementation |
| FR-12-16 | Advanced Config | Connection pooling, proxies, circuit breakers | ❌ Missing | Basic client only |
| FR-17-19 | Request Builders | Fluent API with former crate validation | ❌ Missing | No former integration |
| FR-20-24 | Batch Operations | Parallel processing with rayon | ❌ Missing | No batch support |
| FR-25-29 | WASM Support | Browser compatibility with web-sys | ❌ Missing | No WASM support |
| FR-30-34 | Collaboration | WebSocket real-time features | ❌ Missing | No WebSocket support |
| FR-35-38 | Observability | Metrics, logging, OpenTelemetry | ❌ Missing | No observability |
| FR-39 | Model Discovery | List available models | ✅ Complete | models/api.rs:16 |
| FR-40 | Model Details | Get specific model info | ✅ Complete | models/api.rs:37 |
| FR-41 | Content Generation | Single-turn content generation | ✅ Complete | models/api.rs:108 |
| FR-42 | Multi-turn | Multi-turn conversation support | ✅ Complete | Via GenerateContentRequest |
| FR-43 | Embeddings | Text embedding generation | ✅ Complete | models/api.rs:139 |
| FR-44-47 | Feature Gating | Granular feature flags, minimal default | ❌ Missing | No feature flags |
| FR-48 | Data Structures | Serde structs for API requests/responses | ✅ Complete | models/mod.rs |
| FR-49 | Error Handling | Comprehensive Error enum with error_tools | ✅ Complete | error/mod.rs |
| FR-50 | Testing Strategy | External tests/ directory adherence | ✅ Complete | tests/ directory exists |
| FR-51 | Diagnostics | AsCurl trait re-export | ❌ Missing | No AsCurl integration |

### Non-Functional Requirements

| ID | Category | Requirement | Status | Notes |
|----|----------|-------------|--------|-------|
| NFR-1 | Performance | <50ms overhead per API call | ❓ Unknown | Needs performance testing |
| NFR-2-3 | Streaming/Connection | Streaming <100ms, pooling <10ms | ❌ N/A | No streaming/pooling |
| NFR-4-5 | Ergonomics | Intuitive APIs, builder validation | 🟡 Partial | Basic ergonomics only |
| NFR-6-7 | Reliability | Error handling, circuit breakers | 🟡 Partial | Error handling good, no circuit breakers |
| NFR-8 | Security | API key secrecy | ✅ Complete | workspace_tools integration |
| NFR-9 | Documentation | Comprehensive docs with examples | ✅ Complete | Good documentation present |
| NFR-10 | Compatibility | Latest stable Rust | ✅ Complete | Edition 2021 |
| NFR-11 | Runtime | Tokio compatibility | ✅ Complete | Async implementation |
| NFR-12-13 | Modularity | Ultra-minimal default, feature orthogonality | ❌ Missing | No feature flags |
| NFR-14 | Diagnostics | Optional diagnostic tools | ❌ Missing | No diagnostic features |
| NFR-15 | Transparency | Configurable retry with logging | ❌ Missing | No retry implementation |

### API Coverage (from readme.md)

| Endpoint | Method | Status | Implementation |
|----------|--------|--------|----------------|
| `/v1beta/models` | GET | ✅ Complete | models/api.rs:16 |
| `/v1beta/models/{model}` | GET | ✅ Complete | models/api.rs:37 |
| `/v1beta/models/{model}:generateContent` | POST | ✅ Complete | models/api.rs:108 |
| `/v1beta/models/{model}:streamGenerateContent` | POST | 🚧 Planned | Not implemented |
| `/v1beta/models/{model}:embedContent` | POST | ✅ Complete | models/api.rs:139 |
| `/v1beta/models/{model}:batchEmbedContents` | POST | ✅ Complete | Data structures only |
| `/v1beta/models/{model}:countTokens` | POST | 🚧 Planned | Not implemented |
| `/v1beta/cachedContents` | POST | 🚧 Planned | Not implemented |

---

## Specification Compliance Matrix

### Critical Compliance Gaps

| Priority | Type | Description | Impact | Recommendation |
|----------|------|-------------|--------|----------------|
| 1 | Architecture | No synchronous blocking client (FR-1) | High | Implement BlockingClient struct |
| 1 | Reliability | No retry logic with exponential backoff (FR-4-8) | High | Implement backoff crate integration |
| 1 | Modularity | No feature gating system (FR-44-47) | High | Implement granular Cargo features |
| 2 | Performance | No streaming support (FR-9-11) | Medium | Implement SSE with tokio-stream |
| 2 | Enterprise | No advanced networking features (FR-12-16) | Medium | Add connection pooling, proxies |
| 3 | Ergonomics | No fluent request builders (FR-17-19) | Medium | Integrate former crate |
| 3 | Scale | No batch operations (FR-20-24) | Medium | Add rayon parallel processing |

### Design Rule Violations

| Rule | Violation | Location | Fix Required |
|------|-----------|----------|--------------|
| External Test Directory | ✅ Compliant | tests/ | None |
| snake_case Naming | ✅ Compliant | All files | None |
| error_tools Usage | ✅ Compliant | error/mod.rs | None |
| mod_interface Pattern | ✅ Compliant | All modules | None |

---

## Audit & Risk Log

### High-Risk Findings

| ID | Type | Description | Severity | Mitigation |
|----|------|-------------|----------|------------|
| R-001 | Implementation Gap | No retry mechanism for transient failures | Critical | Implement exponential backoff with configurable policies |
| R-002 | Architecture Gap | Missing synchronous client for blocking applications | High | Add BlockingClient with identical API surface |
| R-003 | Security Risk | API key potentially logged in error messages | Medium | Audit error paths for secret exposure |
| R-004 | Performance Risk | No connection pooling for high-throughput scenarios | Medium | Implement reqwest connection pooling |
| R-005 | Reliability Gap | No circuit breaker pattern for cascade failure prevention | Medium | Add circuit breaker with configurable thresholds |

### Medium-Risk Findings

| ID | Type | Description | Impact | Recommendation |
|----|------|-------------|---------|----------------|
| R-006 | Feature Gap | No batch processing capabilities | Performance | Implement batch endpoints with parallel processing |
| R-007 | Ergonomics | No fluent request builders with validation | Developer Experience | Integrate former crate for type-safe builders |
| R-008 | Observability | No metrics or health check capabilities | Operations | Add metrics collection and health endpoints |
| R-009 | Compatibility | No WebAssembly support for browser deployment | Platform Support | Implement WASM feature flag with web-sys |
| R-010 | Real-time | No streaming content generation | User Experience | Add SSE streaming with tokio-stream |

### Low-Risk Findings

| ID | Type | Description | Impact | Recommendation |
|----|------|-------------|---------|----------------|
| R-011 | Diagnostics | No cURL command generation capability | Developer Experience | Integrate as_curl crate for request debugging |
| R-012 | Collaboration | No real-time multi-user session support | Advanced Features | Add WebSocket collaboration features |
| R-013 | Advanced Config | No proxy support for enterprise environments | Enterprise | Add HTTP/HTTPS/SOCKS5 proxy configuration |

---

## Consolidated Test & Traceability Matrix

### Requirement Coverage Analysis

| Requirement Category | Total | Implemented | Missing | Coverage % |
|---------------------|-------|-------------|---------|------------|
| Core API Endpoints | 8 | 5 | 3 | 62.5% |
| Client Architecture | 10 | 3 | 7 | 30.0% |
| Advanced Features | 15 | 0 | 15 | 0.0% |
| Data Structures | 5 | 5 | 0 | 100.0% |
| Error Handling | 3 | 3 | 0 | 100.0% |
| **Overall** | **41** | **16** | **25** | **39.0%** |

---

## Prioritized Test-Driven Scenarios

### Critical Priority (Security & Reliability)

| Priority | ID | Scenario | Value | Easiness | Advisability | Status |
|----------|----|---------|---------:|----------:|------------:|--------|
| 1 | TDD-001 | **Unit Test: API Key Security** - Verify API keys are not exposed in logs, error messages, or debug output | 10 | 8 | 80 | 🔴 Required |
| 2 | TDD-002 | **Integration Test: Authentication Error Handling** - Test client behavior with invalid API keys, expired tokens, and missing credentials | 10 | 7 | 70 | 🔴 Required |
| 3 | TDD-003 | **Unit Test: HTTP Error Response Parsing** - Verify correct Error enum variants for 4xx/5xx responses with proper error message extraction | 9 | 8 | 72 | 🔴 Required |
| 4 | TDD-004 | **Unit Test: JSON Serialization Safety** - Test request serialization with malformed data, special characters, and edge cases | 9 | 7 | 63 | 🔴 Required |

### High Priority (Core Functionality)

| Priority | ID | Scenario | Value | Easiness | Advisability | Status |
|----------|----|---------|---------:|----------:|------------:|--------|
| 5 | TDD-005 | **Integration Test: Model Discovery API** - Test listing models and retrieving model details with proper response validation | 8 | 9 | 72 | 🟡 Recommended |
| 6 | TDD-006 | **Integration Test: Content Generation Flow** - Test single-turn and multi-turn conversation with various model parameters | 10 | 6 | 60 | 🟡 Recommended |
| 7 | TDD-007 | **Integration Test: Embedding Generation** - Test text embedding with different task types and dimensionality settings | 8 | 8 | 64 | 🟡 Recommended |
| 8 | TDD-008 | **Unit Test: Client Builder Pattern** - Verify all configuration options work correctly with proper validation | 7 | 9 | 63 | 🟡 Recommended |

### Medium Priority (Edge Cases & Validation)

| Priority | ID | Scenario | Value | Easiness | Advisability | Status |
|----------|----|---------|---------:|----------:|------------:|--------|
| 9 | TDD-009 | **Unit Test: Content Structure Validation** - Test Part, Blob, and Content serialization/deserialization with edge cases | 6 | 8 | 48 | 🟢 Optional |
| 10 | TDD-010 | **Integration Test: Large Payload Handling** - Test behavior with large text inputs, multiple images, and complex function calls | 7 | 5 | 35 | 🟢 Optional |
| 11 | TDD-011 | **Unit Test: URL Construction** - Test API key parameter addition with various URL formats and special characters | 6 | 9 | 54 | 🟢 Optional |
| 12 | TDD-012 | **Integration Test: Safety Settings** - Test content filtering with various safety thresholds and categories | 8 | 6 | 48 | 🟢 Optional |

### Lower Priority (Performance & Advanced Features)

| Priority | ID | Scenario | Value | Easiness | Advisability | Status |
|----------|----|---------|---------:|----------:|------------:|--------|
| 13 | TDD-013 | **Performance Test: Response Time Benchmarks** - Measure API call overhead and establish performance baselines | 5 | 4 | 20 | 🟢 Optional |
| 14 | TDD-014 | **Unit Test: Function Calling Structures** - Test FunctionCall and FunctionResponse serialization with JSON schema validation | 6 | 7 | 42 | 🟢 Optional |
| 15 | TDD-015 | **Integration Test: Workspace Secret Management** - Test secret loading from secret directory with various configurations | 7 | 6 | 42 | 🟢 Optional |
| 16 | TDD-016 | **Unit Test: Citation Metadata Processing** - Test CitationSource and CitationMetadata parsing with various source types | 5 | 8 | 40 | 🟢 Optional |

### Critical Priority - Failure Mode Analysis (NEW)

| Priority | ID | Scenario | Value | Easiness | Advisability | Status |
|----------|----|---------|---------:|----------:|------------:|--------|
| 1 | TDD-021 | **Unit Test: Network Connection Failures** - Test behavior when network connection drops during API calls with proper timeout handling | 10 | 8 | 80 | 🔴 Required |
| 2 | TDD-022 | **Integration Test: API Service Unavailability** - Test 503 service unavailable responses and proper error categorization | 9 | 7 | 63 | 🔴 Required |
| 3 | TDD-023 | **Unit Test: Malformed JSON Response Handling** - Test behavior with corrupted/incomplete JSON responses from API | 9 | 8 | 72 | 🔴 Required |
| 4 | TDD-024 | **Unit Test: Memory Exhaustion on Large Responses** - Test handling of extremely large API responses that could cause OOM | 8 | 6 | 48 | 🟡 Recommended |

### High Priority - Failure Mode Analysis (NEW)

| Priority | ID | Scenario | Value | Easiness | Advisability | Status |
|----------|----|---------|---------:|----------:|------------:|--------|
| 5 | TDD-025 | **Integration Test: Rate Limit Boundary Testing** - Test exact rate limit thresholds and proper 429 error handling | 9 | 7 | 63 | 🟡 Recommended |
| 6 | TDD-026 | **Unit Test: Concurrent Request Handling** - Test thread safety when multiple threads use the same client instance | 8 | 5 | 40 | 🟡 Recommended |
| 7 | TDD-027 | **Integration Test: Partial Response Corruption** - Test handling when API returns partial responses due to network issues | 8 | 6 | 48 | 🟡 Recommended |
| 8 | TDD-028 | **Unit Test: Configuration Validation Failures** - Test all possible invalid configurations for ClientBuilder | 7 | 9 | 63 | 🟡 Recommended |

### Medium Priority - Failure Mode Analysis (NEW)

| Priority | ID | Scenario | Value | Easiness | Advisability | Status |
|----------|----|---------|---------:|----------:|------------:|--------|
| 9 | TDD-029 | **Integration Test: Timeout Edge Cases** - Test various timeout scenarios including during response body reading | 7 | 7 | 49 | 🟢 Optional |
| 10 | TDD-030 | **Unit Test: URL Encoding Edge Cases** - Test API key and parameter encoding with special characters and unicode | 6 | 8 | 48 | 🟢 Optional |
| 11 | TDD-031 | **Integration Test: Model Endpoint Failures** - Test behavior when specific models are temporarily unavailable | 7 | 6 | 42 | 🟢 Optional |
| 12 | TDD-032 | **Unit Test: Workspace Secret File Corruption** - Test handling of corrupted or unreadable secret files | 6 | 7 | 42 | 🟢 Optional |

### Future Enhancement Scenarios

| Priority | ID | Scenario | Value | Easiness | Advisability | Status |
|----------|----|---------|---------:|----------:|------------:|--------|
| 13 | TDD-017 | **Unit Test: Retry Logic Implementation** - Test exponential backoff behavior with various failure scenarios | 10 | 5 | 50 | 🔴 Blocked (Not Implemented) |
| 14 | TDD-018 | **Integration Test: Streaming Content Generation** - Test SSE streaming with proper chunk handling and error recovery | 9 | 3 | 27 | 🔴 Blocked (Not Implemented) |
| 15 | TDD-019 | **Unit Test: Feature Flag Combinations** - Test various feature flag combinations for minimal build verification | 8 | 4 | 32 | 🔴 Blocked (Not Implemented) |
| 16 | TDD-020 | **Integration Test: Batch Operations** - Test parallel processing of multiple requests with proper error handling | 7 | 3 | 21 | 🔴 Blocked (Not Implemented) |

### Original Test Scenarios (Re-prioritized)

| Priority | ID | Scenario | Value | Easiness | Advisability | Status |
|----------|----|---------|---------:|----------:|------------:|--------|
| 17 | TDD-001 | **Unit Test: API Key Security** - Verify API keys are not exposed in logs, error messages, or debug output | 10 | 8 | 80 | 🔴 Required |
| 18 | TDD-002 | **Integration Test: Authentication Error Handling** - Test client behavior with invalid API keys, expired tokens, and missing credentials | 10 | 7 | 70 | 🔴 Required |
| 19 | TDD-003 | **Unit Test: HTTP Error Response Parsing** - Verify correct Error enum variants for 4xx/5xx responses with proper error message extraction | 9 | 8 | 72 | 🔴 Required |
| 20 | TDD-004 | **Unit Test: JSON Serialization Safety** - Test request serialization with malformed data, special characters, and edge cases | 9 | 7 | 63 | 🔴 Required |
| 21 | TDD-005 | **Integration Test: Model Discovery API** - Test listing models and retrieving model details with proper response validation | 8 | 9 | 72 | 🟡 Recommended |
| 22 | TDD-006 | **Integration Test: Content Generation Flow** - Test single-turn and multi-turn conversation with various model parameters | 10 | 6 | 60 | 🟡 Recommended |
| 23 | TDD-007 | **Integration Test: Embedding Generation** - Test text embedding with different task types and dimensionality settings | 8 | 8 | 64 | 🟡 Recommended |
| 24 | TDD-008 | **Unit Test: Client Builder Pattern** - Verify all configuration options work correctly with proper validation | 7 | 9 | 63 | 🟡 Recommended |
| 25 | TDD-009 | **Unit Test: Content Structure Validation** - Test Part, Blob, and Content serialization/deserialization with edge cases | 6 | 8 | 48 | 🟢 Optional |
| 26 | TDD-010 | **Integration Test: Large Payload Handling** - Test behavior with large text inputs, multiple images, and complex function calls | 7 | 5 | 35 | 🟢 Optional |
| 27 | TDD-011 | **Unit Test: URL Construction** - Test API key parameter addition with various URL formats and special characters | 6 | 9 | 54 | 🟢 Optional |
| 28 | TDD-012 | **Integration Test: Safety Settings** - Test content filtering with various safety thresholds and categories | 8 | 6 | 48 | 🟢 Optional |
| 29 | TDD-013 | **Performance Test: Response Time Benchmarks** - Measure API call overhead and establish performance baselines | 5 | 4 | 20 | 🟢 Optional |
| 30 | TDD-014 | **Unit Test: Function Calling Structures** - Test FunctionCall and FunctionResponse serialization with JSON schema validation | 6 | 7 | 42 | 🟢 Optional |
| 31 | TDD-015 | **Integration Test: Workspace Secret Management** - Test secret loading from secret directory with various configurations | 7 | 6 | 42 | 🟢 Optional |
| 32 | TDD-016 | **Unit Test: Citation Metadata Processing** - Test CitationSource and CitationMetadata parsing with various source types | 5 | 8 | 40 | 🟢 Optional |

---

## Updated Test Execution Strategy

### Immediate Actions Required (Based on Failure Mode Analysis)

1. **Critical Failure Mode Testing** - Implement TDD-021 through TDD-024 to ensure robust failure handling
2. **High Priority Failure Scenarios** - Implement TDD-025 through TDD-028 for edge case coverage
3. **Security Testing** - Implement TDD-001, TDD-002 (now TDD-017, TDD-018) to ensure API key handling is secure
4. **Core API Testing** - Implement remaining core functionality tests for primary API coverage

### Test Environment Requirements

- **Unit Tests**: No external dependencies, use mock HTTP clients
- **Integration Tests**: Require valid GEMINI_API_KEY for live API testing  
- **Performance Tests**: Dedicated environment with consistent network conditions
- **Security Tests**: Isolated environment with logging capture for verification
- **Failure Mode Tests**: Controlled network conditions to simulate various failure scenarios

### Enhanced Testing Tools (Updated)

- **Network Simulation**: Tools for simulating network failures, timeouts, and partial responses
- **Mocking**: Use existing pattern with HTTP mocking for deterministic failure scenarios
- **Async Testing**: `tokio-test` for async unit tests with precise timing control
- **Integration**: Real API calls with comprehensive timeout and error handling
- **Coverage**: `tarpaulin` for code coverage measurement with failure path emphasis

### Key Insights from Failure Mode Analysis

1. **Network Resilience**: The current implementation lacks proper handling of network connection drops during API calls
2. **JSON Parsing Safety**: Need to test malformed JSON responses that could crash deserialization
3. **Memory Safety**: Large responses could cause memory exhaustion without proper streaming
4. **Concurrency**: Thread safety needs verification for shared client usage
5. **Configuration Validation**: ClientBuilder needs comprehensive validation testing

---

**Test plan status: 242 test functions implemented across 22 test files. Comprehensive coverage including integration tests, unit tests, and advanced feature testing.**

**What is the next `Test Generation Lens` you would like to apply?**

Examples:
- `apply lens User Story Analysis: "As a developer, I want reliable error messages..."`  
- `apply lens Performance Bottleneck Analysis: "Where could the system become slow under load?"`
- `apply lens Security Vulnerability Assessment: "What attack vectors exist in the API client?"`
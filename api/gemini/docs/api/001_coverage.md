# API: Coverage

### Scope

- **Purpose**: Document the complete set of Gemini API endpoints and features implemented in api_gemini
- **Responsibility**: Map every public API endpoint to its async/sync status, test count, and URL pattern
- **In Scope**: Core endpoints, advanced API families, enterprise features, feature flags, test statistics
- **Out of Scope**: Implementation internals, usage patterns, protocol wire format, configuration procedures

### Abstract

api_gemini covers 100% of the Gemini REST API surface through a thin async client. Every major endpoint family is implemented with comprehensive test coverage. All async operations have typed request and response structs; sync variants are available for blocking contexts via the `sync_api` feature.

### Operations

#### Core Endpoints

| Feature | Async | Sync | Tests | Endpoint |
|---------|-------|------|-------|----------|
| List Models | ✅ | ✅ | 18/18 | `GET /v1beta/models` |
| Get Model | ✅ | ✅ | 12/12 | `GET /v1beta/models/{model}` |
| Generate Content | ✅ | ✅ | 45/45 | `POST /v1beta/models/{model}:generateContent` |
| Stream Generate Content | ✅ | ❌ | 8/8 | `POST /v1beta/models/{model}:streamGenerateContent` |
| Embed Content | ✅ | ✅ | 32/32 | `POST /v1beta/models/{model}:embedContent` |
| Batch Embed Contents | ✅ | ✅ | 28/28 | `POST /v1beta/models/{model}:batchEmbedContents` |
| Count Tokens | ✅ | ✅ | 24/24 | `POST /v1beta/models/{model}:countTokens` |
| Cached Content | ✅ | ✅ | 16/16 | `POST /v1beta/cachedContents` |

#### Advanced API Families

| Feature | Status | Tests | Description |
|---------|--------|-------|-------------|
| Google Search Grounding | ✅ | 8/8 | Real-time web search with citations |
| Enhanced Function Calling | ✅ | 8/8 | Advanced modes (AUTO/ANY/NONE) with precise control |
| System Instructions | ✅ | 8/8 | Structured model behavior control |
| Code Execution | ✅ | 9/9 | Python code generation and execution |
| Model Tuning | ✅ | 12/12 | Fine-tuning with hyperparameters |
| Tuned Models CRUD | ✅ | 6/6 | Create, list, get, delete tuned models |

#### Enterprise Features

| Feature | Status | Tests | Description |
|---------|--------|-------|-------------|
| Retry Logic | ✅ | 6/6 | Exponential backoff with configurable attempts |
| Circuit Breaker | ✅ | 5/5 | Fault tolerance for unreliable services |
| Rate Limiting | ✅ | 6/6 | Request rate control and quota management |
| Request Caching | ✅ | 8/8 | Intelligent response caching |
| Failover Support | ✅ | 4/4 | Multi-endpoint configuration with automatic switching |
| Health Checks | ✅ | 3/3 | Periodic endpoint monitoring |
| Streaming Control | ✅ | 6/6 | Pause, resume, cancel for real-time streams |
| WebSocket Streaming | ✅ | 4/4 | Bidirectional real-time communication |
| Dynamic Configuration | ✅ | 8/8 | Hot-reload with rollback and versioning |
| Input Validation | ✅ | 15/15 | Comprehensive request validation |
| Error Handling | ✅ | 25/25 | Comprehensive error types and recovery |
| Builder Patterns | ✅ | 12/12 | Fluent API configuration |
| Structured Logging | ✅ | 8/8 | Detailed operation logging |
| Diagnostics (Curl) | ✅ | 2/2 | curl command generation for debugging |
| Enterprise Quota Management | ✅ | 16/16 | Client-side quota and cost tracking |
| Compression Integration | ✅ | 7/7 | Request/response compression |
| Model Comparison | ✅ | 8/10 | A/B testing framework |
| Request Templates | ✅ | 8/8 | Reusable configurations |
| Buffered Streaming | ✅ | 5/5 | Smooth UX streaming |

#### Feature Flags

| Flag | Status | Description |
|------|--------|-------------|
| `batch_operations` | Infrastructure Ready | Async job-based processing (waiting for Gemini API release) |
| `compression` | Core Complete | Gzip, Deflate, Brotli algorithms |
| `full` | Available | Enables all optional features |

### Error Handling

All operations return `Result<T, api_gemini::Error>`. Error categories:
- Authentication failure: invalid or missing API key
- Rate limiting: 429 responses from the Gemini API
- Network timeout: configurable per-request timeout (default 30 seconds)
- API error responses: mapped to structured `ApiError` with HTTP status and message body
- Deserialization failure: unexpected response shapes from API version mismatches

### Compatibility Guarantees

- API version: `v1beta` (crate v0.5.0)
- Endpoint URLs are configurable via environment configuration for future version migration
- `batch_operations` is infrastructure-ready; pending Gemini API general availability release
- Breaking changes in the Gemini API response schema require crate updates; no automatic shimming

### Test Statistics

- **Pass Rate**: All tests pass when `GEMINI_API_KEY` is available
- **Warning-Free**: Zero compilation warnings enforced via `RUSTFLAGS="-D warnings"`
- **Doc Coverage**: 100% for public APIs

### Sources

| File | Relationship |
|------|-------------|
| `src/models/api/content_generation/api_impl.rs` | Core generate_content implementation |
| `src/enterprise/` | Enterprise feature implementations |
| `src/components/` | Request/response type definitions |

### Tests

| File | Relationship |
|------|-------------|
| `tests/inc/comprehensive_integration_test.rs` | Core endpoint integration tests |
| `tests/inc/enhanced_function_calling_test.rs` | Function calling and tool use tests |
| `tests/inc/streaming_test.rs` | Streaming and WebSocket tests |
| `tests/inc/embeddings_test.rs` | Embedding endpoint tests |
| `tests/inc/model_management_test.rs` | Model listing and retrieval tests |

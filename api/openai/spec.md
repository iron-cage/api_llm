# spec

- **Name:** api_openai
- **Version:** 0.6
- **Date:** 2025-10-08
- **Status:** Production Ready - 100% Complete (89/89 tasks)
- **System Specification:** [../../spec.md](../../spec.md)

### Project Overview

The `api_openai` crate provides a comprehensive, type-safe Rust client library for interacting with OpenAI's API services. This specification defines the architecture, requirements, and standards for the implementation.

**Architecture Decision**: This API crate is designed as a **stateless HTTP client** with no persistence requirements. All operations are direct HTTP calls to the OpenAI API without local data storage, caching, or state management beyond request/response handling.

**Governing Principle**: **"Thin Client, Rich API"** - Expose all server-side functionality transparently while maintaining zero client-side intelligence or **automatic** behaviors. **Key Distinction**: The principle prohibits **automatic/implicit** behaviors but explicitly **allows and encourages** **explicit/configurable** enterprise reliability features.

**Note:** This specification must be implemented in accordance with the ecosystem-wide requirements defined in the [System Specification](../../spec.md).

### Vocabulary

- **API Client**: Comprehensive, type-safe Rust client library for OpenAI API services
- **Chat Completions**: Conversational AI capabilities using GPT models for interactive applications
- **Embeddings**: Text-to-vector conversion for semantic understanding and similarity operations
- **Completions**: Text generation and continuation capabilities for content creation
- **Models**: Discovery and management of available OpenAI model configurations
- **Images**: AI-powered image generation, editing, and variation capabilities
- **Audio**: Speech-to-text transcription and text-to-speech synthesis functionality
- **Files**: Upload and management of files for fine-tuning and assistant operations
- **Fine-tuning**: Custom model training capabilities for specialized use cases
- **Moderation**: Content filtering and safety analysis for responsible AI usage
- **Streaming**: Real-time response delivery for responsive user experiences
- **Function Calling**: Tool integration capabilities for extending model functionality

### Scope and Objectives

### 1.1 Primary Objectives
- Provide complete coverage of OpenAI API endpoints
- Ensure type safety and compile-time error detection
- Support both synchronous and asynchronous operations
- Implement robust error handling and retry mechanisms
- Maintain security best practices for credential management
- Support streaming responses for real-time applications

### 1.2 API Coverage Requirements
The client must support all major OpenAI API endpoints:

### Core Endpoints
- **Responses API**: Create, retrieve, update, cancel, and delete responses
- **Chat Completions**: Standard chat completion functionality
- **Assistants**: Full assistant lifecycle management
- **Files**: File upload, retrieval, and management
- **Fine-tuning**: Model fine-tuning operations
- **Images**: Image generation and manipulation
- **Audio**: Speech-to-text and text-to-speech
- **Embeddings**: Text embedding generation
- **Moderations**: Content moderation capabilities
- **Models**: Model listing and information retrieval

### Advanced Features
- **Realtime API**: WebSocket-based real-time communication
- **Vector Stores**: Vector database operations for RAG applications
- **Tool Usage**: Function calling and tool integration
- **Streaming**: Server-sent events for incremental responses

### 2. Architecture Design

### 2.1 Core Components

### Client Layer
```rust
pub struct Client<E>
where
  E: OpenaiEnvironment + EnvironmentInterface + Send + Sync + 'static
{
  pub environment: E,
}
```

**Custom Base URL Configuration:**
```rust
// Official OpenAI API (default)
let env = OpenaiEnvironmentImpl::build(
  secret,
  None,
  None,
  OpenAIRecommended::base_url().to_string(),     // https://api.openai.com/v1/
  OpenAIRecommended::realtime_base_url().to_string(),
)?;

// Azure OpenAI Service
let env = OpenaiEnvironmentImpl::build(
  secret,
  None,
  None,
  "https://your-resource.openai.azure.com/".to_string(),
  "https://your-resource.openai.azure.com/realtime/".to_string(),
)?;

// OpenAI-compatible API (e.g., LocalAI, Ollama)
let env = OpenaiEnvironmentImpl::build(
  secret,
  None,
  None,
  "http://localhost:8080/v1/".to_string(),
  "http://localhost:8080/realtime/".to_string(),
)?;
```

### Environment Management
- **OpenaiEnvironment**: Trait defining environment interface
- **OpenaiEnvironmentImpl**: Concrete environment implementation with custom base URL support
- **Custom Base URLs**: Configurable API endpoints for Azure OpenAI, OpenAI-compatible APIs, or corporate proxies
- **URL Validation**: Proper URL parsing and validation with error handling
- **Default Configuration**: `OpenAIRecommended::base_url()` provides official OpenAI endpoint
- Support for multiple environments (development, staging, production)

### Secret Management
```rust
pub struct Secret(SecretString);
```
- Integration with `secrecy` crate for credential protection
- Multiple loading mechanisms using workspace_tools fallback chain
- Format validation for API keys
- Audit trail for security monitoring

### 2.2 Module Organization

### Private Namespace Pattern
All modules must follow the wTools ecosystem `mod_interface` pattern:
```rust
mod private {
  // Module declarations
  pub mod assistants;
  pub mod audio;
  // ... other modules
}

crate::mod_interface! {
  exposed use private::assistants;
  exposed use private::audio;
  // ... expose modules
}
```

### Component Structure
- **Core Components**: `src/components/` - Shared data structures and utilities
- **API Modules**: `src/{endpoint}/` - Endpoint-specific implementations
- **Client**: `src/client.rs` - HTTP client wrapper
- **Environment**: `src/environment/` - Configuration management
- **Error Handling**: `src/error.rs` - Unified error types

### 2.3 Error Handling Strategy

### Error Types
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum OpenAIError {
  Api(ApiError),           // API-returned errors
  Http(String),            // HTTP transport errors
  Network(String),         // Network connectivity issues
  Timeout(String),         // Request timeout errors
  InvalidArgument(String), // Client-side validation failures
  // ... additional variants
}
```

### Integration with error_tools
- Leverage `error_tools` crate for ecosystem consistency
- Automatic derive implementations where possible
- Structured error propagation

### 3. Quality Standards

### 3.1 Testing Requirements

### Dual-Layer Testing Strategy

This project implements a **dual-layer testing approach** to balance comprehensive coverage with reliable validation:

### Unit Testing
- Minimum 80% code coverage
- Test all public APIs
- **Test harnesses for reliability mechanisms**: Retry logic, circuit breakers, and rate limiters use controlled test harnesses (`MockHttpClient`) to validate component behavior in isolation with predetermined failure sequences
- Focus on edge cases and error conditions
- **Important**: Test harnesses are NOT mocks of the OpenAI API - they test reliability coordinators' responses to specific failure scenarios

### Integration Testing
- Feature-gated integration tests: `#![cfg(feature = "integration")]`
- Real API endpoint testing with valid credentials
- Environment variable configuration
- Network failure simulation
- **No API mocking**: Integration tests ALWAYS use real OpenAI endpoints

### Mandatory Failing Behavior
Integration tests MUST enforce strict failing behavior:

**REQUIREMENTS:**
- Tests using real APIs MUST fail hard when credentials are unavailable
- NEVER silently fall back to mocks or dummy data when real API access is requested
- Fail immediately on network connectivity issues
- Fail immediately on API authentication/authorization problems

**IMPLEMENTATION:**
- `IsolatedClient::new(test_name, use_real_api=true)` MUST fail if credentials missing
- `Secret::load_with_fallbacks()` MUST fail if no valid OPENAI_API_KEY found
- Test framework validates credentials before proceeding with real API calls
- Clear error messages indicating specific failure reasons

**RATIONALE:**
This ensures integration test results are meaningful and reliable. Test failures
indicate real issues that must be addressed rather than being masked by fallbacks.

### Test Organization
```
tests/
├── integration.rs    # Feature-gated integration tests
├── tests.rs         # Component-level tests
└── unit/           # Granular unit tests
    ├── client.rs
    ├── secret.rs
    └── ...
```

### 3.2 Code Quality Standards

### File Size Limits
- Maximum 1,000 lines per source file
- Split oversized files into logical components
- Current violations must be addressed:
  - `assistants_shared.rs`: 2,078 lines → split required
  - `realtime_shared.rs`: 1,557 lines → split required

### Documentation Requirements
- All public APIs must have rustdoc comments
- Include usage examples in documentation
- Maintain up-to-date README with examples
- API reference documentation generation

### Linting and Formatting
- Custom clippy configuration for project-specific rules
- Strict adherence to wTools codestyle patterns
- Zero-warning builds required

### 4. Enterprise Reliability Features

### 4.1 Overview

The OpenAI API client supports optional enterprise reliability features that enhance production robustness. These features align with the **"Thin Client, Rich API"** governing principle by requiring **explicit configuration** and **transparent operation**.

**Key Requirements**:
- **Feature Gating**: All enterprise features behind cargo features (`retry`, `circuit_breaker`, `rate_limiting`, `failover`, `health_checks`)
- **Explicit Configuration**: Developer must explicitly enable and configure each feature
- **Transparent Operation**: Method names clearly indicate additional behaviors
- **Zero Overhead**: No runtime cost when features are disabled

### 4.2 Configurable Retry Logic

**Cargo Feature**: `retry`

**Requirements**:
- Exponential backoff with jitter for connection failures
- Configurable maximum retry attempts and timeouts
- Error classification (retryable vs non-retryable errors)
- Transparent method naming: `execute_with_retries()`

**Configuration Pattern**:
```rust
let client = Client::builder()
  .api_key("sk-...")
  .max_retries(3)                    // Explicitly configured
  .enable_retry_logic(true)          // Explicitly enabled
  .retry_backoff_multiplier(2.0)     // Optional: backoff configuration
  .build()?;

// Method name clearly indicates retry behavior
client.execute_with_retries(request).await?;
```

### 4.3 Circuit Breaker Pattern

**Cargo Feature**: `circuit_breaker`

**Requirements**:
- Configurable failure thresholds and timeout periods
- State transitions: Closed → Open → Half-Open → Closed
- Automatic recovery testing with configurable success thresholds
- Thread-safe implementation using `Arc<Mutex<>>` patterns

**Configuration Pattern**:
```rust
let client = Client::builder()
  .api_key("sk-...")
  .circuit_breaker_failure_threshold(5)     // Explicitly configured
  .circuit_breaker_timeout(Duration::from_secs(60))
  .enable_circuit_breaker(true)             // Explicitly enabled
  .build()?;

// Method name clearly indicates circuit breaker behavior
client.execute_with_circuit_breaker(request).await?;
```

### 4.4 Rate Limiting

**Cargo Feature**: `rate_limiting`

**Requirements**:
- Token bucket or sliding window algorithms
- Configurable request rates and burst limits
- Request queuing and delay mechanisms
- Support for different rate limits per endpoint type

**Configuration Pattern**:
```rust
let client = Client::builder()
  .api_key("sk-...")
  .rate_limit_requests_per_second(10.0)     // Explicitly configured
  .rate_limit_burst_size(20)                // Optional: burst configuration
  .enable_rate_limiting(true)               // Explicitly enabled
  .build()?;

// Method name clearly indicates rate limiting behavior
client.execute_with_rate_limiting(request).await?;
```

### 4.5 Enterprise Features Integration

**Unified Enterprise Execution**:
```rust
// Single method that applies all explicitly enabled enterprise features
client.execute_with_enterprise_features(request).await?;
```

**Feature Combinations**:
- Features can be enabled independently or in combination
- Each feature maintains its own configuration and state
- Thread-safe concurrent operation across all features
- Optional metrics integration for monitoring and observability

### 4.6 Implementation Standards

**Thread Safety**:
- All enterprise features must be thread-safe for concurrent use
- Use `Arc<Mutex<>>` patterns for shared state management
- Avoid blocking operations in async contexts

**Error Handling**:
- Enterprise features must integrate with existing `OpenAIError` types
- Clear error messages indicating which enterprise feature caused failures
- Proper error propagation without masking underlying API errors

**Testing Requirements**:
- Unit tests for each enterprise feature behind appropriate cargo features
- Integration tests validating feature combinations
- Performance tests ensuring zero overhead when features disabled

### 5. Security Requirements

### 5.1 Credential Management

### Secret Storage
- Never log or expose actual secret values
- Use `secrecy::SecretString` for in-memory protection
- Support multiple credential sources using workspace_tools fallback chain:
  1. **Workspace secrets file**: `../../secret/-secrets.sh` (Primary)
  2. **Environment variables**: `OPENAI_API_KEY` (Fallback)
  3. **Alternative secret files**: `secrets.sh`, `.env` (Compatibility)
  4. **Runtime**: programmatic setting with validation (Direct)
- Automatic format detection: `KEY=VALUE` and `export KEY=VALUE` formats

### Validation Standards
```rust
fn validate_api_key_format(secret: &str) -> Result< (), OpenAIError >
{
  // Enforce OpenAI API key format requirements
  // - Must start with "sk-"
  // - Length constraints: 10-200 characters
  // - Character set: alphanumeric + underscore + hyphen
}
```

### Secret Loading API
```rust
impl Secret
{
  // Recommended: Comprehensive fallback chain using workspace_tools
  pub fn load_with_fallbacks(key_name: &str) -> Result< Self >;

  // Environment variable loading
  pub fn load_from_env(env_var: &str) -> Result< Self >;

  // Workspace-specific loading
  pub fn load_from_workspace(key_name: &str, filename: &str) -> Result< Self >;

  // Direct validation and creation
  pub fn new(secret: String) -> Result< Self >;
  pub fn new_unchecked(secret: String) -> Self;
}
```

### Audit Trail
- Track all secret exposure events
- Hash-based identification without credential leakage
- Global exposure counter for monitoring
- Call stack tracking for debugging

### 5.2 Network Security
- TLS/HTTPS enforcement for all communications
- Certificate validation
- Request signing and authentication
- Rate limiting and retry with exponential backoff

### 6. Performance Requirements

### 6.1 Async-First Design
- All I/O operations must be asynchronous
- Support for `tokio` runtime
- Efficient connection pooling
- Minimal allocation overhead

### 6.2 Streaming Support
- Server-sent events for incremental responses
- WebSocket support for real-time APIs
- Backpressure handling
- Graceful connection management

### 6.3 Compilation Performance
- Build times under 60 seconds for full compilation
- Incremental compilation optimization
- Minimal dependency feature usage
- Parallel compilation support

### 7. Dependency Management

### 7.1 Dependency Strategy: Optional Dependencies with enabled/full Pattern

**Architecture Decision:** This crate follows the **mandatory enabled/full feature pattern** from `crate_distribution.rulebook.md`. ALL dependencies MUST be declared as `optional = true` and activated exclusively through Cargo features.

**Rationale:**
- Enables true no-op compilation when all features disabled
- Prevents unnecessary compilation overhead when crate is a dependency
- Follows wTools ecosystem enabled/full pattern
- Allows fine-grained control over what gets compiled

### 7.2 Feature Configuration

```toml
[features]
default = [ "enabled" ]

# Master switch - activates all core dependencies
enabled = [
  # wTools ecosystem
  "dep:mod_interface",
  "dep:former",
  "dep:error_tools",
  "dep:derive_tools",
  "dep:workspace_tools",

  # Async runtime and traits
  "dep:async-trait",
  "dep:url",

  # Utilities
  "dep:rand",
  "dep:chrono",
  "dep:uuid",

  # Serialization and data handling
  "dep:regex",
  "dep:serde",
  "dep:serde_json",
  "dep:serde_yaml",
  "dep:base64",
  "dep:secrecy",

  # Async execution
  "dep:futures-core",
  "dep:futures-util",
  "dep:futures",
  "dep:backoff",
  "dep:tokio",
  "dep:bytes",
  "dep:eventsource-stream",

  # HTTP and WebSocket
  "dep:reqwest",
  "dep:tracing",
  "dep:tokio-tungstenite",
]

# Convenience feature - enables everything
full = [ "enabled", "integration", "retry", "circuit_breaker", "rate_limiting", "failover", "health_checks", "enterprise", "caching", "batching", "compression" ]

# Additional features
integration = []
retry = []
circuit_breaker = []
rate_limiting = []
failover = []
health_checks = []
caching = [ "sha2", "blake3" ]
compression = [ "flate2" ]
batching = [ "blake3" ]
enterprise = []
```

### 7.3 Dependency Declarations

ALL dependencies MUST be declared with `optional = true`:

```toml
[dependencies]
# wTools ecosystem - all optional
mod_interface = { workspace = true, optional = true }
former = { workspace = true, optional = true }
error_tools = { workspace = true, optional = true }
derive_tools = { workspace = true, optional = true }
workspace_tools = { workspace = true, features = ["secrets"], optional = true }

# Async - all optional
async-trait = { workspace = true, optional = true }
tokio = { workspace = true, features = ["macros", "sync", "time", "rt-multi-thread"], optional = true }
futures = { workspace = true, optional = true }
# ... etc

# All dependencies optional, activated via enabled feature
```

### 7.4 No-Op Compilation

When compiled with `--no-default-features`, the crate produces a minimal no-op build with zero dependencies:

```bash
cargo build --no-default-features
# Compiles successfully with no functional code
```

### 7.5 Feature Testing

```bash
# Test with all features
cargo test --all-features

# Test with only enabled
cargo test --features enabled

# Test no-op build
cargo check --no-default-features
```

### 8. API Design Patterns

### 8.1 Builder Pattern Integration
Use `former` crate for request building:
```rust
let request = CreateResponseRequest::former()
  .model("gpt-5.1-chat-latest".to_string())
  .input(ResponseInput::String("Hello".to_string()))
  .max_output_tokens(100)
  .form();
```

### 8.2 Response Type Safety
Strongly-typed response structures:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseObject {
  pub id: String,
  pub status: ResponseStatus,
  pub output: Option<Vec<OutputItem>>,
  // ... additional fields
}
```

### 8.3 Error Propagation
Consistent error handling across all operations:
```rust
impl Client< E >
{
  pub async fn create_response(&self, request: CreateResponseRequest)
    -> Result< ResponseObject, OpenAIError >
  {
    // Implementation with proper error mapping
  }
}
```

### 9. Release and Versioning

### 9.1 Semantic Versioning
- **Major**: Breaking API changes, OpenAI API version updates
- **Minor**: New features, endpoint additions, backward-compatible changes
- **Patch**: Bug fixes, security updates, documentation improvements

### 9.2 Release Process
1. Automated testing pipeline (unit + integration)
2. Documentation generation and validation
3. Changelog generation
4. Version bumping and tagging
5. Crate publication to crates.io

### 10. Development Workflow

### 10.1 Task Management
Structured task tracking in `task/readme.md`:
- Task prioritization by advisability score
- Clear acceptance criteria
- Status tracking with proper transitions
- Outcome documentation for completed tasks

### 10.2 Code Review Requirements
- All changes must pass automated testing
- Code review for architecture and security aspects
- Documentation updates for public API changes
- Performance impact assessment

### 11. Migration and Compatibility

### 11.1 OpenAI API Version Support
- Support for latest OpenAI API version
- Graceful handling of API deprecations
- Version-specific feature flags where necessary

### 11.2 Backward Compatibility
- Semantic versioning adherence
- Deprecation warnings before breaking changes
- Migration guides for major version updates
- Legacy API support where reasonable

### Implementation Status

**Version:** 0.6
**Date:** 2025-10-08
**Status:** ✅ Production Ready - All Tasks Completed

**Task Completion:**
- **Total Tasks**: 89/89 (100% complete)
- **Test Coverage**: 683/683 tests passing (100%)
- **Code Quality**: Zero clippy warnings, zero compilation warnings
- **Specification Compliance**: Full adherence to all requirements

**Completed Features:**
- ✅ Core API endpoints (Responses, Chat, Embeddings, Files, Models, etc.)
- ✅ Advanced features (Assistants, Realtime, Vector Stores, Tool Usage, Streaming)
- ✅ Enterprise reliability (Retry logic, Circuit breaker, Rate limiting, Failover, Health checks)
- ✅ Custom base URL support (Azure OpenAI, OpenAI-compatible APIs, corporate proxies)
- ✅ Secret management (Comprehensive fallback chain with workspace_tools integration)
- ✅ Sync API variants (Blocking interface for non-async contexts)
- ✅ WebSocket streaming (Real-time bidirectional communication)
- ✅ Performance monitoring (Request metrics and observability)
- ✅ Administrative APIs (Uploads, Organizations, Users, Invites, Projects)
- ✅ Comprehensive documentation (Examples, guides, API references)

**Quality Metrics:**
- **Architecture**: "Thin Client, Rich API" principle fully implemented
- **Error Handling**: Complete error_tools integration with proper propagation
- **Testing**: Mandatory failing behavior enforced, no silent fallbacks
- **Security**: Secure credential management with audit trails
- **Performance**: Efficient async-first design with connection pooling

**Note:** All file size violations have been addressed, all refactoring tasks completed, and comprehensive testing infrastructure is in place. The crate is production-ready for enterprise deployment.

This specification serves as the authoritative source for all development decisions and implementation priorities for the `api_openai` crate.
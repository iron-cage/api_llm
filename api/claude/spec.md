# spec

- **Name:** api_claude
- **Version:** 0.8
- **Date:** 2025-11-07
- **Status:** Production Ready ✅
- **Test Coverage:** 435/435 tests passing (100%)
- **System Specification:** [../../spec.md](../../spec.md)

### Project Overview

The `api_claude` crate provides a comprehensive, type-safe Rust client library for interacting with Anthropic's Claude API services. This specification defines the architecture, requirements, and standards for the implementation.

**Architecture Decision**: This API crate is designed as a **stateless HTTP client** with no persistence requirements. All operations are direct HTTP calls to the Claude API without local data storage, caching, or state management beyond request/response handling.

**Governing Principle**: **"Thin Client, Rich API"** - Expose all server-side functionality transparently while maintaining zero client-side intelligence or **automatic** behaviors. **Key Distinction**: The principle prohibits **automatic/implicit** behaviors but explicitly **allows and encourages** **explicit/configurable** enterprise reliability features.

**Note:** This specification must be implemented in accordance with the ecosystem-wide requirements defined in the [System Specification](../../spec.md).

### Vocabulary

- **API Client**: The main library interface that coordinates all interactions with Anthropic Claude services
- **Environment**: Configuration object that encapsulates API credentials, base URLs, and connection parameters
- **Secret Management**: Secure handling of API keys using `secrecy` crate and workspace integration
- **Streaming**: Real-time delivery of generated content via Server-Sent Events
- **Tool Calling**: Function calling capabilities for integrating external tools and APIs
- **Vision Support**: Image understanding and multimodal content processing capabilities
- **Rate Limiting**: Request throttling mechanisms to manage API usage and prevent abuse
- **Circuit Breaker**: Fault tolerance pattern that prevents cascading failures by monitoring service health
- **Retry Logic**: Automatic retry mechanisms with exponential backoff for transient failures

### Scope and Objectives

### 1.1 Primary Objectives
- Provide complete coverage of Anthropic Claude API endpoints
- Ensure type safety and compile-time error detection
- Support both synchronous and asynchronous operations
- Implement robust error handling and retry mechanisms
- Maintain security best practices for credential management
- Support Server-Sent Events streaming for real-time applications

### 1.2 API Coverage Requirements
The client must support all major Anthropic Claude API endpoints:

### Core Endpoints
- **Messages API**: Create and manage conversational interactions
- **Count Tokens API**: Count message tokens before API calls for cost estimation
- **Models API**: Model listing and information retrieval
- **Content Generation**: Text generation with various parameters
- **Tool Calling**: Function calling and tool integration
- **Vision**: Image understanding capabilities
- **Streaming**: Server-sent events for incremental responses

### Advanced Features
- **System Prompts**: Separate system message handling
- **Message Roles**: User, assistant, and system role management
- **Safety Settings**: Content filtering and safety controls
- **Context Windows**: Support for large context lengths (200k+ tokens)

### 2. Architecture Design

### 2.1 Core Components

### Client Layer
```rust
pub struct Client {
    secret : Secret,
    environment : Environment,
}
```

### Environment Management
```rust
pub struct Environment {
    pub base_url : String,
    pub api_version : String,
    pub timeout : Duration,
}
```

### Secret Management
```rust
pub struct Secret(SecretString);
```
- Integration with `secrecy` crate for credential protection
- Multiple loading mechanisms: environment variables, files, workspace secrets
- Format validation for API keys (sk-ant- prefix)
- Audit trail for security monitoring

### 2.2 Module Organization

### Private Namespace Pattern
All modules must follow the wTools ecosystem `mod_interface` pattern:
```rust
mod private {}

crate::mod_interface! {
  layer client;
  layer environment;
  layer error;
  layer secret;  
  layer messages;
}
```

### Component Structure
- **Client**: `src/client.rs` - HTTP client wrapper and API operations
- **Environment**: `src/environment.rs` - Configuration management
- **Error Handling**: `src/error.rs` - Unified error types
- **Secret Management**: `src/secret.rs` - API key handling
- **Messages**: `src/messages.rs` - Message format definitions

### 2.3 Error Handling Strategy

### Error Types
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum AnthropicError {
  Api(ApiError),           // API-returned errors
  Http(String),            // HTTP transport errors  
  Network(String),         // Network connectivity issues
  Timeout(String),         // Request timeout errors
  InvalidArgument(String), // Client-side validation failures
  Authentication(String),  // API key issues
  RateLimit(String),       // Rate limiting errors
}
```

### Integration with error_tools
- Leverage `error_tools` crate for ecosystem consistency
- Automatic derive implementations where possible
- Structured error propagation

### 3. Quality Standards

### 3.1 Testing Requirements

### Unit Testing
- Minimum 80% code coverage
- Test all public APIs
- Mock external dependencies for isolated component testing
- Focus on edge cases and error conditions
- Must not test API integration behavior

### ⚠️ CRITICAL POLICY: NO MOCKING ALLOWED IN INTEGRATION TESTS ⚠️

**ZERO TOLERANCE FOR MOCK TESTING IN INTEGRATION TESTS**: This crate maintains a strict **NO MOCKING ALLOWED** policy for all integration tests. This policy applies only to integration tests and is non-negotiable.

### Integration Testing: MANDATORY STRICT FAILURE POLICY
- Feature-gated integration tests: `#![cfg(feature = "integration")]`
- **🚫 ABSOLUTE PROHIBITION**: No fake API keys, mock servers, or simulated responses
- **🚫 ZERO MOCK TOLERANCE**: Any mock usage in integration tests is considered a critical policy violation
- **✅ REAL API ONLY**: Integration tests MUST use real Anthropic API endpoints exclusively
- **💥 IMMEDIATE FAILURE REQUIREMENT**: Tests MUST fail immediately and loudly if:
  - API secrets are not available (no graceful fallbacks or silent skips)
  - Network connectivity issues occur
  - API authentication fails
  - Any API endpoint returns errors
  - Mock dependencies are detected
- **📋 DOCUMENTATION MANDATE**: Every integration test file must contain mandatory policy documentation
- **🔐 CREDENTIAL REQUIREMENT**: All integration tests must use `Client::from_workspace()` or equivalent real credential loading

### Policy Enforcement
- **Code Review Requirement**: All PRs must be reviewed for integration test mock usage violations
- **Automated Scanning**: CI/CD pipeline must scan for prohibited patterns in integration tests:
  - `sk-ant-test-` fake API key patterns in integration tests
  - Mock server implementations in integration tests
  - Hardcoded JSON responses mimicking API format in integration tests
  - Simulated error responses in integration tests
- **Violation Response**: Any discovered mock usage in integration tests requires immediate remediation

### Test Organization
```
tests/
├── tests.rs         # Component-level tests
└── inc/
    ├── basic_test.rs
    └── ...
```

### 3.2 Code Quality Standards

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

The Claude API client supports optional enterprise reliability features that enhance production robustness. These features align with the **"Thin Client, Rich API"** governing principle by requiring **explicit configuration** and **transparent operation**.

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
  .api_key("sk-ant-...")
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
  .api_key("sk-ant-...")
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
  .api_key("sk-ant-...")
  .rate_limit_requests_per_second(10.0)     // Explicitly configured
  .rate_limit_burst_size(20)                // Optional: burst configuration
  .enable_rate_limiting(true)               // Explicitly enabled
  .build()?;

// Method name clearly indicates rate limiting behavior
client.execute_with_rate_limiting(request).await?;
```

### 4.5 Failover Support

**Cargo Feature**: `failover`

**Requirements**:
- Multi-endpoint failover configuration
- Multiple failover strategies: Priority, RoundRobin, Random, Sticky
- Endpoint health tracking (Healthy, Degraded, Unhealthy, Unknown)
- Context tracking to prevent retry loops
- Thread-safe implementation

**Configuration Pattern**:
```rust
let failover_config = FailoverConfig::builder()
  .add_endpoint("https://api.anthropic.com", 1)  // Primary
  .add_endpoint("https://backup.anthropic.com", 2)  // Secondary
  .strategy(FailoverStrategy::Priority)
  .max_attempts(3)
  .build()?;

let manager = FailoverManager::new(failover_config);
let endpoint = manager.select_endpoint(context)?;
```

**Implemented**: ✅ Complete (Task 358, 4 strategies, 17 tests)

### 4.6 Health Checks

**Cargo Feature**: `health-checks`

**Requirements**:
- Stateless endpoint health monitoring (no background processes)
- Multiple strategies: Ping (HEAD request), LightweightApi (OPTIONS request)
- Response time-based status determination
- Configurable thresholds for healthy/degraded/unhealthy states
- Concurrent multi-endpoint checking capability
- Metrics aggregation (healthy/degraded/unhealthy percentages)

**Configuration Pattern**:
```rust
let config = HealthCheckConfig::builder()
  .strategy(HealthCheckStrategy::Ping)
  .timeout(Duration::from_secs(5))
  .healthy_threshold(Duration::from_millis(200))
  .degraded_threshold(Duration::from_millis(1000))
  .build()?;

let result = HealthChecker::check_endpoint(&endpoint_url, &config).await?;
```

**Implemented**: ✅ Complete (Task 359, 2 strategies, 14 tests)

### 4.7 Prompt Caching

**Implementation**: Integrated in core client (`src/client.rs`)

**Requirements**:
- CacheControl directive support for system prompts and messages
- Usage metadata tracking (cache_creation_input_tokens, cache_read_input_tokens)
- Cost savings calculation (~90% on cached tokens)
- Ephemeral cache control type
- Cache hit/miss scenario handling

**Configuration Pattern**:
```rust
let request = CreateMessageRequest::builder()
  .model("claude-sonnet-4-5-20250929")
  .max_tokens(1000)
  .system_with_cache(
    "You are an expert Rust developer...",
    CacheControl::ephemeral()
  )
  .messages(messages)
  .build();

// Check cache usage in response
if let Some(cache_read) = response.usage.cache_read_input_tokens {
  println!("Cache hit! Saved {} tokens", cache_read);
}
```

**Implemented**: ✅ Complete (Tasks 716-717, ~90% cost savings, 18 tests)

### 4.8 Enterprise Features Integration

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

**Enterprise Reliability Status**: ✅ 100% Complete (5/5 features)
- Retry Logic ✅
- Circuit Breaker ✅
- Rate Limiting ✅
- Failover ✅
- Health Checks ✅

### 4.9 Implementation Standards

**Thread Safety**:
- All enterprise features must be thread-safe for concurrent use
- Use `Arc<Mutex<>>` patterns for shared state management
- Avoid blocking operations in async contexts

**Error Handling**:
- Enterprise features must integrate with existing `AnthropicError` types
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
- Support multiple credential sources with priority order:
  1. **Workspace secrets**: `workspace_root/secret/-secrets.sh` (Primary location - see [Secret Directory Policy](../../secret/readme.md))
  2. **Environment variables**: `ANTHROPIC_API_KEY` (Fallback)
  3. **Runtime**: programmatic setting (Direct)

**Critical**: The secret directory MUST be named `secret/` at workspace root (NO dot prefix). workspace_tools 0.6.0 uses `secret/` directly.

### Validation Standards
```rust
fn validate_api_key_format(secret: &str) -> Result< (), AnthropicError > {
  // Enforce Anthropic API key format requirements
  // - Must start with "sk-ant-"
  // - Length constraints: reasonable bounds
  // - Character set validation
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
- Backpressure handling
- Graceful connection management
- Event parsing for `content_block_delta` and `message_stop` events

### 6.3 Compilation Performance
- Build times under 60 seconds for full compilation
- Incremental compilation optimization
- Minimal dependency feature usage
- Parallel compilation support

### 7. Dependency Management

### 7.1 Core Dependencies
```toml
[dependencies]
# wTools ecosystem
mod_interface = { workspace = true }
error_tools = { workspace = true }
workspace_tools = { workspace = true, features = ["secret_management"] }

# HTTP and async
reqwest = { workspace = true, features = ["json"] }
tokio = { workspace = true, features = ["fs", "macros"] }
futures-core = { workspace = true }
futures-util = { workspace = true }

# Serialization
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
serde_with = { workspace = true }
```

### 7.2 Optional Features
- `integration`: Integration test support
- `streaming`: Server-sent events support ✅
- `tools`: Tool calling support (included in default features) ✅
- `vision`: Image understanding support ✅
- `authentication`: Advanced authentication functionality ✅
- `content-generation`: Advanced content generation functionality ✅
- `model-management`: Model management functionality ✅
- `error-handling`: Enhanced error handling functionality (included in default features) ✅
- `retry-logic`: Configurable retry mechanisms ✅
- `circuit-breaker`: Circuit breaker fault tolerance ✅
- `rate-limiting`: Token bucket rate limiting ✅
- `failover`: Multi-endpoint failover support ✅
- `health-checks`: Endpoint health monitoring ✅
- `request-caching`: Prompt caching support (integrated in core) ✅
- `count-tokens`: Token counting before API calls for cost estimation ✅
- `streaming-control`: Pause/resume/cancel control for streaming operations ✅
- `compression`: HTTP request/response compression for bandwidth optimization ✅
- `enterprise-quota`: Client-side usage tracking and quota management ✅
- `dynamic-config`: Runtime configuration hot-reloading with file watching ✅
- `sync-api`: Synchronous API wrapper ✅
- `curl-diagnostics`: CURL command generation ✅
- `general-diagnostics`: Comprehensive diagnostics ✅

### 8. Tool Calling Refactoring

### 8.1 Feature Gating and Modular Design
The tool calling functionality has been comprehensively refactored with proper feature gating:

- All tool-related types are conditionally compiled under the `tools` feature
- Feature is included in default features for seamless usage
- Clean separation between core message handling and tool-specific functionality
- Backward compatibility maintained through conditional compilation

### 8.2 Enhanced Validation
Comprehensive validation has been implemented for tool calling:

### Tool Definition Validation
- **Name validation**: Alphanumeric, underscore, hyphen only; max 64 characters
- **Description validation**: Non-empty, max 1024 characters  
- **Duplicate detection**: Prevents duplicate tool names in requests
- **Schema validation**: JSON schema format validation
- **Limits enforcement**: Maximum 64 tools per request

### Tool Choice Validation
- **Reference validation**: Ensures specific tool choices reference existing tools
- **Consistency checks**: Validates tool_choice is only used with tools array
- **Format validation**: Proper tool choice structure enforcement

### Request-Level Validation
```rust
// Comprehensive validation in CreateMessageRequest::validate()
if let Some(ref tools) = self.tools {
    // Check for empty tools array
    // Validate each tool definition
    // Check for duplicate tool names
    // Enforce tool limits
}
```

### 8.3 Improved Error Handling
Enhanced error messages provide actionable feedback:
- Clear validation failure descriptions
- Specific tool name references in error messages
- Structured error types for different validation failures
- Integration with existing error handling framework

### 8.4 Helper Methods and Builder Patterns
New convenience methods for tool definition creation:
```rust
// Simple tool with no parameters
let tool = ToolDefinition::simple("calculator", "Basic math operations");

// Tool with typed parameters  
let tool = ToolDefinition::with_properties(
    "weather", 
    "Get weather information",
    serde_json::json!({
        "location": { "type": "string", "description": "City name" },
        "units": { "type": "string", "enum": ["celsius", "fahrenheit"] }
    }),
    vec!["location".to_string()]
);
```

### 9. Vision Support Refactoring

### 9.1 Feature Gating and Modular Design
The vision support functionality has been comprehensively refactored with proper feature gating:

- All vision-related types are conditionally compiled under the `vision` feature
- Feature is optional and not included in default features for minimal compilation
- Clean separation between core message handling and vision-specific functionality
- Backward compatibility maintained through conditional compilation

### 9.2 Enhanced Type System
Comprehensive type system for vision content:

### ImageContent Structure
- **Type validation**: Always "image" type with compile-time safety
- **Source management**: Integrated ImageSource handling
- **Helper constructors**: Convenient methods for common image formats
- **Validation methods**: Runtime validation with actionable error messages

### ImageSource Validation
- **Format validation**: Only "base64" source type supported
- **Media type validation**: Supports JPEG, PNG, GIF, WebP formats
- **Content validation**: Non-empty data with basic base64 format checking
- **Size estimation**: Approximate byte size calculation from base64 length

### 9.3 Builder Pattern Integration
Vision content seamlessly integrates with message builders:

```rust
#[cfg(feature = "vision")]
let message = Message::user_with_image(
    "What's in this image?",
    ImageContent::jpeg("base64_image_data_here")
);

#[cfg(feature = "vision")]
let multi_image_message = Message::user_with_images(
    "Compare these images",
    vec![
        ImageContent::jpeg("first_image"),
        ImageContent::png("second_image")
    ]
);
```

### 9.4 Validation Framework
Comprehensive validation ensures data integrity:

### Content-Level Validation
```rust
// Automatic validation during content creation
let image = ImageContent::jpeg("valid_base64_data");
image.validate()?; // Returns Result< (), error_tools::Error >

// Helper validation methods
assert!(image.is_valid());
assert_eq!(image.media_type(), "image/jpeg");
```

### Source-Level Validation
```rust
// Direct source validation
let source = ImageSource::png("image_data");
source.validate()?;

// Format and size checks
assert!(source.is_valid_base64());
let estimated_bytes = source.estimated_size_bytes();
```

### 9.5 Message Integration
Vision content works seamlessly with existing message patterns:

### Multi-Modal Messages
- Text and image content in single messages
- Multiple images per message supported
- Proper serialization to Anthropic API format
- Type-safe content enumeration

### Content Type System
- `Content::Image` variant conditionally compiled with vision feature
- Helper methods like `is_image()` feature-gated appropriately
- Consistent API patterns across all content types

### 10. Streaming Support Refactoring

### 10.1 Enhanced Error Handling and Modularity
The streaming support functionality has been comprehensively refactored with improved error handling and modularity:

- **Conditional error handling**: Supports both enhanced `AnthropicError` and simplified `error_tools::Error`
- **Feature compatibility**: Proper integration with existing error-handling feature flag
- **Enhanced validation**: Comprehensive validation for all streaming types
- **Robust parsing**: Improved SSE parsing with better error messages

### 10.2 Comprehensive Type System
Enhanced streaming type system with full validation and helper methods:

### StreamMessage Structure
- **Helper constructors**: Convenient creation methods with validation
- **Status checking**: Methods to check completion state and content presence
- **Validation framework**: Complete validation with actionable error messages
- **Content management**: Methods to manage and inspect content blocks

### StreamContentBlock Enhancements
- **Feature gating**: Tool use blocks properly gated under `tools` feature
- **Type safety**: Helper methods for content type checking and access
- **Validation**: Content-specific validation for text and tool use blocks
- **Builder patterns**: Convenient constructors for different content types

### StreamDelta Improvements
- **Type-safe construction**: Helper methods for different delta types
- **Content access**: Safe methods to access delta content
- **Validation**: Delta-specific validation with proper error handling
- **Feature compatibility**: Tool-related deltas properly feature-gated

### 10.3 Enhanced SSE Parsing
Robust Server-Sent Events parsing with comprehensive error handling:

```rust
// Parse SSE events with validation
let events = parse_sse_events(&sse_data)?;

// Enhanced event construction with validation
let event = StreamEvent::content_block_delta(0, StreamDelta::new_text("Hello"));
event.validate()?;
```

### Parsing Improvements
- **Input validation**: Parameter validation before parsing
- **Enhanced error messages**: Detailed error information with context
- **Content validation**: Automatic validation of parsed content
- **Unknown event handling**: Helpful error messages for unsupported events

### Error Resilience
- **Conditional error types**: Support for both error handling modes
- **Validation integration**: Automatic validation during parsing
- **Context preservation**: Error messages include relevant context
- **Recovery strategies**: Graceful handling of malformed events

### 10.4 Builder Pattern Integration
Streaming types seamlessly integrate with builder patterns:

```rust
// Stream message construction
let message = StreamMessage::new(
    "msg_123",
    "message", 
    "user",
    "claude-sonnet-4-5-20250929",
    usage_stats
);

// Content block creation with validation
let content_block = StreamContentBlock::new_text("Hello, world!");
content_block.validate()?;

// Delta construction
let delta = StreamDelta::new_text("incremental text");
```

### 10.5 Feature Integration
Streaming support properly integrates with other features:

### Tool Integration
- Tool use content blocks feature-gated under `tools` feature
- Tool-related deltas conditionally compiled
- Consistent patterns with message tool handling

### Error Handling Integration
- Conditional compilation based on `error-handling` feature
- Seamless fallback to `error_tools::Error` when needed
- Consistent error patterns across the crate

### Validation Framework
- Comprehensive validation for all streaming types
- Integration with existing validation patterns
- Actionable error messages for debugging

### 11. API Design Patterns

### 11.1 Client Creation
```rust
// Direct secret creation
let client = Client::new(secret)?;

// Load from environment variable (ANTHROPIC_API_KEY)
let client = Client::from_env()?;

// Load from workspace secrets (workspace_root/secret/-secrets.sh with fallback to env)
// See Secret Directory Policy: ../../secret/readme.md
let client = Client::from_workspace()?;
```

### 11.2 Message Format
Anthropic-specific message structure:
```rust
{
  "model": "claude-sonnet-4-5-20250929",
  "max_tokens": 1024,
  "system": "System prompt here",
  "messages": [
    {
      "role": "user",
      "content": "User message content"
    }
  ]
}
```

### 11.3 Streaming Response Handling
Server-Sent Events format:
```rust
data: {"type": "content_block_delta", "delta": {"text": "Hello"}}
data: {"type": "message_stop", "stop_reason": "end_turn"}
data: [DONE]
```

### 11.4 Streaming Control (streaming-control feature)

The `streaming-control` feature provides pause/resume/cancel capabilities for streaming responses.

**Architecture:**
- `StreamState`: Enum tracking stream state (Running, Paused, Cancelled)
- `StreamControl`: Thread-safe handle for controlling stream operations
- `ControlledStream<S>`: Wrapper implementing `Stream` trait with buffering

**Key Features:**
- **Client-side buffering**: Events are buffered when paused using `VecDeque`
- **Configurable buffer limits**: Oldest events dropped when buffer is full (FIFO)
- **Immediate error delivery**: Errors bypass buffering and are returned immediately
- **Thread-safe**: Uses `Arc<Mutex<>>` for state sharing across threads
- **Zero-cost when disabled**: Feature-gated to avoid overhead

**Usage Pattern:**
```rust
let (controlled_stream, control) = ControlledStream::new(inner_stream, buffer_limit);

// In one task: control the stream
control.pause()?;
// ... later ...
control.resume()?;
// ... or ...
control.cancel();

// In another task: consume the stream
while let Some(event) = controlled_stream.next().await {
    // Process events
}
```

**Implementation Details:**
- Pause: Buffers incoming events, returns `Poll::Pending` to consumer
- Resume: Delivers buffered events before polling inner stream
- Cancel: Clears buffer and stops all event delivery
- State checks: `is_paused()`, `is_cancelled()`, `is_running()`, `buffer_size()`

**Dependencies:**
- Requires `streaming` feature (depends on `StreamEvent` type)
- Located in `src/streaming_control.rs` (314 lines)

### 11.5 HTTP Compression (compression feature)

The `compression` feature provides gzip compression for request/response bodies to reduce bandwidth usage and improve performance for large prompts.

**Architecture:**
- `CompressionConfig`: Configuration for compression level and size thresholds
- `compress()`: Compress request bodies using gzip
- `decompress()`: Decompress gzip-compressed responses
- `is_gzip()`: Detect gzip-compressed data
- `add_compression_headers()`: Add appropriate HTTP headers

**Example:**
```rust
use api_claude::{ compress, decompress, CompressionConfig };

// Configure compression
let config = CompressionConfig::new()
    .with_level(6)          // 0-9, where 6 is balanced
    .with_min_size(1024);   // Only compress if >= 1KB

// Compress request body
let data = "Large prompt text...".repeat(1000);
let compressed = compress(data.as_bytes(), &config)?;

// Decompress response (if server returns compressed)
let decompressed = decompress(&compressed_response)?;
```

**Key Features:**
- ~60-80% size reduction for text content
- Configurable compression levels (0=none, 6=default, 9=best)
- Minimum size threshold to avoid compressing small data
- Only uses compression if it actually reduces size
- Automatic gzip magic number detection

**Dependencies:**
- Uses `flate2` crate for gzip compression/decompression
- Located in `src/compression.rs` (200+ lines)

### 11.6 Enterprise Quota Management (enterprise-quota feature)

The `enterprise-quota` feature provides client-side tracking and management of API usage and costs for production deployments.

**Architecture:**
- `UsageMetrics`: Tracks request counts, tokens, and costs
- `QuotaConfig`: Defines quota limits (daily/monthly)
- `CostCalculator`: Calculates costs based on model pricing
- `QuotaManager`: Enforces quotas and tracks usage
- `QuotaExceededError`: Returned when quotas are exceeded

**Example:**
```rust
use api_claude::{ QuotaManager, QuotaConfig };

// Configure quotas
let config = QuotaConfig::new()
    .with_daily_requests(1000)
    .with_daily_tokens(1_000_000)
    .with_daily_cost(50.0)          // $50/day limit
    .with_monthly_cost(1500.0);     // $1500/month limit

let manager = QuotaManager::new(config);

// Record usage before making API call
match manager.record_usage("claude-3-5-sonnet-latest", 1_000, 500) {
    Ok(()) => {
        // Quota OK, proceed with API call
    }
    Err(e) => {
        // Quota exceeded
        eprintln!("Quota exceeded: {}", e.message);
    }
}

// Export metrics
let json = manager.export_json()?;
println!("{}", json);
```

**Key Features:**
- Real-time quota enforcement before API calls
- Automatic cost calculation using current model pricing
- Per-model usage tracking
- Daily and monthly quota limits
- JSON export for monitoring systems
- Thread-safe for concurrent access

**Cost Calculation:**
- Claude 3.5 Sonnet: $3/M input, $15/M output
- Claude 3 Opus: $15/M input, $75/M output
- Claude 3 Haiku: $0.25/M input, $1.25/M output

**Dependencies:**
- Uses `parking_lot` for thread-safe RwLock
- Uses `chrono` for timestamps
- Located in `src/enterprise_quota.rs` (495 lines)

### 11.7 Dynamic Configuration (dynamic-config feature)

The `dynamic-config` feature provides runtime configuration changes with file system monitoring for zero-downtime updates.

**Architecture:**
- `RuntimeConfig`: Hot-reloadable configuration struct
- `ConfigWatcher`: File system watcher for automatic reloading
- Thread-safe updates via `Arc<RwLock<>>`
- JSON-based configuration format
- Configuration validation before applying changes

**Example:**
```rust
use api_claude::{ ConfigWatcher, RuntimeConfig };
use std::path::PathBuf;

// Create config file watcher
let config_path = PathBuf::from("/etc/claude/config.json");
let initial_config = RuntimeConfig::new();
let watcher = ConfigWatcher::new(config_path, initial_config)?;

// Get current config (thread-safe)
let config = watcher.config();
println!("Base URL: {}", config.base_url);
println!("Timeout: {:?}", config.timeout());

// Config automatically reloads when file changes
// Or manually reload
watcher.reload()?;

// Programmatic update
let new_config = RuntimeConfig {
    base_url: "https://custom.api.com".to_string(),
    timeout_ms: 120_000,
    max_retries: 5,
    ..RuntimeConfig::new()
};
watcher.update(new_config)?;
```

**Configuration Options:**
- `base_url`: API endpoint URL (default: "https://api.anthropic.com")
- `api_version`: API version string (default: "2023-06-01")
- `timeout_ms`: Request timeout in milliseconds (default: 300000)
- `enable_retry`: Enable retry logic (default: true)
- `max_retries`: Maximum retry attempts (default: 3, max: 10)
- `enable_circuit_breaker`: Enable circuit breaker (default: true)
- `circuit_breaker_threshold`: Failure threshold (default: 5)
- `enable_rate_limiting`: Enable rate limiting (default: false)
- `rate_limit_rps`: Requests per second limit (default: 10)

**Key Features:**
- Zero-downtime configuration updates
- Automatic file watching and reloading
- Configuration validation before applying
- A/B testing different configurations
- Thread-safe concurrent access
- JSON configuration format

**Dependencies:**
- Uses `notify` crate for file system watching
- Uses `parking_lot` for thread-safe RwLock
- Located in `src/dynamic_config.rs` (300+ lines)

### 12. Release and Versioning

### 12.1 Semantic Versioning
- **Major**: Breaking API changes, Anthropic API version updates
- **Minor**: New features, endpoint additions, backward-compatible changes
- **Patch**: Bug fixes, security updates, documentation improvements

### 12.2 Release Process
1. Automated testing pipeline (unit + integration)
2. Documentation generation and validation
3. Changelog generation
4. Version bumping and tagging
5. Crate publication to crates.io

### 13. Development Workflow

### 13.1 Task Management
Structured task tracking following wTools ecosystem patterns:
- Task prioritization by advisability score
- Clear acceptance criteria
- Status tracking with proper transitions
- Outcome documentation for completed tasks

### 13.2 Code Review Requirements
- All changes must pass automated testing
- Code review for architecture and security aspects
- Documentation updates for public API changes
- Performance impact assessment

### 14. Migration and Compatibility

### 14.1 Anthropic API Version Support
- Support for latest Anthropic Claude API version
- Graceful handling of API deprecations
- Version-specific feature flags where necessary

### 14.2 Backward Compatibility
- Semantic versioning adherence
- Deprecation warnings before breaking changes
- Migration guides for major version updates
- Legacy API support where reasonable

### Implementation Status

As of version 0.7 (2025-10-07):

**✅ Completed Features (Production Ready)**:
- ✅ Core message API (complete HTTP implementation, comprehensive message handling)
- ✅ Secret management (workspace integration, environment variables, validation)
- ✅ Tool calling (20 tests, complete implementation with validation)
- ✅ Vision support (8 tests, multimodal content processing)
- ✅ Streaming responses (5 tests, SSE with tool calling and vision)
- ✅ Prompt caching (18 tests, ~90% cost savings, tasks 716-717)
- ✅ Dynamic configuration (20+ tests, workspace + environment)
- ✅ Enterprise reliability (100% complete):
  - ✅ Retry logic (exponential backoff with jitter)
  - ✅ Circuit breaker (fault tolerance pattern)
  - ✅ Rate limiting (token bucket algorithm)
  - ✅ Failover (4 strategies, 17 tests, task 358)
  - ✅ Health checks (2 strategies, 14 tests, task 359)
- ✅ Sync API (synchronous wrapper for blocking use cases)
- ✅ Diagnostics (CURL generation, comprehensive error diagnostics)

**Test Coverage**: 435/435 tests passing (100%)
- Unit tests
- Integration tests (real API)
- Serialization validation
- Error handling
- Performance benchmarks

**API Feature Coverage**: 62% (26/42 API functionality features implemented)
**Cargo Features**: 21 feature flags defined (see Cargo.toml `[features]` section)
**Production Status**: ✅ Production Ready

**✅ Recently Completed (2025-10-11)**:
- Batch Messages API (tasks 706-707): COMPLETE
  - ✅ Complete batch type system with validation
  - ✅ HTTP client methods (create, retrieve, list, cancel)
  - ✅ Comprehensive test suite (9 tests, 5 integration tests)
  - ✅ All tests passing with zero regressions
- Enhanced Retry Logic (tasks 700-701): COMPLETE
  - ✅ Anthropic rate limit header integration (6 headers)
  - ✅ Server-provided retry-after duration support
  - ✅ Rate limit info parsing (requests/tokens remaining, limits, reset times)
  - ✅ Usage percentage calculations for intelligent retry decisions
  - ✅ Comprehensive test suite (16 tests: header parsing, retry logic, integration)
  - ✅ Error handling improvements (8 new error classification tests)

**🔄 Planned Future Enhancements**:
- Enhanced Circuit Breaker (tasks 702-703): Advanced failure tracking (moved to backlog - conflicts with explicit patterns)
- Enhanced Rate Limiting (tasks 704-705): API header-based rate limit tracking (moved to backlog - conflicts with explicit patterns)
- Enterprise Method Integration (tasks 710-711): Unified enterprise feature methods
- Streaming Control (task 361): Enhanced SSE control operations
- Model Tuning (task 364): Fine-tuning capabilities (managed service only)
- Model Deployment (task 365): Model hosting features (managed service only)

**Known API Limitations**:
- ❌ Embeddings: Not offered by Anthropic (architecture ready for future)
- ❌ WebSocket: Anthropic API uses SSE only (tasks 360 moved to backlog)
- ❌ Audio: Not available in Anthropic API
- ❌ Model Tuning/Deployment: Managed service only (tasks 364-365 moved to backlog)

This specification serves as the authoritative source for all development decisions and implementation priorities for the `api_claude` crate.
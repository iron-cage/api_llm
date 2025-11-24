# spec

- **Name:** api_xai
- **Version:** 0.1
- **Date:** 2025-11-08
- **Status:** In Development
- **System Specification:** [../../spec.md](../../spec.md)

### Project Overview

The `api_xai` crate provides a comprehensive, type-safe Rust client library for interacting with X.AI's Grok API services. This specification defines the architecture, requirements, and standards for the implementation.

**Architecture Decision**: This API crate is designed as a **stateless HTTP client** with no persistence requirements. All operations are direct HTTP calls to the X.AI API without local data storage, caching, or state management beyond request/response handling.

**Governing Principle**: **"Thin Client, Rich API"** - Expose all server-side functionality transparently while maintaining zero client-side intelligence or **automatic** behaviors. **Key Distinction**: The principle prohibits **automatic/implicit** behaviors but explicitly **allows and encourages** **explicit/configurable** enterprise reliability features.

**API Compatibility**: The X.AI Grok API is OpenAI-compatible, using the same REST endpoint patterns and request/response formats as OpenAI's API, but with a simplified feature set focused on core conversational AI capabilities.

**Note:** This specification must be implemented in accordance with the ecosystem-wide requirements defined in the [System Specification](../../spec.md).

### Vocabulary

- **API Client**: The main library interface that coordinates all interactions with X.AI Grok services
- **Environment**: Configuration object that encapsulates API credentials, base URLs, and connection parameters
- **Secret Management**: Secure handling of API keys using `secrecy` crate and workspace integration
- **Streaming**: Real-time delivery of generated content via Server-Sent Events (SSE)
- **Tool Calling**: Function calling capabilities for integrating external tools and APIs
- **Rate Limiting**: Request throttling mechanisms to manage API usage and prevent abuse
- **Circuit Breaker**: Fault tolerance pattern that prevents cascading failures by monitoring service health
- **Retry Logic**: Automatic retry mechanisms with exponential backoff for transient failures
- **Grok**: X.AI's large language model optimized for conversational AI and reasoning

### Scope and Objectives

### 1.1 Primary Objectives

- Provide complete coverage of X.AI Grok API endpoints
- Ensure type safety and compile-time error detection
- Support asynchronous operations (sync API optional for future)
- Implement robust error handling and retry mechanisms
- Maintain security best practices for credential management
- Support Server-Sent Events streaming for real-time applications

### 1.2 API Coverage Requirements

The client must support all major X.AI Grok API endpoints:

### Core Endpoints

- **Chat Completions**: Conversational AI interactions (`/v1/chat/completions`)
- **Models**: Model listing and information retrieval (`/v1/models`)
- **Streaming**: Server-sent events for incremental responses

### Advanced Features

- **Tool Calling**: Function calling and tool integration
- **System Prompts**: Separate system message handling
- **Message Roles**: User, assistant, and system role management
- **Multi-turn Conversations**: Conversation history management

### Out of Scope

The following OpenAI features are NOT supported by X.AI API and will not be implemented:
- Vision/image inputs
- Audio processing (Whisper, TTS)
- Embeddings generation
- Fine-tuning
- Assistants API
- File uploads
- Image generation (DALL-E)

### 2. Architecture Design

### 2.1 Core Components

### Client Layer

```rust
pub struct Client
{
  environment: Environment,
}
```

**Base URL Configuration:**

```rust
// Official X.AI API (default)
let env = Environment::build(
  secret,
  "https://api.x.ai/v1".to_string(),
  Duration::from_secs(30),
)?;

// Custom endpoint (for testing or proxies)
let env = Environment::build(
  secret,
  "http://localhost:8080/v1".to_string(),
  Duration::from_secs(60),
)?;
```

### Environment Management

```rust
pub struct Environment
{
  pub secret: Secret,
  pub base_url: String,
  pub timeout: Duration,
}
```

- Support for custom base URLs (testing, proxies)
- Configurable request timeouts
- URL validation with error handling
- Default: `https://api.x.ai/v1`

### Secret Management

```rust
pub struct Secret( SecretString );
```

- Integration with `secrecy` crate for credential protection
- Multiple loading mechanisms using workspace_tools fallback chain
- Format validation for API keys (xai-* prefix expected)
- Audit trail for security monitoring

### 2.2 Module Organization

### Private Namespace Pattern

All modules must follow the wTools ecosystem `mod_interface` pattern:

```rust
mod private {}

crate::mod_interface!
{
  exposed use ::client;
  exposed use ::environment;
  exposed use ::error;
  exposed use ::secret;
  exposed use ::chat;
  exposed use ::models;
}
```

### Component Structure

- **Client**: `src/client.rs` - HTTP client wrapper and API operations
- **Environment**: `src/environment.rs` - Configuration management
- **Error Handling**: `src/error.rs` - Unified error types
- **Secret Management**: `src/secret.rs` - API key handling
- **Chat**: `src/chat.rs` - Chat completion types and implementations
- **Models**: `src/models.rs` - Model listing and information
- **Streaming**: `src/streaming.rs` - SSE streaming support (feature-gated)
- **Tools**: `src/tools.rs` - Function calling types (feature-gated)

### 2.3 Error Handling Strategy

### Error Types

```rust
#[ derive( Debug, Clone, PartialEq ) ]
pub enum XaiError
{
  Api( ApiError ),           // API-returned errors
  Http( String ),            // HTTP transport errors
  Network( String ),         // Network connectivity issues
  Timeout( String ),         // Request timeout errors
  InvalidArgument( String ), // Client-side validation failures
  Authentication( String ),  // API key issues
  RateLimit( String ),       // Rate limiting errors
  Streaming( String ),       // SSE streaming errors
}
```

### Integration with error_tools

- Leverage `error_tools` crate for ecosystem consistency
- Automatic derive implementations where possible
- Structured error propagation
- Clear error messages with actionable context

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

- Feature-gated integration tests: `#![ cfg( feature = "integration" ) ]`
- **🚫 ABSOLUTE PROHIBITION**: No fake API keys, mock servers, or simulated responses
- **🚫 ZERO MOCK TOLERANCE**: Any mock usage in integration tests is considered a critical policy violation
- **✅ REAL API ONLY**: Integration tests MUST use real X.AI API endpoints exclusively
- **💥 IMMEDIATE FAILURE REQUIREMENT**: Tests MUST fail immediately and loudly if:
  - API secrets are not available (no graceful fallbacks or silent skips)
  - Network connectivity issues occur
  - API authentication fails
  - Any API endpoint returns errors
  - Mock dependencies are detected
- **📋 DOCUMENTATION MANDATE**: Every integration test file must contain mandatory policy documentation
- **🔐 CREDENTIAL REQUIREMENT**: All integration tests must use real credential loading

### Policy Enforcement

- **Code Review Requirement**: All PRs must be reviewed for integration test mock usage violations
- **Automated Scanning**: CI/CD pipeline must scan for prohibited patterns in integration tests
- **Violation Response**: Any discovered mock usage in integration tests requires immediate remediation

### Test Organization

```
tests/
├── integration_tests.rs   # Real API tests (feature-gated)
├── unit_tests.rs          # Component tests
└── manual/
    └── readme.md          # Manual testing procedures
```

### 3.2 Code Quality Standards

### File Size Limits

- Maximum 1,500 lines per source file (hard limit per codebase_hygiene rulebook)
- Target: <1,000 lines per file (soft limit per codebase_hygiene rulebook)
- Split oversized files into logical components

### Documentation Requirements

- All public APIs must have rustdoc comments
- Include usage examples in documentation
- Maintain up-to-date README with examples
- API reference documentation generation

### Linting and Formatting

- Custom clippy configuration for project-specific rules
- Strict adherence to wTools codestyle patterns
- Zero-warning builds required
- **FORBIDDEN**: Never use `cargo fmt` (custom codestyle only per rulebook)

### 4. Enterprise Reliability Features

### 4.1 Overview

The X.AI API client supports optional enterprise reliability features that enhance production robustness. These features align with the **"Thin Client, Rich API"** governing principle by requiring **explicit configuration** and **transparent operation**.

**Key Requirements**:

- **Feature Gating**: All enterprise features behind cargo features (`retry`, `circuit_breaker`, `rate_limiting`, `failover`, `health_checks`, `enhanced_tools`, `structured_logging`)
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
  .api_key( "xai-..." )
  .max_retries( 3 )                    // Explicitly configured
  .enable_retry_logic( true )          // Explicitly enabled
  .retry_backoff_multiplier( 2.0 )     // Optional: backoff configuration
  .build()?;

// Method name clearly indicates retry behavior
client.execute_with_retries( request ).await?;
```

### 4.3 Circuit Breaker Pattern

**Cargo Feature**: `circuit_breaker`

**Requirements**:

- Configurable failure thresholds and timeout periods
- State transitions: Closed → Open → Half-Open → Closed
- Automatic recovery testing with configurable success thresholds
- Thread-safe implementation using `Arc< Mutex<> >` patterns

**Configuration Pattern**:

```rust
let client = Client::builder()
  .api_key( "xai-..." )
  .circuit_breaker_failure_threshold( 5 )     // Explicitly configured
  .circuit_breaker_timeout( Duration::from_secs( 60 ) )
  .enable_circuit_breaker( true )             // Explicitly enabled
  .build()?;

// Method name clearly indicates circuit breaker behavior
client.execute_with_circuit_breaker( request ).await?;
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
  .api_key( "xai-..." )
  .rate_limit_requests_per_second( 10.0 )     // Explicitly configured
  .rate_limit_burst_size( 20 )                // Optional: burst configuration
  .enable_rate_limiting( true )               // Explicitly enabled
  .build()?;

// Method name clearly indicates rate limiting behavior
client.execute_with_rate_limiting( request ).await?;
```

### 4.5 Failover Support

**Cargo Feature**: `failover`

**Status**: ✅ Implemented

**Implementation**:

- **FailoverManager**: Thread-safe endpoint health tracking using `Arc<Mutex<FailoverState>>`
- **Endpoint Health States**: Healthy → Degraded → Unhealthy transitions based on failure counts
- **Automatic Rotation**: Optional auto-rotation when endpoints become unhealthy
- **Cooldown Period**: Configurable retry delay for unhealthy endpoints
- **Manual Control**: Explicit `rotate()` method for application-controlled failover

**Configuration Pattern**:

```rust
use api_xai::{ Client, FailoverConfig };
use std::time::Duration;

let failover_config = FailoverConfig::default()
  .with_max_failures( 3 )                      // Failures before unhealthy
  .with_retry_after( Duration::from_secs( 60 ) )  // Cooldown period
  .with_auto_rotate( true );                   // Auto-rotate on unhealthy

let client = Client::build( env )?
  .with_failover_config(
    vec![
      "https://api.x.ai/v1/".to_string(),
      "https://api-backup.x.ai/v1/".to_string(),
    ],
    failover_config
  );

// Check endpoint health
if let Some( ref manager ) = client.failover_manager
{
  let health = manager.endpoint_health();
  let current = manager.current_endpoint();
}
```

**Design Decisions**:

- **Sequential Rotation**: Implements round-robin rotation through endpoint list (simple, predictable)
- **Degraded State**: Allows detection of intermittent issues without immediate failover
- **Explicit Control**: Auto-rotation disabled by default, following "Thin Client" principle
- **No Strategy Pattern**: Single rotation strategy keeps implementation simple and focused

### 4.6 Health Checks

**Cargo Feature**: `health_checks`

**Status**: ✅ Implemented

**Implementation**:

- **health_check()**: Full health check with 3-state response (Healthy/Degraded/Unhealthy)
- **liveness_check()**: Kubernetes-style liveness probe (always true for stateless client)
- **readiness_check()**: Kubernetes-style readiness probe (checks API endpoint availability)
- **Probe Method**: Uses `list_models()` API call (lightweight, auth-validating, no side effects)
- **Performance Threshold**: 2000ms response time threshold for Degraded state (hardcoded)

**Usage Pattern**:

```rust
use api_xai::{ health_check, liveness_check, readiness_check };

// Full health check with timing and status
let result = health_check( &client ).await;
match result.status
{
  HealthStatus::Healthy => println!( "OK: {}ms", result.response_time_ms ),
  HealthStatus::Degraded => println!( "SLOW: {}ms", result.response_time_ms ),
  HealthStatus::Unhealthy => println!( "FAILED: {:?}", result.message ),
}

// Kubernetes-style probes (boolean)
let is_alive = liveness_check( &client ).await;     // Always true
let is_ready = readiness_check( &client ).await;    // Checks API health
```

**Design Decisions**:

- **list_models() as Probe**: Lightweight endpoint that validates auth without consuming quota
- **3-State Model**: Healthy (< 2s), Degraded (≥ 2s), Unhealthy (failed) for gradual degradation
- **Hardcoded Threshold**: 2000ms threshold keeps API simple, following "Thin Client" principle
- **No Background Probing**: Stateless design - application controls when health checks run
- **Kubernetes Alignment**: Liveness and readiness probes follow K8s conventions

### 4.7 Enhanced Function Calling

**Cargo Feature**: `enhanced_tools`

**Status**: ✅ Implemented

**Implementation**:

- **execute_tool_calls_parallel()**: Concurrent execution of independent tool calls using tokio::spawn
- **execute_tool_calls_sequential()**: Ordered execution for dependent tool calls
- **ToolResult**: Helper struct for standardized tool results with error constructors
- **Independent Error Handling**: Individual tool failures don't stop batch execution

**Usage Pattern**:

```rust
use api_xai::{ execute_tool_calls_parallel, ToolResult };

// Parallel execution for 3x+ speedup on independent tools
let results = execute_tool_calls_parallel(
  tool_calls,
  | call | async move {
    match call.function.name.as_str()
    {
      "get_weather" => Ok( ToolResult::new( call.id, &weather_data ) ),
      "get_time" => Ok( ToolResult::new( call.id, &time_data ) ),
      _ => Err( format!( "Unknown function" ).into() )
    }
  }
).await;

// Results are Vec<Result<ToolResult, Error>> for independent error handling
for result in results
{
  match result
  {
    Ok( tool_result ) => println!( "Success: {}", tool_result.result ),
    Err( e ) => eprintln!( "Tool failed: {}", e ),
  }
}
```

**Design Decisions**:

- **Separate Functions**: Parallel and sequential as distinct functions for type safety and clarity
- **tokio::spawn**: True concurrent execution with error isolation
- **Owned ToolCall**: Executor receives ownership for 'static lifetime compatibility
- **Generic Executor**: Accepts any `Fn(ToolCall) -> Future` for maximum flexibility
- **Partial Results**: Failed tools don't stop execution - maximizes useful work

### 4.8 Structured Logging

**Cargo Feature**: `structured_logging`

**Status**: ✅ Implemented

**Implementation**:

- **tracing Integration**: Uses `tracing` crate for structured event logging
- **Domain Macros**: `log_request!`, `log_response!`, `log_error!`, `log_retry!`, `log_circuit_breaker_state!`, `log_failover!`, `log_rate_limit!`
- **Zero Overhead**: Macros with `#[cfg(feature = "structured_logging")]` guards - compile-time elimination when disabled
- **Span Helpers**: `create_operation_span()` for tracking operation lifecycles

**Usage Pattern**:

```rust
use api_xai::{ log_request, log_response, log_error, create_operation_span };

// Log API request
log_request!( "POST", "/chat/completions", Some( "grok-2-1212" ) );

// Create span for operation tracking
let span = create_operation_span( "chat_completion", Some( "grok-2-1212" ) );
let _guard = span.enter();

// Log response with timing
log_response!( 200, 145 );  // Status, duration_ms

// Log errors
log_error!( "RateLimit", "Rate limit exceeded" );

// Enterprise feature events
log_retry!( 2, 5, 1000 );  // Attempt, max, delay_ms
log_circuit_breaker_state!( "Closed", "Open", "Threshold reached" );
log_failover!( "https://api.x.ai/v1/", "https://backup.x.ai/v1/", "Primary unhealthy" );
```

**Design Decisions**:

- **Macros not Functions**: Compile-time elimination for zero overhead when feature disabled
- **tracing not log**: Structured events instead of string-based logging
- **Feature-Gated Macros**: `#[cfg]` guards ensure no runtime cost when disabled
- **Domain-Specific**: Macros tailored to API client events, not generic logging
- **No Configuration**: Simple API following "Thin Client" principle

### 4.9 Count Tokens

**Cargo Feature**: `count_tokens`

**Status**: ✅ Implemented

**Implementation**:

- **count_tokens()**: Count tokens in text string using tiktoken
- **count_tokens_for_request()**: Count tokens in ChatCompletionRequest including message overhead
- **validate_request_size()**: Validate request fits within context window
- **Model Mapping**: Grok models (grok-3, grok-beta, grok-2) → GPT-4 tokenization (cl100k_base)

**Usage Pattern**:

```rust
use api_xai::{ count_tokens, count_tokens_for_request, validate_request_size };

// Count tokens in text
let count = count_tokens( "Hello, world!", "grok-2-1212" )?;

// Count tokens in full request
let request = ChatCompletionRequest::former()
  .model( "grok-2-1212".to_string() )
  .messages( vec![ Message::user( "Hello!" ) ] )
  .form();

let total_tokens = count_tokens_for_request( &request )?;

// Validate against context window (131K for Grok-3)
validate_request_size( &request, 131072 )?;
```

**Design Decisions**:

- **tiktoken Library**: Uses `tiktoken-rs` for accurate token counting compatible with OpenAI/XAI
- **Local Counting**: No API calls needed - fast offline computation
- **Message Overhead**: Accounts for 4 tokens per message + 3 for reply priming
- **Model Compatibility**: XAI uses same tokenization as OpenAI GPT-4

### 4.10 Cached Content

**Cargo Feature**: `caching`

**Status**: ✅ Implemented

**Implementation**:

- **CachedClient**: Wrapper client with LRU cache for responses
- **cached_create()**: Makes request with automatic caching
- **Cache Key**: Computed from JSON-serialized request for correctness
- **Capacity**: Configurable LRU capacity with automatic eviction

**Usage Pattern**:

```rust
use api_xai::{ CachedClient, Client };

let client = Client::build( env )?;
let cached_client = CachedClient::new( client, 100 ); // Cache 100 responses

let request = ChatCompletionRequest::former()
  .model( "grok-2-1212".to_string() )
  .messages( vec![ Message::user( "Hello!" ) ] )
  .form();

// First call: hits API
let response1 = cached_client.cached_create( request.clone() ).await?;

// Second call: hits cache (instant, no API call)
let response2 = cached_client.cached_create( request ).await?;
```

**Design Decisions**:

- **LRU Eviction**: Least Recently Used policy for bounded memory
- **No Streaming Cache**: Streaming requests bypass cache (responses are incremental)
- **Thread-Safe**: Arc<Mutex<>> for concurrent access
- **Zero Persistence**: Pure in-memory cache, no disk storage

### 4.11 Input Validation

**Cargo Feature**: `input_validation`

**Status**: ✅ Implemented

**Implementation**:

- **validate_request()**: Comprehensive request parameter validation
- **validate_model()**: Check model name against known XAI models
- **validate_messages()**: Ensure non-empty messages with non-empty content
- **validate_temperature/top_p/penalties()**: Range validation
- **validate_tools()**: Function calling schema validation

**Usage Pattern**:

```rust
use api_xai::{ validate_request, ChatCompletionRequest, Message };

let request = ChatCompletionRequest::former()
  .model( "grok-2-1212".to_string() )
  .messages( vec![ Message::user( "Hello!" ) ] )
  .temperature( Some( 0.7 ) )
  .form();

// Validate before sending to API
validate_request( &request )?; // Returns error if invalid
```

**Validation Rules**:

- **Model**: Must be grok-3, grok-beta, or grok-2
- **Messages**: Non-empty array with non-empty content
- **Temperature**: Range [0.0, 2.0]
- **Max Tokens**: Positive values
- **Top P**: Range [0.0, 1.0]
- **Frequency/Presence Penalty**: Range [-2.0, 2.0]
- **Tools**: Valid JSON schemas for function definitions

**Design Decisions**:

- **Client-Side**: Catch errors before API calls for better UX
- **Explicit Opt-In**: Validation must be called explicitly, not automatic
- **Fail Fast**: Returns first error encountered
- **Non-Mutating**: Never modifies requests

### 4.12 CURL Diagnostics

**Cargo Feature**: `curl_diagnostics`

**Status**: ✅ Implemented

**Implementation**:

- **to_curl()**: Convert request to CURL command with environment variable
- **to_curl_with_key()**: Convert with embedded API key (WARNING: security risk)
- **to_curl_with_endpoint()**: Convert with custom endpoint URL
- **to_curl_compact()**: Single-line compact format

**Usage Pattern**:

```rust
use api_xai::{ to_curl, ChatCompletionRequest, Message };

let request = ChatCompletionRequest::former()
  .model( "grok-2-1212".to_string() )
  .messages( vec![ Message::user( "Hello!" ) ] )
  .form();

// Generate CURL command (uses $XAI_API_KEY environment variable)
let curl_command = to_curl( &request );
println!( "{}", curl_command );

// Output:
// curl -X POST https://api.x.ai/v1/chat/completions \
//   -H "Authorization: Bearer $XAI_API_KEY" \
//   -H "Content-Type: application/json" \
//   -d '{ ... }'
```

**Design Decisions**:

- **Security First**: Default uses $XAI_API_KEY to prevent key exposure
- **Multi-Line Format**: Readable with backslash continuation
- **Pretty JSON**: Indented request body
- **Testing Focus**: Primary use case is debugging and issue reproduction

### 4.13 Batch Operations

**Cargo Feature**: `batch_operations`

**Status**: ✅ Implemented

**Implementation**:

- **BatchProcessor**: Client-side parallel request orchestration
- **process_batch()**: Execute multiple requests with concurrency limit
- **process_batch_with_progress()**: Same with progress callbacks
- **Semaphore-Based**: Uses tokio::sync::Semaphore for rate limiting

**Usage Pattern**:

```rust
use api_xai::{ BatchProcessor, Client, ChatCompletionRequest, Message };

let client = Client::build( env )?;
let processor = BatchProcessor::new( client, 5 ); // Max 5 concurrent

let requests = vec!
[
  ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Request 1" ) ] )
    .form(),
  ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Request 2" ) ] )
    .form(),
];

// Process batch with concurrency control
let results = processor.process_batch( requests ).await;

for ( idx, result ) in results.iter().enumerate()
{
  match result
  {
    Ok( response ) => println!( "Request {}: Success", idx ),
    Err( e ) => println!( "Request {}: Failed - {}", idx, e ),
  }
}
```

**Design Decisions**:

- **Client-Side Only**: XAI API doesn't provide native batch endpoint
- **Controlled Concurrency**: Semaphore prevents API overload
- **Partial Success**: One failure doesn't stop other requests
- **In-Order Results**: Results match input request order

### 4.14 Performance Metrics

**Cargo Feature**: `performance_metrics`

**Status**: ✅ Implemented

**Implementation**:

- **MetricsCollector**: Prometheus-compatible metrics collection
- **record_request()**: Record request duration, tokens, and success/failure
- **export()**: Export metrics in Prometheus text format
- **MetricGuard**: RAII pattern for automatic metric recording

**Metrics Collected**:

- `xai_requests_total` - Total number of requests
- `xai_request_duration_seconds` - Request latency histogram
- `xai_tokens_total` - Total tokens consumed
- `xai_errors_total` - Total number of errors

**Usage Pattern**:

```rust
use api_xai::{ MetricsCollector, MetricGuard };
use std::sync::Arc;

let metrics = Arc::new( MetricsCollector::new() );

// Manual recording
metrics.record_request
(
  Duration::from_millis( 250 ),
  1500, // tokens
  true  // success
);

// Automatic recording with RAII
{
  let mut guard = MetricGuard::new( metrics.clone() );

  // ... make API request ...

  guard.set_tokens( 1500 );
  guard.set_success();

  // Metrics recorded automatically on drop
}

// Export for Prometheus scraping
let prometheus_text = metrics.export();
```

**Design Decisions**:

- **Prometheus Standard**: Uses official Prometheus client library
- **Pull-Based**: Application exposes /metrics endpoint, no push required
- **Zero Overhead**: Feature-gated, completely removed when disabled
- **Production Focus**: Essential for monitoring in production deployments

### 4.15 Sync API

**Cargo Feature**: `sync_api`

**Status**: ✅ Implemented (NOT RECOMMENDED)

**⚠️ Design Note**: This feature contradicts Rust async-first design principles and is only provided for compatibility with legacy synchronous codebases. New code should use the async API.

**Implementation**:

- **SyncClient**: Blocking wrapper around async Client
- **create()**: Blocking chat completion
- **list_models()**: Blocking model listing
- **get_model()**: Blocking model retrieval
- **sync_count_tokens()**: Blocking token counting (requires count_tokens feature)
- **sync_count_tokens_for_request()**: Blocking request token counting
- **sync_validate_request_size()**: Blocking size validation
- **SyncCachedClient**: Blocking wrapper for cached operations (requires caching feature)

**Usage Pattern**:

```rust
use api_xai::{ SyncClient, Client, ChatCompletionRequest, Message };

let client = Client::build( env )?;
let sync_client = SyncClient::new( client )?; // Creates tokio Runtime

let request = ChatCompletionRequest::former()
  .model( "grok-2-1212".to_string() )
  .messages( vec![ Message::user( "Hello!" ) ] )
  .form();

// Blocking call (ties up thread)
let response = sync_client.create( request )?;
```

**Design Decisions**:

- **Runtime Ownership**: Each SyncClient owns a tokio::runtime::Runtime
- **Block-on Pattern**: Uses runtime.block_on() to execute async operations
- **Feature Combinations**: Supports sync wrappers for count_tokens and caching features
- **NOT RECOMMENDED**: Documented as non-idiomatic, async API preferred

**Why NOT Recommended**:

- Async is standard in Rust HTTP clients
- Blocks executor threads (performance overhead)
- Runtime creation cost per client instance
- Doesn't compose well with async code
- Better approach: Application-level runtime with block_on() when needed

### 4.16 Implementation Standards

**Thread Safety**:

- All enterprise features must be thread-safe for concurrent use
- Use `Arc< Mutex<> >` patterns for shared state management
- Avoid blocking operations in async contexts

**Error Handling**:

- Enterprise features must integrate with existing `XaiError` types
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
  2. **Environment variables**: `XAI_API_KEY` (Fallback)
  3. **Alternative secret files**: `secrets.sh`, `.env` (Compatibility)
  4. **Runtime**: programmatic setting with validation (Direct)
- Automatic format detection: `KEY=VALUE` and `export KEY=VALUE` formats

### Validation Standards

```rust
fn validate_api_key_format( secret: &str ) -> Result< (), XaiError >
{
  // Enforce X.AI API key format requirements
  // - Expected prefix: "xai-" (may vary based on actual format)
  // - Length constraints: TBD based on actual API keys
  // - Character set: alphanumeric + underscore + hyphen
}
```

### Secret Loading API

```rust
impl Secret
{
  // Recommended: Comprehensive fallback chain using workspace_tools
  pub fn load_with_fallbacks( key_name: &str ) -> Result< Self >;

  // Environment variable loading
  pub fn load_from_env( env_var: &str ) -> Result< Self >;

  // Workspace-specific loading
  pub fn load_from_workspace( key_name: &str, filename: &str ) -> Result< Self >;

  // Direct validation and creation
  pub fn new( secret: String ) -> Result< Self >;
  pub fn new_unchecked( secret: String ) -> Self;
}
```

### 5.2 Network Security

- TLS/HTTPS enforcement for all communications
- Certificate validation
- Request signing and authentication
- Rate limiting and retry with exponential backoff

### 6. Performance Requirements

### 6.1 Async-First Design

- All I/O operations must be asynchronous
- Support for `tokio` runtime
- Efficient connection pooling via reqwest
- Minimal allocation overhead

### 6.2 Streaming Support

- Server-sent events for incremental responses
- Backpressure handling
- Graceful connection management
- Efficient buffer management

### 6.3 Compilation Performance

- Build times under 30 seconds for full compilation
- Incremental compilation optimization
- Minimal dependency feature usage
- Parallel compilation support

### 7. Dependency Management

### 7.1 Dependency Strategy: Optional Dependencies with enabled/full Pattern

**Architecture Decision:** This crate follows the **mandatory enabled/full feature pattern** from `crate_distribution.rulebook.md`. ALL dependencies MUST be declared as `optional = true` and activated exclusively through Cargo features.

**Rationale**:

- Enables true no-op compilation when all features disabled
- Prevents unnecessary compilation overhead when crate is a dependency
- Follows wTools ecosystem enabled/full pattern
- Allows fine-grained control over what gets compiled

### 7.2 Feature Configuration

```toml
[ features ]
default = [ "full" ]

# Master switch - activates all core dependencies
enabled = [
  # wTools ecosystem
  "dep:mod_interface",
  "dep:error_tools",
  "dep:workspace_tools",

  # Serialization and data handling
  "dep:serde",
  "dep:serde_json",
  "dep:secrecy",

  # Async execution
  "dep:futures-core",
  "dep:futures-util",
  "dep:tokio",

  # HTTP
  "dep:reqwest",
]

# Convenience feature - enables everything
full = [
  "enabled",
  "integration",
  "streaming",
  "tool_calling",
  "retry",
  "circuit_breaker",
  "rate_limiting",
  "failover",
  "enhanced_tools",
  "health_checks",
  "structured_logging",
  "count_tokens",
  "caching",
  "input_validation",
  "curl_diagnostics",
  "batch_operations",
  "performance_metrics",
  "sync_api",
]

# Additional features
integration = []
streaming = [ "dep:eventsource-stream", "dep:bytes" ]
tool_calling = []
retry = []
circuit_breaker = []
rate_limiting = [ "tokio/time" ]
failover = []
enhanced_tools = []
health_checks = []
structured_logging = [ "dep:tracing" ]

# Client-Side Enhancement Features
count_tokens = [ "dep:tiktoken-rs" ]
caching = [ "dep:lru" ]
input_validation = []
curl_diagnostics = []
batch_operations = [ "tokio/sync" ]
performance_metrics = [ "dep:prometheus" ]
sync_api = [ "tokio/rt-multi-thread" ]
```

### 7.3 Dependency Declarations

ALL dependencies MUST be declared with `optional = true`:

```toml
[ dependencies ]
# wTools ecosystem - all optional
mod_interface = { workspace = true, optional = true }
error_tools = { workspace = true, optional = true }
workspace_tools = { workspace = true, features = [ "secrets" ], optional = true }

# Async - all optional
tokio = { workspace = true, features = [ "macros" ], optional = true }
futures-core = { workspace = true, optional = true }
futures-util = { workspace = true, optional = true }

# HTTP - all optional
reqwest = { workspace = true, features = [ "json", "rustls-tls" ], default-features = false, optional = true }

# Serialization - all optional
serde = { workspace = true, features = [ "derive" ], optional = true }
serde_json = { workspace = true, optional = true }
secrecy = { workspace = true, optional = true }

# Streaming - all optional
eventsource-stream = { workspace = true, optional = true }
bytes = { workspace = true, optional = true }

# Logging - all optional
tracing = { workspace = true, optional = true }

# Client-Side Enhancement Dependencies - all optional
tiktoken-rs = { workspace = true, optional = true }
lru = { workspace = true, optional = true }
prometheus = { workspace = true, optional = true }

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

### 8.1 Request/Response Types

Strongly-typed request and response structures following OpenAI compatibility:

```rust
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ChatCompletionRequest
{
  pub model: String,
  pub messages: Vec< Message >,
  pub temperature: Option< f32 >,
  pub max_tokens: Option< u32 >,
  pub stream: Option< bool >,
  pub tools: Option< Vec< Tool > >,
  // ... additional fields
}

#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ChatCompletionResponse
{
  pub id: String,
  pub object: String,
  pub created: u64,
  pub model: String,
  pub choices: Vec< Choice >,
  pub usage: Usage,
}
```

### 8.2 Error Propagation

Consistent error handling across all operations:

```rust
impl Client
{
  pub async fn chat_completion( &self, request: ChatCompletionRequest )
    -> Result< ChatCompletionResponse, XaiError >
  {
    // Implementation with proper error mapping
  }

  pub async fn chat_completion_stream( &self, request: ChatCompletionRequest )
    -> Result< impl Stream< Item = Result< ChatCompletionChunk, XaiError > >, XaiError >
  {
    // Streaming implementation with proper error handling
  }
}
```

### 9. Release and Versioning

### 9.1 Semantic Versioning

- **Major**: Breaking API changes, X.AI API version updates
- **Minor**: New features, endpoint additions, backward-compatible changes
- **Patch**: Bug fixes, security updates, documentation improvements

### 9.2 Release Process

1. Automated testing pipeline (unit + integration)
2. Documentation generation and validation
3. Changelog generation
4. Version bumping and tagging
5. Crate publication to crates.io (when ready)

### 10. Development Workflow

### 10.1 Code Review Requirements

- All changes must pass automated testing
- Code review for architecture and security aspects
- Documentation updates for public API changes
- Performance impact assessment

### 10.2 Specification Alignment

- **MANDATORY**: All code changes must align with this specification
- **MANDATORY**: Specification must be updated BEFORE implementing changes that deviate from current spec
- **MANDATORY**: No code implementation without corresponding specification section

### 11. Migration and Compatibility

### 11.1 X.AI API Version Support

- Support for latest X.AI API version
- Graceful handling of API deprecations
- Version-specific feature flags where necessary

### 11.2 OpenAI Compatibility

- Maintain compatibility with OpenAI request/response formats where applicable
- Document deviations from OpenAI API
- Allow migration path from OpenAI to X.AI with minimal code changes

### Implementation Status

**Version:** 0.1
**Date:** 2025-11-08
**Status:** 🚧 In Development - Specification Phase Complete

**Next Steps:**

1. ✅ Specification created and reviewed
2. ⏳ Create Cargo.toml with feature structure
3. ⏳ Create basic module structure (lib.rs, client.rs, error.rs, secret.rs)
4. ⏳ Implement core chat completion API (TDD)
5. ⏳ Implement streaming support (TDD)
6. ⏳ Implement integration tests with real API
7. ⏳ Add examples and documentation
8. ⏳ Production readiness review

This specification serves as the authoritative source for all development decisions and implementation priorities for the `api_xai` crate.

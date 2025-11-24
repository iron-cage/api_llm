# api_xai

[![experimental](https://raster.shields.io/static/v1?label=stability&message=experimental&color=orange&logoColor=eee)](https://github.com/emersion/stability-badges#experimental)

Comprehensive Rust client for X.AI's Grok API with enterprise reliability features.

## 🎯 Architecture: Stateless HTTP Client

**This API crate is designed as a stateless HTTP client with zero persistence requirements.** It provides:
- Direct HTTP calls to the X.AI Grok API
- In-memory operation state only (resets on restart)
- No external storage dependencies (databases, files, caches)
- No configuration persistence beyond environment variables

This ensures lightweight, containerized deployments and eliminates operational complexity.

## 🏛️ Governing Principle: "Thin Client, Rich API"

**Expose all server-side functionality transparently while maintaining zero client-side intelligence or automatic behaviors.**

Key principles:
- **API Transparency**: One-to-one mapping with X.AI Grok API endpoints
- **Zero Automatic Behavior**: No implicit decision-making or magic thresholds
- **Explicit Control**: Developer decides when, how, and why operations occur
- **Configurable Reliability**: Enterprise features available through explicit configuration

## Scope

### In Scope
- Chat completions (single and multi-turn)
- Streaming responses (Server-Sent Events)
- Tool/function calling
- Model listing and details
- Enterprise reliability (retry, circuit breaker, rate limiting, failover)
- Health checks (liveness/readiness probes)
- Token counting (local, using tiktoken)
- Response caching (LRU)
- Input validation
- CURL diagnostics for debugging
- Batch operations (parallel request orchestration)
- Performance metrics (Prometheus)
- Synchronous API wrapper

### Out of Scope
- Vision/multimodal (no XAI API support)
- Audio processing (no XAI API support)
- Embeddings (no XAI API support)
- Safety settings/content moderation (no XAI API endpoints)
- Model tuning/deployment (no XAI API support)
- WebSocket streaming (XAI uses SSE only)

## Features

**Core Capabilities:**
- Chat completions with full conversational support
- SSE streaming responses
- Complete function/tool calling integration
- Model management (list, retrieve)

**Enterprise Reliability:**
- Retry logic with exponential backoff and jitter
- Circuit breaker for failure threshold management
- Rate limiting with token bucket algorithm
- Multi-endpoint failover rotation
- Kubernetes-style health checks
- Structured logging with tracing

**Client-Side Enhancements:**
- Token counting using tiktoken (GPT-4 encoding)
- LRU response caching
- Request parameter validation
- CURL command generation for debugging
- Parallel batch processing
- Prometheus metrics collection

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
api_xai = { version = "0.1.0", features = ["full"] }
```

## Quick Start

### Basic Chat

```rust,no_run
use api_xai::{ Client, Secret, XaiEnvironmentImpl, ChatCompletionRequest, Message, ClientApiAccessors };

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
  let env = XaiEnvironmentImpl::new( secret )?;
  let client = Client::build( env )?;

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Hello, Grok!" ) ] )
    .form();

  let response = client.chat().create( request ).await?;
  println!( "Grok: {:?}", response.choices[ 0 ].message.content );

  Ok( () )
}
```

### Streaming Chat

```rust,no_run
use api_xai::{ Client, Secret, XaiEnvironmentImpl, ChatCompletionRequest, Message, ClientApiAccessors };
use futures_util::StreamExt;

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
  let env = XaiEnvironmentImpl::new( secret )?;
  let client = Client::build( env )?;

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Tell me a story" ) ] )
    .stream( true )
    .form();

  let mut stream = client.chat().create_stream( request ).await?;

  while let Some( chunk ) = stream.next().await
  {
    let chunk = chunk?;
    if let Some( content ) = chunk.choices[ 0 ].delta.content.as_ref()
    {
      print!( "{}", content );
    }
  }

  Ok( () )
}
```

## Authentication

### Option 1: Workspace Secret (Recommended)

Create `secret/-secrets.sh` in your project root:

```bash
#!/bin/bash
export XAI_API_KEY="xai-your-key-here"
```

### Option 2: Environment Variable

```bash
export XAI_API_KEY="xai-your-key-here"
```

The crate uses `workspace_tools` for secret management with automatic fallback chain:
1. Workspace secrets (`./secret/-secrets.sh`)
2. Alternative files (`secrets.sh`, `.env`)
3. Environment variable

## Feature Flags

### Core Features
- `enabled` - Master switch for core functionality
- `streaming` - SSE streaming support
- `tool_calling` - Function calling and tools

### Enterprise Reliability
- `retry` - Exponential backoff retry logic
- `circuit_breaker` - Circuit breaker pattern
- `rate_limiting` - Token bucket rate limiting
- `failover` - Multi-endpoint failover
- `health_checks` - Health monitoring
- `structured_logging` - Tracing integration

### Client-Side Enhancements
- `count_tokens` - Local token counting (requires: tiktoken-rs)
- `caching` - Response caching (requires: lru)
- `input_validation` - Request validation
- `curl_diagnostics` - Debug utilities
- `batch_operations` - Parallel processing
- `performance_metrics` - Metrics collection (requires: prometheus)
- `sync_api` - Sync wrappers

### Presets
- `full` - All features enabled (default)

## Testing

### Test Coverage
- 122 doc tests passing
- 107 integration tests passing
- 229 total tests with real API validation
- No-mockup policy: all tests use real API calls

## Documentation

- **[API Reference](docs/api_reference.md)** - Complete API documentation
- **[OpenAPI Summary](docs/openapi_endpoints_summary.md)** - Endpoint reference
- **[Specification](spec.md)** - Detailed project specification
- **[Examples](https://github.com/Wandalen/api_llm/tree/master/api/xai/examples)** - Real-world usage examples

## Dependencies

- **reqwest**: HTTP client with async support
- **tokio**: Async runtime
- **serde**: Serialization/deserialization
- **workspace_tools**: Secret management
- **error_tools**: Unified error handling
- **tiktoken-rs**: Token counting (optional)
- **lru**: Response caching (optional)
- **prometheus**: Metrics collection (optional)

All dependencies workspace-managed for consistency.

## OpenAI Compatibility

The X.AI Grok API is OpenAI-compatible, using the same REST endpoint patterns and request/response formats. Token counting uses GPT-4 encoding (cl100k_base) via tiktoken for accurate counts.

## Contributing

1. Follow established patterns in existing code
2. Use 2-space indentation consistently
3. Add tests for new functionality
4. Update documentation for public APIs
5. Ensure zero clippy warnings: `cargo clippy -- -D warnings`
6. Follow zero-tolerance mock policy (real API integration only)
7. Follow the "Thin Client, Rich API" principle

## License

MIT

## Links

- **[Specification](spec.md)** - Technical specification
- **[Examples](https://github.com/Wandalen/api_llm/tree/master/api/xai/examples)** - Usage examples
- **[API Reference](docs/api_reference.md)** - Complete documentation

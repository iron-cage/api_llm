# api_claude

[![stable](https://raster.shields.io/static/v1?label=stability&message=stable&color=green&logoColor=eee)](https://github.com/emersion/stability-badges#stable)

Comprehensive Rust client for Anthropic's Claude API with enterprise reliability features.

## 🎯 Architecture: Stateless HTTP Client

**This API crate is designed as a stateless HTTP client with zero persistence requirements.** It provides:
- Direct HTTP calls to the Anthropic Claude API
- In-memory operation state only (resets on restart)
- No external storage dependencies (databases, files, caches)
- No configuration persistence beyond environment variables

This ensures lightweight, containerized deployments and eliminates operational complexity.

## 🏛️ Governing Principle: "Thin Client, Rich API"

**Expose all server-side functionality transparently while maintaining zero client-side intelligence or automatic behaviors.**

Key principles:
- **API Transparency**: One-to-one mapping with Claude API endpoints
- **Zero Automatic Behavior**: No implicit decision-making or magic thresholds
- **Explicit Control**: Developer decides when, how, and why operations occur
- **Configurable Reliability**: Enterprise features available through explicit configuration

## Scope

### In Scope
- Messages API (conversational interface)
- Server-Sent Events streaming
- Tool/function calling
- Vision support (image analysis)
- Prompt caching (~90% cost savings)
- Token counting
- System prompts and safety settings
- Enterprise reliability (retry, circuit breaker, rate limiting, failover, health checks)
- Synchronous API wrapper
- Batch operations

### Out of Scope
- Embeddings (not offered by Anthropic)
- Audio processing (not available in Claude API)
- WebSocket streaming (Claude uses SSE only)
- Model tuning/deployment (managed service only)

## Features

**Core Capabilities:**
- Messages API with full conversational support
- SSE streaming responses with tool calling integration
- Complete function/tool calling with validation
- Vision support for image analysis
- Prompt caching for cost optimization

**Enterprise Reliability:**
- Retry logic with exponential backoff and jitter
- Circuit breaker for failure threshold management
- Rate limiting with token bucket algorithm
- Multi-endpoint failover (4 strategies)
- Health checks with endpoint monitoring

**Client Enhancements:**
- Sync API wrapper for blocking operations
- CURL diagnostics for debugging
- Dynamic configuration with file watching
- Enterprise quota management
- HTTP compression support

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
api_claude = { version = "0.1.0", features = ["full"] }
```

## Quick Start

### Basic Usage

```rust,ignore
use api_claude::{ Client, Secret, CreateMessageRequest, Message, Role, Content };

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let secret = Secret::new( "sk-ant-api03-your-key-here".to_string() )?;
  let client = Client::new( secret );

  let request = CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929".to_string() )
    .max_tokens( 1000 )
    .messages( vec![
      Message
      {
        role : Role::User,
        content : vec![ Content::Text
        {
          r#type : "text".to_string(),
          text : "Hello, Claude!".to_string(),
        } ],
        cache_control : None,
      }
    ] )
    .build();

  let response = client.create_message( request ).await?;
  println!( "Claude: {:?}", response.content );

  Ok( () )
}
```

### Streaming Response

```rust,ignore
use api_claude::{ Client, Secret, CreateMessageRequest, Message, Role, Content };
use futures_util::StreamExt;

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let client = Client::from_workspace()?;

  let request = CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929".to_string() )
    .max_tokens( 1000 )
    .stream( true )
    .messages( vec![ Message::user( "Tell me a story" ) ] )
    .build();

  let mut stream = client.create_message_stream( request ).await?;

  while let Some( event ) = stream.next().await
  {
    let event = event?;
    if let Some( text ) = event.delta_text()
    {
      print!( "{}", text );
    }
  }

  Ok( () )
}
```

## Authentication

### Option 1: Workspace Secret (Recommended)

Create `secret/-secrets.sh` in your workspace root:

```bash
#!/bin/bash
export ANTHROPIC_API_KEY="sk-ant-api03-your-key-here"
```

```rust,ignore
use api_claude::Client;

let client = Client::from_workspace()?;
```

### Option 2: Environment Variable

```bash
export ANTHROPIC_API_KEY="sk-ant-api03-your-key-here"
```

```rust,ignore
use api_claude::Client;

let client = Client::from_env()?;
```

### Option 3: Direct Configuration

```rust,ignore
use api_claude::{ Client, Secret };

let secret = Secret::new( "sk-ant-api03-your-key-here".to_string() )?;
let client = Client::new( secret );
```

See [Secret Loading Guide](docs/operation/001_secret_loading.md) for complete authentication options.

## Feature Flags

### Core Features
- `enabled` - Master switch for core functionality
- `streaming` - SSE streaming support
- `tools` - Function calling and tools
- `vision` - Image understanding capabilities

### Enterprise Reliability
- `retry-logic` - Exponential backoff retry
- `circuit-breaker` - Circuit breaker pattern
- `rate-limiting` - Token bucket rate limiting
- `failover` - Multi-endpoint failover
- `health-checks` - Health monitoring

### Client Enhancements
- `sync-api` - Synchronous wrappers
- `curl-diagnostics` - Debug utilities
- `compression` - HTTP compression
- `enterprise-quota` - Usage tracking
- `dynamic-config` - Runtime configuration

### Presets
- `full` - All features enabled

## Testing

### Test Coverage
- 540 tests (37 unit + 503 integration requiring API credentials)
- Real API integration tests
- No-mockup policy: all integration tests use real API calls

## Supported Models

| Model | Context Window | Capabilities |
|-------|---------------|--------------|
| claude-sonnet-4-5-20250929 | 200k tokens | Full capabilities |
| claude-3-5-sonnet-latest | 200k tokens | Fast, cost-effective |
| claude-3-opus-latest | 200k tokens | Highest capability |
| claude-3-haiku-latest | 200k tokens | Fastest |

## Documentation

- **[API Reference](https://docs.rs/api_claude)** - Complete API documentation
- **[Examples](https://github.com/Wandalen/api_llm/tree/master/api/claude/examples)** - Real-world usage examples
- **[Secret Loading](docs/operation/001_secret_loading.md)** - Authentication and secret management
- **[Testing Guide](tests/readme.md)** - Testing organization and NO MOCKING policy

## Dependencies

- **reqwest**: HTTP client with async support
- **tokio**: Async runtime
- **serde**: Serialization/deserialization
- **workspace_tools**: Secret management
- **error_tools**: Unified error handling
- **secrecy**: Secure credential handling

All dependencies workspace-managed for consistency.

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

## Responsibility

This section documents all files and directories in the crate root, ensuring Complete Entity Coverage per organizational_principles.rulebook.md.

| Path | Purpose |
|------|---------|
| `src/` | Source code implementation - client, Messages API, streaming, tools, error handling |
| `tests/` | Comprehensive test suite with 540 tests, strict NO MOCKING ALLOWED policy |
| `examples/` | API usage examples demonstrating Claude API features and capabilities |
| `docs/` | Technical documentation organized in design collections (operation/) |
| `task/` | Implementation task tracking — tsk-compliant work items for this crate |
| `Cargo.toml` | Crate metadata, dependencies, and feature configuration |
| `readme.md` | Crate overview, quick start, API documentation, and this Responsibility Table |
| `license` | MIT license text |

## Links

- **[Anthropic Console](https://console.anthropic.com/)** - Get your API key
- **[Claude API Documentation](https://docs.anthropic.com/claude/reference)** - Official API docs
- **[Examples](https://github.com/Wandalen/api_llm/tree/master/api/claude/examples)** - Comprehensive usage examples

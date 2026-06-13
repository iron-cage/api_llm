# api_ollama

[![stable](https://raster.shields.io/static/v1?label=stability&message=stable&color=green&logoColor=eee)](https://github.com/emersion/stability-badges#stable)

Rust HTTP client for the Ollama local LLM runtime API.

## 🎯 Architecture: Stateless HTTP Client

**This API crate is designed as a stateless HTTP client with zero persistence requirements.** It provides:
- Direct HTTP calls to the Ollama API
- In-memory operation state only (resets on restart)
- No external storage dependencies (databases, files, caches)
- No configuration persistence beyond environment variables

This ensures lightweight, containerized deployments and eliminates operational complexity.

## 🏛️ Governing Principle: "Thin Client, Rich API"

**Expose Ollama's API directly without abstraction layers, enabling developers to access all capabilities with explicit control.**

Key principles:
- **API Transparency**: Every method directly corresponds to an Ollama API endpoint
- **Zero Client Intelligence**: No automatic decision-making or behavior inference
- **Explicit Control**: Developers control when and how API calls are made
- **Information vs Action**: Clear separation between data retrieval and state changes

## Scope

### In Scope
- Chat completions (single and multi-turn)
- Text generation from prompts
- Model management (list, pull, push, copy, delete)
- Embeddings generation
- Streaming responses
- Tool/function calling
- Vision support (image inputs)
- Enterprise reliability (retry, circuit breaker, rate limiting, failover, health checks)
- Synchronous API wrappers

### Out of Scope
- Audio processing (Ollama API limitation)
- Content moderation (Ollama API limitation)
- High-level abstractions or unified interfaces
- Business logic or application features

## Features

**Core Capabilities:**
- Chat completions with configurable parameters
- Text generation from prompts
- Model listing and information
- Embeddings generation
- Real-time streaming responses
- Tool/function calling support
- Vision support for image inputs
- Builder patterns for request construction

**Enterprise Reliability:**
- Exponential backoff retry logic
- Circuit breaker pattern
- Token bucket rate limiting
- Automatic endpoint failover
- Health monitoring
- Response caching with TTL

**API Patterns:**
- Async API (tokio-based)
- Sync API (blocking wrappers)
- Streaming control (pause/resume/cancel)
- Dynamic configuration

## Installation

```toml
[dependencies]
api_ollama = { version = "0.1.0", features = ["full"] }
```

## Quick Start

```rust,no_run
use api_ollama::{ OllamaClient, ChatRequest, ChatMessage, MessageRole };

#[tokio::main]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let mut client = OllamaClient::new(
    "http://localhost:11434".to_string(),
    std::time::Duration::from_secs( 30 )
  );

  // Check availability
  if !client.is_available().await
  {
    println!( "Ollama is not available" );
    return Ok( () );
  }

  // List available models
  let models = client.list_models().await?;
  println!( "Available models: {:?}", models );

  // Send chat request
  let request = ChatRequest
  {
    model: "llama3.2".to_string(),
    messages: vec![ ChatMessage
    {
      role: MessageRole::User,
      content: "Hello!".to_string(),
      images: None,
      #[cfg( feature = "tool_calling" )]
      tool_calls: None,
    }],
    stream: None,
    options: None,
    #[cfg( feature = "tool_calling" )]
    tools: None,
    #[cfg( feature = "tool_calling" )]
    tool_messages: None,
  };

  let response = client.chat( request ).await?;
  println!( "Response: {:?}", response );

  Ok( () )
}
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `enabled` | Master switch for basic functionality |
| `streaming` | Real-time streaming responses |
| `embeddings` | Text embedding generation |
| `vision_support` | Image inputs for vision models |
| `tool_calling` | Function/tool calling support |
| `builder_patterns` | Fluent builder APIs |
| `retry` | Exponential backoff retry |
| `circuit_breaker` | Circuit breaker pattern |
| `rate_limiting` | Token bucket rate limiting |
| `failover` | Automatic endpoint failover |
| `health_checks` | Endpoint health monitoring |
| `request_caching` | Response caching with TTL |
| `sync_api` | Synchronous blocking API |
| `full` | Enable all features |

## Testing

```bash
# Unit tests
cargo nextest run

# Integration tests (requires running Ollama)
cargo nextest run --features integration

# Full validation
w3 .test level::3
```

**Testing Policy**: Integration tests require a running Ollama instance. Tests fail clearly when Ollama is unavailable.

## Documentation

- **[Implementation Roadmap](docs/implementation_roadmap.md)** - Feature priorities and guidelines
- **[Examples](https://github.com/Wandalen/api_llm/tree/master/api/ollama/examples)** - Runnable code examples
- **[Tests](tests/)** - Test documentation

## Project Structure

This section documents the complete project layout with each entity's responsibility and scope.

### Responsibility Table

#### Production Code & Configuration

| Entity | Responsibility | Scope |
|--------|----------------|-------|
| `src/` | Production source code | Core library implementation (see module docs for details) |
| `Cargo.toml` | Crate manifest and configuration | Dependencies, features, metadata, build configuration |
| `license` | License terms | MIT license text |

#### Testing & Quality

| Entity | Responsibility | Scope |
|--------|----------------|-------|
| `tests/` | Comprehensive test suite | All tests (unit, integration, validation) - see tests/readme.md |
| `benches/` | Performance benchmarks | Regression detection, performance measurements - see benches/readme.md |

#### Documentation & Examples

| Entity | Responsibility | Scope |
|--------|----------------|-------|
| `readme.md` | Project overview and onboarding | Quick start, architecture overview, feature summary, navigation |
| `docs/` | Detailed documentation | Roadmaps, guides, design docs - see docs/readme.md |
| `examples/` | Usage demonstrations | Runnable examples for developers - see examples/readme.md |

#### Temporary Files & Working Directories

| Entity | Responsibility | Scope |
|--------|----------------|-------|
| `-knowledge/` | Organized temporary knowledge | Investigations, explorations, temporary reports |
| `-default_topic/` | Claude Code working directory | Tool-generated temporary workspace |
| `-audit_report_api_ollama.md` | Previous audit findings | Historical compliance audit (temporary) |
| `-audit_report_comprehensive.md` | Comprehensive audit report | Current compliance audit results (temporary) |
| `-remediation_plan_comprehensive.md` | Remediation strategy | Step-by-step compliance fix plan (temporary) |
| `-remediation_summary.md` | Previous remediation tracking | Historical remediation work summary (temporary) |
| `-validation_checklist.md` | Validation checklist | Pre-audit validation items (temporary) |
| `-sample_chart.txt` | Sample data for examples | Example input data (temporary) |
| `-sample_document.txt` | Sample data for examples | Example input data (temporary) |
| `-sample_scene.txt` | Sample data for examples | Example input data (temporary) |

**Note:** All entities prefixed with `-` are temporary and excluded from git tracking.

## Dependencies

- **reqwest**: HTTP client with async support
- **tokio**: Async runtime
- **serde/serde_json**: Serialization
- **error_tools**: Unified error handling

## License

MIT

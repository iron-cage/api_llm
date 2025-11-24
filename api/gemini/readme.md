# api_gemini

[![stable](https://raster.shields.io/static/v1?label=stability&message=stable&color=green&logoColor=eee)](https://github.com/emersion/stability-badges#stable)

Comprehensive Rust client for the Google Gemini API with complete type-safe access to all endpoints.

## 🎯 Architecture: Stateless HTTP Client

**This API crate is designed as a stateless HTTP client with zero persistence requirements.** It provides:
- Direct HTTP calls to the Google Gemini API
- In-memory operation state only (resets on restart)
- No external storage dependencies (databases, files, caches)
- No configuration persistence beyond environment variables

This ensures lightweight, containerized deployments and eliminates operational complexity.

## 🏛️ Governing Principle: "Thin Client, Rich API"

**Expose all server-side functionality transparently while maintaining zero client-side intelligence or automatic behaviors.**

Key principles:
- **API Transparency**: One-to-one mapping with Gemini API endpoints
- **Zero Client Intelligence**: No automatic behaviors or magic thresholds
- **Explicit Control**: Developer decides when, how, and why operations occur
- **Information vs Action**: Clear separation between data retrieval and state changes

## Scope

### In Scope
- Text generation (single and multi-turn conversations)
- Streaming responses with pause/resume/cancel
- Vision and multimodal content processing
- Function calling with AUTO/ANY/NONE modes
- Google Search grounding with citations
- System instructions for behavior control
- Code execution (Python)
- Model tuning and fine-tuning
- Embeddings generation
- Token counting
- Server-side content caching
- Safety settings and content filtering
- Enterprise reliability (retry, circuit breaker, rate limiting)
- Synchronous API wrapper

### Out of Scope
- Model hosting or training infrastructure
- Persistent state management
- Business logic or application features
- Mock servers or test stubs

## Features

**Core Capabilities:**
- Type-safe request/response models with compile-time guarantees
- Async/await built on Tokio for high-performance operations
- Complete synchronous wrapper for blocking operations
- Builder pattern with method chaining

**Advanced Features:**
- Google Search grounding with real-time web search
- Enhanced function calling with precise mode control
- System instructions for model behavior
- Code execution with configurable environments
- Model tuning with hyperparameter optimization
- Server-side caching for context management

**Enterprise Reliability:**
- Automatic retries with exponential backoff
- Circuit breaker for fault tolerance
- Rate limiting and quota management
- Request caching for performance
- Streaming control (pause, resume, cancel)
- Dynamic configuration with hot-reload

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
api_gemini = "0.2.0"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

### Feature Flags

```toml
# Default features
api_gemini = "0.2.0"

# With batch operations (infrastructure ready)
api_gemini = { version = "0.2.0", features = ["batch_operations"] }

# With compression support
api_gemini = { version = "0.2.0", features = ["compression"] }

# All features
api_gemini = { version = "0.2.0", features = ["full"] }
```

## Quick Start

```rust,no_run
use api_gemini::{ client::Client, models::*, error::Error };

#[tokio::main]
async fn main() -> Result< (), Error >
{
  // Create client from GEMINI_API_KEY environment variable
  let client = Client::new().map_err( |_| Error::ConfigurationError( "Failed to create client".to_string() ) )?;

  // Simple text generation
  let request = GenerateContentRequest
  {
    contents: vec!
    [
      Content
      {
        parts: vec![ Part { text: Some( "Write a haiku about programming".to_string() ), ..Default::default() } ],
        role: "user".to_string(),
      }
    ],
    ..Default::default()
  };

  let response = client.models().by_name( "gemini-1.5-pro-latest" ).generate_content( &request ).await?;

  if let Some( text ) = response.candidates.first()
    .and_then( |c| c.content.parts.first() )
    .and_then( |p| p.text.as_ref() )
  {
    println!( "{}", text );
  }

  Ok( () )
}
```

## Authentication

### Option 1: Secret File (Recommended)

Create `secret/-secret.sh` in your project root:

```bash
GEMINI_API_KEY="your-api-key-here"
```

```rust,no_run
use api_gemini::client::Client;

fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let client = Client::new()?; // Automatically reads from secret/-secret.sh
  Ok( () )
}
```

### Option 2: Environment Variable

```bash
export GEMINI_API_KEY="your-api-key-here"
```

### Option 3: Direct Configuration

```rust,no_run
use api_gemini::client::Client;

fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let client = Client::builder()
    .api_key( "your-api-key".to_string() )
    .build()?;
  Ok( () )
}
```

Get your API key from [Google AI Studio](https://makersuite.google.com/app/apikey).

## Error Handling

```rust,no_run
use api_gemini::{ client::Client, error::Error };

async fn example()
{
  let client = Client::new().unwrap();
  match client.models().list().await
  {
    Ok( models ) => println!( "Found {} models", models.models.len() ),
    Err( Error::AuthenticationError( msg ) ) => eprintln!( "Auth failed: {}", msg ),
    Err( Error::RateLimitError( msg ) ) => eprintln!( "Rate limited: {}", msg ),
    Err( Error::ApiError( msg ) ) => eprintln!( "API error: {}", msg ),
    Err( e ) => eprintln!( "Error: {:?}", e ),
  }
}
```

## Testing

### Test Coverage
- 485 tests passing (382 nextest + 103 doctests)
- Zero compilation warnings
- Perfect clippy compliance
- 100% documentation coverage for public APIs
- No-mockup policy: all tests use real API integration

## Supported Models

| Model | Context Window | Vision | Capabilities |
|-------|---------------|--------|--------------|
| gemini-2.5-flash | 1M tokens | Yes | Latest stable |
| gemini-1.5-pro | 1M tokens | Yes | Full capabilities |
| gemini-1.5-flash | 1M tokens | Yes | Fast, cost-effective |
| text-embedding-004 | - | No | Embeddings only |

## Documentation

- **[Usage Examples](docs/usage_examples.md)** - Comprehensive code examples for all features
- **[API Coverage](docs/api_coverage.md)** - Complete endpoint documentation with test counts
- **[Cookbook](docs/cookbook.md)** - Recipe patterns for common use cases
- **[Testing Guide](docs/testing.md)** - Test organization and coverage details
- **[Specification](spec.md)** - Complete technical specification
- **[Examples](https://github.com/Wandalen/api_llm/tree/master/api/gemini/examples)** - Runnable example programs

## Dependencies

- **reqwest**: HTTP client with async support
- **tokio**: Async runtime
- **serde**: Serialization/deserialization
- **workspace_tools**: Secret management
- **error_tools**: Unified error handling

All dependencies workspace-managed for consistency.

## Contributing

1. Follow established patterns in existing code
2. Use 2-space indentation consistently
3. Add tests for new functionality
4. Update documentation for public APIs
5. Ensure zero clippy warnings: `cargo clippy -- -D warnings`
6. Follow zero-tolerance mock policy (real API integration only)

## License

MIT

## Links

- **[Google AI Studio](https://makersuite.google.com/)** - Get your API key
- **[Gemini API Documentation](https://ai.google.dev/api/rest)** - Official API docs
- **[Examples](https://github.com/Wandalen/api_llm/tree/master/api/gemini/examples)** - Comprehensive usage examples
- **[Specification](spec.md)** - Technical specification

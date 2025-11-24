# api_llm

[![stable](https://raster.shields.io/static/v1?label=stability&message=stable&color=green&logoColor=eee)](https://github.com/emersion/stability-badges#stable)

Direct HTTP API bindings for major LLM providers with enterprise reliability features.

## 🎯 Architecture: Stateless HTTP Clients

**All API crates are designed as stateless HTTP clients with zero persistence requirements.** They provide:
- Direct HTTP calls to respective LLM provider APIs
- In-memory operation state only (resets on restart)
- No external storage dependencies (databases, files, caches)
- No configuration persistence beyond environment variables

This ensures lightweight, containerized deployments and eliminates operational complexity.

## 🏛️ Governing Principle: "Thin Client, Rich API"

**Expose all server-side functionality transparently while maintaining zero client-side intelligence or automatic behaviors.**

Key principles:
- **API Transparency**: One-to-one mapping with provider APIs without hidden behaviors
- **Zero Client Intelligence**: No automatic decision-making or magic thresholds
- **Explicit Control**: Developer decides when, how, and why operations occur
- **Information vs Action**: Clear separation between data retrieval and state changes

## Scope

### In Scope
- Text generation (single and multi-turn conversations)
- Streaming responses (SSE and WebSocket where applicable)
- Function/tool calling with full schema support
- Vision and multimodal inputs
- Audio processing (speech-to-text, text-to-speech)
- Embedding generation
- Model listing and information
- Token counting
- Batch operations
- Enterprise reliability (retry, circuit breaker, rate limiting, failover, health checks)
- Synchronous API wrappers

### Out of Scope
- High-level abstractions or unified interfaces (see llm_contract)
- Provider switching or fallback logic
- Business logic or application features
- Persistent state management

## API Crates

| Crate | Provider | Tests | Default Model |
|-------|----------|-------|---------------|
| [api_gemini](api/gemini/) | Google Gemini | 485 | gemini-2.5-flash |
| [api_openai](api/openai/) | OpenAI | 643 | gpt-5.1-chat-latest |
| [api_claude](api/claude/) | Anthropic Claude | 435 | claude-sonnet-4-5-20250929 |
| [api_ollama](api/ollama/) | Ollama (Local) | 378 | llama3.2 |
| [api_huggingface](api/huggingface/) | HuggingFace | 534 | meta-llama/Llama-3.2-3B-Instruct |
| [api_xai](api/xai/) | xAI Grok | 127 | grok-2-1212 |

## Quick Start

```rust
use api_openai::{ OpenAIClient, ChatRequest, ChatMessage, MessageRole };

#[tokio::main]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let client = OpenAIClient::new_from_env()?;

  let request = ChatRequest::new( "gpt-4" )
    .with_message( ChatMessage::new( MessageRole::User, "Hello!" ) );

  let response = client.chat( &request ).await?;
  println!( "{}", response.choices[ 0 ].message.content );
  Ok( () )
}
```

## Features

**Core Capabilities:**
- Text generation with configurable parameters
- Real-time streaming responses
- Multi-turn conversation handling
- Function calling with JSON schema validation
- Vision support (image inputs)
- Audio processing (where supported)
- Embedding generation
- Token counting (where supported)

**Enterprise Reliability:**
- Retry logic with exponential backoff
- Circuit breaker for fault tolerance
- Rate limiting with token bucket algorithm
- Multi-endpoint failover
- Health checks with endpoint monitoring
- Request caching with TTL

**API Patterns:**
- Async API (tokio-based)
- Sync API (blocking wrappers)
- Builder patterns for configuration
- Feature flags for zero-overhead customization

## Secret Management

API keys via environment variables or workspace secrets:

```bash
# Environment (CI/CD)
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export GEMINI_API_KEY="AIza..."
export HUGGINGFACE_API_KEY="hf_..."
export XAI_API_KEY="xai-..."

# Workspace secrets (local development)
source secret/-secrets.sh
```

## Testing

```bash
# Check all crates compile
cargo check --workspace

# Run all tests (requires API keys)
cargo test --workspace

# Run tests for specific crate
cargo test -p api_openai

# Full validation
w3 .test level::3

# Build documentation
cargo doc --workspace --open
```

**Testing Policy**: All tests use real API integration. No mocking allowed.

## Documentation

- **[API Feature Matrix](api/readme.md)** - Complete feature comparison
- **[api_gemini](api/gemini/)** - Google Gemini API client
- **[api_openai](api/openai/)** - OpenAI API client
- **[api_claude](api/claude/)** - Anthropic Claude API client
- **[api_ollama](api/ollama/)** - Ollama local API client
- **[api_huggingface](api/huggingface/)** - HuggingFace Inference API client
- **[api_xai](api/xai/)** - xAI Grok API client

## Dependencies

All crates share common dependencies managed at workspace level:
- **reqwest**: HTTP client with async support
- **tokio**: Async runtime
- **serde/serde_json**: Serialization
- **error_tools**: Unified error handling
- **workspace_tools**: Secret management

## Contributing

1. Follow established patterns in existing code
2. Use 2-space indentation consistently
3. Add tests for new functionality
4. Update documentation for public APIs
5. Ensure zero clippy warnings: `cargo clippy -- -D warnings`
6. Follow zero-tolerance mock policy (real API integration only)

## License

MIT

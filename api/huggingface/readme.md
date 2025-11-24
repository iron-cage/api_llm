# api_huggingface

[![stable](https://raster.shields.io/static/v1?label=stability&message=stable&color=green&logoColor=eee)](https://github.com/emersion/stability-badges#stable)

Comprehensive Rust client for HuggingFace Inference API with Router API support for Pro models.

## 🎯 Architecture: Stateless HTTP Client

**This API crate is designed as a stateless HTTP client with zero persistence requirements.** It provides:
- Direct HTTP calls to the HuggingFace Inference API
- In-memory operation state only (resets on restart)
- No external storage dependencies (databases, files, caches)
- No configuration persistence beyond environment variables

This ensures lightweight, containerized deployments and eliminates operational complexity.

## 🏛️ Governing Principle: "Thin Client, Rich API"

**Expose all server-side functionality transparently while maintaining zero client-side intelligence or automatic behaviors.**

Key principles:
- **API Transparency**: One-to-one mapping with HuggingFace API endpoints
- **Zero Automatic Behavior**: No implicit decision-making or magic thresholds
- **Explicit Control**: Developer decides when, how, and why operations occur
- **Configurable Reliability**: Enterprise features available through explicit configuration

## Scope

### In Scope
- Text generation via Router API (Llama-3, Mistral, Kimi-K2)
- Embeddings generation with similarity calculations
- Model discovery and status checking
- Streaming responses (SSE)
- Vision APIs (classification, detection, captioning)
- Audio APIs (ASR, TTS, classification, transformation)
- Enterprise reliability (circuit breaker, rate limiting, failover, health checks)
- Synchronous API wrapper

### Out of Scope
- Model training (inference only)
- File upload/download (text-based API interactions only)
- Custom model hosting (HuggingFace hosted models only)
- GraphQL support (REST API only)

## Features

**Core Capabilities:**
- Router API for Pro plan models (OpenAI-compatible format)
- Text generation with streaming support
- Embeddings with similarity calculations
- Model availability checking

**Multimodal Features:**
- Vision: Image classification, object detection, captioning
- Audio: Speech recognition, text-to-speech, classification

**Enterprise Reliability:**
- Circuit breaker pattern for failure detection
- Rate limiting with token bucket algorithm
- Multi-endpoint failover (4 strategies)
- Background health checks
- Dynamic configuration with runtime updates
- Performance metrics tracking
- LRU caching with TTL

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
api_huggingface = "0.2.0"
```

## Quick Start

### Basic Usage

```rust
use api_huggingface::
{
  Client,
  environment::HuggingFaceEnvironmentImpl,
  components::{ input::InferenceParameters, models::Models },
  secret::Secret,
};

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let api_key = Secret::load_from_env( "HUGGINGFACE_API_KEY" )?;
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )?;
  let client = Client::build( env )?;

  let params = InferenceParameters::new()
    .with_temperature( 0.7 )
    .with_max_new_tokens( 100 );

  let response = client.inference()
    .create_with_parameters
    (
      "What is the capital of France?",
      Models::llama_3_1_8b_instruct(),
      params
    )
    .await?;

  println!( "Response: {:?}", response );
  Ok( () )
}
```

### Embeddings with Similarity

```rust
use api_huggingface::{ Client, environment::HuggingFaceEnvironmentImpl, secret::Secret };

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let api_key = Secret::load_from_env( "HUGGINGFACE_API_KEY" )?;
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )?;
  let client = Client::build( env )?;

  let embedding1 = client.embeddings().create( "Hello world" ).await?;
  let embedding2 = client.embeddings().create( "Hi there" ).await?;

  let similarity = client.embeddings().similarity( &embedding1, &embedding2 )?;
  println!( "Similarity: {:.4}", similarity );

  Ok( () )
}
```

## Authentication

### Option 1: Workspace Secret (Recommended)

Create `secret/-secrets.sh` in your workspace root:

```bash
#!/bin/bash
export HUGGINGFACE_API_KEY="hf_your-key-here"
```

### Option 2: Environment Variable

```bash
export HUGGINGFACE_API_KEY="hf_your-key-here"
```

Get your API key from [huggingface.co/settings/tokens](https://huggingface.co/settings/tokens).

## Feature Flags

### Core Features
- `default` - Core async inference and embeddings
- `inference` - Text generation API
- `embeddings` - Embeddings generation
- `models` - Model discovery and status

### Streaming and Processing
- `inference-streaming` - SSE streaming support
- `embeddings-similarity` - Similarity calculations
- `embeddings-batch` - Batch processing

### Enterprise Reliability
- `circuit-breaker` - Failure detection and recovery
- `rate-limiting` - Token bucket rate limiting
- `failover` - Multi-endpoint failover
- `health-checks` - Background health monitoring
- `dynamic-config` - Runtime configuration

### Client Enhancements
- `sync` - Synchronous API wrappers
- `caching` - LRU caching with TTL
- `performance-metrics` - Request tracking

### Presets
- `full` - All features enabled
- `integration` - Integration tests with real API

## Testing

### Test Coverage
- Comprehensive unit and integration tests
- Real API integration tests (no mocking)
- No-mockup policy: all tests use real HuggingFace API

## Supported Models

### Router API (Pro Plan)

| Model | Provider | Capabilities |
|-------|----------|--------------|
| moonshotai/Kimi-K2-Instruct-0905 | groq | Chat completions |
| meta-llama/Llama-3.1-8B-Instruct | various | Chat completions |
| mistralai/Mistral-7B-Instruct | various | Chat completions |
| codellama/CodeLlama-34b-Instruct | various | Code generation |

### Legacy Inference API (Free Tier)

| Model | Task |
|-------|------|
| facebook/bart-large-cnn | Summarization |
| gpt2 | Text generation |
| sentence-transformers/all-MiniLM-L6-v2 | Embeddings |

## Documentation

- **[Features Overview](docs/features.md)** - Feature list and cargo features
- **[API Reference](docs/api_reference.md)** - Comprehensive API documentation
- **[Examples](https://github.com/Wandalen/api_llm/tree/master/api/huggingface/examples)** - Working code examples
- **[Specification](spec.md)** - Detailed technical specification

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

## Links

- **[HuggingFace Hub](https://huggingface.co/)** - Model discovery
- **[API Tokens](https://huggingface.co/settings/tokens)** - Get your API key
- **[Inference API Docs](https://huggingface.co/docs/api-inference)** - Official documentation
- **[Specification](spec.md)** - Technical specification

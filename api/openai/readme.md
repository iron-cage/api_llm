<!-- {{# generate.module_header{} #}} -->

# Module :: `api_openai`

[![production](https://raster.shields.io/static/v1?label=stability&message=production&color=green&logoColor=eee)](https://github.com/emersion/stability-badges#production)

Comprehensive, type-safe Rust client for OpenAI's API with enterprise reliability features.

## 🎯 Architecture: Stateless HTTP Client

**This API crate is designed as a stateless HTTP client with zero persistence requirements.** It provides:
- Direct HTTP calls to the OpenAI API
- In-memory operation state only (resets on restart)
- No external storage dependencies (databases, files, caches)
- No configuration persistence beyond environment variables

This ensures lightweight, containerized deployments and eliminates operational complexity.

## 🏛️ Governing Principle: "Thin Client, Rich API"

**Expose all server-side functionality transparently while maintaining zero client-side intelligence or automatic behaviors.**

Key principles:
- **API Transparency**: One-to-one mapping with OpenAI API endpoints
- **Zero Client Intelligence**: No automatic behaviors or magic thresholds
- **Explicit Control**: Developer decides when, how, and why operations occur
- **Information vs Action**: Clear separation between data retrieval and state changes

## Scope

### In Scope
- Chat completions (conversational AI)
- Responses API (create, retrieve, update, delete, stream)
- Realtime API (WebSocket communication)
- Audio (text-to-speech, speech-to-text)
- Images (generation, manipulation)
- Embeddings (text vectorization)
- Files (upload, management)
- Fine-tuning (custom model training)
- Assistants (AI assistant management)
- Vector stores (document storage)
- Models (listing, information)
- Moderations (content safety)
- Enterprise reliability (retry, circuit breaker, rate limiting, failover, health checks)
- Custom base URLs (Azure OpenAI, compatible APIs)

### Out of Scope
- Model hosting or training infrastructure
- Persistent state management
- High-level abstractions beyond API mapping
- Business logic or application features

## Features

- **Comprehensive API Coverage**: Full implementation of OpenAI's REST API (all major endpoints)
- **Type-Safe**: Strong typing for all requests and responses with compile-time validation
- **Async/Await**: Built on `tokio` for high-performance async operations
- **Streaming Support**: Real-time streaming via Server-Sent Events and WebSocket
- **Custom Base URLs**: Support for Azure OpenAI, OpenAI-compatible APIs, and corporate proxies
- **Enterprise Reliability**: Retry logic, circuit breaker, rate limiting, failover, health checks
- **Sync API Variants**: Blocking interface for non-async contexts
- **Secure Secret Management**: Comprehensive fallback chain with workspace_tools integration
- **Error Handling**: Robust error handling using error_tools with detailed error types

## Supported APIs

- **Responses API**: Create, retrieve, update, delete, and stream responses
- **Realtime API**: WebSocket-based real-time communication
- **Chat Completions**: Conversational AI interactions
- **Audio**: Text-to-speech and speech-to-text
- **Images**: Image generation and manipulation
- **Files**: File upload and management
- **Fine-tuning**: Custom model training
- **Assistants**: AI assistant management
- **Vector Stores**: Document storage and retrieval
- **Embeddings**: Text vectorization
- **Models**: Model information and capabilities
- **Moderations**: Content safety and moderation

## Quick Start

### Basic Usage (Official OpenAI API)

```rust
use api_openai::exposed::{Client, Secret, components::responses::*, environment::*};

#[tokio::main]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
    // Initialize client with official OpenAI endpoints
    let secret = Secret::load_from_env("OPENAI_API_KEY")?;
    let env = OpenaiEnvironmentImpl::build(
        secret,
        None,
        None,
        OpenAIRecommended::base_url().to_string(),
        OpenAIRecommended::realtime_base_url().to_string(),
    )?;
    let client = Client::build(env)?;

    // Create a response
    let request = CreateResponseRequest::former()
        .model("gpt-5.1-chat-latest".to_string())
        .input(ResponseInput::String("Hello, world!".to_string()))
        .form();

    let response = client.responses().create(request).await?;
    println!("Response: {}", response.id);

    Ok(())
}
```

### Custom Base URL (Azure OpenAI / Compatible APIs)

```rust
use api_openai::exposed::{Client, Secret, environment::OpenaiEnvironmentImpl};

#[tokio::main]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
    let secret = Secret::load_from_env("OPENAI_API_KEY")?;

    // Azure OpenAI Service
    let env = OpenaiEnvironmentImpl::build(
        secret,
        None,
        None,
        "https://your-resource.openai.azure.com/".to_string(),
        "https://your-resource.openai.azure.com/realtime/".to_string(),
    )?;

    // Or OpenAI-compatible API (LocalAI, Ollama, etc.)
    // let env = OpenaiEnvironmentImpl::build(
    //     secret, None, None,
    //     "http://localhost:8080/v1/".to_string(),
    //     "http://localhost:8080/realtime/".to_string(),
    // )?;

    let client = Client::build(env)?;

    // Use client normally - all APIs work with custom base URLs
    // ...

    Ok(())
}
```

## Examples

See the [`examples/`](examples/) directory for comprehensive examples of all API endpoints:

### Responses API
- `responses_create.rs` - Basic response creation *(planned)*
- `responses_create_stream.rs` - Streaming responses *(planned)*
- [`openai_responses_create_with_tools.rs`](examples/openai_responses_create_with_tools.rs) - Function calling
- [`openai_responses_create_image_input.rs`](examples/openai_responses_create_image_input.rs) - Multimodal input
- [`openai_responses_get.rs`](examples/openai_responses_get.rs) - Retrieve responses
- [`openai_responses_update.rs`](examples/openai_responses_update.rs) - Update responses
- [`openai_responses_delete.rs`](examples/openai_responses_delete.rs) - Delete responses
- [`openai_responses_cancel.rs`](examples/openai_responses_cancel.rs) - Cancel responses

### Realtime API
- [`openai_realtime_response_create.rs`](examples/openai_realtime_response_create.rs) - Real-time responses
- [`openai_realtime_input_audio_buffer_append.rs`](examples/openai_realtime_input_audio_buffer_append.rs) - Audio streaming
- [`openai_realtime_session_update.rs`](examples/openai_realtime_session_update.rs) - Session management

Run any example with:
```bash
cargo run --example responses_create
```

## Testing

The crate includes comprehensive tests for all API endpoints with 100% pass rate:

```bash
# Run all tests (683 tests)
cargo nextest run --all-features

# Run with strict warnings
RUSTFLAGS="-D warnings" cargo nextest run --all-features

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Full verification (ctest3)
RUSTFLAGS="-D warnings" cargo nextest run --all-features && \
RUSTDOCFLAGS="-D warnings" cargo test --doc --all-features && \
cargo clippy --all-targets --all-features -- -D warnings
```

**Test Statistics:**
- Total: 683 tests (100% passing)
- Integration tests: Real API validation with mandatory failing behavior
- Unit tests: Comprehensive coverage of all modules
- Doc tests: 7 documentation examples tested

**Note**: Integration tests require a valid OpenAI API key. Tests fail loudly if credentials are unavailable (no silent fallbacks).

## Authentication

The crate supports multiple authentication methods via comprehensive fallback chain:

### 1. Workspace Secrets (Recommended)
```bash
# Create workspace secrets file
mkdir -p ../../secret
echo 'export OPENAI_API_KEY="your-api-key-here"' > ../../secret/-secrets.sh
chmod 600 ../../secret/-secrets.sh
```

### 2. Environment Variable
```bash
export OPENAI_API_KEY="your-api-key-here"
```

### 3. Programmatic
```rust
use api_openai::Secret;

let secret = Secret::load_with_fallbacks("OPENAI_API_KEY")?; // Tries all methods
// Or explicitly:
let secret = Secret::load_from_env("OPENAI_API_KEY")?;
let secret = Secret::new("sk-...".to_string())?; // With validation
```

**Fallback Chain Order:**
1. Workspace secrets file (`../../secret/-secrets.sh`)
2. Environment variable (`OPENAI_API_KEY`)
3. Alternative secret files (`secrets.sh`, `.env`)
4. Programmatic setting

**Security Features:**
- API key format validation (must start with `sk-`)
- Secure in-memory storage using `secrecy` crate
- Audit trail for secret exposure tracking
- No logging of actual secret values

## Error Handling

The library provides comprehensive error handling:

```rust
use api_openai::exposed::OpenAIError;

match client.responses().create(request).await
{
    Ok(response) => println!("Success: {}", response.id),
    Err(OpenAIError::Api(api_error)) =>
    {
        eprintln!("API Error: {}", api_error.message);
    },
    Err(OpenAIError::Reqwest(http_error)) =>
    {
        eprintln!("HTTP Error: {}", http_error);
    },
    Err(e) => eprintln!("Other Error: {:?}", e),
}
```

## Architecture

The crate follows a layered architecture using the `mod_interface` pattern:

- **Client Layer**: High-level API client (`Client`)
- **API Layer**: Individual API implementations (e.g., `Responses`, `Chat`)
- **Components Layer**: Request/response types and shared components
- **Environment Layer**: Configuration and authentication
- **Error Layer**: Comprehensive error handling

## Contributing

1. All examples must use snake_case naming
2. Include comprehensive documentation and examples
3. Add tests for new functionality
4. Follow the existing code patterns and architecture

## License

See the [license](license) file for details.
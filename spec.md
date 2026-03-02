# api_llm Specification

## Goal

Provide direct, transparent HTTP API bindings for major LLM providers without abstraction layers or automatic behaviors.

## Vision

A collection of thin API clients that developers can confidently use, knowing exactly what HTTP calls are being made and having complete control over all operations.

## Scope

### In Scope

1. **Direct API Bindings** - HTTP clients for LLM provider APIs
2. **Enterprise Features** - Optional reliability features (retry, circuit breaker, rate limiting, caching, etc.)
3. **Workspace Secrets** - Local API key management for development
4. **Comprehensive Testing** - Real API integration tests with zero-tolerance policy

### Out of Scope

1. **Provider Abstraction** - No unified interface across providers
2. **Provider Switching** - No automatic fallback or routing logic
3. **Service Layer** - No proxy services or aggregation layers
4. **Application Modules** - No CLI tools or high-level applications

## Architecture

### Governing Principle: Thin Client, Rich API

All API bindings follow these principles:

1. **API Transparency** - Every method maps directly to an API endpoint
2. **Zero Client Intelligence** - No automatic decision-making
3. **Explicit Control** - Developers control all operations
4. **Information vs Action** - Clear separation of concerns

### State Management Policy

**Allowed: Runtime-Stateful, Process-Stateless**
- Connection pools, circuit breaker state, rate limiting buckets
- Retry logic state, failover state, health check state
- Runtime state that dies with the process
- No persistent storage or cross-process state

**Prohibited: Process-Persistent State**
- File storage, databases, configuration accumulation
- State that survives process restarts

### Enterprise Features

All enterprise features must be:
- **Feature-gated** behind cargo features
- **Explicitly configured** (no automatic enabling)
- **Transparently named** (e.g., `execute_with_retries()`)
- **Zero overhead** when disabled

Available features:
- `retry` - Exponential backoff retry logic
- `circuit_breaker` - Failure threshold management
- `rate_limiting` - Request throttling
- `request_caching` - TTL-based response caching
- `failover` - Multi-endpoint support
- `health_checks` - Endpoint monitoring
- `streaming_control` - Pause/resume/cancel streaming
- `count_tokens` - Token counting before API calls
- `audio_processing` - Speech-to-text and text-to-speech
- `batch_operations` - Multiple request optimization
- `safety_settings` - Content filtering and harm prevention

## Crates

### api_claude

Anthropic Claude API client with support for:
- Chat completion with streaming
- Prompt caching for system prompts and message history
- Tool calling and function invocation
- Vision support for image inputs
- Token counting

**Default Model:** claude-sonnet-4-5-20250929

**Features:**
- `full` - All features enabled
- `streaming` - Streaming responses
- `tool_calling` - Function calling support
- `vision_support` - Image processing
- `cached_content` - Prompt caching
- `count_tokens` - Token counting
- `sync_api` - Blocking API wrappers

### api_gemini

Google Gemini API client with support for:
- Chat completion with streaming
- Content caching for system instructions
- Function calling and tool use
- Vision and multimodal inputs
- File management and uploads
- Code execution
- Audio processing
- Model tuning

**Default Model:** gemini-2.0-flash-exp

**Features:**
- `full` - All features enabled
- `streaming` - Streaming responses
- `tool_calling` - Function calling support
- `vision_support` - Multimodal inputs
- `cached_content` - Content caching
- `count_tokens` - Token counting
- `audio_processing` - Speech-to-text/text-to-speech
- `batch_operations` - Batch request optimization
- `sync_api` - Blocking API wrappers

### api_huggingface

Hugging Face Inference API client with support for:
- Text generation
- Chat completion
- Embeddings
- Token classification
- Vision tasks
- Audio processing
- Streaming responses

**Default Model:** meta-llama/Llama-3.3-70B-Instruct

**Features:**
- `full` - All features enabled
- `streaming` - Streaming responses
- `embeddings` - Embedding generation
- `vision_support` - Image processing
- `audio_processing` - Audio tasks
- `count_tokens` - Token counting
- `sync_api` - Blocking API wrappers

### api_ollama

Ollama local LLM runtime API client with support for:
- Chat completion
- Text generation
- Embeddings
- Model management
- Streaming responses
- Vision support

**Default Model:** llama3.2:latest

**Features:**
- `full` - All features enabled
- `streaming` - Streaming responses
- `embeddings` - Embedding generation
- `vision_support` - Image processing
- `model_details` - Enhanced model information
- `count_tokens` - Token counting
- `cached_content` - Response caching
- `sync_api` - Blocking API wrappers

### api_openai

OpenAI API client with support for:
- Chat completion with streaming
- Text generation
- Embeddings
- Vision inputs
- Function calling
- Audio processing (Whisper)
- Image generation (DALL-E)

**Default Model:** gpt-4o

**Features:**
- `full` - All features enabled
- `streaming` - Streaming responses
- `tool_calling` - Function calling support
- `vision_support` - Image processing
- `audio_processing` - Whisper integration
- `embeddings` - Embedding generation
- `count_tokens` - Token counting
- `sync_api` - Blocking API wrappers

### api_openai_compatible

Shared OpenAI wire-protocol HTTP layer consumed by any OpenAI-compatible API endpoint.
Extracted from `api_xai` and available for reuse by other crates targeting OpenAI-compatible
providers (KIE.ai, xAI, etc.).

Provides:
- Chat completion request/response wire types
- SSE streaming wire types
- Async HTTP client generic over environment
- Synchronous blocking wrapper
- Environment configuration trait and default implementation

**Features:**
- `enabled` — activates all public types and the HTTP client
- `streaming` — Server-Sent Events streaming support
- `sync_api` — blocking wrappers around the async client
- `integration` — real-API integration tests (requires live credentials)
- `full` — enables `enabled`, `streaming`, and `sync_api`

**Architecture Notes:**
- Thin-client: every method maps to exactly one API endpoint
- Generic over `OpenAiCompatEnvironment` to support multiple providers
- `api_openai` wire types structurally differ (i32 vs u32, Role enum vs String, multimodal content)
  and are explicitly NOT consolidated; each crate retains its own type system

### api_xai

X.AI Grok API client with support for:
- Chat completion with streaming
- Function calling and tool use
- Model listing
- OpenAI-compatible REST interface

**Default Model:** grok-beta

**Features:**
- `full` - All features enabled
- `streaming` - Streaming responses via SSE
- `tool_calling` - Function calling support
- `retry` - Exponential backoff retry logic
- `circuit_breaker` - Failure threshold management
- `rate_limiting` - Request throttling
- `failover` - Multi-endpoint support
- `health_checks` - Endpoint health monitoring
- `integration` - Real API integration tests

**Architecture Notes:**
- OpenAI-compatible API (base URL: `https://api.x.ai/v1`)
- Simplified feature set compared to full OpenAI API
- Focus on core chat and tool calling capabilities
- Enterprise reliability features available but optional

## Testing

### Zero-Tolerance Policy

- **No Mocking** - All tests use real API implementations
- **Loud Failures** - Tests fail clearly when APIs unavailable
- **No Silent Passes** - Integration tests never pass silently
- **Real Implementations Only** - No stub/mock servers

### Test Organization

```
api/*/tests/
├── integration_tests.rs     # Real API integration tests
├── unit_tests.rs            # Unit tests for client logic
└── manual/
    └── readme.md            # Manual testing procedures
```

### Running Tests

```bash
# Load API keys
source secret/-secrets.sh

# Run all tests (requires API keys)
cargo test --workspace

# Run specific crate tests
cargo test -p api_openai

# Run with all features
cargo test --workspace --all-features
```

## Secret Management

### Local Development

API keys stored in `secret/-secrets.sh`:

```bash
#!/bin/bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export GEMINI_API_KEY="AIza..."
export HUGGINGFACE_API_KEY="hf_..."
export XAI_API_KEY="xai-..."
```

File is gitignored and never committed.

### CI/CD

API keys provided via environment variables in CI configuration.

## Success Metrics

1. **Compilation** - All crates compile with zero warnings
2. **Test Coverage** - >90% code coverage across all crates
3. **Integration Tests** - All integration tests pass with real APIs
4. **Documentation** - All public APIs documented
5. **Zero Panics** - No unwrap() or expect() in production code paths
6. **Feature Isolation** - All features compile independently

## Future Enhancements

Potential future additions (not currently in scope):
- Additional provider APIs (Cohere, AI21, etc.)
- Async runtime abstraction (support for different executors)
- Custom HTTP client support
- WebSocket streaming for real-time bidirectional communication
- Enhanced observability (tracing, metrics)

## Non-Goals

Explicitly not goals for this workspace:
- Provider abstraction layer
- Unified interface across providers
- Provider routing or fallback logic
- Service orchestration
- Application frameworks
- CLI tools

# api_openai_compatible

[![experimental](https://raster.shields.io/static/v1?label=stability&message=experimental&color=orange&logoColor=eee)](https://github.com/emersion/stability-badges#experimental)

Shared OpenAI wire-protocol HTTP layer for OpenAI-compatible APIs.

## Architecture: Stateless HTTP Client

**This crate is a provider-neutral implementation of the OpenAI REST protocol.** It provides:
- Wire types (request, response, streaming) usable by any OpenAI-compatible endpoint
- Async HTTP client for chat completions
- Optional blocking (sync) wrapper
- SSE streaming support

Extracted from `api_xai` and `api_openai` to eliminate infrastructure duplication across provider crates.

## Governing Principle: "Thin Client, Rich API"

**Expose all server-side functionality transparently while maintaining zero client-side intelligence or automatic behaviors.**

- **API Transparency**: One-to-one mapping with OpenAI-compatible endpoints
- **Zero Automatic Behavior**: No implicit decision-making or magic thresholds
- **Explicit Control**: Developer decides when, how, and why operations occur

## Scope

### In Scope
- Chat completion wire types (request, response, message, tool calls)
- SSE streaming wire types (chunks, deltas)
- Common types (usage, role)
- Async HTTP client for chat completions
- Blocking sync wrapper
- Environment configuration trait

### Out of Scope
- Provider-specific extensions (handled by individual provider crates)
- Enterprise reliability features (retry, circuit breaker, rate limiting)
- Authentication / secret management (provider-specific)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
api_openai_compatible = { version = "0.1.0", features = ["full"] }
```

## Feature Flags

- `enabled` — activates all public types and the HTTP client
- `streaming` — Server-Sent Events streaming support
- `sync_api` — blocking wrappers around the async client
- `integration` — real-API integration tests (requires live credentials)
- `full` — enables `enabled`, `streaming`, and `sync_api` (default)

## Dependencies

- **reqwest** — HTTP client with async support
- **serde** / **serde_json** — serialization
- **error_tools** — unified error handling
- **mod_interface** — module macro pattern
- **former** — builder pattern
- **tokio** — async runtime (sync_api feature)

All dependencies are workspace-managed for consistency.

## License

MIT

## Links

- **[Workspace Repository](https://github.com/Wandalen/api_llm)**
- **[API Documentation](https://docs.rs/api_openai_compatible)**

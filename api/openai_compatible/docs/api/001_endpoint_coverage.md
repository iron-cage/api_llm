# API: Endpoint Coverage

### Scope

- **Purpose**: Define the public API surface of `api_openai_compatible` — client methods, environment trait, and the complete wire-type inventory consumed by downstream crates.
- **Responsibility**: Documents every public method and type that downstream crates depend on.
- **In Scope**: `Client<E>::build`, `post<I,O>`, `get<O>`; `OpenAiCompatEnvironment` trait; `OpenAiCompatEnvironmentImpl`; all wire types in `components/`; `SyncClient<E>`; `OpenAiCompatError`.
- **Out of Scope**: Provider-specific secret loading (downstream crate responsibility), enterprise reliability modules (downstream crate responsibility).

### Abstract

The `api_openai_compatible` crate provides a provider-neutral HTTP layer implementing the OpenAI REST wire protocol. It is consumed by `api_xai`, and any future OpenAI-compatible provider crate. The crate exports a generic `Client<E>` parameterized over an `OpenAiCompatEnvironment` implementor, two HTTP methods (`post` and `get`), and the full set of chat completion wire types. Four Cargo feature flags control compilation: `enabled` (master switch), `streaming` (SSE chunk types), `sync_api` (blocking wrapper), `integration` (live API tests).

### Operations

| Method | Signature | Purpose |
|--------|-----------|---------|
| `Client::build` | `fn build(env: E) -> Result<Self>` | Construct HTTP client from environment; configures timeout and connection pooling |
| `Client::post` | `async fn post<I,O>(&self, path, body) -> Result<O>` | POST JSON body to `base_url + path`, deserialize response |
| `Client::get` | `async fn get<O>(&self, path) -> Result<O>` | GET from `base_url + path`, deserialize response |
| `SyncClient::new` | `fn new(client: Client<E>) -> Result<Self>` | Wrap async client in a dedicated tokio runtime (feature `sync_api`) |
| `SyncClient::post` | `fn post<I,O>(&self, path, body) -> Result<O>` | Blocking POST via the owned runtime |

### Wire Type Inventory

| Type | Module | Purpose |
|------|--------|---------|
| `ChatCompletionRequest` | `components/chat` | Serializable request body; `model` + `messages` required, all other fields optional |
| `ChatCompletionResponse` | `components/chat` | Deserialized success response: `id`, `model`, `choices`, `usage` |
| `Choice` | `components/chat` | Single completion choice: `index`, `message`, `finish_reason` |
| `Message` | `components/chat` | Conversation turn: `role`, `content`, `tool_calls`, `tool_call_id` |
| `Role` | `components/chat` | Enum: `System`, `User`, `Assistant`, `Tool` |
| `Tool` | `components/chat` | Function tool definition wrapping a `Function` struct |
| `Function` | `components/chat` | Tool function: `name`, `description`, `parameters` (as `serde_json::Value`) |
| `ToolCall` | `components/chat` | Response-side tool invocation: `id`, `tool_type`, `function: FunctionCall` |
| `FunctionCall` | `components/chat` | Invocation payload: `name`, `arguments` (raw JSON string) |
| `Usage` | `components/chat` | Token counts: `prompt_tokens`, `completion_tokens`, `total_tokens` |
| `ChatCompletionChunk` | `components/streaming` | SSE streaming chunk (feature `streaming`) |
| `ChunkChoice` | `components/streaming` | Streaming choice with `delta` instead of `message` |
| `Delta` | `components/streaming` | Incremental content: `role`, `content`, `tool_calls` — all optional |

### Environment Trait

The `OpenAiCompatEnvironment` trait defines the configuration contract. Implementors supply `api_key()`, `base_url()`, and `timeout()`; `headers()` has a default implementation producing `Authorization: Bearer <key>` and `Content-Type: application/json`. The trait requires `Send + Sync + 'static` for use across async task boundaries. `OpenAiCompatEnvironmentImpl` is the built-in implementor with `with_base_url()` and `with_timeout()` builder methods.

### Error Handling

All methods return `Result<T>` where the error is `error_tools::untyped::Error`. The typed error enum `OpenAiCompatError` has seven variants: `Api` (non-2xx response body), `Http` (transport failure), `Network` (DNS/TCP), `Timeout` (exceeded configured duration), `Deserialise` (response parsing), `InvalidApiKey` (empty, whitespace-only, or non-ASCII key), `Environment` (misconfiguration). Automatic `From` conversions exist for `reqwest::Error`, `serde_json::Error`, and `reqwest::header::InvalidHeaderValue`.

### Compatibility Guarantees

Wire types follow the OpenAI chat completions schema. Any provider whose REST API accepts the same JSON shapes (X.AI Grok, KIE.ai, local proxies) is compatible. Breaking changes to public types or method signatures require a MAJOR version bump. New error variants are MINOR additions because `OpenAiCompatError` is `#[non_exhaustive]`; new optional fields in wire-type structs require a MAJOR bump since the structs are not `#[non_exhaustive]`.

### Sources

| File | Relationship |
|------|--------------|
| `src/client.rs` | `Client<E>` — `build`, `post`, `get` |
| `src/environment.rs` | `OpenAiCompatEnvironment` trait + `OpenAiCompatEnvironmentImpl` |
| `src/error.rs` | `OpenAiCompatError` enum and `Result` type alias |
| `src/components/chat.rs` | All chat completion wire types |
| `src/components/streaming.rs` | SSE streaming wire types (feature `streaming`) |
| `src/sync_client.rs` | `SyncClient<E>` blocking wrapper (feature `sync_api`) |

### Tests

| File | Relationship |
|------|--------------|
| `tests/wire_test.rs` | Serde round-trips for all wire types; optional-field omission |
| `tests/environment_test.rs` | Environment construction, header building, timeout config |
| `tests/client_test.rs` | Client POST/GET integration tests |
| `tests/sync_client_test.rs` | SyncClient blocking wrapper tests |
| `tests/error_test.rs` | Error Display formatting and From conversions |

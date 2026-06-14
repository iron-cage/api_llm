# Feature: Sync API

### Scope

- **Purpose**: Define the blocking synchronous wrapper around `Client<E>` for use in synchronous contexts.
- **Responsibility**: Documents the Sync API feature — activation, API contract, and behavioral constraints.
- **In Scope**: `SyncClient<E>` struct, `SyncClient::new()`, `SyncClient::post()`, owned tokio runtime lifecycle.
- **Out of Scope**: The underlying async `Client<E>` (always-on via `enabled`), streaming sync wrappers, environment configuration.

### Feature Statement

When the `sync_api` Cargo feature is enabled, callers in synchronous contexts may wrap an async `Client<E>` in a `SyncClient<E>`. Each `SyncClient` instance owns a dedicated `tokio::runtime::Runtime` and blocks the calling thread until the async operation completes. Creating many `SyncClient` instances is expensive; callers should prefer the async `Client<E>` when possible.

### Activation

| Requirement | Detail |
|-------------|--------|
| Cargo feature | `sync_api` — activates `SyncClient<E>` |
| Constructor | `SyncClient::new(client: Client<E>) -> Result<Self>` |
| Default | `full` feature enables `sync_api` |

### API Contract

| Method | Signature | Behavior |
|--------|-----------|----------|
| `SyncClient::new` | `(client: Client<E>) -> Result<Self>` | Creates owned tokio runtime; fails if runtime cannot be created |
| `SyncClient::post` | `(&self, path: &str, body: &I) -> Result<O>` | Blocks calling thread; delegates to `Client::post` on owned runtime |

### Behavioral Constraints

- Each `SyncClient` instance owns exactly one `tokio::runtime::Runtime` wrapped in `Arc`.
- `SyncClient::new` returns `Err(OpenAiCompatError::Environment)` if the tokio runtime cannot be created.
- `SyncClient::post` is a direct blocking delegation to `Client::post` — no additional retry, timeout, or buffering.
- Do not nest `SyncClient` inside an existing tokio runtime context (causes `block_on` panic).
- `SyncClient` does not implement `Clone`; callers must construct one instance and share it via their own synchronization if needed.

### Sources

| File | Relationship |
|------|--------------|
| `src/sync_client.rs` | Defines `SyncClient<E>`, `new()`, `post()` |
| `src/client.rs` | Underlying async `Client<E>` delegated to |
| `src/lib.rs` | `sync_api` feature gate declaration |

### Tests

| File | Relationship |
|------|--------------|
| `tests/sync_client_test.rs` | Unit: `SyncClient::new()` success; Integration: blocking POST round-trip with real API |

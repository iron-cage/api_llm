# Feature: Sync API

### Scope

- **Purpose**: Define the blocking synchronous wrapper around `Client<E>` for use in synchronous contexts.
- **Responsibility**: Documents the Sync API feature — activation, API contract, and behavioral constraints.
- **In Scope**: `SyncClient<E>` struct, `SyncClient::new()`, `SyncClient::post()`, owned tokio runtime lifecycle.
- **Out of Scope**: The underlying async `Client<E>` (always-on via `enabled`), streaming sync wrappers, environment configuration.

### Design

When the `sync_api` Cargo feature is enabled, callers in synchronous contexts may wrap an async client in a blocking wrapper. Each blocking wrapper owns a dedicated runtime and blocks the calling thread until the async operation completes. Creating many blocking wrapper instances is expensive; callers should prefer the async client when possible.

### Activation

| Requirement | Detail |
|-------------|--------|
| Cargo feature | `sync_api` — activates `SyncClient<E>` |
| Constructor | `SyncClient::new(client: Client<E>) -> Result<Self>` |
| Default | `full` feature enables `sync_api` |

### Operations

| Operation | Behavior |
|-----------|----------|
| `SyncClient::new` | Creates owned runtime; fails if runtime cannot be created |
| `SyncClient::post` | Blocks calling thread; delegates to async client on owned runtime |

### Behavioral Constraints

- Each blocking wrapper owns exactly one dedicated runtime.
- `SyncClient::new` returns an environment error if the runtime cannot be created.
- `SyncClient::post` is a direct blocking delegation — no additional retry, timeout, or buffering.
- Do not use the blocking wrapper inside an existing async runtime context — it panics.
- The blocking wrapper does not support cloning; callers must construct one instance and share it via their own synchronization if needed.

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

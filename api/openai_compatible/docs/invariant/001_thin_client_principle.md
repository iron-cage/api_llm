# Invariant: Thin Client Principle

### Scope

- **Purpose**: Enforce that `api_openai_compatible` is a stateless HTTP transport with no automatic or implicit behaviors — every client action is explicit, transparent, and direct.
- **Responsibility**: Documents the Thin Client Principle invariant — statement, enforcement mechanism, and violation consequences.
- **In Scope**: All client methods, environment configuration, streaming wire types, sync wrapper — every source file under `src/`.
- **Out of Scope**: Workspace-level configuration infrastructure, secret file loading mechanics, test helper utilities in `tests/`.

### Invariant Statement

The `api_openai_compatible` client is a **stateless HTTP transport layer**. It exposes OpenAI-compatible API functionality transparently without adding client-side intelligence, automatic behaviors, or implicit side effects. Every client action must be explicit (triggered only by direct caller instruction), transparent (behavior observable from the call site), and direct (one method call maps to at most one API endpoint). Features like streaming and sync-blocking are opt-in via Cargo feature flags; no behavior activates automatically.

### Enforcement Mechanism

The optional features (`streaming`, `sync_api`, `integration`) are behind Cargo feature flags and produce zero overhead when disabled. The `enabled` feature is the only always-required gate for all public types. Code review blocks any automatic or implicit behavior before merge.

| Permitted | Prohibited |
|-----------|------------|
| Streaming when `streaming` feature flag enabled and `stream: true` set in request | Automatic streaming activation |
| Sync wrapper when `sync_api` feature flag enabled | Implicit runtime creation on every call |
| Integration tests when `integration` feature flag enabled | Silent test pass when credentials missing |
| Explicit timeout configuration via `OpenAiCompatEnvironment::timeout()` | Auto-retry on timeout without explicit configuration |
| Environment-supplied headers via `headers()` | Automatic mutation of request headers |

### Violation Consequences

Any automatic or implicit behavior in client code is a blocking code-review violation. The offending code must be removed or refactored to require explicit caller opt-in before merge. There is no deprecation window — the constraint applies to all code at all times.

### Sources

| File | Relationship |
|------|--------------|
| `src/client.rs` | Primary enforcement site — `Client<E>` struct and `post`/`get` methods |
| `src/sync_client.rs` | `SyncClient<E>` — opt-in sync wrapper, one owned runtime per instance |
| `src/environment.rs` | `OpenAiCompatEnvironment` trait — caller supplies all configuration |

### Tests

| File | Relationship |
|------|--------------|
| `tests/environment_test.rs` | Verifies environment construction, header building, timeout configuration |
| `tests/wire_test.rs` | Verifies wire types serialize/deserialize without hidden transformations |
| `tests/client_test.rs` | Verifies GET/POST map directly to single API calls with no implicit behavior |

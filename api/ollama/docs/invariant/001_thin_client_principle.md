# Invariant: Thin Client Principle

### Scope

- **Purpose**: Enforce that `api_ollama` is a stateless HTTP transport with no automatic or implicit behaviors — every client action is explicit, transparent, and direct.
- **Responsibility**: Documents the Thin Client Principle invariant — statement, enforcement mechanism, and violation consequences.
- **In Scope**: All client methods, enterprise feature modules, streaming, error handling — every source file under `src/`.
- **Out of Scope**: Workspace-level configuration infrastructure, secret file loading mechanics, test helper utilities in `tests/`.

### Invariant Statement

The `api_ollama` client is a **stateless HTTP transport layer**. It exposes Ollama API functionality transparently without adding client-side intelligence, automatic behaviors, or implicit side effects. Every client action must be explicit (triggered only by direct caller instruction), transparent (behavior observable from the call site), and direct (one method call maps to at most one Ollama API endpoint). The crate follows the "Thin Client, Rich API" governing principle shared across the `api_llm` workspace.

### Enforcement Mechanism

All enterprise reliability features (retry, circuit_breaker, rate_limiting, failover, health_checks, caching, compression, dynamic_configuration, enterprise_quota) are behind Cargo feature flags. The `sync_api` wrapper is behind the `sync_api` feature flag. Code review blocks any automatic or implicit behavior before merge.

| Permitted | Prohibited |
|-----------|------------|
| Enterprise features when feature flag enabled | Auto-retry without explicit configuration |
| Streaming responses when `streaming` feature enabled | Implicit background connections |
| Caching when feature flag enabled | Implicit response caching |
| Explicit validation at call site | Automatic request mutation or normalization |

### Violation Consequences

Any automatic or implicit behavior in client code is a blocking code-review violation. The offending code must be removed or refactored to require explicit caller opt-in before merge.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | Top-level `mod_interface!` — feature-gated `layer` declarations |
| `src/client.rs` / `src/client/` | Main client implementation demonstrating explicit-only pattern |

### Tests

| File | Relationship |
|------|--------------|
| `tests/` | Integration tests require real Ollama server — no mocking, no automatic fallback |

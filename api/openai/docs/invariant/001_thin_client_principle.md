# Invariant: Thin Client Principle

### Scope

- **Purpose**: Enforce that `api_openai` is a stateless HTTP transport with no automatic or implicit behaviors — every client action is explicit, transparent, and direct.
- **Responsibility**: All contributors and code reviewers; any automatic or implicit behavior in client code is a blocking violation requiring immediate remediation.
- **In Scope**: All client methods, enterprise feature modules, realtime WebSocket, streaming, error handling — every source file under `src/`.
- **Out of Scope**: Workspace-level configuration infrastructure, secret file loading mechanics, test helper utilities in `tests/`.

### Invariant Statement

The `api_openai` client is a **stateless HTTP transport layer**. It exposes OpenAI API functionality transparently without adding client-side intelligence, automatic behaviors, or implicit side effects. Every client action must be explicit (triggered only by direct caller instruction), transparent (behavior observable from the call site), and direct (one method call maps to at most one OpenAI API endpoint). The crate follows the "Thin Client, Rich API" governing principle shared across the `api_llm` workspace.

### Enforcement Mechanism

All enterprise reliability features (retry, circuit_breaker, rate_limiting, failover, health_checks, caching, compression, dynamic_configuration, enterprise_quota) are behind Cargo feature flags. The realtime WebSocket module is behind the `websocket` feature flag. Code review blocks any automatic or implicit behavior before merge.

| Permitted | Prohibited |
|-----------|------------|
| Enterprise features when feature flag enabled | Auto-retry without explicit configuration |
| Realtime WebSocket when `websocket` feature enabled | Implicit background connections |
| Caching when feature flag enabled | Implicit response caching |
| Explicit validation at call site | Automatic request mutation or normalization |

### Violation Consequences

Any automatic or implicit behavior in client code is a blocking code-review violation. The offending code must be removed or refactored to require explicit caller opt-in before merge.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | Top-level `mod_interface!` — feature-gated `layer` declarations including `#[cfg(feature = "websocket")]` |
| `src/client_api_accessors.rs` | `ClientApiAccessors<E>` trait — endpoint accessor pattern with `#[cfg(feature = "websocket")]` guard on `realtime()` |
| `src/enhanced_client.rs` | `EnhancedClient` — demonstrates explicit opt-in pattern for enterprise features |

### Tests

| File | Relationship |
|------|--------------|
| `tests/` | Integration tests require real OpenAI API keys — no mocking, no automatic fallback |
| `tests/websocket_reliability_enhanced_tests.rs` | WebSocket tests gated with `#![cfg(feature = "websocket")]` |

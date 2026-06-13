# Invariant: Thin Client Principle

### Scope

- **Purpose**: Enforce that every `api_llm` crate is a stateless HTTP transport with no automatic or implicit behaviors — every client action is explicit, transparent, and direct.
- **Responsibility**: All contributors and code reviewers across all crates; any automatic or implicit behavior is a blocking violation.
- **In Scope**: All client methods in all crates, enterprise feature modules, streaming, error handling.
- **Out of Scope**: Workspace-level configuration infrastructure, secret file loading mechanics, test helper utilities.

### Invariant Statement

Every crate in the `api_llm` workspace is a **stateless HTTP transport layer**. Each crate exposes one provider's API functionality transparently without adding client-side intelligence, automatic behaviors, or implicit side effects. Every client action must be explicit (triggered only by direct caller instruction), transparent (behavior observable from the call site), and direct (one method call maps to at most one API endpoint). This is the governing principle: **Thin Client, Rich API**.

### Enforcement Mechanism

All enterprise reliability features (retry, circuit_breaker, rate_limiting, failover, health_checks, caching, compression, dynamic_configuration, enterprise_quota) are behind Cargo feature flags in every crate. Code review blocks any automatic or implicit behavior before merge.

| Permitted | Prohibited |
|-----------|------------|
| Enterprise features when feature flag explicitly enabled | Auto-retry without explicit configuration |
| Provider abstraction within a single crate | Unified provider abstraction across crates |
| Runtime-stateful objects (circuit breaker state, rate limiter) | Process-persistent state (file storage, databases) |
| Explicit validation at call site | Automatic request mutation or normalization |

### State Management Boundary

- **Allowed**: Runtime-stateful, process-stateless (connection pools, circuit breaker state, rate limiting buckets — all die with the process)
- **Prohibited**: Process-persistent state (file storage, databases, configuration accumulation across restarts)

### Violation Consequences

Any automatic or implicit behavior in client code is a blocking code-review violation. The offending code must be removed or refactored to require explicit caller opt-in before merge.

### Sources

| File | Relationship |
|------|--------------|
| `api/*/src/lib.rs` | Top-level `mod_interface!` in each crate — feature-gated enterprise layers |
| `Cargo.toml` | Workspace manifest — no cross-crate provider abstraction |

### Tests

| File | Relationship |
|------|--------------|
| `api/*/tests/` | All integration tests require real credentials — no mocking, no automatic fallback |

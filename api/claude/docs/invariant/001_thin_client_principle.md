# Invariant: Thin Client Principle

### Scope

- **Purpose**: Enforce that `api_claude` is a stateless HTTP transport with no automatic or implicit behaviors — every client action is explicit, transparent, and direct.
- **Responsibility**: All contributors and code reviewers; any automatic or implicit behavior in client code is a blocking violation requiring immediate remediation.
- **In Scope**: All client methods, enterprise feature modules, streaming, error handling — every source file under `src/`.
- **Out of Scope**: Workspace-level configuration infrastructure, secret file loading mechanics, test helper utilities in `tests/`.

### Invariant Statement

The `api_claude` client is a **stateless HTTP transport layer**. It exposes Anthropic API functionality transparently without adding client-side intelligence, automatic behaviors, or implicit side effects. Every client action must be explicit (triggered only by direct caller instruction), transparent (behavior observable from the call site), and direct (one method call maps to at most one Anthropic API endpoint).

### Enforcement Mechanism

All enterprise reliability features (retry, circuit-breaker, rate-limiting, failover, health-checks) are behind Cargo feature flags and require explicit opt-in builder construction — `Client::new(secret)` activates zero enterprise features regardless of which feature flags are compiled in. Code review blocks any automatic or implicit behavior before merge.

| Permitted | Prohibited |
|-----------|------------|
| Enterprise features when explicitly constructed via builder | Auto-retry without explicit `RetryConfig` |
| Caching when caller passes `CacheControl` | Implicit response caching |
| Rate limiting when explicitly constructed | Auto rate limiting without `RateLimiterConfig` |
| Explicit error enhancement via `create_message_with_context()` | Silent error swallowing or fallback |

### Violation Consequences

Any automatic or implicit behavior in client code is a blocking code-review violation. The offending code must be removed or refactored to require explicit caller configuration before merge. There is no deprecation window — the constraint applies to all code at all times.

### Sources

| File | Relationship |
|------|--------------|
| `src/client/implementation.rs` | Primary enforcement site — `Client` struct, all constructors, and core API methods |
| `src/enterprise_config.rs` | Enterprise feature configuration — demonstrates explicit-builder requirement |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/invariant/01_thin_client_principle.md` | Behavioral spec — 6 compliance scenarios verifying no automatic or implicit client behaviors |
| `tests/inc/mod.rs` | Aggregates all test modules that verify explicit-only client behaviors |

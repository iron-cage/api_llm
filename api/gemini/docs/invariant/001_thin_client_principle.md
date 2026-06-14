# Invariant: Thin Client Principle

### Scope

- **Purpose**: Enforce that `api_gemini` is a stateless HTTP transport with no automatic or implicit behaviors — every client action is explicit, transparent, and direct.
- **Responsibility**: Documents the Thin Client Principle invariant — statement, enforcement mechanism, and violation consequences.
- **In Scope**: All client methods, enterprise feature modules, streaming, error handling — every source file under `src/`.
- **Out of Scope**: Workspace-level configuration infrastructure, secret file loading mechanics, test helper utilities in `tests/`.

### Invariant Statement

The `api_gemini` client is a **stateless HTTP transport layer** — a transparent window to the Gemini API with optional runtime enhancements. It exposes Gemini API functionality transparently without adding client-side intelligence, automatic behaviors, or implicit side effects.

Every client action must be: explicit (triggered only by direct caller instruction), transparent (behavior observable from the call site), and direct (one method call maps to at most one Gemini API endpoint).

The governing principle is **"Thin Client, Rich API"**: expose all server-side functionality transparently while maintaining zero client-side intelligence or automatic behaviors.

### Enforcement Mechanism

| Permitted | Prohibited |
|-----------|------------|
| Enterprise features when explicitly configured via builder | Auto-retry without explicit `RetryConfig` |
| Caching when caller explicitly passes cache configuration | Implicit response caching |
| Rate limiting when `RateLimitingConfig` is provided | Auto rate limiting without explicit configuration |
| Runtime in-memory state (caches, metrics, counters) | File-based state or cross-process persistence |
| Explicit configuration via `ClientBuilder` or `ClientConfig::former()` | Hidden configuration from environment variables or defaults |

All enterprise reliability features (retry, circuit-breaker, rate-limiting, failover, health-checks) are behind Cargo feature flags and require explicit opt-in configuration. `Client::new()` activates zero enterprise features regardless of which feature flags are compiled in.

### Violation Consequences

Any automatic or implicit behavior in client code is a blocking code-review violation. The offending code must be removed or refactored to require explicit caller configuration before merge. There is no deprecation window — the constraint applies to all code at all times.

### Vocabulary

| Term | Definition |
|------|-----------|
| API Client | High-quality, ergonomic Rust client for Google Gemini API interactions |
| Thin Client | Client with zero client-side intelligence — transparent to server behavior |
| Explicit Configuration | All enterprise features require direct builder/config construction at call site |
| Process-Stateless | No file storage, databases, or cross-process state — runtime in-memory state permitted |
| Runtime State | In-memory operational state (caches, metrics, counters) that dies with the process |

### Sources

| File | Relationship |
|------|--------------|
| `src/client/core.rs` | Primary enforcement site — `Client` struct and all constructors |
| `src/client/config.rs` | Configuration types demonstrating explicit-builder requirement |
| `src/internal/http/enterprise.rs` | Enterprise feature dispatch — all activated only when explicitly configured |

### Tests

| File | Relationship |
|------|--------------|
| `tests/integration_tests.rs` | Core API integration tests verifying thin client behavior |
| `tests/common/mod.rs` | Test infrastructure following the no-implicit-behavior policy |

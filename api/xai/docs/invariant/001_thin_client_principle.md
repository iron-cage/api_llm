# Invariant: Thin Client Principle

### Scope

- **Purpose**: Enforce that `api_xai` is a stateless HTTP transport with no automatic or implicit behaviors — every client action is explicit, transparent, and direct.
- **Responsibility**: All contributors and code reviewers; any automatic or implicit behavior in client code is a blocking violation requiring immediate remediation.
- **In Scope**: All client methods, enterprise feature modules, streaming, error handling — every source file under `src/`.
- **Out of Scope**: Workspace-level configuration infrastructure, secret file loading mechanics, test helper utilities in `tests/`.

### Invariant Statement

The `api_xai` client is a **stateless HTTP transport layer**. It exposes X.AI Grok API functionality transparently without adding client-side intelligence, automatic behaviors, or implicit side effects. Every client action must be explicit (triggered only by direct caller instruction), transparent (behavior observable from the call site), and direct (one method call maps to at most one X.AI API endpoint). The X.AI API is OpenAI-compatible at the wire level; this client follows the "Thin Client, Rich API" governing principle shared across the `api_llm` workspace.

### Enforcement Mechanism

All enterprise reliability features (retry, circuit_breaker, rate_limiting, failover, health_checks, count_tokens, caching, batch_operations, performance_metrics, enhanced_tools) are behind Cargo feature flags and produce zero overhead when disabled. Code review blocks any automatic or implicit behavior before merge.

| Permitted | Prohibited |
|-----------|------------|
| Enterprise features compiled in via feature flag | Auto-retry without explicit configuration |
| Caching when `caching` feature flag enabled | Implicit response caching |
| Rate limiting when `rate_limiting` feature flag enabled | Auto rate limiting without configuration |
| Explicit validation via `validate_request()` | Automatic request mutation or normalization |
| Token counting via `count_tokens()` when feature enabled | Automatic token pre-computation on every request |

### Violation Consequences

Any automatic or implicit behavior in client code is a blocking code-review violation. The offending code must be removed or refactored to require explicit caller opt-in before merge. There is no deprecation window — the constraint applies to all code at all times.

### Sources

| File | Relationship |
|------|--------------|
| `src/client.rs` | Primary enforcement site — `Client` struct and all API methods |
| `src/lib.rs` | Feature-gated `layer` declarations — demonstrates explicit opt-in pattern |
| `src/client_api_accessors.rs` | `ClientApiAccessors<E>` trait — endpoint accessor pattern |

### Tests

| File | Relationship |
|------|--------------|
| `tests/` | Integration tests require real X.AI API keys — no mocking, no automatic fallback |

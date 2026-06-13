# Invariant: Thin Client Principle

### Scope

- **Purpose**: Enforce that `api_huggingface` is a stateless HTTP transport with no automatic or implicit behaviors — every client action is explicit, transparent, and direct.
- **Responsibility**: All contributors and code reviewers; any automatic or implicit behavior in client code is a blocking violation requiring immediate remediation.
- **In Scope**: All client methods, enterprise feature modules, streaming, multimodal handlers — every source file under `src/`.
- **Out of Scope**: Workspace-level configuration infrastructure, secret file loading mechanics, test helper utilities in `tests/`.

### Invariant Statement

The `api_huggingface` client is a **stateless HTTP transport layer**. It exposes HuggingFace Inference API and Router API functionality transparently without adding client-side intelligence, automatic behaviors, or implicit side effects. Every client action must be explicit (triggered only by direct caller instruction), transparent (behavior observable from the call site), and direct (one method call maps to at most one HuggingFace API endpoint). The crate follows the "Thin Client, Rich API" governing principle shared across the `api_llm` workspace.

### Enforcement Mechanism

All enterprise reliability features (circuit_breaker, rate_limiting, failover, health_checks, caching, performance_metrics, dynamic_configuration, token_counting, sync_api) are behind Cargo feature flags. Code review blocks any automatic or implicit behavior before merge.

| Permitted | Prohibited |
|-----------|------------|
| Enterprise features when feature flag enabled | Auto-retry without explicit configuration |
| Caching when `caching` feature enabled | Implicit response caching |
| Rate limiting when `rate_limiting` feature enabled | Auto rate limiting without developer opt-in |
| Explicit validation and model checking | Automatic request mutation or fallback model selection |

### Violation Consequences

Any automatic or implicit behavior in client code is a blocking code-review violation. The offending code must be removed or refactored to require explicit caller opt-in before merge.

### Sources

| File | Relationship |
|------|--------------|
| `src/client.rs` | Primary enforcement site — `Client` struct and all API methods |
| `src/lib.rs` | Feature-gated `layer` declarations — demonstrates explicit opt-in pattern |
| `src/providers.rs` | Router API — explicit model selection per request, no fallbacks |

### Tests

| File | Relationship |
|------|--------------|
| `tests/` | Integration tests require real HuggingFace API keys — no mocking, no automatic fallback |

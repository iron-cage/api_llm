# Invariant: Testing Standards

### Scope

- **Purpose**: Enforce zero-tolerance no-mock policy and loud-failure requirements for all `api_ollama` integration tests.
- **Responsibility**: Documents the Testing Standards invariant — no-mock mandate, loud-failure requirement, and violation consequences.
- **In Scope**: All files under `tests/` — integration tests, unit tests, manual test plans.
- **Out of Scope**: Example files in `examples/`, benchmark harnesses in `benches/`.

### Invariant Statement

Integration tests in `api_ollama` use **real Ollama server endpoints exclusively** and must fail immediately and loudly when a local Ollama server is unavailable. No mock servers, fake endpoints, or simulated responses are permitted in integration tests. Tests are feature-gated with `#[cfg(feature = "integration")]`; `integration` is included in `full` (and therefore in `default`), so `cargo nextest run --all-features` runs all integration tests.

### Enforcement Mechanism

Integration tests must: (1) connect to a real local Ollama server instance, (2) `panic!` with a clear diagnostic message if the server is unreachable, (3) call real Ollama API endpoints with no fallback path. Unit tests for pure logic may run without a server but must be separated via `#[cfg(feature = "integration")]` guards.

| Permitted | Prohibited |
|-----------|------------|
| Real API calls against a live local Ollama server | Mock servers or fake API responses |
| Loud `panic!` on missing server | Silent skip on missing server |
| Unit tests for pure logic (types, serialization) | Tests that silently pass without real API calls |

### Violation Consequences

Any mock usage in integration tests requires immediate remediation before merge. Tests that silently pass without a real Ollama server are a policy violation of equal severity.

### Sources

| File | Relationship |
|------|--------------|
| `src/` (secret/environment modules) | Credential and server URL loading with loud failure pattern |

### Tests

| File | Relationship |
|------|--------------|
| `tests/` | All integration test functions demonstrate `#[cfg(feature = "integration")]` + loud-failure pattern |

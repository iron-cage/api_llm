# Invariant: Testing Standards

### Scope

- **Purpose**: Enforce zero-tolerance no-mock policy and loud-failure requirements for all `api_huggingface` integration tests.
- **Responsibility**: All contributors; any mock usage in integration tests is a critical policy violation.
- **In Scope**: All files under `tests/` — integration tests, unit tests, manual test plans.
- **Out of Scope**: Example files in `examples/`, benchmark harnesses in `benches/`.

### Invariant Statement

Integration tests in `api_huggingface` use **real HuggingFace API endpoints exclusively** and must fail immediately and loudly when credentials are missing or authentication fails. No mock servers, fake API keys, or simulated responses are permitted in integration tests. Tests are feature-gated with `#[cfg(feature = "integration")]`; `integration` is included in `full` (and therefore in `default`), so `cargo nextest run --all-features` runs all integration tests.

### Enforcement Mechanism

Integration tests must: (1) load real credentials via `Secret::load_with_fallbacks("HUGGINGFACE_API_KEY")`, (2) `panic!` with a clear diagnostic message if no credential is found, (3) call real HuggingFace API endpoints with no fallback path. Unit tests for pure logic may run without credentials but must be clearly separated via `#[cfg(feature = "integration")]` guards.

| Permitted | Prohibited |
|-----------|------------|
| Real API calls with valid HuggingFace credentials | Mock servers or fake API responses |
| Loud `panic!` on missing credentials | Silent skip or graceful degradation on missing credentials |
| Unit tests for pure logic (types, validation) without I/O | Tests that silently pass without real API calls |

### Violation Consequences

Any mock usage in integration tests requires immediate remediation before merge. Tests that silently pass without real credentials are a policy violation of equal severity.

### Sources

| File | Relationship |
|------|--------------|
| `src/secret.rs` | `Secret::load_with_fallbacks()` — credential loading with loud failure |

### Tests

| File | Relationship |
|------|--------------|
| `tests/` | All integration test functions demonstrate the `#[cfg(feature = "integration")]` + loud-failure pattern |

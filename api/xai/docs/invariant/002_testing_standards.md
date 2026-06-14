# Invariant: Testing Standards

### Scope

- **Purpose**: Enforce zero-tolerance no-mock policy and loud-failure requirements for all `api_xai` integration tests.
- **Responsibility**: Documents the Testing Standards invariant — no-mock mandate, loud-failure requirement, and violation consequences.
- **In Scope**: All files under `tests/` — integration tests, unit tests, manual test plans.
- **Out of Scope**: Example files in `examples/`, benchmark harnesses in `benches/`.

### Invariant Statement

Integration tests in `api_xai` use **real X.AI API endpoints exclusively** and must fail immediately and loudly when credentials are missing or authentication fails. No mock servers, fake API keys, or simulated responses are permitted in integration tests. Tests are feature-gated with `#[cfg(feature = "integration")]`; `integration` is included in `full` (and therefore in `default`), so `cargo nextest run --all-features` runs all integration tests.

### Enforcement Mechanism

Integration tests must: (1) load real credentials via `Secret::load_with_fallbacks("XAI_API_KEY")`, (2) `panic!` with a clear diagnostic message if no credential is found, (3) call real X.AI API endpoints with no fallback path. Unit tests and logic tests may run without credentials but must be clearly separated from integration tests via `#[cfg(feature = "integration")]` guards.

| Permitted | Prohibited |
|-----------|------------|
| Real API calls with valid credentials | Mock servers or fake API responses in integration tests |
| Loud `panic!` on missing credentials | Silent skip or `eprintln!` on missing credentials |
| Unit tests for pure logic (types, validation) | `try_create_test_client()` returning `None` silently |
| `#[cfg(feature = "integration")]` per test function | File-level `#![cfg(feature = "integration")]` mixing unit + integration |

### Violation Consequences

Any mock usage in integration tests requires immediate remediation before merge. Tests that silently pass without real credentials are a policy violation of equal severity. The "loud failure" requirement is non-negotiable — integration tests are smoke tests against the live API.

### Sources

| File | Relationship |
|------|--------------|
| `src/secret.rs` | `Secret::load_with_fallbacks()` — credential loading with loud failure |

### Tests

| File | Relationship |
|------|--------------|
| `tests/` | All integration test functions demonstrate the `#[cfg(feature = "integration")]` + loud-failure pattern |

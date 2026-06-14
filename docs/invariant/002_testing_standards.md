# Invariant: Testing Standards

### Scope

- **Purpose**: Enforce zero-tolerance no-mock policy and loud-failure requirements for all integration tests across the `api_llm` workspace.
- **Responsibility**: Documents the Testing Standards invariant — no-mock mandate, loud-failure requirement, and violation consequences.
- **In Scope**: All files under `api/*/tests/` — integration tests, unit tests, manual test plans.
- **Out of Scope**: Example files in `api/*/examples/`, benchmark harnesses in `api/*/benches/`.

### Invariant Statement

Integration tests in the `api_llm` workspace use **real provider API endpoints exclusively** and must fail immediately and loudly when credentials are missing or authentication fails. No mock servers, fake API keys, or simulated responses are permitted in integration tests. Tests are feature-gated with `#[cfg(feature = "integration")]`; `integration` is included in `full` (and therefore in `default`), so `cargo nextest run --all-features` runs all integration tests.

### Enforcement Mechanism

Integration tests must: (1) load real credentials via `Secret::load_with_fallbacks("PROVIDER_API_KEY")`, (2) `panic!` with a clear diagnostic message if no credential is found, (3) call real provider API endpoints with no fallback path. Unit tests for pure logic may run without credentials but must be separated via `#[cfg(feature = "integration")]` guards.

| Permitted | Prohibited |
|-----------|------------|
| Real API calls with valid provider credentials | Mock servers or fake API responses |
| Loud `panic!` on missing credentials | Silent skip on missing credentials |
| Unit tests for pure logic (types, serialization) | Tests that silently pass without real API calls |

### Workspace Test Configuration

- `.config/nextest.toml`: `test-threads = 1` (serial execution), 3-minute default timeout, 10-minute for api_ollama integration binaries
- `secret/-secrets.sh` (gitignored): workspace secret file providing all provider API keys

### Violation Consequences

Any mock usage in integration tests requires immediate remediation before merge. Tests that silently pass without real credentials are a policy violation of equal severity.

### Sources

| File | Relationship |
|------|--------------|
| `api/*/src/` (secret modules) | Credential loading with loud failure pattern per crate |
| `.config/nextest.toml` | Workspace-wide test runner configuration |

### Tests

| File | Relationship |
|------|--------------|
| `api/*/tests/` | All integration test functions demonstrate `#[cfg(feature = "integration")]` + loud-failure pattern |

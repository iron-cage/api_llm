# Invariant: Testing Standards

### Scope

- **Purpose**: Enforce zero-tolerance no-mock policy and loud-failure requirements for all `api_huggingface` integration tests.
- **Responsibility**: Zero-tolerance no-mock and loud-failure requirements for all integration tests.
- **In Scope**: All files under `tests/` — integration tests, unit tests, manual test plans.
- **Out of Scope**: Example files in `examples/`, benchmark harnesses in `benches/`.

### Invariant Statement

Integration tests in `api_huggingface` use **real HuggingFace API endpoints exclusively** and must fail immediately and loudly when credentials are missing or authentication fails. No mock servers, fake API keys, or simulated responses are permitted in integration tests. Tests are feature-gated with `#[cfg(feature = "integration")]`; `integration` is included in `full` (and therefore in `default`), so `cargo nextest run --all-features` runs all integration tests.

### Enforcement Mechanism

Integration tests must: (1) load real credentials via `workspace_tools::workspace()?.load_secrets_from_file("-secrets.sh")` or `Secret::load_from_env("HUGGINGFACE_API_KEY")`, then `panic!` immediately if no credential is found, (2) call real HuggingFace API endpoints with no fallback path. Unit tests for pure logic may run without credentials but must be clearly separated via `#[cfg(feature = "integration")]` guards.

| Permitted | Prohibited |
|-----------|------------|
| Real API calls with valid HuggingFace credentials | Mock servers or fake API responses |
| Loud `panic!` on missing credentials | Silent skip or graceful degradation on missing credentials |
| Unit tests for pure logic (types, validation) without I/O | Tests that silently pass without real API calls |

### Violation Consequences

Any mock usage in integration tests requires immediate remediation before merge. Tests that silently pass without real credentials are a policy violation of equal severity.

### APIs

| File | Relationship |
|------|--------------|
| `api/001_reference.md` | All methods documented there require real-API integration tests under this invariant |

### Features

| File | Relationship |
|------|--------------|
| `feature/001_enterprise_reliability.md` | Enterprise features must be tested with real API calls under this invariant |

### Invariants

| File | Relationship |
|------|--------------|
| `invariant/001_thin_client_principle.md` | Companion invariant — governs no-mock and no-implicit behavior requirements |

### Sources

| File | Relationship |
|------|--------------|
| `src/secret.rs` | `Secret::load_from_env()` — environment variable credential loading |

### Tests

| File | Relationship |
|------|--------------|
| `tests/` | All integration tests MUST use `#[cfg(feature = "integration")]` gate and loud-failure pattern (no silent skip/pass permitted) |
| `tests/docs/invariant/02_testing_standards.md` | GWT spec scenarios for this doc instance |

# Invariant: Testing Standards

### Scope

- **Purpose**: Enforce real-implementation testing with no mocking and mandatory loud failure for all tests in `api_openai_compatible`.
- **Responsibility**: Documents the Testing Standards invariant — no-mock mandate, loud-failure requirement, and violation consequences.
- **In Scope**: All files under `tests/` in the `api_openai_compatible` crate.
- **Out of Scope**: Source files under `src/`, build scripts, examples.

### Invariant Statement

All tests in `api_openai_compatible` exercise real implementations — no mocking, no fakes, no stubs that return preset data in place of real behavior. Tests that require live HTTP must be gated with `#[cfg(feature = "integration")]` and must fail loudly when credentials are absent. Wire type tests use real serde implementations. A test that silently passes regardless of the system state is a defect.

### Enforcement Mechanism

Integration tests are gated with `#[cfg(feature = "integration")]` placed before `#[tokio::test]`. This is the canonical ordering per workspace test convention. Unit-level wire tests (serde, field presence) are always-on (`#[cfg(feature = "enabled")]`) and require no credentials. Test helpers that need credentials must panic loudly when credentials are absent — never silently return a dummy client.

| Permitted | Prohibited |
|-----------|------------|
| Real `reqwest::Client` over real HTTP in integration tests | `wiremock` or `mockito` HTTP interception in unit tests |
| Serde round-trip tests against real `serde_json` serialization | Hardcoded expected JSON without exercising the serializer |
| Integration tests gated with `#[cfg(feature = "integration")]` | Empty-body `#[tokio::test]` functions that always pass |
| `create_test_client()` panics on missing credentials | Silent `Option::None` returns when credentials absent |
| Test Matrix table documenting all covered scenarios | Undocumented test scenarios |

### Violation Consequences

A test that always passes regardless of system state is treated as a missing test — it provides no signal. Empty-body test functions, `println!()` used as behavioral verification, and tests that `unwrap_or(default_value)` away failures are all blocking violations.

### Sources

| File | Relationship |
|------|--------------|
| `tests/wire_test.rs` | Reference implementation — 22 always-on + 5 streaming-gated serde tests |
| `tests/environment_test.rs` | Reference implementation — 7 environment unit tests, no HTTP |
| `tests/sync_client_test.rs` | Sync wrapper unit tests — no live HTTP in default configuration |
| `tests/client_test.rs` | Integration tests — GET/POST success and error paths with real HTTP |
| `tests/error_test.rs` | Unit tests — Display formatting and From conversions for all error variants |

### Tests

| File | Relationship |
|------|--------------|
| `tests/wire_test.rs` | Self-enforcing: exercises real serde implementations with explicit assert |
| `tests/environment_test.rs` | Self-enforcing: exercises real header construction with explicit assert |
| `tests/client_test.rs` | Self-enforcing: real HTTP calls, loud failure on missing credentials |
| `tests/error_test.rs` | Self-enforcing: exercises real Display/From impls with explicit assert |

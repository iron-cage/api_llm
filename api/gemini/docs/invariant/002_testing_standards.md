# Invariant: Testing Standards

### Scope

- **Purpose**: Mandate real-API-only integration tests and loud failures — eliminate all silent passes, mocks, and graceful fallbacks from the test suite.
- **Responsibility**: CI gate enforces these standards; any mock usage or silent pass in integration tests is a blocking violation requiring immediate remediation.
- **In Scope**: All test files under `tests/` — integration tests, unit tests, example validation tests.
- **Out of Scope**: Example files in `examples/` (may use simplified patterns); benchmark harnesses in `benches/`.

### Invariant Statement

All integration tests must use real Gemini API endpoints exclusively — no mocks, no fake keys, no hardcoded responses, no graceful fallbacks. Every test must fail loudly and clearly when credentials or network are unavailable; silent passes that do not exercise their stated responsibility are prohibited.

**Missing API Key = Test Failure, not skip.**

### Enforcement Mechanism

The CI gate blocks merges that contain: hardcoded API responses, mock implementations, or any graceful-fallback logic (`Err(_) => { return; }` patterns). Each integration test function must carry `#[ cfg( feature = "integration" ) ]` immediately before its `#[ tokio::test ]` attribute.

Tests must load credentials via `Client::new()` which tries workspace secret file (`secret/-secrets.sh`) then `GEMINI_API_KEY` environment variable. Credentials must never be hardcoded.

| Permitted | Prohibited |
|-----------|------------|
| `panic!("GEMINI_API_KEY required...")` on missing key | `eprintln!(...); return;` silent skip |
| `common::create_integration_client()` shared helper | Duplicate local `create_test_client()` definitions |
| Real HTTP calls to `generativelanguage.googleapis.com` | Mock servers, hardcoded JSON, fake endpoints |
| `#[ cfg( feature = "integration" ) ]` gate per test | `#[ignore]` or conditional skip logic |

### Violation Consequences

Any mock usage or silent pass in integration tests is a blocking violation. The offending test must be rewritten to use real API endpoints and loud failure before the PR can merge. No test may be merged in a disabled, ignored, or silently-passing state.

### Sources

| File | Relationship |
|------|--------------|
| `tests/common/mod.rs` | Canonical integration client factory — enforces loud failure on missing key |
| `src/client/core.rs` | `Client::new()` — primary credential loading path for all tests |

### Tests

| File | Relationship |
|------|--------------|
| `tests/integration_tests.rs` | Core integration tests — primary compliance verification |
| `tests/api_key_failure_test.rs` | Verifies loud failure behavior when API key is absent |

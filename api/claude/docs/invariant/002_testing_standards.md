# Invariant: Testing Standards

### Scope

- **Purpose**: Mandate real-API-only integration tests and loud failures — eliminate all silent passes, mocks, and graceful fallbacks from the test suite.
- **Responsibility**: CI gate enforces these standards; any mock usage or silent pass in integration tests is a blocking violation requiring immediate remediation.
- **In Scope**: All test files under `tests/` — integration tests, unit tests, manual tests.
- **Out of Scope**: Example files in `examples/` (may use simplified patterns); benchmark harnesses in `benches/`.

### Invariant Statement

All integration tests must use real Anthropic API endpoints exclusively — no mocks, no fake keys, no hardcoded responses, no graceful fallbacks. Every test must fail loudly and clearly when credentials or network are unavailable; silent passes that do not exercise their stated responsibility are prohibited.

### Enforcement Mechanism

The CI gate blocks merges that contain: fake API keys (`sk-ant-test-*` patterns), mock HTTP servers (wiremock, httpmock), hardcoded JSON responses, simulated errors that bypass real API paths, or any graceful-fallback logic. Each integration test function must carry `#[cfg(feature = "integration")]` immediately before its `#[tokio::test]` or `#[test]` attribute — exactly one `#[cfg]` gate per test, no duplicates. Tests must load credentials via `Client::from_workspace()` or `Client::from_env()` using `.expect(...)` — never `if let Ok`. Credentials must be loaded via workspace credential files (`secret/-secrets.sh`); no hardcoded credentials anywhere.

### Violation Consequences

Any mock usage or silent pass in integration tests is a blocking violation. CI fails the build. The offending test must be rewritten to use real API endpoints and loud failure before the PR can merge. No test may be merged in a disabled, ignored, or silently-passing state.

### Sources

| File | Relationship |
|------|--------------|
| `tests/tests.rs` | Root test file — contains strict policy declaration and module aggregation |
| `tests/inc/mod.rs` | Test module aggregator — all included modules must comply |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/invariant/02_testing_standards.md` | Behavioral spec — 6 scenarios verifying no-mock mandate and loud-failure requirement |
| `tests/inc/` | All test files — each file is subject to these standards; compliance verified at code review |

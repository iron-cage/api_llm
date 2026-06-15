# Invariant Spec: Testing Standards
**Source:** `../../docs/invariant/002_testing_standards.md`

## Test Cases

### IN-01: Integration test loads real credentials ✅

- **Given:** An integration test function gated with `#[cfg(feature = "integration")]`
- **When:** The test creates a client via `Secret::load_with_fallbacks("XAI_API_KEY")`
- **Then:** A real `Secret` value is returned containing a key starting with `xai-`
- **Test:** `inc/test_helpers.rs::create_test_client` — panics if `XAI_API_KEY` is absent;
  used by all `integration_*.rs` files.

### IN-02: Missing API key causes loud panic ✅

- **Given:** No `XAI_API_KEY` environment variable set and no workspace secrets file present
- **When:** `Secret::load_with_fallbacks("XAI_API_KEY")` is called
- **Then:** The call panics with a diagnostic message identifying the missing key — no silent skip, no `eprintln`-only fallback
- **Test:** `secret_tests.rs::secret_load_with_fallbacks_fails_when_all_sources_unavailable`
  (returns `Err` which `create_test_client` converts to a panic via `.expect(...)`).

### IN-03: Integration gate is per-function not per-file ✅

- **Given:** A test file containing both unit tests and integration tests
- **When:** The crate is compiled without the `integration` feature
- **Then:** Only the `#[cfg(feature = "integration")]`-gated test functions are excluded; unit tests in the same file still compile and run
- **Test:** `components_tests.rs::unit_test_in_file_with_integration_gate_compiles_and_runs`

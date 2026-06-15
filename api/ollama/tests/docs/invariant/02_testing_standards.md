# Invariant Spec: Testing Standards

**Source:** `../../docs/invariant/002_testing_standards.md`

### IN-01: Integration test connects to real local Ollama server ✅

- **Given:** An integration test function gated with `#[cfg(feature = "integration")]`
- **When:** The test executes with the `integration` feature enabled
- **Then:** The test makes at least one real HTTP call to a local Ollama server endpoint
- **Test:** `api_comprehensive_tests.rs::test_integration_simple_chat`,
  `test_integration_list_models`, `test_integration_simple_generation` — all use
  `with_test_server!` which starts a real local Ollama instance.

### IN-02: Missing server causes loud panic, not silent skip ✅

- **Given:** The local Ollama server is unreachable (not running or wrong host)
- **When:** An integration test attempts to create a client or send a request
- **Then:** The test panics with a diagnostic message containing the connection target — it does not skip or pass silently
- **Test:** The `server_helpers.rs::with_test_server!` macro panics on server startup failure
  (verified by the `MANDATORY STRICT FAILURE POLICY` comment block in `api_comprehensive_tests.rs`).
  Non-integration tests confirm error propagation: `error_handling_tests.rs::test_list_models_network_error`.

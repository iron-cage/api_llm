# Invariant Spec: Testing Standards

**Source:** `../../docs/invariant/002_testing_standards.md`

### IN-01: Integration test connects to real local Ollama server

- **Given:** An integration test function gated with `#[cfg(feature = "integration")]`
- **When:** The test executes with the `integration` feature enabled
- **Then:** The test makes at least one real HTTP call to a local Ollama server endpoint

### IN-02: Missing server causes loud panic, not silent skip

- **Given:** The local Ollama server is unreachable (not running or wrong host)
- **When:** An integration test attempts to create a client or send a request
- **Then:** The test panics with a diagnostic message containing the connection target — it does not skip or pass silently

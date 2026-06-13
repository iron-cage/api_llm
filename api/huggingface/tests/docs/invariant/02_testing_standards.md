# Invariant Spec: Testing Standards

Spec scenarios for `docs/invariant/002_testing_standards.md`. Verifies that the test suite uses real API credentials exclusively and fails loudly on missing credentials.

## IN-05: Missing credentials cause immediate loud failure

**Given:** no `HUGGINGFACE_API_KEY` in the environment and no workspace secrets file
**When:** any integration test that calls a real API is executed
**Then:** the test panics immediately with a descriptive message identifying the missing credential; the panic occurs before any network request is attempted

## IN-06: No mock server or fake HTTP client in integration tests

**Given:** all `.rs` files under `tests/` compiled with `--features integration`
**When:** the test source code is inspected for mock usage
**Then:** no `wiremock`, `mockito`, `httpmock`, fake HTTP adapter, or in-process stub server is imported or used in any integration-gated test function

## IN-07: Integration test functions carry cfg feature gate

**Given:** any test function in `tests/` that calls a real HuggingFace endpoint
**When:** the source file is inspected
**Then:** `#[cfg(feature = "integration")]` appears as the line immediately preceding `#[tokio::test]` on that function; no other lines appear between the two attributes

## IN-08: integration feature is reachable through default

**Given:** `Cargo.toml` for `api_huggingface`
**When:** the `default` feature dependency chain is resolved
**Then:** `integration` is reachable (either `default → full → integration` or `default → integration` directly); `cargo nextest run --all-features` therefore runs all integration tests

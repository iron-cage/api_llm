# Invariant Spec: Testing Standards

**Source:** [`docs/invariant/002_testing_standards.md`](../../docs/invariant/002_testing_standards.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| IN-07 | Integration files gated by integration feature | feature gate | ✅ |
| IN-08 | Missing credential causes loud failure | loud failure | ✅ |
| IN-09 | No mock HTTP servers in any test file | no-mock mandate | ✅ |
| IN-10 | No hardcoded API key strings in test files | no-mock mandate | ✅ |
| IN-11 | No conditional skip logic in integration tests | loud failure | ✅ |
| IN-12 | No disabled or ignored tests in any test file | no-disabled mandate | ✅ |

---

### IN-07: Integration test functions gated by integration feature

- **Given:** Test functions that make real API calls exist under `tests/inc/`
- **When:** Each integration test function is inspected
- **Then:** Every integration test function carries exactly one `#[cfg(feature = "integration")]` attribute immediately before its `#[tokio::test]` or `#[test]` attribute — no duplicates, no omissions; no integration test runs without the `integration` feature flag enabled

---

### IN-08: Missing credential causes loud failure

- **Given:** `ANTHROPIC_API_KEY` is absent from the process environment and `secret/-secrets.sh` does not contain a valid key
- **When:** A test that calls `Client::from_workspace()` or `Client::from_env()` runs
- **Then:** The test panics or returns an error immediately with an explicit, actionable message that names the missing credential source; the test never silently passes without actually calling the API

---

### IN-09: No mock HTTP servers in any test file

- **Given:** All test files under `tests/`
- **When:** Their contents are inspected for mock HTTP server usage
- **Then:** No test file imports or references `wiremock`, `httpmock`, or any other HTTP request interceptor crate; all HTTP traffic goes to real Anthropic API endpoints

---

### IN-10: No hardcoded API key strings in test files

- **Given:** All test files under `tests/`
- **When:** Their contents are inspected for hardcoded credential patterns
- **Then:** No test file contains string literals matching `sk-ant-test-*` or any other hardcoded Anthropic API key pattern; all credentials are loaded dynamically from environment or workspace secrets

---

### IN-11: No conditional skip logic in integration tests

- **Given:** Integration test functions that exercise real API endpoints
- **When:** Their source is inspected for conditional branching on credential availability
- **Then:** No test uses `if let Ok(client) = ...` or similar patterns that allow the test body to be skipped when credentials are unavailable; credential loading uses `.expect("...")` or equivalent unconditional assertion; the test either runs fully or fails loudly

---

### IN-12: No disabled or ignored tests in any test file

- **Given:** All test files under `tests/`
- **When:** Their contents are inspected for disabled test attributes
- **Then:** No test function carries `#[ignore]`, no module carries `#[cfg(ignore)]`, and no test is commented out with `//`; every test function is active and either passes or fails loudly; a disabled test is treated as a missing test — remove it or fix it

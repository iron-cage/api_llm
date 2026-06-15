# Invariant Spec: Testing Standards

**Source:** [`docs/invariant/002_testing_standards.md`](../../../docs/invariant/002_testing_standards.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| IN-01 | Integration test loads real credentials | real-credentials | ✅ |
| IN-02 | Missing API key causes panic with diagnostic message | loud-failure | ✅ |

---

### IN-01: Integration test loads real credentials

- **Given:** An integration test function gated with `#[cfg(feature = "integration")]`
- **When:** The test initializes a client via `Secret::load_with_fallbacks("OPENAI_API_KEY")` and calls a real OpenAI endpoint
- **Then:** The client uses a real API key loaded from the environment or workspace secrets; no mock server or fake key is involved; the test exercises the live OpenAI API

---

### IN-02: Missing API key causes panic with diagnostic message

- **Given:** No `OPENAI_API_KEY` environment variable is set and no workspace secret file is present
- **When:** `Secret::load_with_fallbacks("OPENAI_API_KEY")` is called during test setup
- **Then:** The call panics with a clear diagnostic message indicating the missing credential; the test does not silently skip or return `Ok` without exercising an API call

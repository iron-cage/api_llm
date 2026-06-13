# Pattern Spec: Quick Response

**Source:** [`docs/pattern/001_quick_response.md`](../../docs/pattern/001_quick_response.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| PT-01 | Quick Response pattern propagates all errors to caller | quick response | ✅ |
| PT-02 | Quick Response pattern avoids retry and fallback logic | quick response | ✅ |

---

### PT-01: Quick Response pattern propagates all errors to caller

- **Given:** A caller using the Quick Response pattern for single-turn text generation
- **When:** The Gemini API returns an error (rate limit, network timeout, authentication failure)
- **Then:** The error propagates directly to the caller as `Err(...)`; no retry, no fallback, and no default value is substituted by the wrapper; the caller is responsible for all error handling and retry decisions

---

### PT-02: Quick Response pattern avoids retry and fallback logic

- **Given:** A Quick Response wrapper that constructs a `GenerateContentRequest` with defaults and calls `generate_content()`
- **When:** The pattern is inspected for its structure
- **Then:** The wrapper contains no retry loop, no `match` on `Err(_)` that returns a default, and no enterprise feature configuration; it calls `generate_content()` once and returns the `Result` directly; its applicability is limited to call sites where the caller controls error handling

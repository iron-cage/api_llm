# Pattern Spec: Error-Resilient

**Source:** [`docs/pattern/002_error_resilient.md`](../../docs/pattern/002_error_resilient.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| PT-03 | Error-Resilient pattern maps error variants to fallback strings | error resilient | ✅ |
| PT-04 | Error-Resilient pattern absorbs error signal from caller | error resilient | ✅ |

---

### PT-03: Error-Resilient pattern maps error variants to fallback strings

- **Given:** A UI rendering function that cannot propagate `Result` to its caller and requires a `String` return type
- **When:** The Error-Resilient pattern is applied and `generate_content()` returns an error
- **Then:** The error is matched against specific variants (`RateLimitError`, `TimeoutError`, and a general catch-all); each variant maps to a distinct, user-readable fallback string; the function always returns a `String` — never panics, never propagates an error

---

### PT-04: Error-Resilient pattern absorbs error signal from caller

- **Given:** A function using the Error-Resilient pattern that received an authentication error
- **When:** The caller inspects the return value
- **Then:** The caller receives a fallback string rather than the original error; the caller cannot distinguish authentication failure from timeout failure from rate limiting from the return value alone; this is the documented trade-off — error transparency is sacrificed for a guaranteed non-failing return

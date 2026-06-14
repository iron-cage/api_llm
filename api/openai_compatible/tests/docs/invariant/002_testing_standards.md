# Invariant Spec: Testing Standards

**Source:** [`docs/invariant/002_testing_standards.md`](../../../docs/invariant/002_testing_standards.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| IN-05 | Wire tests use real serde_json — no hardcoded byte comparison | real-implementation | ✅ |
| IN-06 | Integration test gate appears before test attribute | gate-ordering | ✅ |
| IN-07 | Assertions use assert_eq!/assert! — not println! | loud-failure | ✅ |
| IN-08 | Test Matrix table covers all test functions in wire_test.rs | matrix-coverage | ✅ |

---

### IN-05: Wire tests use real serde_json — no hardcoded byte comparison

- **Given:** A `ChatCompletionRequest` struct with known field values constructed using the builder
- **When:** Serialized with `serde_json::to_string` and deserialized back with `serde_json::from_str`
- **Then:** The deserialized struct equals the original struct field-by-field; the test exercises the live serde implementation path, not a comparison against a hardcoded expected byte string

---

### IN-06: Integration test gate appears before test attribute

- **Given:** Any test function in `tests/` that makes a live HTTP call to an external API
- **When:** The attribute ordering of the test function is inspected
- **Then:** `#[cfg(feature = "integration")]` appears immediately before `#[tokio::test]`; the integration gate is never placed after the test attribute, at file level only, or omitted; the canonical order is gate then test attribute

---

### IN-07: Assertions use assert_eq!/assert! — not println!

- **Given:** Any test function in the `api_openai_compatible` test suite asserting observable behavior
- **When:** The assertion mechanism is identified
- **Then:** Behavioral verification uses `assert_eq!`, `assert!`, or `assert_ne!` with a diagnostic message; `println!()` is not the sole verification mechanism in any test function; a test that only prints output and never asserts is a defect

---

### IN-08: Test Matrix table covers all test functions in wire_test.rs

- **Given:** The module-level doc comment of `tests/wire_test.rs`
- **When:** The Test Matrix table rows are counted and compared against the number of `#[test]` functions in the file
- **Then:** Every test function in `wire_test.rs` has exactly one corresponding row in the Test Matrix table; the row count equals the function count; no test function is undocumented

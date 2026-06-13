# BUG-009: cosine_similarity returns values outside [-1.0, 1.0]

- **Severity:** Minor
- **State:** Fixed
- **Affects:** `cosine_similarity()` for nearly-collinear high-dimensional vectors
- **Component:** `src/embeddings.rs`
- **Filed:** 2026-06-13
- **Updated:** 2026-06-13

## Symptom

```rust
// For nearly-identical high-dimensional float vectors:
cosine_similarity(&vec_a, &vec_b)  // returned 1.0000001 instead of 1.0
```

## Impact

Results outside `[-1.0, 1.0]` violate the mathematical invariant of cosine similarity and can cause `acos()` callers to receive `NaN` (since `acos` is only defined for `[-1.0, 1.0]`). Downstream similarity comparisons expecting the bounded range silently produce wrong results.

## How Discovered

Discovered during test review: the test helper was clamping but the production implementation was not, masking the gap.

## Root Cause

IEEE 754 floating-point rounding accumulates error in the dot product and magnitude calculations. For nearly-collinear vectors, `dot / (|a| * |b|)` can slightly exceed 1.0. The implementation had no clamp.

## Fix Location

`src/embeddings.rs` — return expression.

Before: `dot / norm`

After: `(dot / norm).clamp(-1.0, 1.0)`

## Prevention

- **Pitfall:** Cosine similarity is mathematically bounded to `[-1.0, 1.0]`, but IEEE 754 arithmetic can violate this for nearly-collinear vectors. Always clamp at the production site, not only in test helpers.
- **Pitfall:** If a test helper applies clamping that the production function doesn't, tests will pass while production silently produces out-of-range values.

## History

- 2026-06-13 REPORT — bug filed; MRE test `test_cosine_similarity_clamping` written
- 2026-06-13 CLOSE — fix applied; all tests pass

## Refs Tests

- `tests/embeddings_tests.rs` — `test_cosine_similarity_clamping`

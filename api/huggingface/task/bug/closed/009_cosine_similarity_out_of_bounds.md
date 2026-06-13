# BUG-009: cosine_similarity returns values outside [-1.0, 1.0]

- **Severity:** Medium
- **State:** Fixed
- **Affects:** `cosine_similarity()` called with nearly-identical high-dimensional vectors
- **Component:** `embeddings.rs`
- **Filed:** 2026-06-13
- **Validated By:** cargo nextest
- **Validation Date:** 2026-06-13

## Symptom

```bash
# cosine_similarity(a, b) where a ≈ b, high-dimensional → 1.0000001
# Expected invariant: result ∈ [-1.0, 1.0]
# AP-03 documents this invariant explicitly
```

## Impact

Any code comparing or processing similarity scores assumes values in [-1.0, 1.0]. A value like 1.0000001 can silently corrupt downstream computations (e.g., `acos(1.0000001)` returns NaN in many float libraries). Medium severity because the violation is rare and the API documents the bound.

## How Discovered

Code review of `cosine_similarity` during audit revealed the return expression `dot / (mag_a * mag_b)` had no clamping, while a test helper locally applied clamping — masking the production gap.

## Minimum Reproducible Example

```bash
# High-dimensional vector of small equal values; accumulated fp rounding pushes result over 1.0
let v: Vec<f32> = vec![0.001; 1000];
let sim = cosine_similarity(&v, &v).unwrap();
assert!(sim <= 1.0);  # could fail pre-fix
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|-------|---------|----------|
| H1 | IEEE 754 rounding accumulates across high-dimensional dot products | ✅ Root Cause | Floating-point arithmetic well-known property | E1 |
| H2 | No clamping in production code; clamping only in test helper | ✅ Root Cause | Source inspection: no `.clamp()` at return site | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | IEEE 754 spec | Accumulated rounding in high-dim dot products can exceed exact value | H1 ✅ |
| E2 | `src/embeddings.rs:258` pre-fix | `Ok(dot / (mag_a * mag_b))` — no `.clamp(-1.0, 1.0)` | H2 ✅ |

## Root Cause

```
cosine_similarity(v, v) where v has 1000 elements
  → dot_product = Σ v[i]² — accumulated fp rounding
  → mag_a * mag_b = (Σ v[i]²) — same accumulation
  → ratio = dot / (mag_a * mag_b) = 1.0 + ε for some tiny ε > 0
  → returned without clamping → violates [-1.0, 1.0] invariant
```

## Why Not Caught

The test helper function added clamping locally, masking the gap between test behavior and production behavior. The difference between the helper's output and the production function's output was not tested directly.

## Fix Location

`src/embeddings.rs:258`

```rust
// Before:
Ok(dot_product / (magnitude_a * magnitude_b))

// After:
Ok((dot_product / (magnitude_a * magnitude_b)).clamp(-1.0, 1.0))
```

## Prevention

When implementing mathematical operations with invariant bounds, apply clamping at the production site — not only in test helpers.

**Pitfall:** Cosine similarity is mathematically bounded to [-1.0, 1.0], but IEEE 754 floating-point arithmetic can yield values slightly outside this range for nearly-collinear vectors due to accumulated rounding.

## Generalized Version

**Broken assumption:** Exact mathematical bounds hold for all IEEE 754 computations.
**Failure conditions:** High-dimensional dot products with accumulated rounding; nearly-identical vectors.
**Detection invariant:** Any function that returns a value with a documented range bound must `.clamp()` the result at the production return site.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-06-13 | filed | Discovered during review; test helper masking prevented earlier detection |
| 2026-06-13 | closed | Fix verified by `test_cosine_similarity_clamping` passing |

## Refs: src/

- `src/embeddings.rs` — `cosine_similarity()`: `.clamp(-1.0, 1.0)` added to return expression

## Refs: tests/

- `tests/embeddings_tests.rs` — `test_cosine_similarity_clamping`: MRE reproducer

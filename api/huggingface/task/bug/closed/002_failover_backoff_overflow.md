# BUG-002: Failover backoff u64 overflow

- **Severity:** High
- **State:** Fixed
- **Affects:** `execute_with_failover()` when `max_retries >= 57`
- **Component:** `reliability/failover.rs`
- **Filed:** 2026-06-13
- **Validated By:** cargo nextest
- **Validation Date:** 2026-06-13

## Symptom

```bash
# max_retries = 57 → attempts-1 = 56 → 500 * 2^56 overflows u64
# Debug build: thread panic (arithmetic overflow)
# Release build: silent u64 wraparound → incorrect (very small) delay
```

## Impact

Any configuration with `max_retries >= 57` triggers a panic in debug builds or silent wraparound in release. Severity is High because release mode silently produces a corrupted backoff delay, masking the fault.

## How Discovered

```bash
$ cargo nextest run -- test_failover_backoff_delay_no_overflow_for_high_attempt_counts
# panicked at 'attempt to multiply with overflow'
```

## Minimum Reproducible Example

```bash
# 500 * 2u64.pow(56) = 500 * 72_057_594_037_927_936 = 36_028_797_018_963_968_000
# u64::MAX                                           = 18_446_744_073_709_551_615
# → overflow: 36e18 > 18e18
let delay = 500 * 2u64.pow(56);  # panics in debug
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|-------|---------|----------|
| H1 | `2u64.pow(56)` exceeds u64::MAX when multiplied by 500 | ✅ Root Cause | Arithmetic confirms overflow | E1 |
| H2 | `.min(5000)` post-multiply cannot prevent overflow | ✅ Root Cause | Overflow occurs before `.min()` | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/reliability/failover.rs:360` | `500 * 2u64.pow(attempts - 1)` with uncapped exponent | H1 ✅ |
| E2 | Rust semantics | Panic happens at the `*` operator, before any `.min()` | H2 ✅ |

## Root Cause

```
execute_with_failover(max_retries=57)
  → attempts reaches 57
  → exponent = attempts - 1 = 56
  → 500 * 2u64.pow(56) → overflow (debug: panic; release: wrap)
  → `.min(5000)` never reached
```

## Why Not Caught

No test used `max_retries >= 57`. Typical values in tests are 1–5. The extreme boundary was never exercised.

## Fix Location

`src/reliability/failover.rs:360`

```rust
// Before:
let delay_ms = 500 * 2u64.pow(attempts - 1);

// After:
let exp = (attempts - 1).min(13);  // 500 * 2^13 = 4_096_000 > 5000; .min(5000) still caps
let delay_ms = 500 * 2u64.pow(exp);
```

## Prevention

When computing exponential backoff with a capped result, cap the exponent *before* the multiplication — not the product after. Post-multiply clamping cannot prevent arithmetic overflow.

**Pitfall:** `x.min(cap)` after `a * b` is too late if `a * b` already overflows.

## Generalized Version

**Broken assumption:** `attempts - 1` always stays within exponent range for u64.
**Failure conditions:** `max_retries >= 57` → exponent ≥ 56 → overflow.
**Detection invariant:** `exp.max(0).min(floor(log2(u64::MAX / base)))` before exponentiation.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-06-13 | filed | Discovered during code_hyg_l1 audit of failover.rs |
| 2026-06-13 | closed | Fix verified by `test_failover_backoff_delay_no_overflow_for_high_attempt_counts` passing |

## Refs: src/

- `src/reliability/failover.rs` — `execute_with_failover()`: cap exponent before multiply

## Refs: tests/

- `tests/failover_tests.rs` — `test_failover_backoff_delay_no_overflow_for_high_attempt_counts`: MRE reproducer

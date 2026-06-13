# BUG-003: Temperature validator dead NaN/Inf check

- **Severity:** Minor
- **State:** Fixed
- **Affects:** `validate_temperature()` called with `f32::NAN` or `f32::INFINITY`
- **Component:** `validation.rs`
- **Filed:** 2026-06-13
- **Validated By:** cargo nextest
- **Validation Date:** 2026-06-13
- **Related Bugs:** [BUG-004](./004_top_p_nan_dead_code.md), [BUG-005](./005_frequency_penalty_nan_dead_code.md), [BUG-006](./006_presence_penalty_nan_dead_code.md), [BUG-007](./007_repetition_penalty_nan_dead_code.md)

## Symptom

```bash
# validate_temperature(f32::NAN) → Err("temperature must be between 0.0 and 2.0")
# Expected:          → Err("temperature must be a valid number")
# The explicit is_nan() / is_infinite() guard fires last (dead code) instead of first.
```

## Impact

Users passing `NaN` or `Inf` receive a misleading error message about range bounds rather than "valid number". Minor severity — validation still rejects the input; only the error message is wrong.

## How Discovered

Code review of `validate_temperature` noticed `is_nan()` / `is_infinite()` checks appeared *after* the `contains()` range check. NaN comparisons are always false — `contains()` silently catches them and emits the wrong message.

## Minimum Reproducible Example

```bash
# In Rust: the range check catches NaN before the explicit NaN guard
assert_eq!(
  validate_temperature(f32::NAN),
  Err("temperature must be between 0.0 and 2.0")  # wrong message
);
# After fix:
assert_eq!(
  validate_temperature(f32::NAN),
  Err("temperature must be a valid number")  # correct message
);
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|-------|---------|----------|
| H1 | Range `contains()` catches NaN silently (NaN comparisons always false) | ✅ Root Cause | IEEE 754 semantics confirmed | E1 |
| H2 | `is_nan()` guard appeared after `contains()`, making it unreachable for NaN inputs | ✅ Root Cause | Source inspection at pre-fix line 179 | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | IEEE 754 | `NaN >= 0.0` evaluates to `false`; `contains()` returns `false` for NaN | H1 ✅ |
| E2 | `src/validation.rs` pre-fix | `is_nan()` check appears after the `(0.0..=2.0).contains()` check | H2 ✅ |

## Root Cause

```
validate_temperature(f32::NAN)
  → (0.0..=2.0).contains(&NaN) == false  (NaN comparisons are false)
  → early return Err("...between 0.0 and 2.0")
  → is_nan() / is_infinite() guards never reached
```

## Why Not Caught

Tests covered wrong-range values (e.g., `3.0`, `-0.1`) and valid values. No test passed `NaN` or `Infinity` to validate the error *message* (only the error *existence* was asserted elsewhere).

## Fix Location

`src/validation.rs:179`

```rust
// Before (NaN/Inf guards after range check — dead code for NaN):
if !(0.0..=2.0).contains(&temperature) { return Err("...between 0.0 and 2.0"); }
if temperature.is_nan() { return Err("...valid number"); }  // unreachable for NaN

// After (NaN/Inf guards before range check):
if temperature.is_nan() || temperature.is_infinite() { return Err("...valid number"); }
if !(0.0..=2.0).contains(&temperature) { return Err("...between 0.0 and 2.0"); }
```

## Prevention

In float validators: always check `is_nan()` / `is_infinite()` **before** any range check. Range `contains()` catches NaN silently with a misleading "out of range" error.

**Pitfall:** `(a..=b).contains(&x)` returns `false` for NaN — same as out-of-range — so `is_nan()` after `contains()` is unreachable dead code.

## Generalized Version

**Broken assumption:** `is_nan()` after a range check is reachable.
**Failure conditions:** Any float validator where explicit NaN guard follows `contains()`.
**Detection invariant:** In all float validators, `is_nan()` and `is_infinite()` must precede every `contains()` call. Clippy's `clippy::float_cmp` and dead-code analysis can help detect these.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-06-13 | filed | Discovered during code_hyg_l1 audit; same pattern in 4 other validators |
| 2026-06-13 | closed | Fix verified by `test_validate_temperature_nan_gives_valid_number_error` passing |

## Refs: src/

- `src/validation.rs` — `validate_temperature()`: NaN/Inf guard moved before range check

## Refs: tests/

- `tests/validation_tests.rs` — `test_validate_temperature_nan_gives_valid_number_error`: MRE reproducer

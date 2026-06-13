# BUG-003: Temperature NaN/Inf passes with wrong error message

- **Severity:** Medium
- **State:** Fixed
- **Affects:** `validate_temperature()` with `f32::NAN` or infinite inputs
- **Component:** `src/validation.rs`
- **Filed:** 2026-06-13
- **Updated:** 2026-06-13

## Symptom

```
// Expected: "must be a valid number"
// Got:      "must be between 0.0 and 2.0"
validate_temperature(f32::NAN)  // returns Err with wrong message
```

## Impact

NaN and Inf inputs are rejected (good), but the error message says "between 0.0 and 2.0" instead of "must be a valid number". Callers receive a misleading diagnostic. Additionally, the `is_nan()` / `is_infinite()` guards after the range check are dead code.

## Root Cause

The NaN/Inf guard came *after* the range check. `NaN < 0.0` is `false` and `NaN > 2.0` is `false`, so NaN accidentally passes the range check, but `NaN` is not *in* the `0.0..=2.0` range either — Rust's `RangeInclusive::contains` uses `<=` comparison which returns `false` for NaN, causing the range check to fire with "between 0.0 and 2.0". The subsequent NaN check was therefore unreachable.

## Fix Location

`src/validation.rs:179`

Before: range check first, then `is_nan()` / `is_infinite()` guard.

After: `if v.is_nan() || v.is_infinite() { return Err(...) }` placed **before** the range check.

## Prevention

- **Pitfall:** `f32/f64 RangeInclusive::contains()` silently catches NaN because NaN comparisons return `false`, making the range check look like it fired on NaN when actually only the containing membership logic triggered. Always guard NaN/Inf *before* range checks in float validators.

## History

- 2026-06-13 REPORT — bug filed; MRE test `test_validate_temperature_nan_gives_valid_number_error` written
- 2026-06-13 CLOSE — fix applied; all tests pass

## Refs Tests

- `tests/validation_tests.rs` — `test_validate_temperature_nan_gives_valid_number_error`

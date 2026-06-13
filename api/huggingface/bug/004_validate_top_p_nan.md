# BUG-004: top_p NaN/Inf passes with wrong error message

- **Severity:** Medium
- **State:** Fixed
- **Affects:** `validate_top_p()` with `f32::NAN` or infinite inputs
- **Component:** `src/validation.rs`
- **Filed:** 2026-06-13
- **Updated:** 2026-06-13

## Symptom

Same pattern as BUG-003: NaN/Inf inputs to `validate_top_p()` produce "must be between 0.0 and 1.0" instead of "must be a valid number".

## Impact

Same as BUG-003: valid rejection, wrong error message; NaN/Inf guard is dead code.

## Root Cause

Same as BUG-003: NaN/Inf guard placed after range check. `0.0..=1.0` range check fires first for NaN, producing wrong message.

## Fix Location

`src/validation.rs:238`

Before: range check first.

After: `if v.is_nan() || v.is_infinite()` guard placed before range check.

## Prevention

See BUG-003 — same pitfall applies to all float range validators.

## History

- 2026-06-13 REPORT — bug filed; MRE test `test_validate_top_p_nan_gives_valid_number_error` written
- 2026-06-13 CLOSE — fix applied; all tests pass

## Refs Tests

- `tests/validation_tests.rs` — `test_validate_top_p_nan_gives_valid_number_error`

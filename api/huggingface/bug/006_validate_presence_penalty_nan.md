# BUG-006: presence_penalty NaN/Inf passes with wrong error message

- **Severity:** Medium
- **State:** Fixed
- **Affects:** `validate_presence_penalty()` with `f32::NAN` or infinite inputs
- **Component:** `src/validation.rs`
- **Filed:** 2026-06-13
- **Updated:** 2026-06-13

## Symptom

Same pattern as BUG-003: NaN/Inf inputs to `validate_presence_penalty()` produce "must be between -2.0 and 2.0" instead of "must be a valid number".

## Impact

Same as BUG-003: valid rejection, wrong error message; NaN/Inf guard is dead code.

## Root Cause

Same as BUG-003: NaN/Inf guard placed after range check.

## Fix Location

`src/validation.rs:410`

Before: range check first.

After: `if v.is_nan() || v.is_infinite()` guard placed before range check.

## Prevention

See BUG-003 — same pitfall applies to all float range validators.

## History

- 2026-06-13 REPORT — bug filed; MRE test `test_validate_presence_penalty_nan_gives_valid_number_error` written
- 2026-06-13 CLOSE — fix applied; all tests pass

## Refs Tests

- `tests/validation_tests.rs` — `test_validate_presence_penalty_nan_gives_valid_number_error`

# BUG-007: repetition_penalty NaN/Inf dead-code guard (wrong order)

- **Severity:** Medium
- **State:** Fixed
- **Affects:** `validate_repetition_penalty()` with `f32::NAN` or `f32::INFINITY` inputs
- **Component:** `src/validation.rs`
- **Filed:** 2026-06-13
- **Updated:** 2026-06-13

## Symptom

`+Inf` was silently caught by `penalty > 10.0` producing "too high", and `-Inf` by `penalty <= 0.0` producing "must be positive" — neither produced "must be a valid number". NaN produced "must be positive" (because `NaN <= 0.0` is `false`, then fell through to next check).

## Impact

Non-finite values produce misleading error messages pointing callers to adjust the value range rather than fix the input type.

## Root Cause

The `is_infinite()` check was placed after comparisons that already caught `±Inf` with wrong messages. `is_nan()` check similarly reachable only if NaN somehow passed all prior checks.

## Fix Location

`src/validation.rs:268`

Before: `> 10.0` / `<= 0.0` checks first, then `is_nan()` / `is_infinite()`.

After: `if v.is_nan() || v.is_infinite()` guard placed at the top, before all range checks.

## Prevention

See BUG-003 — same root pattern, different manifestation (Inf caught by value boundary instead of range `contains()`).

## History

- 2026-06-13 REPORT — bug filed; MRE test `test_validate_repetition_penalty_nan_gives_valid_number_error` written
- 2026-06-13 CLOSE — fix applied; all tests pass

## Refs Tests

- `tests/validation_tests.rs` — `test_validate_repetition_penalty_nan_gives_valid_number_error`

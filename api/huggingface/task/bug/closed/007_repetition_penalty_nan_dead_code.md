# BUG-007: repetition_penalty validator dead NaN/Inf check

- **Severity:** Minor
- **State:** Fixed
- **Affects:** `validate_repetition_penalty()` called with `f32::NAN` or `f32::INFINITY`
- **Component:** `validation.rs`
- **Filed:** 2026-06-13
- **Validated By:** cargo nextest
- **Validation Date:** 2026-06-13
- **Related Bugs:** [BUG-003](./003_temperature_nan_dead_code.md)

## Symptom

```bash
# validate_repetition_penalty(f32::NAN) → wrong "must be between" error instead of "valid number"
```

## Impact

Same as BUG-003 — wrong error message for NaN/Inf. Validation still rejects the input.

## How Discovered

Code review of validation.rs; identical dead-check pattern as BUG-003.

## Minimum Reproducible Example

```bash
assert_eq!(validate_repetition_penalty(f32::NAN), Err("repetition_penalty must be a valid number"));
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|-------|---------|----------|
| H1 | Same root cause as BUG-003 | ✅ Root Cause | Code review at validation.rs pre-fix line 410 | E1 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/validation.rs:410` pre-fix | `is_nan()` check after `contains()` — dead code for NaN inputs | H1 ✅ |

## Root Cause

Same as BUG-003. See [BUG-003](./003_temperature_nan_dead_code.md).

## Why Not Caught

Same gap as BUG-003.

## Fix Location

`src/validation.rs:410` — NaN/Inf guard moved before `(0.0..=2.0).contains()`.

## Prevention

See BUG-003. **Pitfall:** See BUG-003.

## Generalized Version

See BUG-003.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-06-13 | filed | Same pattern as BUG-003; filed in same audit session |
| 2026-06-13 | closed | Fix verified by `test_validate_repetition_penalty_infinity_gives_valid_number_error` passing |

## Refs: src/

- `src/validation.rs` — `validate_repetition_penalty()`: NaN/Inf guard moved before range check

## Refs: tests/

- `tests/validation_tests.rs` — `test_validate_repetition_penalty_infinity_gives_valid_number_error`: MRE reproducer

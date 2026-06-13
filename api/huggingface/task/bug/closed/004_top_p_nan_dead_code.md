# BUG-004: top_p validator dead NaN/Inf check

- **Severity:** Minor
- **State:** Fixed
- **Affects:** `validate_top_p()` called with `f32::NAN` or `f32::INFINITY`
- **Component:** `validation.rs`
- **Filed:** 2026-06-13
- **Validated By:** cargo nextest
- **Validation Date:** 2026-06-13
- **Related Bugs:** [BUG-003](./003_temperature_nan_dead_code.md)

## Symptom

```bash
# validate_top_p(f32::NAN) → Err("top_p must be between 0.0 and 1.0")
# Expected:                 → Err("top_p must be a valid number")
```

## Impact

Same as BUG-003 — wrong error message for NaN/Inf inputs. Validation still rejects the input.

## How Discovered

Code review of validation.rs; identical dead-check pattern as BUG-003.

## Minimum Reproducible Example

```bash
# Same pattern as BUG-003 but for validate_top_p
assert_eq!(validate_top_p(f32::NAN), Err("top_p must be a valid number"));
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|-------|---------|----------|
| H1 | Same root cause as BUG-003 — `contains()` catches NaN before explicit guard | ✅ Root Cause | Code review at validation.rs pre-fix line 238 | E1 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/validation.rs:238` pre-fix | `is_nan()` check after `(0.0..=1.0).contains()` — same dead-code pattern | H1 ✅ |

## Root Cause

Same mechanism as BUG-003. See [BUG-003](./003_temperature_nan_dead_code.md) for full root cause analysis.

## Why Not Caught

Same gap as BUG-003: no test asserted the specific error *message* for NaN/Inf inputs.

## Fix Location

`src/validation.rs:238` — NaN/Inf guard moved before `(0.0..=1.0).contains()`.

## Prevention

See BUG-003 Prevention. **Pitfall:** See BUG-003 Pitfall.

## Generalized Version

See BUG-003 Generalized Version.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-06-13 | filed | Same pattern as BUG-003; filed in same audit session |
| 2026-06-13 | closed | Fix verified by `test_validate_top_p_nan_gives_valid_number_error` passing |

## Refs: src/

- `src/validation.rs` — `validate_top_p()`: NaN/Inf guard moved before range check

## Refs: tests/

- `tests/validation_tests.rs` — `test_validate_top_p_nan_gives_valid_number_error`: MRE reproducer

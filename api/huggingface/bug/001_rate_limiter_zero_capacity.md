# BUG-001: Rate limiter panics on zero capacity

- **Severity:** High
- **State:** Fixed
- **Affects:** `RateLimiter::time_until_token()` when constructed with `capacity = 0`
- **Component:** `src/reliability/rate_limiter.rs`
- **Filed:** 2026-06-13
- **Updated:** 2026-06-13

## Symptom

```
thread 'test_rate_limiter_zero_capacity_try_acquire_no_panic' panicked at 'not yet implemented',
or: attempt to create Duration from a +Inf float value
```

`Duration::from_secs_f64(+Inf)` panics at runtime because `refill_rate = 0.0` causes division by zero.

## Impact

Any call to `time_until_token()` on a `RateLimiter` with zero capacity panics unconditionally, crashing the calling thread. Silent in production if the caller does not explicitly test capacity = 0.

## How Discovered

Found during reliability test suite review (code_hyg_l1 audit 2026-06-13).

## Root Cause

`refill_rate` is derived from `capacity as f64`; when capacity is 0, `refill_rate = 0.0`. The waiting time calculation divides by `refill_rate`, yielding `+Inf`. `Duration::from_secs_f64(+Inf)` panics because IEEE 754 `+Inf` is not representable as a `Duration`.

## Fix Location

`src/reliability/rate_limiter.rs:140`

Before: no guard — proceeded to `Duration::from_secs_f64(seconds)` directly.

After: `if !seconds.is_finite() { return Some(Duration::MAX); }`

## Prevention

- **Pitfall:** `Duration::from_secs_f64()` panics on non-finite floats. Always guard `seconds.is_finite()` before calling it.
- When a rate is derived from user-controlled capacity, guard the zero case before any arithmetic that divides by it.

## History

- 2026-06-13 REPORT — bug filed; MRE test `test_rate_limiter_zero_capacity_try_acquire_no_panic` written
- 2026-06-13 CLOSE — fix applied; all tests pass

## Refs Tests

- `tests/rate_limiting_tests.rs` — `test_rate_limiter_zero_capacity_try_acquire_no_panic`

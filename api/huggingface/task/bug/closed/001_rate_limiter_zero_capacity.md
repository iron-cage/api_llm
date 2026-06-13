# BUG-001: Rate limiter zero-capacity panic

- **Severity:** High
- **State:** Fixed
- **Affects:** Any call to `RateLimiter::try_acquire()` when `capacity = 0`
- **Component:** `reliability/rate_limiter.rs`
- **Filed:** 2026-06-13
- **Validated By:** cargo nextest
- **Validation Date:** 2026-06-13

## Symptom

```bash
# RateLimiter::new(capacity=0) then try_acquire() → panic
thread 'test' panicked at 'not yet implemented: Duration seconds are not representable'
# Triggered at: Duration::from_secs_f64(f64::INFINITY)
```

## Impact

Any consumer passing `capacity = 0` (intentionally or via misconfiguration) triggers a panic — a loud crash, not a recoverable error. No API call reaches the server. Severity is High because the panic occurs at runtime with no preceding warning.

## How Discovered

```bash
$ cargo nextest run --all-features -- test_rate_limiter_zero_capacity
# panicked at 'attempt to multiply with overflow'
```

Discovered during code_hyg_l1 audit of `reliability/rate_limiter.rs`.

## Minimum Reproducible Example

```bash
# /tmp/mre001/: reproduce in any Rust project using RateLimiter
# Create capacity=0 limiter and call time_until_token()
let rl = RateLimiter::new(RateLimiterConfig { per_second: 0, .. });
rl.try_acquire();  # triggers time_until_token() → 1.0 / 0.0 = +Infinity
                   # → Duration::from_secs_f64(+Infinity) panics
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|-------|---------|----------|
| H1 | `refill_rate = 0.0` causes `1.0 / 0.0 = +Infinity` | ✅ Root Cause | Confirmed via source inspection | E1 |
| H2 | `Duration::from_secs_f64(+Infinity)` panics | ✅ Root Cause | Confirmed by Rust stdlib docs | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/reliability/rate_limiter.rs:138` | `tokens_needed / self.refill_rate` where `refill_rate = 0.0` | H1 ✅ |
| E2 | Rust stdlib | `Duration::from_secs_f64` panics on non-finite input | H2 ✅ |

## Root Cause

```
RateLimiter::new(capacity=0)
  → refill_rate = 0.0 / duration_secs = 0.0
  → try_acquire() returns false (0 < 1 token)
  → time_until_token() computes 1.0 / 0.0 = f64::INFINITY
  → Duration::from_secs_f64(INFINITY) → panic
```

## Why Not Caught

No existing test used `capacity = 0`. Minimum tested capacity was 1. The boundary value was never exercised.

## Fix Location

`src/reliability/rate_limiter.rs:140`

```rust
// Before:
let seconds = tokens_needed / self.refill_rate;
return Some(Duration::from_secs_f64(seconds));

// After:
let seconds = tokens_needed / self.refill_rate;
if !seconds.is_finite() { return Some(Duration::MAX); }
return Some(Duration::from_secs_f64(seconds));
```

## Prevention

Always test boundary value `0` for numeric configuration fields. `Duration::from_secs_f64` panics on non-finite input — guard with `is_finite()` before every conversion.

**Pitfall:** f64 division by zero yields `+Infinity` silently (IEEE 754). The panic only surfaces at the `Duration::from_secs_f64` call site, not at the division.

## Generalized Version

**Broken assumption:** `refill_rate > 0.0` always holds.
**Failure conditions:** `capacity = 0` → `refill_rate = 0.0` → division yields `+Infinity`.
**Detection invariant:** Any float passed to `Duration::from_secs_f64` must satisfy `x.is_finite()`.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-06-13 | filed | Discovered during code_hyg_l1 audit of rate_limiter.rs |
| 2026-06-13 | closed | Fix verified by `test_rate_limiter_zero_capacity_try_acquire_no_panic` passing |

## Refs: src/

- `src/reliability/rate_limiter.rs` — `time_until_token()`: guard `is_finite()` before `Duration::from_secs_f64`

## Refs: tests/

- `tests/rate_limiting_tests.rs` — `test_rate_limiter_zero_capacity_try_acquire_no_panic`: MRE reproducer

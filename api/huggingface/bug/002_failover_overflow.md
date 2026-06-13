# BUG-002: Failover backoff overflows u64 for high retry counts

- **Severity:** High
- **State:** Fixed
- **Affects:** `failover` exponential backoff delay calculation when `max_retries >= 57`
- **Component:** `src/reliability/failover.rs`
- **Filed:** 2026-06-13
- **Updated:** 2026-06-13

## Symptom

```
thread 'test_failover_backoff_delay_no_overflow_for_high_attempt_counts' panicked at
'attempt to multiply with overflow'
```

`500 * 2u64.pow(attempts-1)` overflows u64 when `attempts-1 >= 63`.

## Impact

Any failover configuration with `max_retries >= 57` panics during delay calculation after the first few retries. The crash is unrecoverable and prevents the failover mechanism from functioning.

## How Discovered

Found during reliability test suite review (code_hyg_l1 audit 2026-06-13).

## Root Cause

`2u64.pow(n)` overflows u64 for `n >= 64`. The exponent was not capped, so a sufficiently high `max_retries` value makes the shift overflow.

## Fix Location

`src/reliability/failover.rs:360`

Before: `let delay_ms = 500 * 2u64.pow(attempts - 1);`

After: `let exp = (attempts - 1).min(13); let delay_ms = 500 * 2u64.pow(exp);`

`2^13 * 500 = 4,096,000 ms ≈ 68 minutes` — a sensible cap for any retry scenario.

## Prevention

- **Pitfall:** `u64::pow(n)` panics in debug mode and silently wraps in release mode for large `n`. Always cap the exponent before use.
- Exponential backoff must always cap both the exponent and the resulting delay independently.

## History

- 2026-06-13 REPORT — bug filed; MRE test `test_failover_backoff_delay_no_overflow_for_high_attempt_counts` written
- 2026-06-13 CLOSE — fix applied; all tests pass

## Refs Tests

- `tests/failover_tests.rs` — `test_failover_backoff_delay_no_overflow_for_high_attempt_counts`

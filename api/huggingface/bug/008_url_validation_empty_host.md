# BUG-008: URL validation accepts bare scheme with no hostname

- **Severity:** Minor
- **State:** Fixed
- **Affects:** `validate_base_url()` with inputs like `"http://"` or `"https://"`
- **Component:** `src/validation.rs`
- **Filed:** 2026-06-13
- **Updated:** 2026-06-13

## Symptom

```rust
validate_base_url("http://")  // returned Ok(()) — should return Err
```

## Impact

A bare scheme URL (no hostname) was accepted as valid and could be passed to the HTTP client, causing a confusing connection error downstream rather than a clear validation error at input time.

## Root Cause

The validation only checked `starts_with("http://")` and length `> 8`. `"http://"` is exactly 7 characters, so that check rejected it. But `"http://x"` (8 chars) passes, and the logic didn't verify that anything meaningful followed the scheme prefix.

## Fix Location

`src/validation.rs:602`

After: extract `after_scheme` slice; require `!after_scheme.is_empty()` to guarantee at least one hostname character.

## Prevention

- **Pitfall:** `starts_with("http://")` is necessary but not sufficient for a valid URL — the string could end immediately after the scheme separator. Always verify the remainder is non-empty.

## History

- 2026-06-13 REPORT — bug filed
- 2026-06-13 CLOSE — fix applied; tests pass

## Refs Tests

- `tests/validation_tests.rs` — URL validation tests covering bare scheme edge cases

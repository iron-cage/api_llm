# BUG-008: URL validator accepts protocol-only strings

- **Severity:** Medium
- **State:** Fixed
- **Affects:** `validate_url()` called with `"http://"` or `"https://"`
- **Component:** `validation.rs`
- **Filed:** 2026-06-13
- **Validated By:** cargo nextest
- **Validation Date:** 2026-06-13

## Symptom

```bash
# validate_url("http://") → Ok(())  (incorrectly accepted)
# Expected:               → Err("...must include a hostname")
```

## Impact

A protocol-only string with no hostname passes URL validation and could propagate to network calls, which will then fail at the HTTP layer with a confusing error — the invalid URL is not caught at the validation boundary where the error would be most informative.

## How Discovered

Code review of `validate_url` during code_hyg_l1 audit revealed it checked only: (1) non-empty, (2) starts with `http://` or `https://`, (3) length ≤ 2048. `"http://"` satisfies all three — 7 chars, starts with `"http://"`, not empty.

## Minimum Reproducible Example

```bash
# Before fix: passes validation
assert!(validate_url("http://").is_ok());

# After fix: rejected
assert!(validate_url("http://").is_err());
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|-------|---------|----------|
| H1 | Only prefix check performed; no hostname structural check | ✅ Root Cause | Source inspection: no check after scheme | E1 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/validation.rs:602` pre-fix | After `starts_with("http://")`, no check that remainder is non-empty | H1 ✅ |

## Root Cause

```
validate_url("http://")
  → non-empty: true (7 chars)
  → starts_with("http://"): true
  → len <= 2048: true
  → Ok(())  ← no structural component check
```

## Why Not Caught

Tests covered wrong-protocol strings and too-long strings, but no test used a structurally malformed URL that passed the prefix check.

## Fix Location

`src/validation.rs:602`

```rust
// After confirming http(s):// prefix, verify remainder is non-empty:
let scheme_len = if url.starts_with("https://") { 8 } else { 7 };
if url.len() <= scheme_len {
    return Err("...must include a hostname after the scheme");
}
```

## Prevention

URL validators must check structural components (scheme, host) individually — not just whether the string starts with a known prefix.

**Pitfall:** `starts_with("http://")` is necessary but not sufficient for a valid URL. A 7-character string matches `"http://"` exactly.

## Generalized Version

**Broken assumption:** A URL starting with `http://` has a hostname.
**Failure conditions:** Any protocol-only string (`"http://"`, `"https://"`) passes prefix-only validation.
**Detection invariant:** After extracting the scheme, assert `remainder.len() > 0`.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-06-13 | filed | Discovered during code_hyg_l1 audit of validation.rs |
| 2026-06-13 | closed | Fix verified by `test_validate_url_protocol_only_rejected` passing |

## Refs: src/

- `src/validation.rs` — `validate_url()`: structural hostname check added after scheme prefix check

## Refs: tests/

- `tests/validation_tests.rs` — `test_validate_url_protocol_only_rejected`: MRE reproducer

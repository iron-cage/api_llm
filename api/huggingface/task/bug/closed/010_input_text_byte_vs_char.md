# BUG-010: validate_input_text rejects valid multibyte strings (byte len vs char count)

- **Severity:** Medium
- **State:** Fixed
- **Affects:** Any caller passing non-ASCII text (multibyte Unicode) to `validate_input_text` or `validate_message_content`
- **Component:** `validation.rs`
- **Filed:** 2026-06-14
- **Validated By:** cargo nextest
- **Validation Date:** 2026-06-14

## Symptom

```bash
# validate_input_text("é".repeat(25001)) → Err("Input text is too long (50002 characters)")
# Expected: Ok(()) — 25001 chars is well under MAX_INPUT_LENGTH=50000
```

A string of 25 001 two-byte characters (total 50 002 UTF-8 bytes) failed validation even
though it contains only 25 001 Unicode code points — well under the 50 000-character limit.

## Impact

Any user sending non-ASCII text (e.g., French, Chinese, Arabic, emoji-heavy content) near
the character limit would be incorrectly rejected. The failure is silent: the error message
reports a byte count and labels it "characters", making the limit appear to be for bytes.
Medium severity because the affected strings must be both non-ASCII and large.

## How Discovered

Manual testing corner-case analysis: the `MAX_INPUT_LENGTH` constant is documented as
"characters" and the error message says "characters", but the check uses `input.len()`
which returns UTF-8 byte count in Rust.

## Minimum Reproducible Example

```rust
use api_huggingface::validation::{ validate_input_text, MAX_INPUT_LENGTH };

// 2-byte char repeated at half the limit → byte_len = MAX_INPUT_LENGTH, char_count = half
let text = "é".repeat( MAX_INPUT_LENGTH ); // MAX_INPUT_LENGTH code points, 2× bytes
assert!( validate_input_text( &text ).is_ok() ); // FAILED pre-fix
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|-------|---------|----------|
| H1 | `input.len()` counts bytes, not characters | ✅ Root Cause | Rust `str::len()` is always UTF-8 bytes | E1 |
| H2 | Constant and error message both say "characters" → semantic contract is code points | ✅ Supporting | Documentation and error text consistent | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/validation.rs:42` pre-fix | `if input.len() > MAX_INPUT_LENGTH` — uses byte count | H1 ✅ |
| E2 | `src/validation.rs:8` | `/// Maximum allowed input text length (characters)` | H2 ✅ |

## Root Cause

```
"é".len()         = 2   (UTF-8 bytes)
"é".chars().count() = 1   (Unicode code point)

MAX_INPUT_LENGTH = 50_000
"é".repeat(25_001).len()          = 50_002  → fails input.len() > 50_000
"é".repeat(25_001).chars().count() = 25_001  → passes if counted correctly
```

Rust `str::len()` returns byte count, not code-point count. For any non-ASCII character
(U+0080 and above), `len()` > `chars().count()`. The constant `MAX_INPUT_LENGTH` is
documented and displayed as a "character" limit, so the correct implementation must use
`chars().count()`.

## Why Not Caught

Every existing boundary test used ASCII-only strings (`"a".repeat(N)`) where
`len() == chars().count()`. No test exercised multibyte Unicode near the boundary.

## Fix Location

`src/validation.rs` — `validate_input_text` and `validate_message_content`

```rust
// Before (both functions):
if input.len() > MAX_INPUT_LENGTH { ... input.len() ... }

// After:
let char_count = input.chars().count();
if char_count > MAX_INPUT_LENGTH { ... char_count ... }
```

## Prevention

When a limit is documented as "characters", always use `.chars().count()` to count.
Reserve `.len()` for byte-length limits and name/document them clearly as "bytes".

**Pitfall:** In Rust, `str::len()` returns UTF-8 byte count, never Unicode code-point count.
These are equal only for ASCII text (U+0000–U+007F).

## Generalized Version

**Broken assumption:** `str::len()` measures "text size" in the human sense.
**Failure conditions:** Any non-ASCII character (2–4 bytes in UTF-8).
**Detection invariant:** Character-limit validators must use `.chars().count()`.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-06-14 | filed | Discovered during exhaustive corner-case analysis |
| 2026-06-14 | closed | Fix verified by `test_validate_input_text_multibyte_character_boundary` passing |

## Refs: src/

- `src/validation.rs` — `validate_input_text`: `input.len()` → `input.chars().count()` + matching format arg
- `src/validation.rs` — `validate_message_content`: same fix

## Refs: tests/

- `tests/validation_tests.rs` — `test_validate_input_text_multibyte_character_boundary`: MRE reproducer for `validate_input_text` (bug_reproducer BUG-010)
- `tests/validation_tests.rs` — `test_validate_message_content_multibyte_character_boundary`: MRE reproducer for `validate_message_content` (bug_reproducer BUG-010)

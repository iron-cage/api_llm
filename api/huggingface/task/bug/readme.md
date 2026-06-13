# Bug Reports — api_huggingface

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `readme.md` | Bug index — open and closed bug tracking |
| `closed/` | Resolved bug reports archive |

## Open Bugs

| ID | Title | State | Severity | Component | Filed | Root Cause | Reopen Count |
|----|-------|-------|----------|-----------|-------|------------|--------------|

## Closed Bugs

| ID | Title | Severity | Component | Filed | Closed | Root Cause | Reopen Count | Validated By |
|----|-------|----------|-----------|-------|--------|------------|--------------|--------------|
| BUG-001 | [Rate limiter zero-capacity panic](./closed/001_rate_limiter_zero_capacity.md) | High | `reliability/rate_limiter.rs` | 2026-06-13 | 2026-06-13 | `1/0.0 = +Inf` passed to `Duration::from_secs_f64` | 0 | cargo nextest |
| BUG-002 | [Failover backoff u64 overflow](./closed/002_failover_backoff_overflow.md) | High | `reliability/failover.rs` | 2026-06-13 | 2026-06-13 | `500 * 2u64.pow(n-1)` overflows when `n >= 57` | 0 | cargo nextest |
| BUG-003 | [Temperature validator dead NaN/Inf check](./closed/003_temperature_nan_dead_code.md) | Minor | `validation.rs` | 2026-06-13 | 2026-06-13 | `contains()` catches NaN before explicit guard | 0 | cargo nextest |
| BUG-004 | [top_p validator dead NaN/Inf check](./closed/004_top_p_nan_dead_code.md) | Minor | `validation.rs` | 2026-06-13 | 2026-06-13 | Same dead-check pattern as BUG-003 | 0 | cargo nextest |
| BUG-005 | [frequency_penalty validator dead NaN/Inf check](./closed/005_frequency_penalty_nan_dead_code.md) | Minor | `validation.rs` | 2026-06-13 | 2026-06-13 | Same dead-check pattern as BUG-003 | 0 | cargo nextest |
| BUG-006 | [presence_penalty validator dead NaN/Inf check](./closed/006_presence_penalty_nan_dead_code.md) | Minor | `validation.rs` | 2026-06-13 | 2026-06-13 | Same dead-check pattern as BUG-003 | 0 | cargo nextest |
| BUG-007 | [repetition_penalty validator dead NaN/Inf check](./closed/007_repetition_penalty_nan_dead_code.md) | Minor | `validation.rs` | 2026-06-13 | 2026-06-13 | Same dead-check pattern as BUG-003 | 0 | cargo nextest |
| BUG-008 | [URL validator accepts protocol-only strings](./closed/008_url_protocol_only.md) | Medium | `validation.rs` | 2026-06-13 | 2026-06-13 | Prefix check passes `"http://"` with no hostname | 0 | cargo nextest |
| BUG-009 | [cosine_similarity returns values outside [-1, 1]](./closed/009_cosine_similarity_out_of_bounds.md) | Medium | `embeddings.rs` | 2026-06-13 | 2026-06-13 | IEEE 754 rounding yields values like 1.0000001 | 0 | cargo nextest |
| BUG-010 | [validate_input_text rejects valid multibyte strings](./closed/010_input_text_byte_vs_char.md) | Medium | `validation.rs` | 2026-06-14 | 2026-06-14 | `input.len()` (bytes) used instead of `chars().count()` for character limit | 0 | cargo nextest |

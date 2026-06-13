# Bug Reports — api_huggingface

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `readme.md` | Bug registry index — open and closed bug tracking |

## Open Bugs

No open bugs.

## Closed Bugs

| ID | Title | Severity | State | Filed | Component |
|----|-------|----------|-------|-------|-----------|
| [BUG-001](001_rate_limiter_zero_capacity.md) | Rate limiter panics on zero capacity | High | Fixed | 2026-06-13 | `src/reliability/rate_limiter.rs` |
| [BUG-002](002_failover_overflow.md) | Failover backoff overflows u64 for high retry counts | High | Fixed | 2026-06-13 | `src/reliability/failover.rs` |
| [BUG-003](003_validate_temperature_nan.md) | Temperature NaN/Inf passes with wrong error message | Medium | Fixed | 2026-06-13 | `src/validation.rs` |
| [BUG-004](004_validate_top_p_nan.md) | top_p NaN/Inf passes with wrong error message | Medium | Fixed | 2026-06-13 | `src/validation.rs` |
| [BUG-005](005_validate_frequency_penalty_nan.md) | frequency_penalty NaN/Inf passes with wrong error message | Medium | Fixed | 2026-06-13 | `src/validation.rs` |
| [BUG-006](006_validate_presence_penalty_nan.md) | presence_penalty NaN/Inf passes with wrong error message | Medium | Fixed | 2026-06-13 | `src/validation.rs` |
| [BUG-007](007_validate_repetition_penalty_nan.md) | repetition_penalty NaN/Inf dead-code guard (wrong order) | Medium | Fixed | 2026-06-13 | `src/validation.rs` |
| [BUG-008](008_url_validation_empty_host.md) | URL validation accepts bare scheme with no hostname | Minor | Fixed | 2026-06-13 | `src/validation.rs` |
| [BUG-009](009_cosine_similarity_no_clamping.md) | cosine_similarity returns values outside [-1.0, 1.0] | Minor | Fixed | 2026-06-13 | `src/embeddings.rs` |

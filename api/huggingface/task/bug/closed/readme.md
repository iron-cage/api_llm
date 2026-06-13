# Bug Reports — Closed Archive

Resolved bug reports for `api_huggingface`. All bugs in this directory have `State: Fixed`.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `readme.md` | Closed bugs archive index |
| `001_rate_limiter_zero_capacity.md` | BUG-001: panic on capacity=0 in rate limiter |
| `002_failover_backoff_overflow.md` | BUG-002: u64 overflow in failover exponential backoff |
| `003_temperature_nan_dead_code.md` | BUG-003: dead NaN/Inf guard in temperature validator |
| `004_top_p_nan_dead_code.md` | BUG-004: dead NaN/Inf guard in top_p validator |
| `005_frequency_penalty_nan_dead_code.md` | BUG-005: dead NaN/Inf guard in frequency_penalty validator |
| `006_presence_penalty_nan_dead_code.md` | BUG-006: dead NaN/Inf guard in presence_penalty validator |
| `007_repetition_penalty_nan_dead_code.md` | BUG-007: dead NaN/Inf guard in repetition_penalty validator |
| `008_url_protocol_only.md` | BUG-008: URL validator accepts protocol-only strings |
| `009_cosine_similarity_out_of_bounds.md` | BUG-009: cosine_similarity returns values outside [-1.0, 1.0] |

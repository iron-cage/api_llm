# Implement Client Enterprise Fields in core.rs

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** ❓ (Unverified)
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** src/client/
- **Validated By:** N/A
- **Validation Date:** N/A

## Goal

Nine `// xxx : @team` markers in `src/client/core.rs` identify Client struct fields that have `#[allow(dead_code)]` annotations because their integration into the HTTP dispatch layer is incomplete. Implement or route each marked field through the enterprise dispatch path so the fields are actively used, removing all `#[allow(dead_code)]` annotations. Observable: `grep "// xxx" src/client/core.rs | wc -l` → 0.

## In Scope

- `src/client/core.rs` — all 9 `// xxx : @team` markers
- `src/internal/http/enterprise.rs` — integrate fields into dispatch where applicable
- Remove `#[allow(dead_code)]` attributes from affected Client struct fields

## Out of Scope

- WebSocket markers (see task 008)
- Batch API qqq: markers (see task 004)
- New public API surface changes
- Tests — no new test files added in this task

## Markers to Resolve

| Location | Marker | Action |
|----------|--------|--------|
| `src/client/core.rs:58` | Implement per-request timeout override mechanism | Route timeout override through `HttpConfig` |
| `src/client/core.rs:65` | Implement retry metrics collection and aggregation | Wire retry counter into `execute_with_optional_retries` |
| `src/client/core.rs:72` | Integrate circuit breaker from internal/http.rs into Client API | Use shared `Arc<CircuitBreaker>` — see task 003 |
| `src/client/core.rs:87` | Expose circuit breaker metrics through `get_circuit_breaker_metrics()` | Add method; delegate to shared breaker |
| `src/client/core.rs:91` | Implement Arc-based shared circuit breaker state | See task 003 — coordinate |
| `src/client/core.rs:95` | Implement general HTTP response caching layer | Wire `request_cache` field into execute path |
| `src/client/core.rs:107` | Track cache hit/miss rates, eviction statistics | Expose from cache implementation |
| `src/client/core.rs:114` | Integrate rate limiter from internal/http.rs into Client API | Route through enterprise dispatch |
| `src/client/core.rs:129` | Expose rate limiting metrics through `get_rate_limiter_metrics()` | Add method; delegate to rate limiter |

## History

- **2026-06-13** `CREATED` — L1 hygiene audit identified 9 untracked `// xxx : @team` markers in `src/client/core.rs` with no task IDs. Created to track resolution.

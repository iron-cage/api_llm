# Fix Circuit Breaker Per-Call State Reset

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** 🎯 (Verified)
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** src/internal/http/
- **Validated By:** null
- **Validation Date:** null

## Goal

Fix the structural bug in `execute_with_optional_retries` (`src/internal/http/enterprise.rs:67`) where a fresh `CircuitBreaker` instance is constructed on every HTTP dispatch call via `full_client.to_circuit_breaker_config().map(|config| CircuitBreaker::new(config))`. Because `CircuitBreaker::new` initializes a clean Closed state with zero counters, failure history never accumulates across calls — the circuit can never open regardless of how many failures occur, making the `circuit_breaker` feature silently inoperative for any `Client` using `execute_with_optional_retries`. Fix: replace per-call instantiation with a shared `Arc<CircuitBreaker>` owned by `Client`, initialized once at construction time, reused across all dispatch calls. Observable outcome: a `Client` configured with `failure_threshold = N` transitions to Open state after N consecutive failures when using `execute_with_optional_retries`; unit test proves the state persists between calls.

## In Scope

- `src/client/core.rs` — add `circuit_breaker: Option<Arc<CircuitBreaker>>` field under `#[cfg(feature = "circuit_breaker")]`; remove `#[allow(dead_code)]` from the circuit breaker config fields once they feed initialization
- `src/client/builder/` — initialize the new `circuit_breaker` field using existing `circuit_breaker_*` config fields during `build()`
- `src/internal/http/enterprise.rs:67` — replace per-call `CircuitBreaker::new(config)` with `full_client.circuit_breaker.as_ref()` (read the shared instance, do not construct)
- `src/client/config.rs` (`ClientConfig::former()` path) — initialize `circuit_breaker` field in the former-based builder path, matching the `ClientBuilder` path
- MRE test — demonstrate state persistence: call `execute_with_optional_retries` twice on an invalid URL with `failure_threshold = 1`; second call must return `Error::CircuitBreakerOpen`

## Out of Scope

- Rate limiter per-call reset — distinct feature, separate task if confirmed
- Circuit breaker for `execute_legacy` callers — those paths bypass enterprise features entirely; tracked in task 006
- Caching, retry — other enterprise features, separate scope
- Exposing circuit breaker state via public `Client` methods — separate task

## Requirements

- All work must adhere to applicable rulebooks (`kbase .rulebooks`)
- The shared `CircuitBreaker` must be `Arc`-wrapped so it can be referenced across async tasks without cloning state
- `CircuitBreaker` already uses `Arc<Mutex<...>>` internally (confirmed in `src/internal/http/circuit_breaker.rs`) — wrapping in `Arc` is sufficient; cloning the `Arc` in `execute_with_optional_retries` gives a reference to the same instance
- Remove `#[allow(dead_code)]` from `enable_circuit_breaker`, `circuit_breaker_failure_threshold`, `circuit_breaker_success_threshold`, `circuit_breaker_timeout` in `core.rs` once they are consumed at init time — the suppression must not remain after the fix
- `w3 .test l::3` must pass with zero failures and zero new warnings

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Create bug report** — file bug per `bug.rulebook.md § Lifecycle : Procedure - Report New Bug`; yields BUG-NNN.
2. **Write MRE test** — in `tests/inc/`, write `test_circuit_breaker_state_persists_across_calls` marked `bug_reproducer(BUG-NNN)` and `#[cfg(feature = "circuit_breaker")]`. Build a `Client` with `enable_circuit_breaker = true`, `failure_threshold = 1`, pointed at an invalid base URL (e.g., `http://127.0.0.1:1`). Call `execute_with_optional_retries` once — expect a network error. Call it again — assert `Err(Error::CircuitBreakerOpen(...))`. Run test; confirm it currently fails (second call returns network error instead of open circuit, because CB state was reset).
3. **Add `circuit_breaker` field to `Client`** — in `src/client/core.rs`, add:
   ```
   #[ cfg( feature = "circuit_breaker" ) ]
   pub( crate ) circuit_breaker : Option< std::sync::Arc< crate::internal::http::CircuitBreaker > >,
   ```
   Remove `#[allow(dead_code)]` from all `circuit_breaker_*` config fields now that they will be consumed.
4. **Initialize in builder** — in `src/client/builder/` (and `ClientConfig::former()` path in `config.rs`): when `enable_circuit_breaker` is true, create `CircuitBreaker::new(config)` once and wrap in `Arc`; assign to the new field. When false, assign `None`.
5. **Update enterprise dispatch** — in `src/internal/http/enterprise.rs:67`, replace:
   ```
   let circuit_breaker = full_client.to_circuit_breaker_config().map( |config| CircuitBreaker::new( config ) );
   ```
   with:
   ```
   #[ cfg( feature = "circuit_breaker" ) ]
   let circuit_breaker = full_client.circuit_breaker.as_ref().map( |arc| arc.as_ref() );
   ```
   Adjust the `execute_with_enterprise_features` call site to pass the reference type correctly (confirm it already accepts `Option<&CircuitBreaker>`).
6. **Run MRE test** — `cargo nextest run test_circuit_breaker_state_persists_across_calls --features circuit_breaker`; confirm it now passes.
7. **Document fix** — add `// Fix(BUG-NNN): ...` comment above the field in `core.rs`; add 5-section fix doc in test file per `bug.rulebook.md`.
8. **Full verification** — `w3 .test l::3`.
9. **Update task state** — update `task/readme.md`; move file to `task/completed/`.

## Acceptance Criteria

- `execute_with_optional_retries` no longer calls `CircuitBreaker::new` — `grep -n "CircuitBreaker::new" src/internal/http/enterprise.rs` → 0 results
- `Client` struct has `circuit_breaker: Option<Arc<CircuitBreaker>>` field under `#[cfg(feature = "circuit_breaker")]`
- MRE test passes: second call to a failed endpoint returns `Error::CircuitBreakerOpen`
- Zero `#[allow(dead_code)]` remain on circuit breaker config fields in `core.rs` (they feed the initializer, not dead)
- `w3 .test l::3` passes with zero failures and zero new warnings

## Validation

**Execution:** Independent validator walks this section after SUBMIT transition.

### Checklist

**Structural fix**
- [ ] C1 — Does `grep -n "CircuitBreaker::new" src/internal/http/enterprise.rs` return 0 results?
- [ ] C2 — Does `Client` struct in `src/client/core.rs` contain a `circuit_breaker` field of type `Option<Arc<CircuitBreaker>>`?
- [ ] C3 — Is that field initialized (not just declared) in the builder path?

**Dead code cleanup**
- [ ] C4 — Are zero `#[allow(dead_code)]` attributes present on `enable_circuit_breaker`, `circuit_breaker_failure_threshold`, `circuit_breaker_success_threshold`, `circuit_breaker_timeout` in `core.rs`?

**Behavioral correctness**
- [ ] C5 — Does the MRE test `test_circuit_breaker_state_persists_across_calls` pass?

**Fix documentation**
- [ ] C6 — Is there a `// Fix(BUG-NNN): ...` comment in `core.rs` or `enterprise.rs` at the fix site?
- [ ] C7 — Is there a 5-section fix doc in the test file?

### Measurements

- [ ] M1 — per-call CB construction gone: `grep -c "CircuitBreaker::new" src/internal/http/enterprise.rs` → 0
- [ ] M2 — shared field present: `grep -c "circuit_breaker.*Arc" src/client/core.rs` → ≥ 1
- [ ] M3 — `w3 .test l::3` → 0 failures

### Invariants

- [ ] I1 — `cargo check --all-features` → 0 errors, 0 warnings
- [ ] I2 — `cargo check --no-default-features` → 0 errors, 0 warnings (feature-gated code compiles correctly without feature)

### Anti-faking checks

- [ ] AF1 — The MRE test is not marked `#[ignore]` or skipped
- [ ] AF2 — The `circuit_breaker` field is not `#[allow(dead_code)]` — it must be actively read in `execute_with_optional_retries`
- [ ] AF3 — `grep "CircuitBreaker::new" src/internal/http/enterprise.rs` returns 0 — the per-call constructor is fully removed, not just commented out

## Related Documentation

| Path | Role |
|------|------|
| `src/internal/http/enterprise.rs` | Primary fix site — line 67 per-call construction removed |
| `src/internal/http/circuit_breaker.rs` | CircuitBreaker type — uses Arc<Mutex<...>> internally |
| `src/client/core.rs` | Client struct — add shared circuit_breaker field |
| `src/client/builder/` | Builder path — initialize circuit_breaker once |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — Confirmed during crate audit: `execute_with_optional_retries` in `src/internal/http/enterprise.rs:67` calls `CircuitBreaker::new(config)` on every HTTP call. Because each call starts with a fresh Closed-state circuit breaker with zero failure counter, the circuit can never transition to Open. The `circuit_breaker` feature is structurally inoperative for any code using this dispatch path.
- **2026-06-13** `VERIFY PASS` — User authorization: confirmed from crate audit with exact file:line. All 4 dimensions pass: scope bounded (enterprise.rs + core.rs + builder), goal observable (CB state persists, grep = 0 for per-call construction), YAGNI satisfied (active enterprise feature broken), procedure executable (Arc field + init + read-from-client).

## Verification Record

- **Date:** 2026-06-13
- **Method:** User authorization — confirmed bug from crate audit, exact line documented
- **Dim 1 (Scope Coherence):** PASS — In Scope: enterprise.rs per-call constructor + Client field + builder init; Out of Scope: execute_legacy paths, other enterprise features. Observable outcome: MRE test passes, grep = 0.
- **Dim 2 (MOST Goal Quality):** PASS — Motivated: circuit_breaker feature inoperative for all execute_with_optional_retries callers; Observable: MRE test passes + grep verifies removal; Scoped: single dispatch function + Client struct field; Testable: `grep -c "CircuitBreaker::new" src/internal/http/enterprise.rs` = 0 + MRE pass.
- **Dim 3 (Value/YAGNI — adversarial):** PASS — Null hypothesis: any code enabling circuit_breaker feature and calling execute_with_optional_retries gets zero protection; failure_threshold is never reached; active enterprise feature produces false safety signal.
- **Dim 4 (Implementation Readiness):** PASS — Work Procedure steps are executable; Arc<CircuitBreaker> pattern is unambiguous; dead_code cleanup identified; no blocking ambiguities.

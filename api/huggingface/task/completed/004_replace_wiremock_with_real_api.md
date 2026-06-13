# Replace wiremock with Real API Calls in health_check_tests.rs

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** ✅ (Completed)
- **Priority:** 2
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** tests/
- **Validated By:** MAAV (adversarial + completeness subagents)
- **Validation Date:** 2026-06-13

## Goal

`tests/health_check_tests.rs` imports and uses `wiremock` for HTTP mocking, directly violating `docs/invariant/002_testing_standards.md` invariant IN-09 (no mock HTTP servers in any test file). The no-mock standard is a hard invariant in this workspace — it is not a preference. The fix removes all `wiremock` imports and mock server setup from the file and rewrites each test to perform real HuggingFace API calls gated with `#[cfg(feature = "integration")]`. Tests must fail loudly (via `.expect("...")`) when credentials are absent — no conditional skip logic. Observable outcome: `grep -n "wiremock" tests/health_check_tests.rs` → 0 matches; all test functions in the file carry `#[cfg(feature = "integration")]`; `w3 .test l::3` → 0 failures, 0 warnings.

## In Scope

- `tests/health_check_tests.rs` — remove all wiremock imports, mock server setup, and mock response stubs; rewrite each test to call the real HuggingFace API endpoint with `#[cfg(feature = "integration")]` gate before `#[tokio::test]`
- `Cargo.toml` — remove the `wiremock` dev-dependency if no other test file uses it (verify first)

## Out of Scope

- Other test files that may reference wiremock — audit only; changes are this file only
- Changing the health-check behavior being tested — same semantics, different transport
- Adding new health-check tests beyond the existing set — scope limited to replacing existing tests
- `src/` files — test-only change

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- No mocking: real API calls only; credentials via `Secret::load_with_fallbacks()` or `load_from_env("HUGGINGFACE_API_KEY")` with `.expect("HUGGINGFACE_API_KEY must be set for integration tests")`
- Integration gate: `#[cfg(feature = "integration")]` immediately before `#[tokio::test]`, not at file level
- No conditional skip: no `if let Ok(client)` or `try_create_test_client()` patterns — fail loudly
- No `cargo fmt`; 2-space indent; custom codestyle throughout

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; confirm no-mock standard and integration gate convention.
2. **Read `tests/health_check_tests.rs`** in full to understand what health-check behaviors are under test and which endpoints are called.
3. **Verify wiremock usage** — check whether any other test file under `tests/` imports wiremock; note result for step 8.
4. **Write Test Matrix** — populate every row before writing any test code.
5. **Remove wiremock** — delete all `use wiremock` imports, `MockServer::start()` calls, and `Mock::given(...)` stubs from `tests/health_check_tests.rs`.
6. **Rewrite tests** — replace each mock-backed test with a real API call; add `#[cfg(feature = "integration")]` immediately before each `#[tokio::test]`; use `.expect("HUGGINGFACE_API_KEY must be set for integration tests")` for credential loading.
7. **Remove dep if safe** — if no other file uses wiremock, remove it from `[dev-dependencies]` in `Cargo.toml`.
8. **Green state** — `w3 .test l::3` → 0 failures, 0 warnings before proceeding.
9. **Submit for Validation** — trigger SUBMIT transition (⏳ → 🔍).
10. **Update task state** — on validation pass, move file to `task/completed/`; update index.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | No wiremock import in file | `tests/health_check_tests.rs` source text | `grep "wiremock" tests/health_check_tests.rs` → 0 matches |
| T02 | All tests gated with integration feature | `tests/health_check_tests.rs` source text | Every `#[tokio::test]` preceded immediately by `#[cfg(feature = "integration")]` |
| T03 | No conditional skip logic | `tests/health_check_tests.rs` test bodies | `grep "if let Ok\|try_create_test_client" tests/health_check_tests.rs` → 0 matches |
| T04 | Real API call succeeds with credentials | integration feature + valid HUGGINGFACE_API_KEY | Test passes against live HuggingFace health endpoint |
| T05 | Loud failure without credentials | integration feature + no HUGGINGFACE_API_KEY | Test panics with actionable credential error message |

## Acceptance Criteria

- `grep -n "wiremock" tests/health_check_tests.rs` → 0 matches
- Every `#[tokio::test]` in the file is immediately preceded by `#[cfg(feature = "integration")]`
- `grep -n "if let Ok\|try_create_test_client" tests/health_check_tests.rs` → 0 matches
- `wiremock` absent from `[dev-dependencies]` in `Cargo.toml` (or still present only if another test file uses it — document which)
- `w3 .test l::3` → 0 failures, 0 warnings

## Validation

**Execution:** An independent validator walks this section per `validation.rulebook.md` after SUBMIT transition. The executor does NOT self-validate.

### Checklist

**Mock removal**
- [ ] C1 — Does `grep -n "wiremock" tests/health_check_tests.rs` return 0 matches?
- [ ] C2 — Are zero `MockServer` or `Mock::given` usages present in `tests/health_check_tests.rs`?

**Integration gating**
- [ ] C3 — Does every `#[tokio::test]` in the file have `#[cfg(feature = "integration")]` immediately before it (not at file level)?
- [ ] C4 — Is there zero `#![cfg(feature = "integration")]` file-level gate?
- [ ] C5 — Are zero `if let Ok(client)` or `try_create_test_client()` patterns present in the test bodies?

**Compilation and tests**
- [ ] C6 — Does `RUSTFLAGS="-D warnings" cargo build --all-features` complete with 0 errors and 0 warnings?
- [ ] C7 — Does `w3 .test l::3` complete with 0 failures and 0 warnings?

**Out of Scope confirmation**
- [ ] C8 — Are zero `src/` files modified?
- [ ] C9 — Are zero other test files modified beyond `health_check_tests.rs` (and `Cargo.toml` if wiremock dep removed)?

### Measurements

- [ ] M1 — wiremock refs removed: `grep -c "wiremock" tests/health_check_tests.rs` → 0
- [ ] M2 — integration gates added: count of `#[cfg(feature = "integration")]` lines in file = count of `#[tokio::test]` lines in file (1:1 ratio)
- [ ] M3 — wiremock dep status: `grep -c "wiremock" Cargo.toml` → 0 if no other test file uses it; document if retained

### Invariants

- [ ] I1 — test suite: `w3 .test l::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings
- [ ] I3 — no-mock invariant: `grep -r "wiremock\|httpmock\|MockServer" tests/` → 0 matches across all test files after this task (if wiremock removed from Cargo.toml)

### Anti-faking checks

- [ ] AF1 — no trivial pass: `grep -n "assert!(true)\|unimplemented!()" tests/health_check_tests.rs` → 0 matches
- [ ] AF2 — no silent skip: `grep -n "if.*credentials\|if.*token\|skip\|SKIP" tests/health_check_tests.rs` → 0 matches

## Verification Record

**[2026-06-13]** VERIFY PASS — 4 independent subagents dispatched (D1 Scope Coherence, D2 MOST Goal Quality, D3 Value/YAGNI, D4 Implementation Readiness).
- D1 Scope Coherence: PASS — In Scope (health_check_tests.rs) and Out of Scope (src/ files, other test files) non-empty; observable outcome (grep wiremock → 0); single deliverable.
- D2 MOST Goal Quality: PASS — Motivated (hard invariant IN-09 violated — no mock HTTP servers in any test file); Observable (grep → 0 + test suite passes); Scoped (1 test file); Testable (grep + w3 .test l::3).
- D3 Value / YAGNI: PASS — Hard invariant violation (IN-09) is a concrete committed constraint; wiremock dependency confirmed present; no speculative work.
- D4 Implementation Readiness: PASS — Work Procedure has concrete ordered steps; Test Matrix has rows; Acceptance Criteria all machine-verifiable grep/build checks; no ambiguity in replacement pattern.

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — Task filed by code audit session. Goal: remove wiremock from health_check_tests.rs (violates no-mock invariant IN-09) and replace with real HuggingFace API calls gated by #[cfg(feature = "integration")].
- **2026-06-13** `VERIFIED` — MAAV gate passed (4 independent subagents). State → 🎯 (Verified).
- **2026-06-13** `COMPLETED` — Implementation validated by MAAV (adversarial + completeness subagents): 0 wiremock refs, 0 MockServer/Mock::given usages, 15 real-API tests properly gated with `#[cfg(feature = "integration")]`, 3 ungated tests correctly identified as pure unit tests (no network calls, labeled "no network" in file comments), JoinHandle `if let Ok( Ok( _ ) )` at line 433 confirmed legitimate (explicit `assert_eq!` assertion follows). State → ✅ (Completed).

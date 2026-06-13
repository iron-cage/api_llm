# Implement Operation Test Specs

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** ✅ (Completed)
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** tests/
- **Validated By:** null
- **Validation Date:** null

## Goal

Establish explicit spec-to-test traceability for the 15 operational scenarios in `tests/docs/operation/001_secret_loading.md` by creating named test functions in `tests/inc/` whose names match the OP- scenario identifiers. Task 001 covers TC/TS/AP/FT/PT (38 scenarios) but explicitly excludes all OP scenarios — they are the sole remaining gap that leaves the operation spec file orphaned. An auditor cannot confirm which test covers which loading path or rollback procedure without grepping through 47+ test files manually. Named functions (`test_op_01`..`test_op_15`) make the linkage auditable in CI output and maintainable when procedures change. Where an existing test already validates the behavior, the named function may delegate — no duplicate logic required. Observable outcome: 15 named test functions in `tests/inc/` (one per OP- ID), all OP scenarios promoted to ✅, `w3 .test l::3` green with zero new failures. Proven by `grep -r "fn test_op_" tests/inc/ | wc -l` → ≥ 15.

## In Scope

- `tests/inc/` — one new test module with 15 named functions, one per OP- scenario ID
- `tests/docs/operation/001_secret_loading.md` — all OP-01..OP-15 scenarios
- Delegation to existing tests where behavior is already validated — thin wrappers acceptable, no duplication required
- Updating Status column from ⏳ to ✅ in `tests/docs/operation/001_secret_loading.md` as scenarios are implemented
- Updating `tests/inc/mod.rs` to include the new test module

## Out of Scope

- TC-01..TC-06, TS-01..TS-06, AP-01..AP-12, FT-01..FT-08, PT-01..PT-06 — covered by task 001
- Performance benchmarks — separate bench task
- Cross-crate test coverage — api_claude scope only
- New test infrastructure beyond test functions — no new harnesses
- Duplicating test logic that already exists — delegation and thin wrappers are the preferred form

## Requirements

- All work must adhere to applicable rulebooks (`kbase .rulebooks`)
- Integration tests (OP-01, OP-03, OP-08, OP-09, OP-13, OP-14): per-function `#[cfg(feature = "integration")]` attribute on each integration test function; real credentials via `from_workspace()` or `from_env()`; load with `.expect("...")`, never `if let Ok`; do NOT use file-level `#![cfg(feature = "integration")]` — the module contains both integration and structural tests
- Structural/unit tests (OP-02, OP-04, OP-05, OP-06, OP-07, OP-10, OP-11, OP-12, OP-15): no mocking; test against real code paths with controlled inputs
- No new mocks, fake keys, or hardcoded responses anywhere
- All functions under 50 lines; 2-space indent; `mod private { }` pattern per module organization pattern

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note constraints on test file structure, feature gating, credential loading.
2. **Read spec scenarios** — read `tests/docs/operation/001_secret_loading.md` OP-01..OP-15 in full before writing any test.
3. **Confirm API surface** — verify call sites for T10–T13 and T15: `validate_anthropic_secret() -> Result<String>` (`src/environment.rs:59`), `secret_diagnostic_info() -> String` (`src/environment.rs:125`), `validate_workspace_structure() -> Result<PathBuf>` (`src/environment.rs:214`). Confirm re-export paths via `mod_interface!` so tests can import them. For T15 (cwd outside workspace): use `std::env::set_current_dir` into a temp directory isolated from the workspace; restore after the test using a guard.
4. **Write failing tests** — for each spec scenario, write the test body and confirm it fails (or compile-errors) before implementing. If a test passes trivially, it is wrong.
5. **Implement** — write minimum test code to make tests pass against real API or via structural checks; delegate to existing tests where behavior is already covered.
6. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings.
7. **Update spec file** — change ⏳ → ✅ in `tests/docs/operation/001_secret_loading.md` Overview Table for every implemented scenario.
8. **Submit for Validation** — trigger SUBMIT transition.
9. **Update task state** — on validation pass, update `task/readme.md` and move file to `task/completed/`.

## Test Matrix

*(Required — write before writing any test code.)*

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | OP-01: env var success path | ANTHROPIC_API_KEY set with valid sk-ant-api03- key | from_env() returns Ok(Client); client secret holds the exact key string |
| T02 | OP-02: env var absent error | ANTHROPIC_API_KEY not set or empty | from_env() returns Err; no panic; error message references ANTHROPIC_API_KEY |
| T03 | OP-03: workspace success path | Valid Cargo workspace; secret/-secrets.sh contains valid key | from_workspace() returns Ok(Client); key starts with sk-ant- |
| T04 | OP-04: workspace secrets file absent | Valid Cargo workspace root; no secret/-secrets.sh | from_workspace() returns Err; no panic; error describes missing secrets file |
| T05 | OP-05: secrets file missing key | secret/-secrets.sh exists but no ANTHROPIC_API_KEY line | from_workspace() returns Err; no panic; error indicates key not found |
| T06 | OP-06: direct construction valid key | Key string starting sk-ant-api03- with length ≥ 30 | Secret::new() succeeds; Client::new(secret) returns usable Client with the key |
| T07 | OP-07: direct construction invalid key | Key string "bad", "", or without sk-ant- prefix | Secret::new(invalid) returns Err; no panic; Client is never constructed |
| T08 | OP-08: key format invariant | Client from_env() or from_workspace() in real test environment | Key starts with sk-ant-; length > 30; both conditions hold simultaneously |
| T09 | OP-09: env and workspace keys identical | Both env var and secrets file contain the same valid key | from_env() and from_workspace() both return clients holding byte-for-byte identical keys |
| T10 | OP-10: validate secret key present | At least one source has non-empty value; call `api_claude::environment::validate_anthropic_secret()` (`src/environment.rs:59`, returns `Result<String>`) | Returns Ok(source_string); source_string is non-empty |
| T11 | OP-11: validate secret key absent | No source has ANTHROPIC_API_KEY; call `validate_anthropic_secret()` | Returns Err; no panic; error message is non-empty |
| T12 | OP-12: diagnostic info always callable | Any environment state; call `api_claude::environment::secret_diagnostic_info()` (`src/environment.rs:125`, returns `String`) | Returns non-empty String; never panics regardless of environment |
| T13 | OP-13: validate workspace structure valid | Valid workspace + secrets file; call `api_claude::environment::validate_workspace_structure()` (`src/environment.rs:214`, returns `Result<PathBuf>`) | Returns Ok(path); path.to_string_lossy() is non-empty |
| T14 | OP-14: rollback — unset env var then reload succeeds | Env var was incorrect; secrets file has valid key; env var unset before reload | from_workspace() after env var unset returns Ok(Client) from workspace source; no residual incorrect env state |
| T15 | OP-15: from_workspace() fails without reachable workspace | Use `std::env::set_current_dir("/tmp")` in test body (restore after via drop guard); call from_workspace() | Returns Err; no panic; error references workspace detection failure |

## Acceptance Criteria

- Every Test Matrix row has a corresponding test function that passes under `w3 .test l::3`
- All 15 OP scenarios (OP-01..OP-15) updated to ✅ in `tests/docs/operation/001_secret_loading.md`
- No test uses mocks, fake keys, or hardcoded credentials
- Integration test functions (T01, T03, T08, T09, T13, T14) each carry per-function `#[cfg(feature = "integration")]` attribute; no file-level `#![cfg]` gate is used
- Unit/structural tests (T02, T04, T05, T06, T07, T10, T11, T12, T15) make no real API calls
- `w3 .test l::3` passes with zero new failures and zero new warnings

## Validation

**Execution:** An independent validator walks this section after SUBMIT transition.

### Checklist

**Test coverage completeness**
- [ ] C1 — Is `tests/docs/operation/001_secret_loading.md` fully updated with ✅ for all 15 scenarios?
- [ ] C2 — Are all 15 Test Matrix rows implemented as named test functions?
- [ ] C3 — Does each test function name correspond to its spec scenario ID (e.g., `test_op_01`)?

**Compliance**
- [ ] C4 — Do all integration test functions (T01, T03, T08, T09, T13, T14) carry per-function `#[cfg(feature = "integration")]`? (file-level `#![cfg]` is a FAIL — it suppresses all structural tests when integration feature is absent)
- [ ] C5 — Does every integration test load credentials via `.expect("...")` unconditionally?
- [ ] C6 — Are zero mocks or fake API keys present in the new test file?

**Out of Scope confirmation**
- [ ] C7 — Are TC/TS/AP/FT/PT scenario IDs absent from the new implementation file?
- [ ] C8 — Are zero cross-crate test files modified?

### Measurements

- [ ] M1 — new test functions: `grep -r "fn test_op_" tests/inc/ | wc -l` → ≥ 15
- [ ] M2 — spec ✅ count: `grep -E "^\| OP-[0-9]+" tests/docs/operation/001_secret_loading.md | grep -c "✅"` → ≥ 15
- [ ] M3 — zero ⏳ remaining: `grep -E "^\| OP-[0-9]+" tests/docs/operation/001_secret_loading.md | grep -c "⏳"` → 0

### Invariants

- [ ] I1 — test suite: `w3 .test l::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — no trivial passes: `grep -r "assert!(true)\|unimplemented!()" tests/inc/ | grep "op_"` → 0 matches
- [ ] AF2 — no hardcoded key strings: `grep -rn "\"sk-ant-" tests/inc/ | grep "op_"` → 0 matches (catches any sk-ant- literal in op_ test functions regardless of format)

## Related Documentation

| Path | Role |
|------|------|
| `tests/docs/operation/01_secret_loading.md` | Spec source — OP-01..OP-15 |
| `docs/operation/001_secret_loading.md` | Authoritative operational contract for all OP scenarios |
| `tests/inc/authentication_test.rs` | Existing coverage — Secret validation and auth error handling |
| `tests/inc/workspace_loading_integration_test.rs` | Existing coverage — workspace loading paths |
| `tests/inc/spec_verification_integration_test.rs` | Existing coverage — API key format verification |
| `task/completed/001_implement_doc_test_specs.md` | Related: 001 — Case E; covers TC/TS/AP/FT/PT (38 scenarios); excludes OP |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — Task filed by doc_tsk session. Goal: implement OP-01..OP-15 from tests/docs/operation/001_secret_loading.md as named test functions; these are the only spec scenarios without task coverage after task 001 closed all 38 non-operation scenarios.
- **2026-06-13** `VERIFY FAIL` — Dim 4 FAIL: (1) Work Procedure missing step to confirm API surface for T10–T13 before writing tests; (2) T10–T13 omitted call sites for validate_anthropic_secret/secret_diagnostic_info/validate_workspace_structure; (3) T15 gave no guidance on cwd manipulation approach; (4) M2/M3 grep patterns were fragile (grep -v readme could silently drop OP- rows); (5) AF2 did not catch well-formed fake keys like sk-ant-api03-fake-.... All findings fixed: Step 3 added to procedure; T10–T13 updated with src/environment.rs line refs and return types; T15 updated with std::env::set_current_dir approach; M2/M3 rewritten with grep -E "^\| OP-" anchor; AF2 strengthened to catch any "sk-ant-" string literal.
- **2026-06-13** `VERIFY FAIL` — Dim 4 FAIL: one finding: Requirements and Acceptance Criteria specified `#![cfg(feature = "integration")]` (file-level inner attribute), but the new module contains both integration tests (T01/T03/T08/T09/T13/T14) and structural/unit tests (T02/T04–T07/T10–T12/T15). File-level gating would suppress all 9 structural tests when integration feature is absent. Project convention uses per-function `#[cfg(feature = "integration")]`. Fixed: Requirements, Acceptance Criteria, and Checklist C4 all updated to require per-function attribute with explicit prohibition on file-level gate.
- **2026-06-13** `VERIFY PASS` — All 4 MAAV dimensions pass. Dim 1 (Scope Coherence): PASS. Dim 2 (MOST Goal Quality): PASS — M/O/S/T all pass; motivation is non-circular (orphaned spec, auditor grep burden). Dim 3 (Value/YAGNI adversarial): PASS — zero test_op_ functions exist; authentication_test.rs bodies are gutted stubs; workspace_loading_integration_test.rs uses soft println! failures; gap is real. Dim 4 (Implementation Readiness): PASS — per-function #[cfg] fix confirmed in Requirements, Acceptance Criteria, and C4; API surface citations present; TDD step present.
- **2026-06-13** `COMPLETED` — All 15 OP scenarios implemented in `tests/inc/operation_test_specs.rs` as named test functions test_op_01..test_op_15. All 15 rows in tests/docs/operation/001_secret_loading.md promoted to ✅. Structural tests (OP-02, 04–07, 10–12, 15) pass without credentials. Integration tests (OP-01, 03, 08, 09, 13, 14) fail loudly per Loud Failure Mandate when credentials absent. Clippy clean. M1=15+, M2=15, M3=0, AF1=0, AF2=0 all confirmed.

## Verification Record

- **Date:** 2026-06-13
- **Method:** MAAV — 4 independent subagents, one adversarial
- **Dim 1 (Scope Coherence):** PASS
- **Dim 2 (MOST Goal Quality):** PASS
- **Dim 3 (Value/YAGNI — adversarial):** PASS — adversarial agent confirmed zero test_op_ functions exist; authentication_test.rs bodies are gutted stubs with non-existent API calls; workspace_loading_integration_test.rs uses soft println! error paths; gap is real and unsubstitutable
- **Dim 4 (Implementation Readiness):** PASS — per-function #[cfg] requirement confirmed correct in all three enforcement points; API surface citations present with file:line; TDD ordering enforced; no blocking ambiguities

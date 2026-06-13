# Test Suite Compliance

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** ❓ (Unverified)
- **Priority:** 1
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** tests/, src/
- **Validated By:** null
- **Validation Date:** null

## Goal

Fix six active violation categories discovered during the 2026-06-13 audit (V01–V05, V07; V06 pre-resolved — `providers_api_integration_test.rs` was deleted before task execution): missing `#[cfg(feature = "integration")]` guards on test files where no module-level gate exists, silent-pass/skip patterns (~34 functions returning instead of panicking), duplicated integration test helper code (get_api_key_* ×12), wrong model identifier in two integration test calls in `inference_tests.rs`, and inline `#[cfg(test)] mod tests { }` block in `src/reliability/circuit_breaker.rs`. Observable outcome: `w3 .test l::3` → 0 failures, 0 warnings; all integration tests are gated; no integration helper duplication; no src-level inline test modules.

## In Scope

- All `.rs` files in `tests/` that lack any `#[cfg(feature = "integration")]` gate on their integration test functions — add function-level `#[cfg(feature = "integration")]` before each `#[tokio::test]` that makes real API calls. Files that already wrap integration tests in a `#[cfg(feature = "integration")] mod integration_tests { }` block are COMPLIANT and need no change.
- `tests/` files with silent `return` or `return Ok(())` inside integration-gated function bodies — replace with explicit `panic!` per invariant IN-07
- `tests/inc/` — consolidate `get_api_key_for_integration() -> String` (×6) and `get_api_key_for_testing() -> Option<String>` (×6) into single shared definitions. Unit-test helpers that use fake/stub credentials (not loading real workspace secrets) must NOT be modified.
- `tests/inference_tests.rs` lines 441 and 473 ONLY — replace `"gpt2"` with `"meta-llama/Llama-3.2-1B-Instruct"`. Do NOT change line 127 (validation unit test that intentionally tests identifier acceptance).
- `src/reliability/circuit_breaker.rs` lines 323-500 — move the `#[cfg(test)] mod tests { … }` block to `tests/circuit_breaker_tests.rs`; delete the inline block from src

## Out of Scope

- Files already using module-level `#[cfg(feature = "integration")] mod integration_tests { }` blocks — these are compliant; converting to function-level is NOT required
- Unit-test helpers using fake/stub credentials — do not consolidate or touch these
- `tests/inference_tests.rs` line 127 — intentionally validates `"gpt2"` as an identifier; must remain unchanged
- Other `src/` files with `#[cfg(test)] mod tests { }` blocks (17 total) — only `circuit_breaker.rs` is in scope
- `src/reliability/mod.rs` mod_interface violation — separate structural task
- `src/components/inference_shared.rs` mod_interface violation — separate structural task
- `Cargo.toml` — no changes required

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Module-level `#[cfg(feature = "integration")] mod { }` gates are acceptable and compliant — do NOT require function-level gates where module-level already exists
- When adding function-level gates: `#[ cfg( feature = "integration" ) ]` must appear immediately before `#[ tokio::test ]`; no file-level `#![cfg]`
- Silent pass prohibition: integration-gated test functions must `panic!` (not `return`) when credentials are absent or API calls fail unexpectedly
- No `cargo fmt`; custom codestyle throughout (2-space indent, braces on new line)
- Integration credential helpers live in `tests/inc/mod.rs`; no duplicates across test files

## Violations (Audit Reference — 2026-06-13)

| # | Category | Count | Example Location |
|---|----------|-------|-----------------|
| V01 | Missing integration gate (no module-level or function-level guard) | Subset of test files | Files with ungated `#[tokio::test]` that call real endpoints |
| V02 | Silent pass/skip (`return` / `return Ok(())`) inside integration-gated fns | ~34 functions | `tests/embeddings_tests.rs`, `tests/cache_tests.rs`, … |
| V03 | Duplicate `get_api_key_for_integration() -> String` | ×6 | across `tests/*.rs` |
| V04 | Duplicate `get_api_key_for_testing() -> Option<String>` | ×6 | across `tests/*.rs` |
| V05 | Wrong model "gpt2" in integration test calls | lines 441, 473 only | `tests/inference_tests.rs` |
| V06 | `async fn main()` instead of `#[tokio::test]` — **RESOLVED**: target file deleted before task execution | 0 | N/A |
| V07 | Inline `#[cfg(test)] mod tests {}` in src/ | 1 file (in scope) | `src/reliability/circuit_breaker.rs:323-500` |

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; confirm integration gate convention and loud-failure requirement.
2. **Audit test files** — for each file in `tests/`, determine if real-API tests are already covered by a module-level `#[ cfg( feature = "integration" ) ] mod { }` block. Mark such files COMPLIANT. For remaining files with ungated `#[ tokio::test ]` functions that call real endpoints, note them for step 4.
3. **Consolidate integration credential helpers** — in `tests/inc/mod.rs`, create one canonical `get_api_key_for_integration() -> String` (panics on missing key) and one `get_api_key_for_testing() -> Option< String >`. Delete the 12 duplicate definitions from individual test files. Do NOT touch unit-test stub helpers.
4. **Add integration gates** — for each file identified in step 2 as needing gates, add `#[ cfg( feature = "integration" ) ]` immediately before each `#[ tokio::test ]` that makes real API calls. Do NOT gate unit tests that only construct clients with fake credentials or test serialization/validation logic.
5. **Fix silent passes** — inside integration-gated test function bodies, replace every silent `return` / `return Ok(())` with `panic!( "integration test failed: {e}" )` or `panic!( "expected error but test passed" )` as appropriate.
6. **Fix wrong model** — in `tests/inference_tests.rs`, at lines 441 and 473 ONLY, replace `"gpt2"` with `"meta-llama/Llama-3.2-1B-Instruct"`. Do NOT touch line 127.
7. **V06 resolved — skip** — `tests/providers_api_integration_test.rs` was deleted before task execution; no implementation work required for this violation category.
8. **Move inline tests** — copy the entire `#[cfg(test)] mod tests { … }` block (lines 323-500) from `src/reliability/circuit_breaker.rs` to `tests/circuit_breaker_tests.rs`; add necessary imports; delete the inline block from src.
9. **Green state** — `w3 .test l::3` → 0 failures, 0 warnings before proceeding.
10. **Update task state** — move file to `task/completed/`; update index.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Every real-API test is gated | All `tests/*.rs` | Every integration test function is inside a `#[cfg(feature = "integration")]` guard (module-level or function-level) |
| T02 | No file-level cfg gate in any test file | All `tests/*.rs` | `grep -rc "#!\[cfg"` tests/ → 0 |
| T03 | No silent `return Ok(())` in integration-gated fn bodies | Integration-gated fns | `grep` for bare `return Ok(())` inside such functions → 0 |
| T04 | Single `get_api_key_for_integration` definition | `tests/inc/mod.rs` | `grep -r "fn get_api_key_for_integration"` tests/ → exactly 1 match |
| T05 | No "gpt2" in integration test calls | `tests/inference_tests.rs` lines 441, 473 | Those lines use `"meta-llama/Llama-3.2-1B-Instruct"` |
| T06 | Line 127 of inference_tests.rs unchanged | `tests/inference_tests.rs` | `grep -n "gpt2"` still returns line 127 |
| T07 | No `async fn main` in tests/ | N/A — target file deleted (V06 pre-resolved) | `grep -r "async fn main"` tests/ → 0 |
| T08 | No `#[ cfg( test ) ] mod tests` in circuit_breaker.rs | `src/reliability/circuit_breaker.rs` | `grep -c "#\[ cfg( test ) \]" src/reliability/circuit_breaker.rs` → 0 |
| T09 | Full test suite passes | All features | `w3 .test l::3` → 0 failures |

## Acceptance Criteria

- Every real-API test in `tests/` is covered by a `#[cfg(feature = "integration")]` guard (module-level block or function-level attribute)
- `grep -rc "#!\[cfg(feature"` across `tests/` → 0 (no file-level gates)
- `grep -r "fn get_api_key_for_integration"` across `tests/` → exactly 1 definition (in `tests/inc/mod.rs`)
- `grep -r "fn get_api_key_for_testing"` across `tests/` → exactly 1 definition (in `tests/inc/mod.rs`)
- Lines 441 and 473 of `tests/inference_tests.rs` use `"meta-llama/Llama-3.2-1B-Instruct"` (not `"gpt2"`)
- Line 127 of `tests/inference_tests.rs` is unchanged (still contains `"gpt2"` in the validation list)
- All integration-gated test function bodies use `panic!` (not bare `return` or `return Ok(())`) when handling missing credentials or unexpected API errors
- `grep -r "async fn main"` across `tests/` → 0 (V06 pre-resolved: target file deleted; this check confirms no regression)
- `grep -c "#\[ cfg( test ) \]" src/reliability/circuit_breaker.rs` → 0 (17 other src/ files with inline test blocks are out of scope)
- `w3 .test l::3` → 0 failures, 0 warnings

## Validation

**Execution:** An independent validator walks this section per `validation.rulebook.md` after SUBMIT transition. The executor does NOT self-validate.

### Checklist

**Integration gate compliance**
- [ ] C1 — Is every real-API test covered by a `#[cfg(feature = "integration")]` guard (module-level or function-level)?
- [ ] C2 — Is there zero `#![cfg(feature = "integration")]` file-level gate in any file?
- [ ] C3 — Where function-level gates are added: does `#[cfg(feature = "integration")]` appear immediately before `#[tokio::test]` with no lines between?

**Helper deduplication**
- [ ] C4 — Is `fn get_api_key_for_integration` defined in exactly one place (`tests/inc/mod.rs`)?
- [ ] C5 — Is `fn get_api_key_for_testing` defined in exactly one place (`tests/inc/mod.rs`)?

**Specific violation fixes**
- [ ] C6 — Do lines 441 and 473 of `tests/inference_tests.rs` use `"meta-llama/Llama-3.2-1B-Instruct"`?
- [ ] C7 — Does line 127 of `tests/inference_tests.rs` still contain `"gpt2"` in the validation list (UNCHANGED)?
- [ ] C8 — Does `grep -r "async fn main" tests/` → 0? (V06 pre-resolved: `providers_api_integration_test.rs` was deleted; check confirms no new `async fn main` was introduced.)
- [ ] C9 — Does `src/reliability/circuit_breaker.rs` contain zero `#[cfg(test)] mod tests` blocks?

**Loud failure**
- [ ] C10 — Do all integration-gated test function bodies use `panic!` (not bare `return`) when credentials are absent or API calls fail unexpectedly?

**Compilation and tests**
- [ ] C11 — Does `RUSTFLAGS="-D warnings" cargo build --all-features` complete with 0 errors and 0 warnings?
- [ ] C12 — Does `w3 .test l::3` complete with 0 failures and 0 warnings?

### Measurements

- [ ] M1 — `grep -rc "fn get_api_key_for_integration"` tests/ → `1`
- [ ] M2 — `grep -rc "fn get_api_key_for_testing"` tests/ → `1`
- [ ] M3 — `grep -r "async fn main"` tests/ → empty
- [ ] M4 — `grep -c "#\[ cfg( test ) \]" src/reliability/circuit_breaker.rs` → 0
- [ ] M5 — `grep -n "gpt2" tests/inference_tests.rs` → line 127 present; lines 441, 473 absent

### Invariants

- [ ] I1 — test suite: `w3 .test l::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — no file-level gates: `grep -rn "#!\[cfg" tests/*.rs` → 0 matches
- [ ] AF2 — integration test coverage preserved: `grep -c "#\[tokio::test\]" tests/providers_api_tests.rs` > 0 (V06 pre-resolved: `providers_api_integration_test.rs` was deleted; `providers_api_tests.rs` retains unit test coverage)
- [ ] AF3 — circuit_breaker test content preserved: the 8 test function names from src/reliability/circuit_breaker.rs now exist in `tests/circuit_breaker_tests.rs`
- [ ] AF4 — unit tests unaffected: tests in `inference_tests.rs` outside the `mod integration_tests { }` block remain ungated (unit test coverage preserved)

## Pre-Filing Review

**MAAV run:** 2026-06-13 (4 independent subagents — scope, goal quality, YAGNI adversarial, implementation readiness)

### Findings Applied

| Finding | Severity | Resolution |
|---------|----------|-----------|
| `simple_inference_integration_test.rs` excluded by task but file does not exist | CRITICAL | Removed exclusion; file is absent from tests/ |
| V01 ambiguous: many files already have module-level `#[cfg] mod integration_tests { }` gates | MAJOR | Clarified: module-level gates are compliant; only add function-level where no gate exists |
| V06 scope error: `"gpt2"` at line 127 is a validation unit test that must not change | MAJOR | Scoped to lines 441 and 473 only; line 127 explicitly preserved in AC and Checklist |
| V02 orphaned: T03 test had no corresponding AC | CRITICAL | Added explicit AC for silent-pass remediation (C10) |
| Step 4 ambiguous: could incorrectly gate unit tests | HIGH | Clarified: only gate functions that make real HTTP calls; unit tests with fake credentials excluded |
| Step 6 ambiguous: `Models::llama_3_2_1b_instruct()` not found in codebase | HIGH | Corrected to exact string literal `"meta-llama/Llama-3.2-1B-Instruct"` |
| Helper consolidation for `create_test_client()` may break unit/integration separation | MEDIUM | Removed `create_test_client` from consolidation scope; only consolidate `get_api_key_*` helpers |
| Step 7: fn main() conversion needs specificity | MEDIUM | Clarified: convert to single `test_providers_api_integration()` function |

## Verification Record

**[2026-06-13]** VERIFY FAIL — D1, D2, and D3 blocking findings below. State remains ❓ (Unverified).
- D1 Scope Coherence: FAIL — Task bundles 6 independent violation categories (V01 missing gates, V02 silent passes, V03/V04 helper duplication, V05 wrong model, V07 inline src tests) into a single deliverable. Each category targets a different set of files and has a different acceptance criterion. A reviewer cannot independently verify one fix without verifying all. This violates the single-deliverable principle required for the VERIFY gate.
- D2 MOST Goal Quality: FAIL — No single MOST-compliant goal can be stated for 6 independent violation categories. The observable outcome is a conjunction of 6 disjoint grep/count checks, not a single observation. Each violation should be its own task with a single MOST goal.
- D3 Value / YAGNI: FAIL — Priority=1 (< 2 required for VERIFY gate). Additionally, V05 (wrong model "gpt2" at lines 441/473) may be stale — those line numbers should be verified against the current codebase before re-filing. The Pre-Filing Review has been completed (8 findings applied), but the structural bundling issue was not addressed.
- D4 Implementation Readiness: PASS — Work Procedure steps are concrete and executable for each individual violation category; Test Matrix has 9 rows.
- **Required to unblock:** Split into individual tasks (one per violation category: V01/V02/V03+V04/V05/V07); raise each to Priority ≥ 2; verify V05 line numbers against current codebase. OR: raise Priority to ≥ 2 AND narrow scope to a single violation category.

**[2026-06-13 — Re-VERIFY]** VERIFY FAIL (confirmed + expanded) — D1, D2, D3 still blocking; mechanical checks reveal multiple stale claims. State remains ❓ (Unverified).
- D1 Scope Coherence: FAIL (confirmed) — Task still bundles 5 independent violation categories; structural bundling issue unaddressed.
- D2 MOST Goal Quality: FAIL (confirmed) — No single MOST-compliant goal; 5 disjoint observable outcomes; structural issue unaddressed.
- D3 Value / YAGNI: FAIL (confirmed + expanded) — Priority remains 1 (< 2 required). Mechanical checks reveal: V05 stale ("gpt2" at lines 441/473 already corrected — only occurrence is line 129 in unit test `test_model_identifier_validation`); V03/V04 stale (get_api_key_* helpers already consolidated — exactly 1 definition each in `tests/inc/mod.rs`); V07 stale (`src/reliability/circuit_breaker.rs` is 322 lines with no inline test block; claimed lines 323-500 do not exist); V06 factual error (`tests/providers_api_integration_test.rs` was NOT deleted — file exists and is properly gated). Only V01 and V02 remain potentially valid and require fresh audit.
- D4 Implementation Readiness: PASS — Work Procedure steps concrete; Test Matrix populated.
- **Required to unblock (updated):** (1) Raise Priority to ≥ 2; (2) Run fresh audit to confirm which violations among V01/V02 still apply and at which file/line locations; (3) Rewrite task scope around surviving violations only (V03/V04/V05/V07/V06 all stale); (4) Split into individual tasks per surviving category OR narrow to one category.

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — Task filed following audit identifying violation categories across tests/ and src/.
- **2026-06-13** `REVISED` — MAAV (4 agents) identified 8 actionable issues.
- **2026-06-13** `NOTE` — V06 pre-resolved: `tests/providers_api_integration_test.rs` was deleted before task execution (git status: `D`). No implementation work needed for this violation category. Applied corrections: removed non-existent file exclusion; clarified module-level gate compliance; scoped V06 to lines 441/473 only (line 127 preserved); added C10 AC for V02; clarified step 4 (no unit test gating); corrected step 6 replacement to string literal; removed create_test_client from consolidation scope; specified single test function for fn main() conversion.
- **2026-06-13** `VERIFY FAIL` — MAAV gate blocked by D1 (bundles 6 independent deliverables), D2 (no single MOST goal), D3 (Priority=1 < 2; stale V05 line numbers). Remains ❓ (Unverified) until task is split or scope is narrowed to a single violation category.
- **2026-06-13** `VERIFY FAIL (Re-VERIFY)` — Second MAAV run + mechanical checks confirm all 3 blocking dimensions unchanged. New findings: V05 stale (gpt2 only at line 129 unit test; lines 441/473 already use correct model); V03/V04 stale (helpers already consolidated in tests/inc/mod.rs — 1 definition each); V07 stale (circuit_breaker.rs is only 322 lines, no inline test block); V06 factual error (providers_api_integration_test.rs exists and is properly gated — was NOT deleted). Only V01 and V02 potentially still apply. Fresh audit required before re-filing.

# Bring api_gemini Test Suite into Compliance with Testing Standards Invariant

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** ❓ (Unverified)
- **Closes:** null

## MOST Goal

**Motivated**: INV-002 (Testing Standards) mandates two constraints currently violated:
(1) Every integration test function must carry `#[ cfg( feature = "integration" ) ]` immediately before `#[ tokio::test ]`. Current tests carry no such gate.
(2) `common::create_integration_client()` is the sole canonical integration client factory; `tests/integration_tests.rs` has a duplicate local `create_test_client()` definition violating the "no duplicate test helper" prohibition.

**Observable**: After implementation, all three hold — each is independently re-runnable:
- `grep -rn "fn create_test_client" tests/ | grep -v "/common/"` → zero matches (all non-canonical duplicates removed).
- `for F in $(grep -rln --include="*.rs" -E "create_integration_client|Client::new|Client::builder" tests/ | grep -v "/common/"); do grep -cq "#\[ cfg( feature = \"integration\" ) \]" "$F" || echo "UNGATED: $F"; done` → no UNGATED lines printed. (The `--include="*.rs"` excludes documentation and aggregator files that match the grep but contain no `#[ tokio::test ]` annotations.)
- `GEMINI_API_KEY="" cargo nextest run --no-default-features --features enabled` run from the `api_gemini` crate root → exit code 0.

**Scoped**: All files returned by `grep -rln "fn create_test_client" tests/ | grep -v "/common/"` for duplicate removal; all files in `INTEGRATION_FILES` (Rust files only) for gate addition. No changes to `src/`, `benches/`, or `examples/`.

**Testable**: Each observable is a single re-runnable command or pipeline producing deterministic output without retained session state.

## In Scope

- All files returned by `grep -rln "fn create_test_client" tests/ | grep -v "/common/"` — remove each local `create_test_client()` definition; migrate call sites in the same file to `common::create_integration_client()`
- All files in `INTEGRATION_FILES` = `grep -rln --include="*.rs" -E "create_integration_client|Client::new|Client::builder" tests/ | grep -v "/common/"` — add `#[ cfg( feature = "integration" ) ]` gate before every `#[ tokio::test ]` in those files
- `tests/common/mod.rs` — read-only; confirm `create_integration_client()` panics on missing key

## Out of Scope

- `src/` — no source changes
- `benches/` — exempt per INV-002 Scope
- `examples/` — exempt per INV-002 Scope
- Files NOT in `INTEGRATION_FILES` — no gating needed (they neither call `create_integration_client()` nor construct a `Client` directly)
- `common::create_integration_client()` implementation — canonical; not modified

## Work Procedure

0. **Enumerate scope**: From the `api_gemini` crate root (directory containing `api_gemini/Cargo.toml`), run:
   - `DUPLICATE_FILES = grep -rln "fn create_test_client" tests/ | grep -v "/common/"` — files containing non-canonical duplicate helpers.
   - `INTEGRATION_FILES = grep -rln --include="*.rs" -E "create_integration_client|Client::new|Client::builder" tests/ | grep -v "/common/"` — Rust files containing API-calling tests.
   Both sets are the definitive scope for subsequent steps.
1. **Confirm pre-condition**: Read `tests/common/mod.rs`. Verify `create_integration_client()` contains an explicit `panic!` call for the missing-key case. If absent, stop and report — that is a separate bug outside this task's scope. If present, proceed.
2. **Locate duplicates**: Read each file in `DUPLICATE_FILES` (from Step 0). For each, note the `create_test_client()` function definition and all its call sites within that file.
3. **Remove duplicates**: Delete each local `create_test_client()` function body from every file in `DUPLICATE_FILES`.
4. **Migrate call sites**: In each file from `DUPLICATE_FILES`, replace every `create_test_client()` call with `common::create_integration_client()`. Confirm `mod common;` (or the appropriate module import path) appears at the top of each modified file.
5. **Gate all INTEGRATION_FILES**: For each file in `INTEGRATION_FILES` (from Step 0): read the file; for every `#[ tokio::test ]` annotation — add `#[ cfg( feature = "integration" ) ]` on the line directly above with no other attributes, blank lines, or comments between the two attributes.
6. **Verify unit build**: From the `api_gemini` crate root, run `GEMINI_API_KEY="" cargo nextest run --no-default-features --features enabled`. Must exit code 0. If it fails, diagnose and fix before proceeding.
7. **Verify full suite**: From the `api_gemini` crate root, with `GEMINI_API_KEY` set to a valid key, run `cargo nextest run --all-features`. All integration tests must pass.
8. **Verify grep checks**: (a) `grep -rn "fn create_test_client" tests/ | grep -v "/common/"` → zero matches; (b) `for F in $(grep -rln --include="*.rs" -E "create_integration_client|Client::new|Client::builder" tests/ | grep -v "/common/"); do grep -cq "#\[ cfg( feature = \"integration\" ) \]" "$F" || echo "UNGATED: $F"; done` → no UNGATED lines printed.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|
| `GEMINI_API_KEY="" cargo nextest run --no-default-features --features enabled` (Step 6) | Unit build — `integration` feature absent | Exit code 0; no API key needed; gated tests excluded from compilation |
| `cargo nextest run --all-features` with valid `GEMINI_API_KEY` (Step 7) | Full suite with credentials | All integration tests execute and pass |
| `cargo nextest run --all-features` with `integration` feature, no `GEMINI_API_KEY` | Full suite, key absent | Tests using `create_integration_client()` panic loudly — pre-existing behavior confirmed in Step 1 |
| `grep -rn "fn create_test_client" tests/ \| grep -v "/common/"` (Step 8a) | Duplicate definition check — all test files | Zero matches |
| `for F in $(grep -rln --include="*.rs" -E "create_integration_client\|Client::new\|Client::builder" tests/ \| grep -v "/common/"); do grep -cq "cfg.*integration" "$F" \|\| echo "UNGATED: $F"; done` (Step 8b) | Gate coverage — all INTEGRATION_FILES (.rs only) | No UNGATED lines printed |

## Related Documentation

- `docs/invariant/002_testing_standards.md` — INV-002: Testing Standards invariant (governing constraint)
- `docs/invariant/001_thin_client_principle.md` — INV-001: Thin Client Principle (context)
- `docs/operation/003_testing.md` — NO MOCKUP TESTS policy and testing procedures
- `task/decisions.md` — Decision registry (no open questions matched)

## Verification Findings

### Attempt 1 — 2026-06-13 (FAIL)

Dimension 2 FAIL — three findings from independent subagent:
1. **Observable**: grep cross-reference cannot verify line-adjacency; build check proves compile success not gate coverage; `common/mod.rs` panic confirmation is a pre-condition not a completion check.
2. **Scoped**: "any function making network calls" is judgment-based, not structural; test files not enumerated.
3. **Testable**: Test Matrix row for "loud panic on missing credentials" not addressed by any Work Procedure step.

Fixes applied: replaced Observable with three deterministic checks (grep for duplicate, per-file grep for gate, unit build); introduced `INTEGRATION_FILES` as a mechanical enumeration (grep-based, not judgment-based); added Step 1 pre-condition check for panic behavior; added Step 8 for full-suite credential run.

### Attempt 5 — 2026-06-13 (FAIL — pre-investigation required)

Two adversarial subagents ran actual greps against the codebase and found structural complexity that invalidates the task's current framing:

1. **DUPLICATE_FILES grep lacks `--include="*.rs"`**: `grep -rln "fn create_test_client" tests/` returns both `tests/safety/mod.rs` (real duplicate, Rust file) AND `tests/readme.md` (documentation prose code example). Steps 2-4 applied to `tests/readme.md` would corrupt documentation. Fix: add `--include="*.rs"` to the DUPLICATE_FILES grep.

2. **`tests/safety/` submodule structure breaks after duplicate removal**: `tests/safety/integration_part1.rs` and `tests/safety/integration_part2.rs` call `create_test_client()` via `use super::*` from `safety/mod.rs`. After Step 3 removes the duplicate from `safety/mod.rs`, these files' `super::create_test_client()` calls become undefined. They are NOT in `INTEGRATION_FILES` (they don't match the grep patterns), so Step 5 never touches them. The task as scoped would break compilation of these files.

3. **`tests/deployment/mod.rs` has a broken `mod common;` path**: `tests/deployment/mod.rs` declares `mod common;` which resolves to a non-existent `tests/deployment/common/` directory. This is a pre-existing compilation error unrelated to this task. Step 6 (`GEMINI_API_KEY="" cargo nextest run --no-default-features`) would fail on this pre-existing issue.

4. **Test Matrix row 3 (loud panic on missing key) has no execution step**: Step 1 reads source code to confirm `panic!` exists — it does not run the scenario. Row 3 is verified by code inspection only, not by running `cargo nextest run` with `integration` feature enabled and no key.

These findings require a pre-investigation task to establish accurate scope before task 009 can be written correctly. Specifically: the `tests/safety/` submodule internal helper chain and the `tests/deployment/` compilation issue must be understood before a gate-addition task can be safely scoped.

### Attempt 2 — 2026-06-13 (FAIL)

Dimension 2 and Dimension 4 FAIL — four findings across two subagents:
1. **Observable 2**: `grep -rn "#\[ cfg ]" tests/` compared to Step 0 output requires cross-reference judgment.
2. **Scoped**: "any `Client` method" is judgment-dependent for indirect callers.
3. **Testable**: Test Matrix row 2 (full-suite credential run) had no corresponding Work Procedure step.
4. **Readiness**: Step 7 missing crate root path and clean-env precondition; `common/mod.rs` exclusion not expressed as a computable rule.

Fixes applied: `INTEGRATION_FILES` now uses `grep -rln "create_integration_client" tests/` as the sole mechanical criterion (no "Client method" judgment); gate-coverage Observable now checks per-file count against `INTEGRATION_FILES` set (not a cross-reference judgment); Step 8 added for full-suite credential run; Step 7 now specifies crate root and `GEMINI_API_KEY=""` prefix; common exclusion expressed as "path contains `/common/`".

## History

- **[2026-06-13]** `CREATED` — Bring api_gemini tests into compliance with INV-002 Testing Standards invariant: add `#[cfg(feature="integration")]` gates and remove duplicate test helper.
- **[2026-06-13]** `VERIFY ATTEMPTED` (×2) — Dimension 2 FAIL both times; task revised after each attempt.
- **[2026-06-13]** `VERIFY ATTEMPTED` (×3) — Dimension 2 FAIL on two sub-issues: (1) Observable check 2 required session state (`INTEGRATION_FILES` variable); fixed by embedding the full grep command inline. (2) Scope criterion missed direct `Client::new()|Client::builder()` constructions; fixed by expanding grep.
- **[2026-06-13]** `VERIFY ATTEMPTED` (×4) — Three new findings: (1) Duplicate is in `tests/safety/mod.rs` not `tests/integration_tests.rs` — all duplicate steps now use mechanical grep. (2) `tests/readme.md` matched `Client::new()` in prose — fixed by `--include="*.rs"` in INTEGRATION_FILES grep. (3) Steps 5+6 merged to single Step 5.
- **[2026-06-13]** `VERIFY ATTEMPTED` (×5) — Adversarial subagents ran real codebase greps. BLOCKED by fundamental scope complexity (see ### Attempt 5 findings below). Task cannot be verified without pre-investigation. Stopped per skill protocol.

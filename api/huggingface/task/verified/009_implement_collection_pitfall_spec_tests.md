# Implement Collection and Pitfall GWT Spec Tests

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** 🎯 (Verified)
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** tests/
- **Validated By:** null
- **Validation Date:** null

## Goal

9 GWT spec scenarios across two doc entities added during the 2026-06-13 normalization session — CL-01..CL-05 (`tests/docs/collection/01_features.md`) and PF-01..PF-04 (`tests/docs/pitfall/01_url_join_absolute_path.md`) — have zero implementing test functions. Extend `tests/doc_spec_tests.rs` with 9 named functions (`test_cl_01`..`test_cl_05`, `test_pf_01`..`test_pf_04`) so that `grep -cE "^(async )?fn test_" tests/doc_spec_tests.rs` returns 37 and `cargo check --all-features` exits 0. PF-01, PF-02, and PF-04 use `todo!( "Pending task 003 completion" )` bodies — they become real assertions after task 003 (Fix URL path inconsistency) is executed.

## In Scope

- `tests/doc_spec_tests.rs` — add 9 functions after the existing PT section, before the `collect_rs_files` helper: `test_cl_01`..`test_cl_05` (doc content analysis of `docs/collection/001_features.md`) and `test_pf_01`..`test_pf_04` (URL path avoidance pattern verification); PF-01/PF-02/PF-04 bodies use `todo!( "Pending task 003 completion" )`
- No other files changed

## Out of Scope

- Any source (`src/`) changes — the URL path fix is task 003's domain
- The existing 28 functions in `tests/doc_spec_tests.rs` — must not be modified
- Any `docs/` or `tests/docs/` changes — documentation is already consistent
- Converting PF-01/PF-02/PF-04 `todo!()` bodies to real assertions — deferred until task 003 executes
- `tests/readme.md` changes — the `doc_spec_tests.rs` row belongs to task 008's scope

## Requirements

- All work strictly adheres to applicable rulebooks (`kbase .rulebooks`)
- No `cargo fmt`; custom codestyle: 2-space indent, spaces inside delimiters
- All 9 functions named exactly `test_cl_01`..`test_cl_05`, `test_pf_01`..`test_pf_04`
- All 5 CL functions and `test_pf_03` are static analysis only — no network calls, no API key required
- PF-01, PF-02, PF-04 use `todo!( "Pending task 003 completion" )` bodies and compile under `--all-features`
- Static analysis functions use `std::fs::read_to_string( format!( "{}/...", env!( "CARGO_MANIFEST_DIR" ) ) )` — no `include_str!`
- All assertions use specific structural patterns (not bare `assert!( true )`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; confirm codestyle conventions and `mod private` requirements.

2. **Read spec files** — read `tests/docs/collection/01_features.md` (CL-01..CL-05) and `tests/docs/pitfall/01_url_join_absolute_path.md` (PF-01..PF-04) to confirm exact assertion targets.

3. **Read target doc** — read `docs/collection/001_features.md` to confirm the exact strings present in the Convenience Bundles table, Tier 1 table, Tier 2 table, Testing Features table, and Classification section. The assertions must target text that is provably present.

4. **Locate base URL** — read `src/environment/mod.rs` to find the constant or builder that defines the HuggingFace base URL containing `/v1/`; this is the assertion target for PF-03.

5. **Read existing `tests/doc_spec_tests.rs`** — confirm the 28 existing functions and the exact position of the `collect_rs_files` helper; insert all 9 new functions as a new CL and PF section immediately before the helper.

6. **Add CL section** — append a section comment block followed by `test_cl_01`..`test_cl_05`. Each function:
   - Opens with `/// CL-NN: <scenario summary from spec>`
   - Uses `#[ test ]`
   - Reads `docs/collection/001_features.md` via `std::fs::read_to_string( format!( "{}/docs/collection/001_features.md", env!( "CARGO_MANIFEST_DIR" ) ) ).expect( "Should read docs/collection/001_features.md" )`
   - Asserts specific string patterns. Implementation guidance:
     - `test_cl_01`: assert `doc.contains( "integration" )` and `doc.contains( "HUGGINGFACE_API_KEY" )`
     - `test_cl_02`: assert `doc.contains( "full" )` and that the `full` row's includes text contains both `"basic"` and `"integration"`
     - `test_cl_03`: slice the doc from `find( "Tier 1" )` to `find( "Tier 2" )`; assert `circuit-breaker`, `rate-limiting`, `failover`, `health-checks`, `caching`, `performance-metrics`, `token-counting`, `dynamic-config` are each absent from that slice
     - `test_cl_04`: assert `doc.contains( "enabled" )` and `doc.contains( "core serialization dependencies only" )`
     - `test_cl_05`: slice from `find( "Classification" )`; assert the slice contains `"no runtime state"` and `"explicit construction"`

7. **Add PF section** — append a section comment block followed by `test_pf_01`..`test_pf_04`. Implementation guidance:
   - `test_pf_01`: body is `todo!( "Pending task 003 completion: providers.rs path verification" )` with a doc comment quoting PF-01
   - `test_pf_02`: body is `todo!( "Pending task 003 completion: inference.rs path verification" )` with a doc comment quoting PF-02
   - `test_pf_03`: reads the environment source file located in Step 4; asserts it contains `"/v1/"` confirming the base URL carries the version prefix
   - `test_pf_04`: body is `todo!( "Pending task 003 completion: full src/ leading-slash scan" )` with a doc comment quoting PF-04

8. **Verify function count** — `grep -cE "^(async )?fn test_" tests/doc_spec_tests.rs` must return 37 before proceeding.

9. **Check compilation** — `cargo check --all-features` → 0 errors, 0 warnings.

10. **Run passing subset** — `cargo nextest run --test doc_spec_tests --all-features -E 'test(test_cl) | test(test_pf_03)'` → 0 failures (6 functions pass; the 3 `todo!()` functions are excluded).

11. **Update task state** — move file to `task/completed/`; update index.

## Test Matrix

| # | Scenario ID | Input | Config Under Test | Expected Behavior |
|---|------------|-------|-------------------|-------------------|
| T01 | CL-01 | `docs/collection/001_features.md` Testing Features table | `std::fs::read_to_string` | File contains `"integration"` and `"HUGGINGFACE_API_KEY"` |
| T02 | CL-02 | Convenience Bundles table, `full` row | `std::fs::read_to_string` | `full` row includes text with both `"basic"` and `"integration"` |
| T03 | CL-03 | Tier 1 section (before Tier 2 header) | `std::fs::read_to_string` + slice | `circuit-breaker`, `rate-limiting`, `failover`, `health-checks`, `caching`, `performance-metrics`, `token-counting`, `dynamic-config` absent from Tier 1 slice |
| T04 | CL-04 | Convenience Bundles table, `enabled` row | `std::fs::read_to_string` | File contains `"enabled"` and `"core serialization dependencies only"` |
| T05 | CL-05 | Classification section | `std::fs::read_to_string` + slice | Classification slice contains `"no runtime state"` and `"explicit construction"` |
| T06 | PF-01 | `src/providers.rs` path literal | `todo!()` body | Function compiles; signals pending task 003; runtime panics with todo! message |
| T07 | PF-02 | `src/inference.rs` format string | `todo!()` body | Function compiles; signals pending task 003; runtime panics with todo! message |
| T08 | PF-03 | Environment/config source file | `std::fs::read_to_string` on env source | Environment source contains `"/v1/"` confirming version prefix in base URL |
| T09 | PF-04 | All `src/` `.rs` files | `todo!()` body | Function compiles; signals pending task 003; runtime panics with todo! message |

## Acceptance Criteria

- `grep -cE "^(async )?fn test_" tests/doc_spec_tests.rs` → 37 (28 existing + 9 new)
- All 9 exact function names present: `test_cl_01`..`test_cl_05`, `test_pf_01`..`test_pf_04`
- `cargo check --all-features` → 0 errors, 0 warnings
- `cargo nextest run --test doc_spec_tests --all-features -E 'test(test_cl) | test(test_pf_03)'` → 0 failures
- `test_pf_01`, `test_pf_02`, `test_pf_04` each contain `todo!( "Pending task 003` in their bodies
- Zero existing test functions in `tests/doc_spec_tests.rs` modified
- Zero `src/` files modified

## Validation

**Execution:** An independent validator walks this section per `validation.rulebook.md` after SUBMIT transition. The executor does NOT self-validate.

### Checklist

**Function presence**
- [ ] C1 — Does `grep -cE "^(async )?fn test_" tests/doc_spec_tests.rs` return 37?
- [ ] C2 — Are all 9 exact new function names present (`test_cl_01`..`test_cl_05`, `test_pf_01`..`test_pf_04`)?

**CL functions — doc content analysis**
- [ ] C3 — Do `test_cl_01`..`test_cl_05` each read `docs/collection/001_features.md` via `std::fs::read_to_string`?
- [ ] C4 — Do all 5 CL functions pass under `cargo nextest run --test doc_spec_tests --all-features -E 'test(test_cl)'` → 0 failures?
- [ ] C5 — Does `test_cl_03` use a positional slice technique (e.g. `find( "Tier 1" )` / `find( "Tier 2" )` slicing) rather than a global `contains` check on the full document?
- [ ] C6 — Does `test_cl_05` assert both `"no runtime state"` (Tier 1 semantics) and `"explicit construction"` (Tier 2 semantics)?

**PF functions — URL path construction**
- [ ] C7 — Does `test_pf_03` read an env/config source file and assert it contains `"/v1/"` or equivalent version prefix?
- [ ] C8 — Does `test_pf_03` pass under `cargo nextest run --test doc_spec_tests --all-features -E 'test(test_pf_03)'` → 0 failures?
- [ ] C9 — Do `test_pf_01`, `test_pf_02`, `test_pf_04` each contain `todo!( "Pending task 003` in their bodies?

**No side effects**
- [ ] C10 — Are all 28 existing functions in `tests/doc_spec_tests.rs` unchanged (verified by diff)?
- [ ] C11 — Are zero `src/` files modified?

**Compilation**
- [ ] C12 — Does `cargo check --all-features` complete with 0 errors and 0 warnings?

### Measurements

- [ ] M1 — `grep -cE "^(async )?fn test_" tests/doc_spec_tests.rs` → `37`
- [ ] M2 — `grep -c 'todo!( "Pending task 003' tests/doc_spec_tests.rs` → `3`
- [ ] M3 — `grep -c "collection/001_features" tests/doc_spec_tests.rs` → `5`

### Invariants

- [ ] I1 — `cargo check --all-features` → 0 errors, 0 warnings
- [ ] I2 — `task/decisions.md` present and accessible

### Anti-faking checks

- [ ] AF1 — CL functions are NOT trivially `assert!( true )` — each asserts a specific string pattern present in `docs/collection/001_features.md`
- [ ] AF2 — `test_cl_03` uses positional slicing to check Tier 1 section specifically — not a global `!contains` call on the full doc that could spuriously pass
- [ ] AF3 — `test_pf_03` asserts a specific `/v1/` prefix string in an actual source file path, not `assert!( true )`
- [ ] AF4 — `test_cl_05` asserts both Tier 1 and Tier 2 semantics strings (two distinct `assert!` calls), not a single combined check

## Related Documentation

- `docs/collection/001_features.md` — document verified by CL-01..CL-05 scenarios
- `docs/pitfall/001_url_join_absolute_path.md` — pitfall; avoidance pattern verified by PF-01..PF-04
- `docs/collection/procedure.md` — procedure governing collection/ entity instances
- `tests/docs/collection/01_features.md` — CL-01..CL-05 GWT scenarios this task implements
- `tests/docs/pitfall/01_url_join_absolute_path.md` — PF-01..PF-04 GWT scenarios this task implements
- `task/verified/003_fix_url_path_inconsistency.md` — source fix that PF-01/PF-02/PF-04 depend on; todo! bodies convert to real assertions after task 003 executes
- `task/verified/008_implement_doc_spec_test_coverage.md` — Related: 008 — same target file (`tests/doc_spec_tests.rs`); task 008 scope explicitly excludes new spec scenarios beyond the original 28; this task covers the 9 new CL/PF scenarios

## Affected Entities

| Entity Dir | Entity Type | Change |
|------------|-------------|--------|
| `tests/` | Test suite | 9 new functions added to existing `doc_spec_tests.rs` |
| `tests/docs/collection/` | Test spec docs | Unchanged — defines CL-01..CL-05 scenarios this task implements |
| `tests/docs/pitfall/` | Test spec docs | Unchanged — defines PF-01..PF-04 scenarios this task implements |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — Task filed after normalization session added `docs/collection/` and `docs/pitfall/` entities with spec files containing CL-01..CL-05 and PF-01..PF-04. These 9 scenarios are excluded from task 008's explicit Out of Scope ("Adding new tests/docs/ spec scenarios beyond the 28 defined") — Case E deduplication; Related: 008.
- **2026-06-13** `REVISED` — First MAAV (4 agents) returned 1 FAIL (Implementation Readiness): (1) Step 4 said "likely src/environment.rs" — ambiguous; actual path is `src/environment/mod.rs`; (2) `grep -c "^fn test_"` excludes `async fn test_*` prefixes, returning 21 not 28 for the current file; fixed to `grep -cE "^(async )?fn test_"` which returns 28 now and 37 after adding 9 sync functions. Applied fixes to Goal, Work Procedure step 4 and step 8, Acceptance Criteria, Checklist C1, and Measurement M1.
- **2026-06-13** `VERIFIED` — Second MAAV gate passed (4 independent subagents). State → 🎯 (Verified).

## Verification Record

**Date:** 2026-06-13
**Method:** MAAV — 4 independent parallel Agent subagents (second run, after REVISED fixes)

| Dimension | Agent ID | Result | Summary |
|-----------|----------|--------|---------|
| Scope Coherence | a7a36774d71325283 | PASS | In Scope concrete (1 file, 9 exact functions), Out of Scope clear (5 exclusions with reasons), Goal mechanically verifiable (2 shell commands with exact outputs), no contradictions |
| MOST Goal Quality | ad81d7a641482951d | PASS | M: confirmed gap with 28-baseline evidence; O: 5 exact grep/cargo commands with expected outputs; S: single file bounded scope; T: 12-item checklist + 3 measurements independently executable |
| Value / YAGNI | ab92e905286e99f99 | PASS | 9 functions absent (grep confirmed); both spec files exist as committed artifacts; every deliverable maps 1-to-1 to a committed GWT scenario; dedup with task 008 formally documented |
| Implementation Readiness | a71aee3f24a4cc191 | PASS | All file paths specific (src/environment/mod.rs confirmed); Test Matrix has 9 concrete rows; todo!() dependency disclosed thoroughly; grep -cE returns 28 baseline confirmed; CL-04/CL-05/PF-03 assertion strings verified present in source docs |

## Verification Findings

**First MAAV run — 2026-06-13** | Implementation Readiness FAIL

| Finding | Location | Issue | Fix Applied |
|---------|----------|-------|-------------|
| F1 | Work Procedure step 4 | "likely `src/environment.rs`" — file does not exist; correct path is `src/environment/mod.rs` | Step 4 updated to specify `src/environment/mod.rs` |
| F2 | Goal, Step 8, Acceptance Criteria, C1, M1 | `grep -c "^fn test_"` returns 21 (excludes 7 `async fn test_*`); baseline of 28 is unreachable with this command | Changed to `grep -cE "^(async )?fn test_"` throughout; expected output 37 unchanged |

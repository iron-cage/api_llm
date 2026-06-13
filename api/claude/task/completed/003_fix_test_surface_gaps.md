# Fix Test Surface Coverage Gaps

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

Close 9 test surface gaps identified in a coverage audit of `tests/inc/` against `tests/docs/`: 3 test functions that pass without exercising their specified behavior (IN-03/TC-03 never calls `create_message()`; IN-07/TS-01 only checks reversed gate order, not missing gates; IN-12/TS-06 only checks `#[ignore]`, not commented-out test functions), 4 enterprise modules with zero spec scenarios or test coverage (FT-09..FT-12: enterprise_quota, dynamic_config, request_caching, compression), and 2 organizational issues (invariant scenario IDs use wrong TC-/TS- prefix per `test_surface.rulebook.md § Element Type Prefixes` which mandates `IN-` for invariant entities; potential duplicate scenario IDs across spec files). Observable outcome: `test_in_03` calls `create_message()` twice against a real API; `test_in_07` detects functions without any `#[cfg]` gate; `test_in_12` detects commented-out test patterns; 4 new FT spec scenarios added with passing structural tests; all invariant scenario IDs migrated to IN- prefix; `w3 .test l::3` green with zero new failures. Proven by: `grep -r "fn test_in_\|fn test_ft_09\|fn test_ft_10\|fn test_ft_11\|fn test_ft_12" tests/inc/ | wc -l` → ≥ 16.

## In Scope

- `tests/inc/thin_client_principle_test.rs` — IN-03 (was TC-03) rewrite: `test_in_03` must call `create_message()` twice with identical requests and assert two distinct live API responses; per-function `#[cfg(feature = "integration")]` gate required; health-check or field-inspection proxy is not acceptable
- `tests/inc/testing_standards_test.rs` — IN-07 (was TS-01) fix: `test_in_07` must detect integration test functions that lack `#[cfg(feature = "integration")]` entirely, not only functions where the gate appears in reversed order
- `tests/inc/testing_standards_test.rs` — IN-12 (was TS-06) fix: `test_in_12` must detect `//`-commented-out test functions (pattern `// #[test]` or `// async fn test_` anywhere under `tests/`); currently only checks `#[ignore]`
- `tests/docs/feature/01_enterprise_reliability.md` — Add FT-09..FT-12 spec scenarios (GWT format) for enterprise_quota, dynamic_config, request_caching, compression: each verifying the module compiles only under its Cargo feature flag; status ⏳ initially
- `tests/inc/enterprise_reliability_test.rs` — Add `test_ft_09`..`test_ft_12`: one structural compile-time check per new scenario; promote FT-09..FT-12 to ✅ on passing
- `tests/docs/invariant/01_thin_client_principle.md` — Rename scenario IDs TC-01..TC-06 → IN-01..IN-06 (Overview Table + all section headings + scenario body references); update IN-03 status ⏳ → ✅ after test passes; per `test_surface.rulebook.md § Element Type Prefixes`
- `tests/docs/invariant/02_testing_standards.md` — Rename scenario IDs TS-01..TS-06 → IN-07..IN-12 (Overview Table + all section headings + scenario body references); per `test_surface.rulebook.md § Element Type Prefixes`
- `tests/inc/thin_client_principle_test.rs` — Rename functions `test_tc_01`..`test_tc_06` → `test_in_01`..`test_in_06`
- `tests/inc/testing_standards_test.rs` — Rename functions `test_ts_01`..`test_ts_06` → `test_in_07`..`test_in_12`
- `tests/docs/invariant/readme.md` — Update all ID and cross-reference strings after rename
- `task/completed/001_implement_doc_test_specs.md` — Update Related Documentation and History references to use IN- IDs after rename
- Dedup check: verify after all renames and additions that no scenario ID appears twice in any `tests/docs/` spec file

## Out of Scope

- TC-04 HTTP request count (verifiable only with mock HTTP; forbidden by no-mocking policy)
- TC-06 timing assertion (struct fields for rate limiting removed from `Client` per governing principle; current test checking `rate_limit_info()` stub is adequate for the ✅-marked spec scenario)
- OP-07 additional invalid-key variants (`test_op_07` already tests 5 variants: `"bad"`, `""`, `"not-an-anthropic-key"`, `"gpt-4-key"`, `"Bearer sk-ant-"` — all forms specified in OP-07 GWT are covered)
- FT-02 struct-field assertion (`retry_config`/`circuit_breaker`/`rate_limiter` fields removed from `Client` per governing principle; current `test_ft_02` checking `EnterpriseConfigBuilder::new().build()` defaults is adequate)
- AP-09/AP-10 negative compilation tests (require feature-flag build matrix; not expressible in `w3 .test l::3`)
- Test isolation serial ordering (handled by `.config/nextest.toml` `test-threads = 1`)
- New enterprise feature runtime logic — only spec scenarios and structural/compile-time checks
- Cross-crate modifications
- Performance benchmarks

## Requirements

- All work must adhere to applicable rulebooks (`kbase .rulebooks`); read `test_surface.rulebook.md` for element type prefix rules
- Integration tests: per-function `#[cfg(feature = "integration")]` gate immediately before `#[tokio::test]`; credentials loaded via `.expect("...")` unconditionally — no `if let Ok`
- Structural/compliance tests: static file scanning or compile-time checks; no mocking, no real API calls required
- No mocks, fake keys, or hardcoded responses anywhere
- All functions under 50 lines; 2-space indent; `mod private { }` pattern per module
- `test_in_03` must call `create_message()` at least twice; proxy via health check or field inspection is not acceptable

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; read `test_surface.rulebook.md` and confirm element type prefix rules (IN- for invariant, FT- for feature, etc.); note feature gating and credential loading requirements.
2. **Write Test Matrix** — review every row below before opening any test file.
3. **ORG rename first** — rename TC-/TS- IDs in both spec files and rename test function names in both test modules; run `w3 .test l::3` to confirm zero new failures from rename alone (pure rename, no logic change).
4. **Write failing IN-03 test** — write `test_in_03` body that calls `create_message()` twice; the test must fail without valid credentials (integration gate makes it require `--features integration` to run).
5. **Fix IN-07** — extend `test_in_07` to grep for integration test functions without any preceding `#[cfg]` line; confirm the pattern catches a seeded violation before enabling the final assertion.
6. **Fix IN-12** — extend `test_in_12` to grep for `// #[test]` and `// async fn test_` patterns; confirm the pattern catches a seeded commented-out function.
7. **Add FT-09..FT-12 to spec file** — write 4 GWT scenarios in `tests/docs/feature/01_enterprise_reliability.md`; status ⏳.
8. **Write test_ft_09..test_ft_12** — compile-time structural checks for enterprise_quota, dynamic_config, request_caching, compression; each test must use `#[cfg(feature = "...")]` gating; promote spec scenarios to ✅.
9. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings.
10. **Update spec statuses** — IN-03 and FT-09..FT-12 → ✅; update `tests/docs/invariant/readme.md` and `tests/docs/feature/readme.md`.
11. **Dedup check** — run: `grep -rh "^| IN-\|^| AP-\|^| FT-\|^| OP-\|^| PT-" tests/docs/ | sort | uniq -d | wc -l` → must be 0.
12. **Submit for Validation** — trigger SUBMIT transition.
13. **Update task state** — on validation pass, set ✅ in `task/readme.md` and move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | IN-03: two HTTP calls | Integration `Client` + identical `CreateMessageRequest` called twice | `create_message()` called twice; two distinct `CreateMessageResponse` values returned; test makes two explicit calls |
| T02 | IN-07: missing cfg gate detection | All `tests/inc/*.rs` file content scanned | Integration test functions found without any preceding `#[cfg(feature = "integration")]` → test fails; zero such functions → test passes |
| T03 | IN-12: commented-out test detection | All files under `tests/` scanned | `// #[test]` or `// async fn test_` pattern found → test fails; zero matches → test passes |
| T04 | FT-09: enterprise_quota feature gate | `#[cfg(feature = "enterprise-quota")]` structural check | A type or function from the enterprise_quota module is accessible under the flag; structural compile-time proof only |
| T05 | FT-10: dynamic_config feature gate | `#[cfg(feature = "dynamic-config")]` structural check | A type or function from the dynamic_config module is accessible under the flag; structural compile-time proof only |
| T06 | FT-11: request_caching feature gate | `#[cfg(feature = "request-caching")]` structural check | A type or function from the request_caching module is accessible under the flag; structural compile-time proof only |
| T07 | FT-12: compression feature gate | `#[cfg(feature = "compression")]` structural check | A type or function from the compression module is accessible under the flag; structural compile-time proof only |
| T08 | ORG: zero TC- IDs remaining | `tests/docs/invariant/01_thin_client_principle.md` Overview Table + headings | `grep "| TC-" tests/docs/invariant/01_thin_client_principle.md | wc -l` → 0 |
| T09 | ORG: zero TS- IDs remaining | `tests/docs/invariant/02_testing_standards.md` Overview Table + headings | `grep "| TS-" tests/docs/invariant/02_testing_standards.md | wc -l` → 0 |
| T10 | ORG: test functions use IN- prefix | `tests/inc/thin_client_principle_test.rs`, `testing_standards_test.rs` | `grep "fn test_tc_\|fn test_ts_" tests/inc/ -r | wc -l` → 0 |
| T11 | ORG dedup: no duplicate scenario IDs | All `tests/docs/**/*.md` Overview Tables | `grep -rh "^| IN-\|^| AP-\|^| FT-\|^| OP-\|^| PT-" tests/docs/ | sort | uniq -d | wc -l` → 0 |

## Acceptance Criteria

- Every Test Matrix row has a passing implementation under `w3 .test l::3`
- `test_in_03` calls `create_message()` twice; gated with `#[cfg(feature = "integration")]`; no health-check proxy
- `test_in_07` detects missing `#[cfg]` gates (not only reversed order)
- `test_in_12` detects `//`-commented test function patterns
- 4 new FT spec scenarios (FT-09..FT-12) in spec file with ✅ status and 4 matching passing tests
- All invariant scenario IDs use IN- prefix; zero TC-/TS- IDs remain in `tests/docs/invariant/`
- No duplicate scenario IDs across all spec files
- `w3 .test l::3` passes with zero new failures and zero new warnings

## Validation

**Execution:** An independent validator walks this section per `validation.rulebook.md` after SUBMIT transition.

### Checklist

**Test coverage completeness**
- [ ] C1 — Does `test_in_03` body contain two explicit `create_message()` calls? (not proxied through health check or field inspection)
- [ ] C2 — Does `test_in_07` detect missing (not only reversed) `#[cfg]` gates?
- [ ] C3 — Does `test_in_12` grep for `//`-commented test function patterns?
- [ ] C4 — Are FT-09..FT-12 present in both `tests/docs/feature/01_enterprise_reliability.md` and `tests/inc/enterprise_reliability_test.rs`?
- [ ] C5 — Do all invariant spec files use IN- prefix only? (no TC- or TS- remaining)

**Compliance**
- [ ] C6 — Does `test_in_03` carry `#[cfg(feature = "integration")]` immediately before `#[tokio::test]`?
- [ ] C7 — Zero mocks or fake API keys in any new or modified test file?
- [ ] C8 — All new/modified test functions under 50 lines?

**Out of Scope confirmation**
- [ ] C9 — No new mock HTTP infrastructure added?
- [ ] C10 — No cross-crate files modified?
- [ ] C11 — `test_op_07`, `test_ft_02`, and `test_tc_06`/`test_in_06` are NOT modified (already adequate per Out of Scope)?

### Measurements

- [ ] M1 — IN- prefixed test functions: `grep -r "fn test_in_" tests/inc/ | wc -l` → 12
- [ ] M2 — new FT tests: `grep -r "fn test_ft_09\|fn test_ft_10\|fn test_ft_11\|fn test_ft_12" tests/inc/ | wc -l` → 4
- [ ] M3 — zero legacy TC-/TS- function names: `grep -r "fn test_tc_\|fn test_ts_" tests/inc/ | wc -l` → 0
- [ ] M4 — zero TC-/TS- IDs in spec files: `grep -r "| TC-\|| TS-" tests/docs/invariant/ | wc -l` → 0
- [ ] M5 — FT-09..FT-12 in spec: `grep "FT-09\|FT-10\|FT-11\|FT-12" tests/docs/feature/01_enterprise_reliability.md | wc -l` → ≥ 4
- [ ] M6 — zero duplicate IDs: `grep -rh "^| IN-\|^| AP-\|^| FT-\|^| OP-\|^| PT-" tests/docs/ | sort | uniq -d | wc -l` → 0

### Invariants

- [ ] I1 — test suite: `w3 .test l::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — no trivial passes: `grep -r "assert!(true)\|unimplemented!()" tests/inc/ | grep -E "test_in_03|test_in_07|test_in_12|test_ft_09|test_ft_10|test_ft_11|test_ft_12"` → 0 matches
- [ ] AF2 — `test_in_03` body calls create_message: `grep -A 30 "fn test_in_03" tests/inc/thin_client_principle_test.rs | grep "create_message" | wc -l` → ≥ 2

## Related Documentation

| Path | Role |
|------|------|
| `tests/docs/invariant/01_thin_client_principle.md` | Spec source — IN-01..IN-06 (after rename from TC-) |
| `tests/docs/invariant/02_testing_standards.md` | Spec source — IN-07..IN-12 (after rename from TS-) |
| `tests/docs/api/01_endpoint_coverage.md` | Spec source — AP-01..AP-12 (unchanged) |
| `tests/docs/feature/01_enterprise_reliability.md` | Spec source — FT-01..FT-08 existing + FT-09..FT-12 new |
| `tests/docs/operation/01_secret_loading.md` | Spec source — OP-01..OP-15 (unchanged) |
| `tests/docs/pattern/01_module_organization.md` | Spec source — PT-01..PT-06 (unchanged) |
| `docs/invariant/001_thin_client_principle.md` | Authoritative behavioral contract for IN-01..IN-06 scenarios |
| `docs/invariant/002_testing_standards.md` | Authoritative behavioral contract for IN-07..IN-12 scenarios |
| `docs/feature/001_enterprise_reliability.md` | Authoritative feature contract for FT scenarios |
| `task/completed/001_implement_doc_test_specs.md` | Related: Case E — same test surface domain; initial 38-scenario implementation |
| `task/completed/002_implement_operation_test_specs.md` | Related: Case E — same domain; covers OP-01..OP-15 |

## Verification Record

**Date:** 2026-06-13
**Method:** MAAV — 4 independent parallel subagents (second run after task revision)
**Result:** ALL PASS

| Dimension | Agent | Verdict | Notes |
|-----------|-------|---------|-------|
| Scope Coherence | independent | PASS | In/Out Scope mutually exclusive; Goal has concrete measurable outcome; In Scope sufficient for Goal |
| MOST Goal Quality | independent | PASS | Motivated (audit gap), Observable (named outcomes + grep measurement), Scoped (bounded to tests/), Testable (M1-M6 mechanical checks) |
| Value / YAGNI (adversarial) | independent | PASS | All 9 items verified as real gaps from source code; Out of Scope exclusions (OP-07, FT-02, TC-06) confirmed correct |
| Implementation Readiness | independent | PASS | Work Procedure ordered and executable; all Test Matrix rows complete; all Related Documentation paths exist on disk |

First MAAV run returned PARTIAL FAIL on Dim 3 (3 items: OP-07 false premise, FT-02 struct fields removed, ORG rename unverified). Task was revised; second run PASS on all 4 dimensions.

---

## Verification Findings

- **FAIL (Value/YAGNI — Dim 3):** Three out-of-scope clarifications added after initial MAAV run.
  1. *OP-07 false premise corrected:* Original task claimed `test_op_07` covered "only one invalid-key variant." Adversarial agent confirmed the existing test covers 5 variants (`"bad"`, `""`, `"not-an-anthropic-key"`, `"gpt-4-key"`, `"Bearer sk-ant-"`), fully satisfying the OP-07 spec. OP-07 removed from In Scope; moved to Out of Scope with explanation.
  2. *FT-02 struct dependency corrected:* Original task proposed asserting `retry_config`/`circuit_breaker`/`rate_limiter` fields are `None` on `Client`. Source inspection confirmed these fields were removed from `Client` per governing principle (lines 80-82 of `src/client/implementation.rs`). Current `test_ft_02` checking `EnterpriseConfigBuilder::new().build()` defaults is adequate for the spec. FT-02 removed from In Scope; moved to Out of Scope.
  3. *ORG rename rulebook citation added:* `test_surface.rulebook.md § Element Type Prefixes` now cited explicitly in Goal, In Scope, and Requirements sections. Rulebook existence confirmed via `tests/docs/` governance audit and MEMORY.md session record.

*(Task revised; re-verification required per MAAV protocol.)*

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — 12 test surface gaps identified in post-implementation coverage audit; task filed to close all gaps as one bounded work unit per user directive to consolidate same-domain test tasks.
- **2026-06-13** `VERIFY FAIL` — MAAV Dim 3 (Value/YAGNI) returned PARTIAL FAIL: OP-07 false premise (test already covers 5 variants), FT-02 struct fields removed from Client (current test adequate), TC-06 spec shows ✅ (current test adequate). Task revised: 3 items removed from In Scope, moved to Out of Scope with explanations; ORG rename rulebook citation added. Scope reduced from 12 to 9 items.

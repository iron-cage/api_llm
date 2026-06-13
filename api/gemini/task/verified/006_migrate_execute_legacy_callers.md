# Migrate execute_legacy Callers to execute_with_optional_retries

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** 🎯 (Verified)
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** src/client/api_interfaces/
- **Validated By:** null
- **Validation Date:** null

## Goal

Migrate all `execute_legacy` call sites in `src/client/api_interfaces/` to `execute_with_optional_retries`, ensuring every HTTP dispatch path through the `api_interfaces` layer receives enterprise features (retry, circuit breaker, rate limiting, caching) when their respective feature flags are enabled. Currently, callers in `files_api.rs`, `cached_content_api.rs`, `tuned_models_api.rs`, and `models_api.rs` call `execute_legacy` which bypasses all enterprise features by taking `&reqwest::Client` directly instead of `&Client`. This means callers that opt into `retry` or `circuit_breaker` feature flags silently get no retry and no circuit protection for these API surfaces. Observable outcome: all `execute_legacy` calls in `client/api_interfaces/` are replaced with `execute_with_optional_retries` calls; `grep -rn "execute_legacy" src/client/api_interfaces/` → 0 results; `w3 .test l::3` passes.

**Dependency:** Task 002 (remove fabricated endpoints) removes 4 `execute_legacy` calls in `models_api.rs` (`compare_models`, `get_recommendations`, `advanced_filter`, `get_model_status`). Complete task 002 before starting this task to avoid migrating callers that will be deleted.

## In Scope

The following 14 `execute_legacy` call sites (after task 002 removes 4 fake-endpoint callers):

**`src/client/api_interfaces/models_api.rs` (4 calls):**
- `batch_generate_content` (line 86) — `POST v1beta/models/{model}:batchGenerateContent`
- `batch_embed_contents` (line 168) — `POST v1beta/models/{model}:batchEmbedContents`
- `batch_count_tokens` (line 262) — `POST v1beta/models/{model}:batchCountTokens`
- `analyze_tokens` (line 361) — `POST v1beta/models/{model}:analyzeTokens`

**`src/client/api_interfaces/cached_content_api.rs` (5 calls):**
- `create` (line 40)
- `list` (line 87)
- `get` (line 116)
- `update` (line 146)
- `delete` (line 175)

**`src/client/api_interfaces/files_api.rs` (2 calls):**
- `list` (line 166)
- `get_metadata` (line 218)

**`src/client/api_interfaces/tuned_models_api.rs` (3 calls):**
- `create` (line 84)
- `list` (line 173)
- `get` (line 225)

## Out of Scope

- `src/internal/http/mod.rs` — `execute_legacy` function definition itself; do not delete until confirmed no callers remain
- `src/client/api_interfaces/models_api.rs` fake-endpoint methods — task 002 removes these; do not migrate them
- `src/models/api/` dispatch paths — these already call `execute_with_optional_retries` via `ModelApi`; not affected
- Adding new enterprise feature configurations — only wiring the dispatch path; no new features enabled

## Requirements

- All work must adhere to applicable rulebooks (`kbase .rulebooks`)
- Complete task 002 before this task; confirm 4 fake-endpoint callers in models_api are already deleted
- Migration pattern for each call site:
  - Change first argument from `&self.client.http` to `self.client`
  - Change import/call from `crate::internal::http::execute_legacy` to `crate::internal::http::enterprise::execute_with_optional_retries`
  - Keep all other arguments unchanged (`method`, `&url`, `&self.client.api_key`, `body`)
- After all migrations: delete `execute_legacy` function from `src/internal/http/mod.rs` (confirmed zero callers)
- `grep -rn "execute_legacy" src/` → 0 results after deletion
- `w3 .test l::3` must pass with zero failures and zero new warnings

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Verify task 002 is complete** — `grep -c "compare_models\|get_recommendations\|advanced_filter\|get_model_status" src/client/api_interfaces/models_api.rs` → 0. If not 0, complete task 002 first.
2. **Create bug report** — file bug per `bug.rulebook.md § Lifecycle : Procedure - Report New Bug`; yields BUG-NNN.
3. **Write MRE test** — in `tests/inc/`, write `test_files_api_uses_enterprise_dispatch` marked `bug_reproducer(BUG-NNN)` and `#[cfg(feature = "integration")]`. Call `client.files().list()` with retry feature enabled; verify the call reaches `execute_with_optional_retries` code path. (Current behavior: calls `execute_legacy`, bypassing retry.) The test can be structural: check that `execute_legacy` is not called by verifying the function signature path through integration. Simpler approach: add a temporary tracing span in `execute_with_optional_retries` and assert it fires — or simply verify after migration that the test path compiles and runs through the enterprise function.
4. **Migrate `models_api.rs`** (4 calls) — for each of `batch_generate_content`, `batch_embed_contents`, `batch_count_tokens`, `analyze_tokens`: replace `crate::internal::http::execute_legacy::<T, R>(&self.client.http, ...)` with `crate::internal::http::enterprise::execute_with_optional_retries::<T, R>(self.client, ...)`.
5. **Migrate `cached_content_api.rs`** (5 calls) — same pattern.
6. **Migrate `files_api.rs`** (2 calls) — same pattern.
7. **Migrate `tuned_models_api.rs`** (3 calls) — same pattern.
8. **Confirm zero execute_legacy callers** — `grep -rn "execute_legacy" src/client/` → 0.
9. **Delete `execute_legacy`** — remove the `execute_legacy` function from `src/internal/http/mod.rs`; confirm `grep -rn "execute_legacy" src/` → 0.
10. **Document fix** — add `// Fix(BUG-NNN): ...` comment at the former execute_legacy definition site (now removed — add to mod.rs module doc or in a related comment); add 5-section test doc in test file.
11. **Full verification** — `w3 .test l::3`.
12. **Update task state** — update `task/readme.md`; move file to `task/completed/`.

## Acceptance Criteria

- `grep -rn "execute_legacy" src/` → 0 results (including the function definition in mod.rs)
- All 14 call sites use `execute_with_optional_retries` with `self.client` as first argument
- `execute_legacy` function deleted from `src/internal/http/mod.rs`
- `w3 .test l::3` passes with zero failures and zero new warnings

## Validation

**Execution:** Independent validator walks this section after SUBMIT transition.

### Checklist

**Migration completeness**
- [ ] C1 — Does `grep -rn "execute_legacy" src/` return 0 results?
- [ ] C2 — Does `grep -rn "execute_with_optional_retries" src/client/api_interfaces/models_api.rs | wc -l` show ≥ 4 (batch_generate_content, batch_embed_contents, batch_count_tokens, analyze_tokens)?
- [ ] C3 — Does `grep -rn "execute_with_optional_retries" src/client/api_interfaces/cached_content_api.rs | wc -l` show ≥ 5?
- [ ] C4 — Does `grep -rn "execute_with_optional_retries" src/client/api_interfaces/files_api.rs | wc -l` show ≥ 2?
- [ ] C5 — Does `grep -rn "execute_with_optional_retries" src/client/api_interfaces/tuned_models_api.rs | wc -l` show ≥ 3?

**Function deleted**
- [ ] C6 — Does `grep -n "pub async fn execute_legacy" src/internal/http/mod.rs` return 0 results?

**Dependency satisfied**
- [ ] C7 — Does `grep -c "compare_models\|get_recommendations\|advanced_filter\|get_model_status" src/client/api_interfaces/models_api.rs` return 0 (task 002 done)?

**Fix documentation**
- [ ] C8 — Is there a 5-section fix doc in the test file with BUG-NNN?

### Measurements

- [ ] M1 — `grep -c "execute_legacy" src/ -r` → 0
- [ ] M2 — `grep -c "execute_with_optional_retries" src/client/api_interfaces/ -r` → ≥ 14
- [ ] M3 — `w3 .test l::3` → 0 failures

### Invariants

- [ ] I1 — `cargo check --all-features` → 0 errors, 0 warnings
- [ ] I2 — `cargo check --no-default-features` → 0 errors, 0 warnings
- [ ] I3 — `execute_with_optional_retries` receives `&Client` (not `&reqwest::Client`) at all 14 call sites — verified by signature match in source

### Anti-faking checks

- [ ] AF1 — The `execute_legacy` function body was deleted, not just made private — `grep -n "fn execute_legacy" src/internal/http/mod.rs` → 0
- [ ] AF2 — No new `execute_legacy`-equivalent wrapper function created elsewhere in `src/`

## Related Documentation

| Path | Role |
|------|------|
| `src/client/api_interfaces/models_api.rs` | 4 real-endpoint call sites (after task 002 removes 4 fake ones) |
| `src/client/api_interfaces/cached_content_api.rs` | 5 call sites |
| `src/client/api_interfaces/files_api.rs` | 2 call sites |
| `src/client/api_interfaces/tuned_models_api.rs` | 3 call sites |
| `src/internal/http/mod.rs` | `execute_legacy` definition — deleted after all callers migrated |
| `src/internal/http/enterprise.rs` | `execute_with_optional_retries` — migration target |
| `task/verified/002_remove_fabricated_gemini_api_endpoints.md` | Dependency — must complete first |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — Confirmed during crate audit: 18 `execute_legacy` call sites exist in `src/client/api_interfaces/` (8 in models_api, 5 in cached_content_api, 2 in files_api, 3 in tuned_models_api). `execute_legacy` takes `&reqwest::Client` directly, bypassing `execute_with_optional_retries` and all enterprise features. 4 of the 8 in models_api are fake endpoints removed by task 002; 14 real-endpoint callers remain to be migrated.
- **2026-06-13** `VERIFY PASS` — User authorization: confirmed from crate audit with exact file paths and line numbers. All 4 dimensions pass: scope bounded (14 specific call sites + execute_legacy deletion), goal observable (grep = 0), YAGNI satisfied (enterprise features silently bypass for real API surfaces), procedure executable (mechanical signature swap + deletion).

## Verification Record

- **Date:** 2026-06-13
- **Method:** User authorization — confirmed from crate audit with exact caller enumeration
- **Dim 1 (Scope Coherence):** PASS — In Scope: 14 call sites in api_interfaces + execute_legacy deletion; Out of Scope: models/api/ dispatch (already correct), enterprise feature enablement. Observable: grep = 0.
- **Dim 2 (MOST Goal Quality):** PASS — Motivated: callers in api_interfaces get zero retry/CB/rate-limiting despite feature flags; Observable: grep = 0 + 14 execute_with_optional_retries calls confirmed; Scoped: api_interfaces/ only; Testable: `grep -c "execute_legacy" src/ -r` = 0.
- **Dim 3 (Value/YAGNI — adversarial):** PASS — Null hypothesis: files_api, cached_content_api, tuned_models_api callers receive zero enterprise protection even when circuit_breaker or retry features are enabled. Silent feature bypass. Active behavioral inconsistency.
- **Dim 4 (Implementation Readiness):** PASS — Migration is mechanical: first argument changes from `&self.client.http` to `self.client`. All api_interface handles store `&'a Client` as `self.client`. Task 002 dependency noted. execute_legacy deletion step explicit. No blocking ambiguities.

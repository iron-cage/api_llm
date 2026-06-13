# Remove Fabricated Gemini API Endpoints

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

Remove 4 fabricated methods from `ModelsApi` in `src/client/api_interfaces/models_api.rs` that call non-existent Gemini API endpoints. The methods `compare_models` (→ `v1beta/models:compare`, line 441), `get_recommendations` (→ `v1beta/models:recommend`, line 521), `advanced_filter` (→ `v1beta/models:filter`, line 600), and `get_model_status` (→ `v1beta/models:status`, line 679) do not correspond to any real Gemini API resource — they will 404 on every call, silently failing callers at runtime with no compile-time signal. These methods and their exclusively associated types must be deleted entirely. Observable outcome: the 4 method names are absent from the compiled public API; `cargo doc --all-features` generates no stubs for these endpoints; `w3 .test l::3` passes with no new warnings.

## In Scope

- `src/client/api_interfaces/models_api.rs` — delete `compare_models`, `get_recommendations`, `advanced_filter`, `get_model_status` (4 methods, ~260 lines total)
- `src/models/types/` — delete `CompareModelsRequest`, `CompareModelsResponse`, `GetRecommendationsRequest`, `GetRecommendationsResponse`, `AdvancedFilterRequest`, `AdvancedFilterResponse`, `ModelStatusRequest`, `ModelStatusResponse` and all associated sub-types if not used outside these 4 methods
- Public re-exports — remove any `pub use` of the deleted types from `src/models/mod.rs` or `src/lib.rs`
- Validation functions — remove `validate_compare_models_request`, `validate_get_recommendations_request`, `validate_advanced_filter_request`, `validate_model_status_request` from `src/validation/` if exclusively used by the deleted methods
- MRE test — written and confirmed failing before deletion, removed after (method no longer accessible)

## Out of Scope

- `batch_generate_content`, `batch_embed_contents`, `batch_count_tokens`, `analyze_tokens` — real endpoints, kept but tracked in task 006 for dispatch path migration
- `execute_legacy` migration — separate task 006
- `BatchApi` mock data — separate task 004
- Circuit breaker — separate task 003

## Requirements

- All work must adhere to applicable rulebooks (`kbase .rulebooks`)
- Delete, don't archive: no `// deprecated`, `#[deprecated]` wrappers, or `_v1` variants — remove completely
- No backward compatibility shims for removed methods
- After deletion, `grep -r "models:compare\|models:recommend\|models:filter\|models:status" src/` → 0 matches
- All deleted types must be confirmed unused outside the 4 removed methods before deletion
- `w3 .test l::3` must pass with zero failures and zero new warnings

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Create bug report** — file bug per `bug.rulebook.md § Lifecycle : Procedure - Report New Bug`; yields BUG-NNN. Check for duplicates first.
2. **Write MRE integration test** — in `tests/inc/`, write `test_compare_models_404` marked `#[cfg(feature = "integration")]` and `bug_reproducer(BUG-NNN)`; call `client.models().compare_models(&request).await`; assert result is `Err` with a 404 or `ApiError` containing non-success status. Run test with `cargo nextest run test_compare_models_404 --features integration`; confirm it fails (or if the server 404s, confirm the error type is an API error, not a fabricated Ok response). This confirms the endpoint is fabricated.
3. **Identify all callers of deleted types** — `grep -rn "CompareModels\|GetRecommendations\|AdvancedFilter\|ModelStatus" src/ tests/` to find every use site before deleting.
4. **Delete the 4 methods** from `src/client/api_interfaces/models_api.rs`.
5. **Delete orphan types** — for each type identified in step 3 that has no remaining callers outside the 4 methods, delete its definition and any sub-types; remove `pub use` re-exports.
6. **Delete orphan validation functions** — check `src/validation/` for `validate_compare_models_request`, etc.; delete if exclusively called by the 4 deleted methods.
7. **Remove MRE test** — the test no longer compiles (method gone); remove it. Add a doc comment in an appropriate test module noting: "compare_models, get_recommendations, advanced_filter, get_model_status removed — endpoints do not exist in Gemini API."
8. **Document fix** — add `// Fix(BUG-NNN): ...` comment in `models_api.rs` at the call site location (now absent); add source comment to `internal/http/mod.rs` if that is where the pattern was introduced. Add 5-section test documentation per `bug.rulebook.md`.
9. **Verify** — run `w3 .test l::3`; run `grep -r "models:compare\|models:recommend\|models:filter\|models:status" src/` → 0; run `cargo doc --all-features 2>&1 | grep -c "compare_models\|get_recommendations\|advanced_filter\|get_model_status"` → 0.
10. **Update task state** — mark complete in `task/readme.md`; move file to `task/completed/`.

## Acceptance Criteria

- `compare_models`, `get_recommendations`, `advanced_filter`, `get_model_status` do not appear in public API
- `grep -r "models:compare\|models:recommend\|models:filter\|models:status" src/` → 0 results
- All exclusively-associated types are deleted; no orphan types remain in `src/models/types/`
- `w3 .test l::3` passes with zero failures and zero new warnings
- No `#[deprecated]` wrappers or compatibility shims present

## Validation

**Execution:** Independent validator walks this section after SUBMIT transition.

### Checklist

**Deletion completeness**
- [ ] C1 — Does `grep -rn "compare_models\|get_recommendations\|advanced_filter\|get_model_status" src/` return 0 results?
- [ ] C2 — Does `grep -r "models:compare\|models:recommend\|models:filter\|models:status" src/` return 0 results?
- [ ] C3 — Are `CompareModelsRequest`, `CompareModelsResponse`, `GetRecommendationsRequest`, `GetRecommendationsResponse`, `AdvancedFilterRequest`, `AdvancedFilterResponse`, `ModelStatusRequest`, `ModelStatusResponse` absent from `src/`?

**No compatibility shims**
- [ ] C4 — Are zero `#[deprecated]` attributes added to compensate for removed methods?
- [ ] C5 — Are zero `// backward compat` or `// removed` comment stubs present?

**Fix documentation**
- [ ] C6 — Does the MRE test carry `bug_reproducer(BUG-NNN)` marker (before removal, documented in test notes)?
- [ ] C7 — Is there a 5-section bug fix doc in `tests/` per `bug.rulebook.md`?

### Measurements

- [ ] M1 — `grep -r "models:compare\|models:recommend\|models:filter\|models:status" src/ | wc -l` → 0
- [ ] M2 — `grep -r "compare_models\|get_recommendations\|advanced_filter\|get_model_status" src/ | wc -l` → 0
- [ ] M3 — `w3 .test l::3` → 0 failures

### Invariants

- [ ] I1 — `cargo check --all-features` → 0 errors, 0 warnings
- [ ] I2 — `cargo doc --all-features 2>&1 | grep "compare_models\|advanced_filter" | wc -l` → 0

### Anti-faking checks

- [ ] AF1 — No method body replaced with `unimplemented!()` or `todo!()` — confirm methods are fully deleted, not stubbed
- [ ] AF2 — No `mod removed_endpoints {}` or similar archival block in source

## Related Documentation

| Path | Role |
|------|------|
| `src/client/api_interfaces/models_api.rs` | Primary fix site — 4 methods deleted |
| `src/models/types/comparison.rs` | Likely location of orphan types |
| `task/verified/006_migrate_execute_legacy_callers.md` | Task 006 — remaining execute_legacy callers in models_api (batch_* + analyze_tokens) |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — Confirmed during crate audit: 4 methods in `ModelsApi` call `v1beta/models:compare`, `v1beta/models:recommend`, `v1beta/models:filter`, `v1beta/models:status` — none of which exist in the real Gemini API. Any caller will receive a 404 at runtime. No compile-time signal. Methods and exclusively-associated types must be deleted.
- **2026-06-13** `VERIFY PASS` — User authorization: confirmed from crate audit with exact file paths and line numbers. All 4 dimensions pass: scope bounded (models_api.rs + orphan types), goal observable (grep = 0), YAGNI satisfied (actively misleading public API), procedure executable (delete + grep verify).

## Verification Record

- **Date:** 2026-06-13
- **Method:** User authorization — confirmed bug from crate audit, exact line numbers provided
- **Dim 1 (Scope Coherence):** PASS — In Scope: 4 methods + orphan types; Out of Scope: other execute_legacy callers, BatchApi, circuit breaker. Observable outcome: methods absent from public API.
- **Dim 2 (MOST Goal Quality):** PASS — Motivated: 404 at runtime with no compile signal; Observable: grep = 0, cargo doc = 0; Scoped: single file + orphan types; Testable: `grep -r "models:compare|..." src/ | wc -l` = 0.
- **Dim 3 (Value/YAGNI — adversarial):** PASS — Null hypothesis: without this fix, any caller of compare_models/get_recommendations/advanced_filter/get_model_status receives a 404 error, silently. Active user-facing bug. Concrete committed need.
- **Dim 4 (Implementation Readiness):** PASS — Work Procedure steps are executable; exact method names and lines documented; grep commands specified; orphan type discovery via grep before deletion.

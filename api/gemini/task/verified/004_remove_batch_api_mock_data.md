# Remove Mock Data from BatchApi

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** 🎯 (Verified)
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** src/
- **Validated By:** null
- **Validation Date:** null

## Goal

Replace all mock-returning method bodies in `src/batch_api.rs` with explicit `Err(Error::NotImplemented(...))` returns, eliminating the policy violation where `BatchApi` silently returns fabricated responses to callers. Every affected method currently has a `// Mock implementation - replace with real API call` comment and returns hardcoded `Ok(...)` data (e.g., `"Mock response"`, `vec![0.1, 0.2, 0.3]`), which a caller cannot distinguish from a real API response. The no-mock policy prohibits this regardless of whether the real endpoint exists. The correct behavior when a real endpoint is unavailable is an explicit, informative error — not a fabricated success response. Observable outcome: every `BatchApi` method that currently returns a mock `Ok(...)` instead returns `Err(Error::NotImplemented("Batch API not yet available in Gemini v1beta: {method}"))` or equivalent; `grep -n "Mock response\|Mock implementation" src/batch_api.rs` → 0; `w3 .test l::3` passes.

## In Scope

- `src/batch_api.rs` — replace mock `Ok(...)` bodies in: `create_inline`, `get_status`, `retrieve_results` (private), `cancel`, `list_with_page_size` (private), `retrieve_embedding_results` (private), `create_embedding_batch` (7 methods)
- Each replacement returns `Err(Error::NotImplemented("Batch API endpoint not yet available in Gemini v1beta: {method_description}".to_string()))` — caller receives a clear, specific error
- Module-level doc comment in `src/batch_api.rs` — update to reflect honest status (endpoint not yet available; returns NotImplemented for all operations)
- `Error` enum — confirm `NotImplemented` variant exists in `src/error/`; add it if absent (single-field tuple variant carrying the message string)
- MRE test — call `client.batches().create_inline(...)`, assert result is `Err(Error::NotImplemented(...))`

## Out of Scope

- Removing `BatchApi` from the public API surface — callers may want to compile against the type even when the endpoint is unavailable; the error response is the correct signal
- Polling logic in `wait_and_retrieve` and `wait_and_retrieve_embeddings` — these methods call `get_status` which will now return `NotImplemented`, causing the poll loop to propagate the error correctly; no changes needed to the loop control flow
- Real Batch API implementation — that requires the endpoint to exist in Gemini v1beta; out of scope until Google publishes it
- `models_api.rs` `batch_generate_content` / `batch_embed_contents` / `batch_count_tokens` — these call the real `batchGenerateContent`/`batchEmbedContents`/`batchCountTokens` endpoints; not BatchApi; not affected

## Requirements

- All work must adhere to applicable rulebooks (`kbase .rulebooks`)
- Every mock `Ok(...)` return in `batch_api.rs` must be replaced — no partial fix
- Error message must be specific: include both the method name and the reason ("Batch API endpoint not yet available in Gemini v1beta")
- Do NOT replace `Ok(...)` with `unimplemented!()` or `todo!()` — panics are not errors; callers expect `Result`
- `grep -n "Mock response\|mock response\|Mock implementation" src/batch_api.rs` → 0 after fix
- `w3 .test l::3` must pass with zero failures and zero new warnings

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Create bug report** — file bug per `bug.rulebook.md § Lifecycle : Procedure - Report New Bug`; yields BUG-NNN.
2. **Write MRE test** — in `tests/inc/`, write `test_batch_api_returns_not_implemented` marked `bug_reproducer(BUG-NNN)`. Call `client.batches().create_inline("gemini-2.5-flash", vec![])`, assert `Err(...)`, and in particular that the error IS NOT an `Ok` with mock data. (Current behavior: returns `Ok(BatchJob { job_id: "batch_job_...", state: Pending, ... })` from fake UUID — a fabricated success.) Run test; confirm it fails (current code returns Ok).
3. **Check/add `NotImplemented` variant** — inspect `src/error/` for `Error::NotImplemented(String)`. If absent, add it to the `Error` enum and update the `Display` impl.
4. **Replace mock bodies** — for each of the 7 methods in `src/batch_api.rs`:
   - `create_inline` — replace body with `Err(Error::NotImplemented("Batch API: create_inline endpoint not yet available in Gemini v1beta".to_string()))`
   - `get_status` — replace body with `Err(Error::NotImplemented("Batch API: get_status endpoint not yet available in Gemini v1beta".to_string()))`
   - `retrieve_results` — replace body with `Err(Error::NotImplemented("Batch API: retrieve_results endpoint not yet available in Gemini v1beta".to_string()))`
   - `cancel` — replace body with `Err(Error::NotImplemented("Batch API: cancel endpoint not yet available in Gemini v1beta".to_string()))`
   - `list_with_page_size` — replace body with `Err(Error::NotImplemented("Batch API: list endpoint not yet available in Gemini v1beta".to_string()))`
   - `retrieve_embedding_results` — replace body with `Err(Error::NotImplemented("Batch API: retrieve_embedding_results endpoint not yet available in Gemini v1beta".to_string()))`
   - `create_embedding_batch` — replace body with `Err(Error::NotImplemented("Batch API: create_embedding_batch endpoint not yet available in Gemini v1beta".to_string()))`
5. **Update module doc** — revise the `//!` header comment in `src/batch_api.rs` to state: "All methods return `Error::NotImplemented` until the real Batch API endpoint becomes available in Gemini v1beta. Replace method bodies when endpoints are published."
6. **Run MRE test** — confirm `test_batch_api_returns_not_implemented` now passes (receives `Err(NotImplemented(...))`).
7. **Document fix** — add `// Fix(BUG-NNN): ...` comment in `batch_api.rs`; add 5-section test documentation.
8. **Full verification** — `w3 .test l::3`.
9. **Update task state** — update `task/readme.md`; move file to `task/completed/`.

## Acceptance Criteria

- `grep -n "Mock response\|mock response\|Mock implementation" src/batch_api.rs` → 0 results
- Every `create_inline`, `get_status`, `cancel`, `list` call returns `Err(Error::NotImplemented(...))`
- MRE test `test_batch_api_returns_not_implemented` passes
- No `unimplemented!()` or `todo!()` panics in `batch_api.rs`
- `w3 .test l::3` passes with zero failures and zero new warnings

## Validation

**Execution:** Independent validator walks this section after SUBMIT transition.

### Checklist

**Mock elimination**
- [ ] C1 — Does `grep -n "Mock response\|Mock implementation" src/batch_api.rs` return 0 results?
- [ ] C2 — Does every public `async fn` in `BatchApi` return `Err(Error::NotImplemented(...))` when the endpoint is unavailable?
- [ ] C3 — Does `grep -n "unimplemented!\|todo!" src/batch_api.rs` return 0 results (no panic-based stubs)?

**Error quality**
- [ ] C4 — Does each `NotImplemented` error message contain both the method name and "v1beta"?

**Fix documentation**
- [ ] C5 — Does the MRE test `test_batch_api_returns_not_implemented` carry `bug_reproducer(BUG-NNN)`?
- [ ] C6 — Is there a 5-section fix doc in the test file?

### Measurements

- [ ] M1 — `grep -c "Mock response\|Mock implementation" src/batch_api.rs` → 0
- [ ] M2 — `grep -c "NotImplemented" src/batch_api.rs` → ≥ 7 (one per previously-mocked method)
- [ ] M3 — `w3 .test l::3` → 0 failures

### Invariants

- [ ] I1 — `cargo check --all-features` → 0 errors, 0 warnings
- [ ] I2 — `grep -c "Ok(" src/batch_api.rs` → 0 (no Ok-returning mock bodies remain; only Err returns)

### Anti-faking checks

- [ ] AF1 — `grep -c "unimplemented!\|todo!\|panic!" src/batch_api.rs` → 0 (panics are not error returns)
- [ ] AF2 — The MRE test actually calls a `BatchApi` method and asserts `Err`; it does not assert `Ok` or skip the assertion

## Related Documentation

| Path | Role |
|------|------|
| `src/batch_api.rs` | Primary fix site — 7 methods with mock Ok returns |
| `src/error/` | Error enum — confirm or add NotImplemented variant |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — Confirmed during crate audit: `src/batch_api.rs` contains 7 methods that return hardcoded mock `Ok(...)` responses. Module doc states "NOTE: As of 2025-10-11, this implementation uses mock responses. The real Batch API endpoints (e.g., `/v1/batches`) are not yet available in v1beta." This violates the no-mock policy — callers receive fabricated success responses indistinguishable from real API data.
- **2026-06-13** `VERIFY PASS` — User authorization: confirmed from crate audit. All 4 dimensions pass: scope bounded (batch_api.rs, 7 methods), goal observable (grep = 0 for mock strings), YAGNI satisfied (active policy violation), procedure executable (replace Ok with Err(NotImplemented)).

## Verification Record

- **Date:** 2026-06-13
- **Method:** User authorization — confirmed from crate audit, methods and mock strings identified
- **Dim 1 (Scope Coherence):** PASS — In Scope: 7 mocked methods in batch_api.rs; Out of Scope: real batch endpoints in models_api, BatchApi removal. Observable: grep = 0 for mock strings.
- **Dim 2 (MOST Goal Quality):** PASS — Motivated: callers receive fabricated Ok() they cannot distinguish from real responses; Observable: grep = 0 + MRE asserts Err; Scoped: single file, 7 methods; Testable: `grep -c "Mock response" src/batch_api.rs` = 0.
- **Dim 3 (Value/YAGNI — adversarial):** PASS — Null hypothesis: callers of BatchApi receive fake data silently. No way to distinguish from real data without reading source. Active no-mock violation.
- **Dim 4 (Implementation Readiness):** PASS — Replacement is mechanical: 7 method bodies replaced with `Err(Error::NotImplemented(...))`. Error variant may need to be added. No blocking ambiguity.

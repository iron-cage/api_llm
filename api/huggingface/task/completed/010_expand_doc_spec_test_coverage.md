# Expand GWT Spec Test Coverage — AP-07, FE-06, CL-06, CL-07

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

4 GWT spec scenarios added during the 2026-06-13 doc_tsk normalization session — AP-07 (`tests/docs/api/01_reference.md`), FE-06 (`tests/docs/feature/01_enterprise_reliability.md`), CL-06 and CL-07 (`tests/docs/catalog/01_features.md`) — have zero implementing test functions. Extend `tests/doc_spec_tests.rs` with 4 named functions (`test_ap_07`, `test_fe_06`, `test_cl_06`, `test_cl_07`) so that `grep -cE "^(async )?fn test_" tests/doc_spec_tests.rs` returns 42 and `cargo check --all-features` exits 0.

## In Scope

- `tests/doc_spec_tests.rs` — add 4 functions after the existing PF section: `test_ap_07` (integration — chat completions via providers), `test_fe_06` (static analysis — no cross-module state between reliability and cache), `test_cl_06` (Cargo.toml static — basic bundle composition), `test_cl_07` (Cargo.toml static — default = full alias)
- No other files changed

## Out of Scope

- Any `src/` changes
- The existing 38 functions in `tests/doc_spec_tests.rs` — must not be modified
- Any `docs/` or `tests/docs/` changes — documentation is already consistent after doc_tsk
- Fixing pre-existing failing integration tests (AP-02, AP-03, AP-04 HTTP 404 failures) — unrelated to this task's coverage scope
- `tests/readme.md` changes — the `doc_spec_tests.rs` row is owned by prior tasks

## Requirements

- All work strictly adheres to applicable rulebooks (`kbase .rulebooks`)
- No `cargo fmt`; custom codestyle: 2-space indent, spaces inside delimiters
- All 4 functions named exactly: `test_ap_07`, `test_fe_06`, `test_cl_06`, `test_cl_07`
- `test_ap_07`: gated with `#[ cfg( feature = "integration" ) ]` BEFORE `#[ tokio::test ]`; uses `inc::get_api_key_for_integration()` (panics on missing key)
- `test_fe_06`, `test_cl_06`, `test_cl_07`: static analysis only — no network calls, no API key required
- Static analysis functions use `std::fs::read_to_string( format!( "{}/...", env!( "CARGO_MANIFEST_DIR" ) ) )` — no `include_str!`
- All assertions use specific structural patterns (not bare `assert!( true )`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; confirm codestyle conventions and `#[ cfg ]` ordering requirements.

2. **Read spec files** — read `tests/docs/api/01_reference.md` (AP-07), `tests/docs/feature/01_enterprise_reliability.md` (FE-06), `tests/docs/catalog/01_features.md` (CL-06, CL-07) to confirm exact assertion targets for each function.

3. **Confirm providers chat API** — the method is `chat_completion` (not `chat`). Confirmed signature: `pub async fn chat_completion(&self, model: impl AsRef<str>, messages: Vec<ChatMessage>, max_tokens: Option<u32>, temperature: Option<f32>, top_p: Option<f32>) -> Result<ChatCompletionResponse>`. Read `src/providers.rs` to confirm the `ChatMessage` struct fields (`role: String`, `content: String`) and the exact path through `ChatCompletionResponse.choices[0].message.content` to the reply content field.

4. **Inspect reliability and cache source files** — read `src/reliability/circuit_breaker.rs`, `src/reliability/rate_limiter.rs`, and `src/cache/implementation.rs` to identify the actual `use` import paths in each file. Confirm the specific cross-module reference strings to assert absent in `test_fe_06` (e.g., check that circuit_breaker does not import from rate_limiter or cache, and that cache/implementation does not import from reliability).

5. **Confirm Cargo.toml feature entries** — read `Cargo.toml` features section to confirm the exact `basic` and `default` entries. Record the exact strings used (feature names and quoted members) as assertion targets for `test_cl_06` and `test_cl_07`.

6. **Read existing `tests/doc_spec_tests.rs`** — confirm the 38 existing functions (`grep -cE "^(async )?fn test_"` returns 38) and the exact position at the end of the PF section; this is the insertion point. Also read the `#[ cfg( feature = "integration" ) ]` use block at the top of the file (lines 8-15); `test_ap_07` requires adding `components::inference_shared::ChatMessage` to this block. Extend the block with `components::inference_shared::ChatMessage,` before the closing brace — this is an addition to the existing use block, not a modification of any existing function.

7. **Add `test_ap_07`** — integration test for AP-07 (chat completion returns assistant reply):
   - Doc comment: `/// AP-07: Chat completion returns assistant reply`
   - Gates: `#[ cfg( feature = "integration" ) ]` then `#[ tokio::test ]`
   - Body: call `inc::get_api_key_for_integration()`; build client via `build_integration_client()`; construct a single-element messages vec: `vec![ ChatMessage { role: "user".to_string(), content: "What is 2+2?".to_string(), tool_calls: None, tool_call_id: None } ]`; call `client.providers().chat_completion( "meta-llama/Llama-3.3-70B-Instruct", messages, None, None, None ).await.expect( "chat_completion call should succeed" )`; assert `!response.choices[ 0 ].message.content.is_empty()`

8. **Add `test_fe_06`** — static analysis for FE-06 (no cross-module shared state):
   - Doc comment: `/// FE-06: Enterprise feature modules do not share global static state`
   - Gate: `#[ test ]` only (no integration gate)
   - Body: read each of the three source files via `std::fs::read_to_string`; assert that `circuit_breaker` source does not contain use-paths to `rate_limiter` or the cache module; assert that `rate_limiter` source does not contain use-paths to `circuit_breaker` or the cache module; assert that `cache/implementation` source does not contain use-paths to `reliability`

9. **Add `test_cl_06`** — static analysis for CL-06 (basic bundle composition):
   - Doc comment: `/// CL-06: basic bundle composes exactly inference, embeddings, models, and env-config`
   - Gate: `#[ test ]` only
   - Body: read `Cargo.toml`; locate the `basic` feature line; assert it contains `"inference"`, `"embeddings"`, `"models"`, `"env-config"`; assert it does not contain enterprise feature names (`"circuit-breaker"`, `"rate-limiting"`, `"failover"`, `"health-checks"`, `"caching"`, `"performance-metrics"`, `"token-counting"`, `"dynamic-config"`, `"integration"`)

10. **Add `test_cl_07`** — static analysis for CL-07 (default = full alias):
    - Doc comment: `/// CL-07: default feature is an alias for full`
    - Gate: `#[ test ]` only
    - Body: read `Cargo.toml`; assert the file contains the literal `default = ["full"]` confirming `default` lists only `"full"` as its single member

11. **Verify function count** — `grep -cE "^(async )?fn test_" tests/doc_spec_tests.rs` must return 42 before proceeding.

12. **Check compilation** — `cargo check --all-features` → 0 errors, 0 warnings.

13. **Run non-integration subset** — `cargo nextest run --test doc_spec_tests --all-features -E 'test(test_fe_06) | test(test_cl_06) | test(test_cl_07)'` → 0 failures (3 static analysis functions pass; `test_ap_07` excluded as it requires live API).

14. **Update task state** — move file to `task/completed/`; update index.

## Test Matrix

| # | Scenario ID | Input | Config Under Test | Expected Behavior |
|---|------------|-------|-------------------|-------------------|
| T01 | AP-07 | `ChatMessage { role: "user", content: "What is 2+2?", tool_calls: None, tool_call_id: None }`, model `meta-llama/Llama-3.3-70B-Instruct` | `#[cfg(integration)]` + `#[tokio::test]`; `chat_completion(model, messages, None, None, None)` on live Router API | Returns `ChatCompletionResponse` with `choices[0].message.content` non-empty; no panic |
| T02 | FE-06 | `src/reliability/circuit_breaker.rs`, `src/reliability/rate_limiter.rs`, `src/cache/implementation.rs` | `std::fs::read_to_string`; cross-module `use` path search | No file contains `use` paths importing from a sibling enterprise module; each module is self-contained |
| T03 | CL-06 | `Cargo.toml`, `basic` feature line | `std::fs::read_to_string`; substring search on feature line | `basic` line contains all 4 Core features; no enterprise or integration feature names present in the line |
| T04 | CL-07 | `Cargo.toml`, `default` feature line | `std::fs::read_to_string`; exact substring match | File contains `default = ["full"]` confirming single-member alias |

## Acceptance Criteria

- `grep -cE "^(async )?fn test_" tests/doc_spec_tests.rs` → 42 (38 existing + 4 new)
- All 4 exact function names present: `test_ap_07`, `test_fe_06`, `test_cl_06`, `test_cl_07`
- `cargo check --all-features` → 0 errors, 0 warnings
- `cargo nextest run --test doc_spec_tests --all-features -E 'test(test_fe_06) | test(test_cl_06) | test(test_cl_07)'` → 0 failures
- `test_ap_07` is gated with `#[ cfg( feature = "integration" ) ]` before `#[ tokio::test ]`
- Zero existing test functions in `tests/doc_spec_tests.rs` modified
- Zero `src/` files modified

## Validation

**Execution:** An independent validator walks this section per `validation.rulebook.md` after SUBMIT transition. The executor does NOT self-validate.

### Checklist

**Function presence**
- [ ] C1 — Does `grep -cE "^(async )?fn test_" tests/doc_spec_tests.rs` return 42?
- [ ] C2 — Are all 4 exact new function names present: `test_ap_07`, `test_fe_06`, `test_cl_06`, `test_cl_07`?

**AP-07 — integration test**
- [ ] C3 — Is `test_ap_07` gated with `#[ cfg( feature = "integration" ) ]` immediately before `#[ tokio::test ]`?
- [ ] C4 — Does `test_ap_07` call `inc::get_api_key_for_integration()` (not the graceful Option variant)?
- [ ] C5 — Does `test_ap_07` assert that the response content field is non-empty (not bare `assert!( true )`)?

**FE-06 — module isolation**
- [ ] C6 — Does `test_fe_06` read all three source files via `std::fs::read_to_string`?
- [ ] C7 — Does `test_fe_06` assert that each module's source does not import from sibling enterprise modules?
- [ ] C8 — Is `test_fe_06` a `#[ test ]` function (no integration gate)?

**CL-06 — basic bundle composition**
- [ ] C9 — Does `test_cl_06` read `Cargo.toml` via `std::fs::read_to_string`?
- [ ] C10 — Does `test_cl_06` assert all 4 Core feature names present AND absence of at least one enterprise feature from the `basic` line?
- [ ] C11 — Does `test_cl_06` pass under `cargo nextest run --test doc_spec_tests --all-features -E 'test(test_cl_06)'`?

**CL-07 — default alias**
- [ ] C12 — Does `test_cl_07` assert the literal `default = ["full"]` substring in `Cargo.toml`?
- [ ] C13 — Does `test_cl_07` pass under `cargo nextest run --test doc_spec_tests --all-features -E 'test(test_cl_07)'`?

**No side effects**
- [ ] C14 — Are all 38 existing functions in `tests/doc_spec_tests.rs` unchanged (verified by diff)?
- [ ] C15 — Are zero `src/` files modified?

**Compilation**
- [ ] C16 — Does `cargo check --all-features` complete with 0 errors and 0 warnings?

### Measurements

- [ ] M1 — `grep -cE "^(async )?fn test_" tests/doc_spec_tests.rs` → `42`
- [ ] M2 — `grep -c 'cfg( feature = "integration"' tests/doc_spec_tests.rs` → count increases by 1 (only `test_ap_07` added)
- [ ] M3 — `grep -c 'Cargo.toml' tests/doc_spec_tests.rs` → count increases by 2 (`test_cl_06` + `test_cl_07`)

### Invariants

- [ ] I1 — `cargo check --all-features` → 0 errors, 0 warnings
- [ ] I2 — `task/decisions.md` present and accessible

### Anti-faking checks

- [ ] AF1 — `test_ap_07` is NOT `assert!( true )` — it makes a live API call and asserts specific response structure
- [ ] AF2 — `test_fe_06` asserts specific module path strings are absent from specific files — not a trivial always-true check
- [ ] AF3 — `test_cl_06` uses a targeted assertion on the `basic` feature line content, not a global `contains( "inference" )` on the whole file that might accidentally match another feature's description
- [ ] AF4 — `test_cl_07` asserts the exact literal `default = ["full"]` — not just `contains( "full" )` which could match other occurrences

## Related Documentation

- `docs/api/001_reference.md` — API reference; AP-07 verifies `providers().chat_completion()` contract
- `docs/feature/001_enterprise_reliability.md` — enterprise reliability feature group; FE-06 verifies module isolation
- `docs/catalog/001_features.md` — feature flag catalog; CL-06 and CL-07 verify bundle and alias entries
- `tests/docs/api/01_reference.md` — AP-07 GWT scenario this task implements
- `tests/docs/feature/01_enterprise_reliability.md` — FE-06 GWT scenario this task implements
- `tests/docs/catalog/01_features.md` — CL-06 and CL-07 GWT scenarios this task implements
- `task/completed/008_implement_doc_spec_test_coverage.md` — Related: 008 — original 28-function doc_spec_tests.rs creation
- `task/completed/009_implement_collection_pitfall_spec_tests.md` — Related: 009 — added CL-01..05 and PF-01..04 (same target file)

## Affected Entities

| Entity Dir | Entity Type | Change |
|------------|-------------|--------|
| `tests/` | Test suite | 4 new functions added to existing `doc_spec_tests.rs` |
| `tests/docs/api/` | Test spec docs | Unchanged — AP-07 scenario already defined in `01_reference.md` |
| `tests/docs/feature/` | Test spec docs | Unchanged — FE-06 scenario already defined in `01_enterprise_reliability.md` |
| `tests/docs/catalog/` | Test spec docs | Unchanged — CL-06, CL-07 scenarios already defined in `01_features.md` |

## Verification Findings

**First MAAV run — 2026-06-13** | Implementation Readiness FAIL

| Finding | Location | Issue | Fix Applied |
|---------|----------|-------|-------------|
| F1 | Work Procedure step 7, Test Matrix T01 | `client.providers().chat(messages, model)` — method `chat` does not exist in `src/providers.rs`; actual method is `chat_completion(model, messages, max_tokens, temperature, top_p)` with model as first argument | Step 3 updated with confirmed signature; step 7 updated to `chat_completion("meta-llama/Llama-3.3-70B-Instruct", messages, None, None, None)`; T01 updated with correct call |
| F2 | `tests/docs/api/01_reference.md` AP-07 When clause | Spec used `client.providers().chat(messages, model)` — wrong method name and wrong argument order | AP-07 When clause updated to `client.providers().chat_completion(model, messages, None, None, None)` |

**Second MAAV run — 2026-06-13** | Implementation Readiness FAIL

| Finding | Location | Issue | Fix Applied |
|---------|----------|-------|-------------|
| F3 | Work Procedure step 7 | `ChatMessage { role: ..., content: ... }` struct literal omits required fields `tool_calls: Option<Vec<ToolCall>>` and `tool_call_id: Option<String>` — struct has no `Default` impl; compile error without all 4 fields | Step 7 updated to `ChatMessage { role: ..., content: ..., tool_calls: None, tool_call_id: None }`; T01 updated |
| F4 | Work Procedure (missing step) | No instruction to extend the `#[cfg(feature = "integration")]` use block with `ChatMessage` import; test_ap_07 uses `ChatMessage` but the existing use block only imports `Client`, `HuggingFaceEnvironmentImpl`, `Secret`, `HuggingFaceError` — unresolved name compile error | Step 6 extended with explicit instruction to add `components::inference_shared::ChatMessage` to the existing integration use block |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — Task filed after doc_tsk normalization session added AP-07, FE-06, CL-06, CL-07 spec scenarios with zero implementing functions. Extends the same target file as task 008 (Related: 008) and task 009 (Related: 009); those tasks' Out of Scope explicitly excluded new scenarios added after their execution. No dedup match (Case A — all prior similar tasks are Completed).
- **2026-06-13** `REVISED` — First MAAV (4 agents) returned 1 FAIL (Implementation Readiness): `src/providers.rs` has no `chat()` method — the actual method is `chat_completion(model, messages, None, None, None)` with model as the first argument. AP-07 spec in `tests/docs/api/01_reference.md` also used the wrong method name — fixed to `client.providers().chat_completion(model, messages, None, None, None)`. Applied fixes to Work Procedure step 3 and step 7, Test Matrix T01, and Related Documentation.
- **2026-06-13** `REVISED` — Second MAAV (4 agents) returned 1 FAIL (Implementation Readiness): (1) `ChatMessage` has 4 fields (`role`, `content`, `tool_calls`, `tool_call_id`) — struct literal in step 7 only specified 2, causing compile error; fixed to include all 4 with `None` for the optional pair. (2) No instruction to add `ChatMessage` to the existing integration use block — fixed by extending step 6 with an explicit import block amendment instruction. Applied fixes to step 6, step 7, and Test Matrix T01.
- **2026-06-13** `VERIFIED` — Third MAAV gate passed (4 independent subagents). State → 🎯 (Verified).
- **2026-06-13** `EXECUTED` — 4 functions inserted into `tests/doc_spec_tests.rs` before Helpers section. Edit 1: `ChatMessage` added to integration use block. Edit 2: `test_ap_07`, `test_fe_06`, `test_cl_06`, `test_cl_07` inserted. `grep -cE` → 42. `cargo nextest run -E 'test(test_fe_06)|test(test_cl_06)|test(test_cl_07)'` → 3/3 PASS. Full unit suite (473 tests) → 473/473 PASS. Status readmes updated to ✅. State → ✅ (Completed).

## Verification Record

**Date:** 2026-06-13
**Method:** MAAV — 4 independent parallel Agent subagents (third run, after two REVISED passes)

| Dimension | Result | Summary |
|-----------|--------|---------|
| Scope Coherence | PASS | In Scope: 1 file, 4 named functions, concrete insertion point. Out of Scope: 5 explicit exclusions covering realistic drift areas. End-state: 3 machine-verifiable shell commands. No contradictions found. |
| MOST Goal Quality | PASS | M: gap stated with specific session and scenario IDs. O: 3 exact grep/cargo commands with expected outputs. S: bounded to single file (`tests/doc_spec_tests.rs`). T: 16-item checklist + 3 measurements + 4 anti-faking checks independently executable. |
| Value / YAGNI | PASS | All 4 scenarios confirmed in committed spec files; zero implementing functions confirmed by grep. Prior tasks 008/009 explicitly exclude these scenarios. No speculative work. |
| Implementation Readiness | PASS | Baseline 38 confirmed. `chat_completion` signature confirmed (model first, then messages, 3 Option params). `ChatMessage` 4-field struct literal confirmed in step 7. Step 6 import-block amendment instruction present. All 3 FE-06 source files confirmed to exist. CL-06 targets `basic` feature line specifically. CL-07 asserts exact `default = ["full"]` literal. Test Matrix T01-T04 present. |

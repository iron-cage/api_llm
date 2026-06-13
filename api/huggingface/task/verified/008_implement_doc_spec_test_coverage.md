# Implement GWT Spec Tests for 28 Doc Entity Scenarios

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

28 GWT spec scenarios in `tests/docs/` (FE-01..FE-04, AP-01..AP-06, OP-01..OP-06, IN-01..IN-08, PT-01..PT-04) have zero implementing test functions; create `tests/doc_spec_tests.rs` with one named function per scenario so that `grep -c "^fn test_" tests/doc_spec_tests.rs` returns 28 and `cargo nextest run --test doc_spec_tests --all-features` exits 0.

## In Scope

- `tests/doc_spec_tests.rs` — new file; one test function per scenario, named `test_fe_NN`, `test_ap_NN`, `test_op_NN`, `test_in_NN`, `test_pt_NN` matching scenario IDs exactly
- `tests/readme.md` — add row for `doc_spec_tests.rs`
- `Cargo.toml` — register `[[test]] name = "doc_spec_tests" path = "tests/doc_spec_tests.rs"` if not auto-discovered
- No changes to any existing test file

## Out of Scope

- Modifying existing test files (`inference_tests.rs`, `embeddings_tests.rs`, etc.) — those are handled by task 007
- Changing any `docs/` or `tests/docs/` file — documentation is already consistent
- Adding new `tests/docs/` spec scenarios beyond the 28 defined
- Any source (`src/`) changes

## Requirements

- All work strictly adheres to applicable rulebooks (`kbase .rulebooks`)
- No `cargo fmt`; custom codestyle: 2-space indent, spaces inside delimiters
- Function-level integration gates use canonical attribute order: `#[ cfg( feature = "integration" ) ]` immediately before `#[ tokio::test ]`
- Compile-time absence tests use `#[ cfg( not( feature = "..." ) ) ]` gate on the function itself — function is present in source (grep finds it), absent from `--all-features` build (excluded by cfg), and present in feature-absent build where it runs and passes
- Static analysis tests use `include_str!( "../../src/..." )` to read source and `assert!( content.contains( ... ) )` — no use of `proc_macro`, trybuild, or compile-time assertions
- Integration tests: `#[ cfg( feature = "integration" ) ]` + `#[ tokio::test ]`; call `get_api_key_for_integration()` (from `tests/inc/mod.rs`) or create a `Client` with the key; `panic!` on failure, never `return`
- No mocks; no wiremock; all API tests call real HuggingFace endpoints
- **Prerequisite dependency for IN-06 and IN-07**: As of 2026-06-13, `tests/health_check_tests.rs` uses `wiremock` (task 004 open) and several test files lack integration gates (task 007 open). Implement `test_in_06` and `test_in_07` with `todo!("Pending task 004 / task 007 completion")` bodies so they compile but signal the dependency without silently passing; convert to real `include_str!`-based assertions only after tasks 004 and 007 complete

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; confirm codestyle, integration gate convention, loud-failure requirement.

2. **Read all 28 spec scenarios** — read all five spec files in `tests/docs/`:
   - `tests/docs/feature/01_enterprise_reliability.md` (FE-01..FE-04)
   - `tests/docs/api/01_reference.md` (AP-01..AP-06)
   - `tests/docs/operation/01_features.md` (OP-01..OP-06)
   - `tests/docs/invariant/01_thin_client_principle.md` (IN-01..IN-04)
   - `tests/docs/invariant/02_testing_standards.md` (IN-05..IN-08)
   - `tests/docs/pattern/01_module_organization.md` (PT-01..PT-04)

3. **Read source files** needed for static analysis tests — at minimum:
   - `src/lib.rs` (feature gates on `pub mod`)
   - All individual feature module `.rs` files under `src/` (e.g., `src/inference.rs`, `src/providers.rs`, `src/embeddings.rs`, `src/models.rs`, `src/error.rs`, `src/validation.rs`, `src/token_counter/counter.rs`, `src/diagnostics.rs`) — confirm `mod private { }` block and `crate::mod_interface!` presence
   - `Cargo.toml` — confirm `default` → `full` → `integration` feature chain
   - `tests/inc/mod.rs` — confirm `get_api_key_for_integration()` signature

4. **Create `tests/doc_spec_tests.rs`** with all 28 functions. Apply these implementation patterns by scenario category:

   **Category A — Compile-time absence checks** (6 functions: FE-01, FE-04, IN-01, OP-01, OP-02, OP-03):
   Gate each function with `#[ cfg( not( feature = "..." ) ) ]` so it compiles and passes when the named feature is absent; it disappears from `--all-features` builds. Body: a doc comment quoting the scenario + `assert!( true )` to confirm the compile gate itself is the test (the function would not compile if the type were referenced and the feature absent). Example:
   ```
   #[ cfg( not( feature = "circuit-breaker" ) ) ]
   #[ test ]
   fn test_fe_01()
   {
     // FE-01: CircuitBreaker type is absent when circuit-breaker feature is disabled.
     // Verification: this function compiles only when the feature is absent; if the
     // feature were present and CircuitBreaker were referenced here, compilation would
     // fail — the cfg gate is the mechanism.
     assert!( true );
   }
   ```

   **Category B — Static source analysis** (12 functions: IN-02, IN-03, IN-04, IN-05, IN-06, IN-07, IN-08, PT-01, PT-02, PT-03, PT-04, OP-04):
   Use `include_str!` to load source text; assert on structural patterns. No network or integration feature gates needed. Example:
   ```
   #[ test ]
   fn test_pt_03()
   {
     // PT-03: No private.rs files or private/ directories in src/.
     let entries: Vec< _ > = std::fs::read_dir( "src" )
       .expect( "src/ must be readable" )
       .flatten()
       .collect();
     let has_private = entries.iter().any( |e| {
       let name = e.file_name();
       name == "private.rs" || name == "private"
     } );
     assert!( !has_private, "found private.rs or private/ in src/ — PT-03 violation" );
   }
   ```
   For `IN-08` (Cargo.toml default chain): `include_str!( "../../Cargo.toml" )`; assert the source contains `"full"` in the `default` line and `"integration"` in the `full` line.
   For `IN-05` (missing credentials loud failure): `include_str!( "../inc/mod.rs" )`; assert `get_api_key_for_integration` body contains `.expect(` (indicating it panics) and does NOT contain `return None` or `return Ok(())` — this statically verifies the function never silently absorbs a missing-key condition. No runtime credential manipulation needed.
   For `OP-04` (full feature type presence): plain `#[ test ] fn test_op_04()` with `#[ cfg( feature = "circuit-breaker" ) ]`-gated inner blocks that perform `let _: Option< api_huggingface::reliability::CircuitBreaker > = None;` style compile-presence assertions. The outer function always compiles; the inner type references compile only when the relevant features are present (as they are under `--all-features`).
   For `IN-06` and `IN-07`: use `todo!( "Pending task 004 / task 007 completion" )` per the prerequisite dependency noted in Requirements.

   **Category C — Unit tests** (3 functions: FE-02, FE-03, OP-05):
   - FE-02: construct a client with a dummy key, retrieve the inference accessor, verify it returns without circuit-breaking or wrapping. Constructor call: `let secret = Secret::new( "hf_test" ); let env = HuggingFaceEnvironmentImpl::build( secret, None ).unwrap(); let client = Client::build( env ).unwrap(); let _inference = client.inference();`. Requires `client` + `env-config` + `inference` features. No network call.
   - FE-03: construct a `RateLimiter` with `RateLimiter::new( RateLimiterConfig::default() )` (from `api_huggingface::reliability::{ RateLimiter, RateLimiterConfig }`); do NOT call `.acquire().await`; assert that a timing check (`std::time::Instant::now()` before and after the construction) shows < 100ms elapsed — confirming the rate limiter is silent without explicit acquisition. Requires `rate-limiting` feature.
   - OP-05: plain `#[ test ] fn test_op_05() { /* OP-05: minimal build compiles; this fn body documents that compilation with --no-default-features --features enabled is verified at build time */ assert!( true ); }`.

   **Category D — Real API integration tests** (7 functions: AP-01..AP-06, OP-06):
   Gate with `#[ cfg( feature = "integration" ) ]` + `#[ tokio::test ]`. Use `get_api_key_for_integration()` from `tests/inc/mod.rs` to obtain the key. Assert response contents; `panic!` on error, never `return`.
   - OP-06: call `client.inference().create(prompt, model)` with the integration key; assert `generated_text` is non-empty.

5. **Verify function count** — `grep -c "^fn test_" tests/doc_spec_tests.rs` must return 28 before proceeding.

6. **Run test suite** — `cargo nextest run --test doc_spec_tests --all-features` → 0 failures, 0 warnings. Note: compile-time absence functions (Category A) will NOT appear in `--all-features` run; that is expected and correct. Run separately with `--no-default-features --features enabled` to exercise Category A functions if desired.

7. **Update `tests/readme.md`** — add row for `doc_spec_tests.rs` with responsibility description.

8. **Green state** — `w3 .test l::3` → 0 failures, 0 warnings.

9. **Update task state** — move file to `task/completed/`; update index.

## Test Matrix

| # | Scenario ID | Input Scenario | Config Under Test | Expected Behavior |
|---|------------|----------------|-------------------|-------------------|
| T01 | FE-01 | Compiled without `circuit-breaker` feature | `#[cfg(not(feature = "circuit-breaker"))]` | `test_fe_01` compiles; no `CircuitBreaker` type reference compiles through |
| T02 | FE-02 | `Client` constructed, inference accessor retrieved, no `CircuitBreaker` wired | `--all-features` | `client.inference()` returns accessor; no automatic wrapping or throttling occurs |
| T03 | FE-03 | `RateLimiter` constructed, but `acquire()` NOT called before call | `rate-limiting` feature | Request proceeds without delay; `RateLimiter` is silent |
| T04 | FE-04 | Compiled with only `failover` feature (not `circuit-breaker`, `rate-limiting`, `health-checks`) | `#[cfg(not(feature = "circuit-breaker"))]` etc. | Only failover logic present; other modules excluded from compilation |
| T05 | AP-01 | Valid API key + `meta-llama/Llama-3.2-1B-Instruct` model | `integration` feature + real HuggingFace endpoint | `InferenceResponse.generated_text` is non-empty `String` |
| T06 | AP-02 | Valid API key + `sentence-transformers/all-MiniLM-L6-v2` model | `integration` feature | `EmbeddingResponse.embeddings[0]` is `Vec<f32>` with len ≥ 1 |
| T07 | AP-03 | Valid API key + identical text twice + embedding model | `integration` + `embeddings-similarity` feature | Returned `f32` in `[-1.0, 1.0]` and ≥ 0.99 for identical texts |
| T08 | AP-04 | Valid API key + generation model | `integration` + `inference-streaming` feature | At least one `StreamingChunk` with non-`None` token before stream yields `None` |
| T09 | AP-05 | Client constructed with `"hf_invalid"` key | `integration` feature | Returns `HuggingFaceError::Authentication` or `HuggingFaceError::Http`; no panic |
| T10 | AP-06 | Valid API key + known model identifier | `integration` feature | Returns model metadata OR `HuggingFaceError::ModelUnavailable`; no panic |
| T11 | OP-01 | Compiled without `inference-streaming` feature | `#[cfg(not(feature = "inference-streaming"))]` | `create_stream` symbol absent from compiled output |
| T12 | OP-02 | Compiled without `embeddings-similarity` feature | `#[cfg(not(feature = "embeddings-similarity"))]` | `similarity` method absent from compiled output |
| T13 | OP-03 | Compiled without `sync` feature | `#[cfg(not(feature = "sync"))]` | `SyncClient` type absent from compiled output |
| T14 | OP-04 | All tier-1 and tier-2 feature types referenced | `--features full` | All capability types compile and link without errors |
| T15 | OP-05 | `cargo build` with `--no-default-features --features enabled` | Minimal feature set | Compilation succeeds; `test_op_05` body asserts true |
| T16 | OP-06 | `HUGGINGFACE_API_KEY` in env; `cargo nextest run --all-features` | `integration` feature | Integration tests run and call real HuggingFace endpoints |
| T17 | IN-01 | Compiled without `rate_limiting` feature | `#[cfg(not(feature = "rate-limiting"))]` | `rate_limiting` module not compiled in; no throttling |
| T18 | IN-02 | Source of each non-enterprise module inspected | `include_str!` on `src/*.rs` files | No retry logic found outside `retry-logic` feature-gated module |
| T19 | IN-03 | Source of `src/inference.rs` inspected for HTTP call count | `include_str!` | Exactly one HTTP call path per public method; no background dispatch |
| T20 | IN-04 | Source of `src/inference.rs` inspected for model substitution | `include_str!` | No aliasing, fallback, or substitution logic present in the call path |
| T21 | IN-05 | `tests/inc/mod.rs` source inspected for panic behavior | `include_str!( "../inc/mod.rs" )` | Source contains `.expect(` in `get_api_key_for_integration`; does NOT contain `return None` — statically confirms loud-failure mechanism |
| T22 | IN-06 | All `tests/*.rs` source files inspected | `include_str!` on test directory files | No `wiremock`, `mockito`, `httpmock`, or fake adapter import found |
| T23 | IN-07 | All integration test functions in `tests/*.rs` inspected | `include_str!` | Every real-API test function is preceded by `#[cfg(feature = "integration")]` immediately before `#[tokio::test]` |
| T24 | IN-08 | `Cargo.toml` default feature chain inspected | `include_str!("../../Cargo.toml")` | `default` feature includes `"full"`; `full` includes `"integration"` (or direct path) |
| T25 | PT-01 | Individual feature module `.rs` files in `src/` inspected | `include_str!` on each | All type definitions appear inside `mod private {` block |
| T26 | PT-02 | Feature module `.rs` files inspected for mod_interface! | `include_str!` | Each file ends with `crate::mod_interface! {` invocation listing exposed symbols |
| T27 | PT-03 | `src/` directory tree walked via `std::fs::read_dir` | Filesystem | Zero `private.rs` files and zero `private/` directories found |
| T28 | PT-04 | `src/lib.rs` inspected for optional `pub mod` declarations | `include_str!("../../src/lib.rs")` | Every optional-feature `pub mod` is preceded by `#[cfg(feature = "...")]` |

## Acceptance Criteria

- `grep -c "^fn test_" tests/doc_spec_tests.rs` → 28
- All 28 function names follow the pattern: `test_fe_01`, `test_fe_02`, `test_fe_03`, `test_fe_04`, `test_ap_01`..`test_ap_06`, `test_op_01`..`test_op_06`, `test_in_01`..`test_in_08`, `test_pt_01`..`test_pt_04`
- `cargo nextest run --test doc_spec_tests --all-features` → 0 failures (compile-time absence functions excluded by cfg; unit and integration functions pass)
- No `wiremock`, `mockito`, or mock HTTP dependency introduced in `Cargo.toml`
- No existing test file modified
- `tests/readme.md` contains a row for `doc_spec_tests.rs`
- `w3 .test l::3` → 0 failures, 0 warnings

## Validation

**Execution:** An independent validator walks this section per `validation.rulebook.md` after SUBMIT transition. The executor does NOT self-validate.

### Checklist

**Function presence**
- [ ] C1 — Does `grep -c "^fn test_" tests/doc_spec_tests.rs` return 28?
- [ ] C2 — Are all 28 exact function names present (`test_fe_01`..`test_fe_04`, `test_ap_01`..`test_ap_06`, `test_op_01`..`test_op_06`, `test_in_01`..`test_in_08`, `test_pt_01`..`test_pt_04`)?

**Category A — Compile-time gates**
- [ ] C3 — Are `test_fe_01`, `test_fe_04`, `test_in_01`, `test_op_01`, `test_op_02`, `test_op_03` each gated with `#[cfg(not(feature = "..."))]`?
- [ ] C4 — Do these six functions compile and pass when the named feature is absent?

**Category B — Static analysis**
- [ ] C5 — Do `test_in_02`, `test_in_03`, `test_in_04`, `test_in_05`, `test_in_06`, `test_in_07`, `test_in_08`, `test_op_04`, `test_pt_01`, `test_pt_02`, `test_pt_03`, `test_pt_04` use `include_str!`, `std::fs`, or compile-presence techniques — no runtime API calls?
- [ ] C6 — Do these twelve functions (excluding `test_in_06`/`test_in_07` which use `todo!`) pass under `--no-default-features --features enabled`?

**Category C — Unit tests**
- [ ] C7 — Do `test_fe_02`, `test_fe_03`, `test_op_05` pass without a real API key?

**Category D — Integration tests**
- [ ] C8 — Are `test_ap_01`..`test_ap_06` and `test_op_06` (7 functions total) gated with `#[cfg(feature = "integration")]`?
- [ ] C9 — Do these 7 integration tests use `get_api_key_for_integration()` and `panic!` (not `return`) on failure?

**No side effects**
- [ ] C10 — Are zero existing test files modified?
- [ ] C11 — Is no mock HTTP dependency added to `Cargo.toml`?

**Compilation and tests**
- [ ] C12 — Does `cargo nextest run --test doc_spec_tests --all-features` complete with 0 failures?
- [ ] C13 — Does `w3 .test l::3` complete with 0 failures and 0 warnings?

### Measurements

- [ ] M1 — `grep -c "^fn test_" tests/doc_spec_tests.rs` → `28`
- [ ] M2 — `grep -c "cfg(not(feature" tests/doc_spec_tests.rs` → `6` (one per Category A function)
- [ ] M3 — `grep -c "cfg(feature = \"integration\")" tests/doc_spec_tests.rs` → `7` (one per Category D function: AP-01..AP-06 + OP-06)
- [ ] M4 — `grep -r "wiremock\|mockito\|httpmock" Cargo.toml tests/doc_spec_tests.rs` → empty

### Invariants

- [ ] I1 — test suite: `w3 .test l::3` → 0 failures
- [ ] I2 — task system: `task/decisions.md` present and accessible (I2 invariant)

### Anti-faking checks

- [ ] AF1 — static analysis tests are NOT trivially `assert!(true)` — each asserts a specific structural pattern from the source text
- [ ] AF2 — integration tests make a real network call (AP-01..AP-06, OP-06 use `client.inference().create(...)` or equivalent)
- [ ] AF3 — compile-time absence functions contain a comment identifying the specific scenario and compile gate mechanism
- [ ] AF4 — `test_in_05` asserts a specific structural pattern in `tests/inc/mod.rs` source (not just `assert!(true)`) — confirms `.expect(` present and `return None` absent

## Related Documentation

- `docs/feature/001_enterprise_reliability.md` — enterprise reliability design (FE scenarios)
- `docs/api/001_reference.md` — API contract and client interface (AP scenarios)
- `docs/operation/001_features.md` — feature flag management procedure (OP scenarios)
- `docs/invariant/001_thin_client_principle.md` — thin client constraints (IN-01..IN-04)
- `docs/invariant/002_testing_standards.md` — testing standards constraints (IN-05..IN-08)
- `docs/pattern/001_module_organization.md` — module organization pattern (PT scenarios)
- `tests/docs/feature/01_enterprise_reliability.md` — FE-01..FE-04 GWT scenarios
- `tests/docs/api/01_reference.md` — AP-01..AP-06 GWT scenarios
- `tests/docs/operation/01_features.md` — OP-01..OP-06 GWT scenarios
- `tests/docs/invariant/01_thin_client_principle.md` — IN-01..IN-04 GWT scenarios
- `tests/docs/invariant/02_testing_standards.md` — IN-05..IN-08 GWT scenarios
- `tests/docs/pattern/01_module_organization.md` — PT-01..PT-04 GWT scenarios

## Affected Entities

| Entity Dir | Entity Type | Change |
|------------|-------------|--------|
| `tests/` | Test suite | New file `doc_spec_tests.rs` added |
| `tests/docs/` | Test spec docs | Unchanged — these define the 28 scenarios this task implements |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — Task filed after doc normalization session added/updated 6 doc instances and 5 tests/docs/ spec files across feature/, api/, operation/, invariant/, and pattern/ entities; 28 GWT scenarios confirmed unimplemented by grep across tests/.
- **2026-06-13** `REVISED` — First MAAV (4 agents) returned 2 FAILs: (1) IN-05 category conflict (C vs D), unspecified constructors for FE-02/FE-03, M3 count inconsistency (9→7), OP-04 mislabeled as Category D; (2) IN-06/IN-07 ordering dependency on tasks 004/007 not disclosed. Applied fixes: reclassified IN-05 → Category B (static analysis of tests/inc/mod.rs); reclassified OP-04 → Category B (compile-presence); moved OP-05 from Category D to Category C; specified exact constructors (Secret::new, HuggingFaceEnvironmentImpl::build, Client::build, RateLimiter::new); fixed M3 to 7; added prerequisite dependency note for IN-06/IN-07 with todo! bodies.

## Verification Record

**Date:** 2026-06-13
**Method:** MAAV — 4 independent parallel Agent subagents (second run, after REVISED fixes)

| Dimension | Agent ID | Result | Summary |
|-----------|----------|--------|---------|
| Scope Coherence | af90c7fc3c7f5f03b | PASS | In Scope concrete (3 deliverables), Out of Scope clear (4 exclusions), Goal mechanically verifiable, no contradictions |
| MOST Goal Quality | a50d597a111fe3bf5 | PASS | M: grounded in confirmed grep evidence; O: two mechanical commands with expected outputs; S: one new file + two housekeeping artifacts; T: 28 scenario-level criteria with anti-fake checks |
| Value / YAGNI | a2f097ce7553f0467 | PASS | `doc_spec_tests.rs` absent (Read 404); zero implementing functions (Grep confirmed); all 28 scenarios in committed spec files; no overlap with tasks 001/003/004/005/006/007; IN-06/IN-07 dependency disclosed with todo! mechanism |
| Implementation Readiness | a6ef20e4f1e643f42 | PASS | IN-05 consistently Category B in all locations; FE-02/FE-03 constructors confirmed against source; M3=7 matches Category D (AP-01..AP-06 + OP-06); all 6 spec files readable; `get_api_key_for_integration` confirmed to use `.expect(` |

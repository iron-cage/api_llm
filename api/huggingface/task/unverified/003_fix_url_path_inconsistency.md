# Fix URL Path Inconsistency in providers.rs

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** ❓ (Unverified)
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** src/
- **Validated By:** null
- **Validation Date:** null

## Goal

`src/inference.rs` constructs endpoint URLs using a relative path `"chat/completions"` (correct for `Url::join` when the base URL ends in `/`), but `src/providers.rs` uses the absolute path `"/v1/chat/completions"` with a leading slash. `Url::join` treats a path starting with `/` as an absolute path and discards the base URL's path component entirely, producing a wrong URL. For example, if the provider base is `https://router.huggingface.co/hf-inference/models/{model}/v1/`, the join with `"/v1/chat/completions"` yields `https://router.huggingface.co/v1/chat/completions` — stripping the model path. This is a live runtime bug: requests routed through `providers.rs` hit the wrong endpoint. The fix is to audit `src/providers.rs` for all URL path strings starting with `/v1/` and replace them with relative counterparts consistent with `inference.rs`. Observable outcome: `grep -n '"/v1/' src/providers.rs` → 0 matches; `w3 .test l::3` → 0 failures, 0 warnings.

## In Scope

- `src/providers.rs` — audit all URL path string literals; replace any absolute `/v1/...` paths with relative paths matching the convention in `src/inference.rs`

## Out of Scope

- `src/inference.rs` — already uses correct relative paths; no changes needed
- Base URL configuration changes — only the path join argument is changed, not the base URL
- Adding new URL construction helpers or abstractions — path string fix only
- Other source files — audit scope is limited to providers.rs

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- No `cargo fmt` — custom codestyle applies throughout
- Path strings must be consistent with the `Url::join` semantics used in `inference.rs`
- All modified code must compile clean under `RUSTFLAGS="-D warnings"`

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note thin-client and URL construction principles.
2. **Read both files** — read `src/inference.rs` to confirm the relative path convention; read `src/providers.rs` to locate all URL path literals.
3. **Write Test Matrix** — populate every row before writing any test code.
4. **Write a failing test** — write a grep or unit test that fails while absolute paths exist in `providers.rs`.
5. **Implement** — replace each `"/v1/..."` path literal in `providers.rs` with the correct relative form (e.g., `"chat/completions"` — no leading slash and no `v1/` prefix, matching the pattern used in `inference.rs`).
6. **Green state** — `w3 .test l::3` → 0 failures, 0 warnings before proceeding.
7. **Submit for Validation** — trigger SUBMIT transition (⏳ → 🔍).
8. **Update task state** — on validation pass, move file to `task/completed/`; update index.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Scan for absolute path literals | `src/providers.rs` source text | `grep -n '"/v1/' src/providers.rs` → 0 matches |
| T02 | Confirm relative path scheme in inference.rs | `src/inference.rs` source text | `grep -n '"chat/completions"' src/inference.rs` → ≥ 1 match (reference form) |
| T03 | Relative path consistent between files | Both src files | Path join argument in providers.rs does not start with `/` |
| T04 | Compile after path change | All features | `RUSTFLAGS="-D warnings" cargo build --all-features` → 0 errors, 0 warnings |
| T05 | Full test suite passes | Default + all features | `w3 .test l::3` → 0 failures |

## Acceptance Criteria

- `grep -n '"/v1/' src/providers.rs` → 0 matches (no absolute path literals)
- Path string arguments in `src/providers.rs` do not start with `/`
- `RUSTFLAGS="-D warnings" cargo build --all-features` → 0 errors, 0 warnings
- `w3 .test l::3` → 0 failures, 0 warnings
- `src/inference.rs` is unmodified

## Validation

**Execution:** An independent validator walks this section per `validation.rulebook.md` after SUBMIT transition. The executor does NOT self-validate.

### Checklist

**URL path correctness**
- [ ] C1 — Does `grep -n '"/v1/' src/providers.rs` return 0 matches?
- [ ] C2 — Are all URL path string literals in `src/providers.rs` relative (no leading `/`)?
- [ ] C3 — Are the relative path forms in `providers.rs` consistent with the scheme used in `inference.rs`?

**Compilation and tests**
- [ ] C4 — Does `RUSTFLAGS="-D warnings" cargo build --all-features` complete with 0 errors and 0 warnings?
- [ ] C5 — Does `w3 .test l::3` complete with 0 failures and 0 warnings?

**Out of Scope confirmation**
- [ ] C6 — Is `src/inference.rs` unmodified?
- [ ] C7 — Are zero source files outside `src/providers.rs` modified?

### Measurements

- [ ] M1 — absolute paths removed: `grep -c '"/v1/' src/providers.rs` → 0
- [ ] M2 — relative paths present: `grep -c '"chat/completions"' src/providers.rs` → same count as the former absolute paths (2); confirms `"chat/completions"` is the replacement form (no leading slash, no `v1/` prefix)

### Invariants

- [ ] I1 — test suite: `w3 .test l::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — no leading-slash path strings anywhere in providers.rs: `grep -n '"/' src/providers.rs` → 0 matches (no string literals beginning with `/`)
- [ ] AF2 — inference.rs baseline unchanged: `git diff src/inference.rs` → empty (no changes to the reference file)

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — Task filed by code audit session. Goal: fix runtime URL construction bug in providers.rs where leading-slash absolute paths cause Url::join to strip the base URL path; replace with relative paths matching inference.rs.

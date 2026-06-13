# Add Secret Workspace-Secrets Fallback

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

`src/secret.rs` exposes only `load_from_env(var_name)`, which reads the API key exclusively from an environment variable. Every other crate in the workspace (api_claude, api_gemini, api_xai, api_openai) exposes a `load_with_fallbacks()` pattern that tries the env var first, then falls back to `workspace_tools::workspace()?.load_secret_key("HUGGINGFACE_API_KEY", "-secrets.sh")`. Without this fallback, contributors running integration tests locally must manually `export HUGGINGFACE_API_KEY=...` — they cannot rely on the workspace `secret/-secrets.sh` file that all other crates use. The fix adds `load_with_fallbacks()` to `src/secret.rs` following the existing workspace pattern, and updates the integration test client factory to use it. Observable outcome: `grep -n "fn load_with_fallbacks" src/secret.rs` → ≥ 1 match; `w3 .test l::3` → 0 failures, 0 warnings.

## In Scope

- `src/secret.rs` — add `pub fn load_with_fallbacks() -> Result<Self, ...>` that tries `HUGGINGFACE_API_KEY` env var first, then falls back to `workspace_tools::workspace()?.load_secret_key("HUGGINGFACE_API_KEY", "-secrets.sh")`
- Integration test client factory (wherever `create_test_client()` or equivalent is defined in `tests/`) — update to call `load_with_fallbacks()` instead of `load_from_env()`
- `Cargo.toml` — verify `workspace_tools` is already in `[dependencies]`; add it if missing

## Out of Scope

- Other crates' `secret.rs` implementations — already correct; not touched
- Changing the secret key name or env var name — `HUGGINGFACE_API_KEY` is fixed
- Removing `load_from_env()` — it may be needed by non-test callers; keep it
- Adding a new CLI or configuration mechanism — fallback pattern only

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Pattern must match the workspace convention: env var first, workspace secrets file second, loud error if both missing
- No `cargo fmt`; 2-space indent; custom codestyle throughout
- `load_with_fallbacks()` must return `Err(...)` (not panic) when neither source has the key — tests call `.expect("HUGGINGFACE_API_KEY not found; set env var or add to secret/-secrets.sh")` on the returned `Result`
- All modified code must compile clean under `RUSTFLAGS="-D warnings"`

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note secret loading and workspace_tools patterns.
2. **Read `src/secret.rs`** and a comparable `secret.rs` (e.g., `api/xai/src/secret.rs` or `api/claude/src/secret.rs`) to confirm the `load_with_fallbacks` signature and error type.
3. **Check `Cargo.toml`** — verify `workspace_tools` is listed in `[dependencies]`; add if missing.
4. **Write Test Matrix** — populate every row before writing any test code.
5. **Write a failing test** — write a test confirming `load_with_fallbacks` exists; it fails to compile until the function is added.
6. **Implement** — add `load_with_fallbacks()` to `src/secret.rs`; update test client factory.
7. **Green state** — `w3 .test l::3` → 0 failures, 0 warnings before proceeding.
8. **Submit for Validation** — trigger SUBMIT transition (⏳ → 🔍).
9. **Update task state** — on validation pass, move file to `task/completed/`; update index.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | load_with_fallbacks function defined | `src/secret.rs` source | `grep "fn load_with_fallbacks" src/secret.rs` → ≥ 1 match |
| T02 | Env var takes priority | `HUGGINGFACE_API_KEY` set in env + secrets file present | Returns env var value, not secrets file value |
| T03 | Falls back to workspace secrets | `HUGGINGFACE_API_KEY` not set + valid secrets file | Returns value from `secret/-secrets.sh` |
| T04 | Loud failure on missing credentials | Neither env var nor secrets file | Returns `Err(...)` with actionable message; test client `.expect(...)` panics with message |
| T05 | load_from_env still works | `HUGGINGFACE_API_KEY` set in env | Returns env var value (existing behavior unchanged) |
| T06 | Full test suite passes | All features | `w3 .test l::3` → 0 failures |

## Acceptance Criteria

- `grep -n "fn load_with_fallbacks" src/secret.rs` → ≥ 1 match
- `workspace_tools` present in `[dependencies]` in `Cargo.toml`
- Integration test client factory calls `load_with_fallbacks()` (not only `load_from_env()`)
- `load_from_env()` still exists and is unmodified — backward-compatible addition
- `RUSTFLAGS="-D warnings" cargo build --all-features` → 0 errors, 0 warnings
- `w3 .test l::3` → 0 failures, 0 warnings

## Validation

**Execution:** An independent validator walks this section per `validation.rulebook.md` after SUBMIT transition. The executor does NOT self-validate.

### Checklist

**Function presence**
- [ ] C1 — Does `grep -n "fn load_with_fallbacks" src/secret.rs` return ≥ 1 match?
- [ ] C2 — Does `load_with_fallbacks()` try the env var first before the workspace secrets file?
- [ ] C3 — Does `load_with_fallbacks()` return `Err(...)` (not panic) when neither source has the key?
- [ ] C4 — Does `load_from_env()` still exist and remain unchanged?

**Integration**
- [ ] C5 — Does the integration test client factory call `load_with_fallbacks()` (or a wrapper that calls it)?
- [ ] C6 — Is `workspace_tools` in `[dependencies]` in `Cargo.toml`?

**Compilation and tests**
- [ ] C7 — Does `RUSTFLAGS="-D warnings" cargo build --all-features` complete with 0 errors and 0 warnings?
- [ ] C8 — Does `w3 .test l::3` complete with 0 failures and 0 warnings?

**Out of Scope confirmation**
- [ ] C9 — Are zero other crates' `secret.rs` files modified?
- [ ] C10 — Is the `HUGGINGFACE_API_KEY` key name unchanged?

### Measurements

- [ ] M1 — function defined: `grep -c "fn load_with_fallbacks" src/secret.rs` → ≥ 1
- [ ] M2 — workspace_tools dep: `grep -c "workspace_tools" Cargo.toml` → ≥ 1
- [ ] M3 — test factory updated: `grep -c "load_with_fallbacks" tests/` → ≥ 1 (at least one call site in tests/)

### Invariants

- [ ] I1 — test suite: `w3 .test l::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings
- [ ] I3 — existing API preserved: `grep -c "fn load_from_env" src/secret.rs` → ≥ 1 (not removed)

### Anti-faking checks

- [ ] AF1 — not a stub: `grep -A5 "fn load_with_fallbacks" src/secret.rs` shows actual implementation (env var lookup + workspace fallback), not `unimplemented!()` or `todo!()`
- [ ] AF2 — workspace_tools actually used: `grep -n "workspace_tools" src/secret.rs` → ≥ 1 match (import and usage present)

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — Task filed by code audit session. Goal: add load_with_fallbacks() to src/secret.rs following workspace pattern (env var first, then secret/-secrets.sh via workspace_tools); update integration test client factory to use it.

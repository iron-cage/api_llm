# Export streaming_control via mod_interface

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

`src/lib.rs` declares `pub mod streaming_control` as a bare module declaration outside the `mod_interface!` block, bypassing the crate's canonical export mechanism. The module organization invariant (`docs/invariant/001_module_organization.md`) requires all modules to flow through `mod_interface!` as layer entries. The current bare declaration means `streaming_control` is not re-exported through the standard `mod_interface!` path and is invisible to consumers relying on the crate's structured export surface. The fix is a one-line addition: add `streaming_control` as a layer entry in the `mod_interface!` block under the appropriate feature flag and remove the bare `pub mod` declaration outside the block. Observable outcome: `grep "streaming_control" src/lib.rs | grep "layer"` → ≥ 1 match; no bare `pub mod streaming_control` outside the block; `w3 .test l::3` → 0 failures, 0 warnings.

## In Scope

- `src/lib.rs` — add `streaming_control` as a layer entry inside the `mod_interface!` block (under the `streaming-control` feature flag if the module is feature-gated); remove any bare `pub mod streaming_control` declaration outside the block to eliminate duplication

## Out of Scope

- `src/streaming_control.rs` — module contents are correct; no changes to its implementation
- New public API additions to the streaming_control module — export registration only, no new functionality
- Other modules that may also be missing from mod_interface — separate audit task

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- mod_interface layer declaration follows the pattern: `#[ cfg( feature = "streaming-control" ) ] exposed use streaming_control` (or the feature name used by the crate)
- No `cargo fmt` — custom codestyle applies
- All modified code must compile clean under `RUSTFLAGS="-D warnings"`

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; confirm mod_interface layer syntax and feature-gate conventions.
2. **Read `src/lib.rs`** — identify the existing mod_interface! block structure and the feature flag governing `streaming_control` (check `Cargo.toml` for the feature name).
3. **Write Test Matrix** — populate every row before writing any test code.
4. **Write a failing test** — write a grep-based pattern test that currently fails because streaming_control is not in the mod_interface! block.
5. **Implement** — add streaming_control as a layer entry inside the mod_interface! block; remove the bare `pub mod` declaration.
6. **Green state** — `w3 .test l::3` → 0 failures, 0 warnings before proceeding.
7. **Submit for Validation** — trigger SUBMIT transition (⏳ → 🔍).
8. **Update task state** — on validation pass, move file to `task/completed/`; update index.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | streaming_control in mod_interface! block | `src/lib.rs` source text | `grep "streaming_control" src/lib.rs \| grep "exposed use"` → ≥ 1 match |
| T02 | No bare pub mod outside block | `src/lib.rs` source text | `grep "^pub mod streaming_control" src/lib.rs` → 0 matches |
| T03 | Compile with streaming-control feature | `--features streaming-control` | `RUSTFLAGS="-D warnings" cargo build --features streaming-control` → 0 errors |
| T04 | Full test suite passes | All features | `w3 .test l::3` → 0 failures |

## Acceptance Criteria

- `grep "streaming_control" src/lib.rs | grep "exposed use"` → ≥ 1 match
- `grep "^pub mod streaming_control" src/lib.rs` → 0 matches (no bare declaration outside mod_interface! block)
- `RUSTFLAGS="-D warnings" cargo build --all-features` → 0 errors, 0 warnings
- `w3 .test l::3` → 0 failures, 0 warnings

## Validation

**Execution:** An independent validator walks this section per `validation.rulebook.md` after SUBMIT transition. The executor does NOT self-validate.

### Checklist

**mod_interface registration**
- [ ] C1 — Does `src/lib.rs` contain a layer entry for `streaming_control` inside the `mod_interface!` block?
- [ ] C2 — Is the layer entry gated with the correct feature flag (matching the `streaming-control` feature in `Cargo.toml`)?
- [ ] C3 — Is there zero bare `pub mod streaming_control` declaration outside the `mod_interface!` block in `src/lib.rs`?

**Compilation and tests**
- [ ] C4 — Does `RUSTFLAGS="-D warnings" cargo build --all-features` complete with 0 errors and 0 warnings?
- [ ] C5 — Does `w3 .test l::3` complete with 0 failures and 0 warnings?

**Out of Scope confirmation**
- [ ] C6 — Is `src/streaming_control.rs` unmodified (no changes to module contents)?
- [ ] C7 — Are zero other modules added or removed from `lib.rs` beyond streaming_control?

### Measurements

- [ ] M1 — layer entry present: `grep "streaming_control" src/lib.rs | grep -c "exposed use"` → ≥ 1
- [ ] M2 — bare mod absent: `grep -c "^pub mod streaming_control" src/lib.rs` → 0
- [ ] M3 — lines changed in lib.rs: net change ≤ 3 lines (add layer entry, remove bare pub mod)

### Invariants

- [ ] I1 — test suite: `w3 .test l::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — layer entry not commented out: `grep "streaming_control" src/lib.rs | grep "exposed use" | grep -v "^//"` → ≥ 1 match
- [ ] AF2 — streaming_control module still reachable: `grep -r "use.*streaming_control\|crate::streaming_control" src/ tests/` verifies the module resolves (count > 0 OR crate compiles without dead-code warnings for the module)

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — Task filed by code audit session. Goal: add streaming_control as a layer entry in the mod_interface! block in lib.rs; remove bare pub mod declaration outside the block.

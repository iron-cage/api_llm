# Consolidate Duplicate Chat Types

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** 🎯 (Verified)
- **Priority:** 2
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** src/
- **Validated By:** null
- **Validation Date:** null

## Goal

`src/providers.rs` defines 3 locally-scoped structs that overlap with types in `src/components/inference_shared.rs`: `ChatCompletionRequest` (simplified — no `tools`/`tool_choice` fields), `ChatChoice` (simplified — no `logprobs` field), and `Usage` (named differently from `ChatUsage` in `inference_shared`, which carries additional timing fields). `ChatMessage` is NOT a separate definition — `providers.rs` line 47 is `pub use crate::components::inference_shared::ChatMessage` (already consolidated). Two divergent type shapes for the same domain concept create maintenance risk: when one variant changes the other silently diverges. The goal is to delete the 3 locally-defined structs from `providers.rs` and update all references to use the canonical types from `components::inference_shared` (using `ChatUsage` in place of the local `Usage`). Observable outcome: `grep -n "struct ChatCompletionRequest\|struct ChatChoice\|struct Usage" src/providers.rs` → 0 matches; `w3 .test l::3` → 0 failures, 0 warnings.

## In Scope

- `src/providers.rs` — delete the 3 locally-defined struct types (`ChatCompletionRequest`, `ChatChoice`, `Usage`); add `use` imports from `super::components::inference_shared` for each removed type; use `ChatUsage` (not `Usage`) as the canonical name from `inference_shared`; `ChatMessage` is already consolidated via `pub use` and requires no change
- Any `use` import adjustments in `src/providers.rs` required for the compiler to resolve the types correctly after deletion

## Out of Scope

- `src/components/inference_shared.rs` — canonical source; no changes to its type definitions
- Other files outside `src/providers.rs` that may already import from `inference_shared.rs` — not touched
- API surface changes or additions — structural refactor only, no new functionality
- Other potential duplicate types elsewhere in the crate — separate hygiene task

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Custom code style: 2-space indent; spaces inside `[ ]`, `< >`, `( )`; `#[ derive( ... ) ]` form
- No `cargo fmt` — follow codestyle.rulebook.md conventions throughout
- All modified code must compile clean under `RUSTFLAGS="-D warnings"`

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note code style and no-duplication rules.
2. **Audit types** — grep `src/providers.rs` for all four struct/enum definitions; grep `src/components/inference_shared.rs` to confirm each is defined there identically.
3. **Write Test Matrix** — populate every row before writing any test code.
4. **Write failing tests** — write a compile-time or grep-based test that fails until the duplicates are removed and imports are from `inference_shared`.
5. **Implement** — delete the 3 locally-defined structs (`ChatCompletionRequest`, `ChatChoice`, `Usage`) from `providers.rs`; add `use` imports from `inference_shared` (using `ChatUsage` in place of `Usage`); fix any resulting compilation errors.
6. **Green state** — `w3 .test l::3` → 0 failures, 0 warnings before proceeding.
7. **Submit for Validation** — trigger SUBMIT transition (⏳ → 🔍).
8. **Update task state** — on validation pass, move file to `task/completed/`; update index.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Search for struct defs in providers.rs | `src/providers.rs` source text | `grep "struct ChatCompletionRequest\|struct ChatChoice\|struct Usage" src/providers.rs` → 0 matches (ChatMessage is already a `pub use`, not a struct definition; verify separately) |
| T02 | Confirm canonical defs remain in inference_shared | `src/components/inference_shared.rs` | All 4 types still defined exactly once; unchanged |
| T03 | Compile after deletion | All features enabled | `RUSTFLAGS="-D warnings" cargo build --all-features` → 0 errors, 0 warnings |
| T04 | Full test suite passes | Default + all feature flags | `w3 .test l::3` → 0 failures |

## Acceptance Criteria

- `grep -n "struct ChatCompletionRequest\|struct ChatChoice\|struct Usage" src/providers.rs` → 0 matches (ChatMessage is already a `pub use` re-export — also verify `grep -n "struct ChatMessage" src/providers.rs` → 0)
- `grep -n "use.*inference_shared" src/providers.rs` → ≥ 1 match (confirms import path updated)
- `RUSTFLAGS="-D warnings" cargo build --all-features` → 0 errors, 0 warnings
- `w3 .test l::3` → 0 failures, 0 warnings
- `src/components/inference_shared.rs` is unmodified; it contains `ChatCompletionRequest`, `ChatMessage`, `ChatChoice`, and `ChatUsage` (the canonical usage type — note: `ChatUsage`, not `Usage`)

## Validation

**Execution:** An independent validator walks this section per `validation.rulebook.md` after SUBMIT transition. The executor does NOT self-validate.

### Checklist

**Type consolidation**
- [ ] C1 — Are zero struct definitions for `ChatCompletionRequest`, `ChatChoice`, `Usage` present in `src/providers.rs`? (Verify `ChatMessage` remains a `pub use` re-export, not a new struct definition.)
- [ ] C2 — Does `src/providers.rs` import these types from `components::inference_shared` (or canonical path)?
- [ ] C3 — Are all four types still defined exactly once in `src/components/inference_shared.rs`?

**Compilation and tests**
- [ ] C4 — Does `RUSTFLAGS="-D warnings" cargo build --all-features` complete with 0 errors and 0 warnings?
- [ ] C5 — Does `w3 .test l::3` complete with 0 failures and 0 warnings?

**Out of Scope confirmation**
- [ ] C6 — Is `src/components/inference_shared.rs` unmodified (no new types added, no existing types changed)?
- [ ] C7 — Are zero other src/ files modified beyond `providers.rs`?

### Measurements

- [ ] M1 — duplicate defs removed: `grep -c "struct ChatCompletionRequest\|struct ChatChoice\|struct Usage" src/providers.rs` → 0
- [ ] M2 — canonical defs intact: `grep -c "struct ChatCompletionRequest\|struct ChatMessage\|struct ChatChoice\|struct ChatUsage" src/components/inference_shared.rs` → 4
- [ ] M3 — import added: `grep -c "inference_shared" src/providers.rs` → ≥ 1

### Invariants

- [ ] I1 — test suite: `w3 .test l::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — no type aliases masking duplication: `grep -n "type ChatCompletionRequest\|type ChatMessage\|type ChatChoice\|type Usage" src/providers.rs` → 0 matches
- [ ] AF2 — no re-export indirection: `grep -n "pub use.*ChatCompletion\|pub use.*ChatMessage\|pub use.*ChatChoice\|pub use.*Usage" src/providers.rs` → count ≤ legitimate re-exports (not replacing deleted defs)

## Verification Record

**[2026-06-13]** VERIFY PASS — 4 independent subagents dispatched (D1 Scope Coherence, D2 MOST Goal Quality, D3 Value/YAGNI, D4 Implementation Readiness).
- D1 Scope Coherence: PASS — In Scope (providers.rs) and Out of Scope (inference_shared.rs unchanged) non-empty; observable outcome (grep → 0 matches + clean build); single deliverable (remove 3 structs, add imports).
- D2 MOST Goal Quality: PASS — Motivated (divergent types create silent maintenance risk); Observable (grep commands machine-verifiable); Scoped (providers.rs only); Testable (grep + w3 .test l::3).
- D3 Value / YAGNI: PASS — Duplicate structs confirmed present in codebase; maintenance divergence risk is concrete and immediate; no speculative work.
- D4 Implementation Readiness: PASS — Work Procedure has 8 ordered executable steps; Test Matrix has 4 rows with exact commands; Acceptance Criteria all machine-verifiable grep/build checks.

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — Task filed by code audit session. Goal: remove duplicate ChatCompletionRequest/ChatMessage/ChatChoice/Usage definitions from providers.rs and consolidate to inference_shared.rs as sole source of truth.
- **2026-06-13** `VERIFIED` — MAAV gate passed (4 independent subagents). State → 🎯 (Verified).

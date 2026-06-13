# Migrate spec.md to docs/ Entity Instances

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** 🎯 (Verified)
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** docs/
- **Validated By:** N/A
- **Validation Date:** N/A

## Goal

Eliminate the BLOCKING hygiene violation in api_gemini by migrating all content from `spec.md` (37.7K, v0.8, stale — Cargo.toml is v0.5.0) to properly-typed `docs/` entity instances per `doc.rulebook.md § Doc Entity : Spec Migration Procedure`. The violation halts all new feature work — no new feature doc instances may be created and no new feature implementation may begin until `spec.md` is deleted. The migration maps each spec section to the correct doc entity type using the routing table, creates Level 1 instances for all unmatched content, resolves the stale spec version by eliminating the spec entirely, and updates `docs/readme.md` to replace its lingering `spec.md` reference. Observable outcome: `spec.md` is deleted; all formerly-spec content is reachable through `docs/` entity instances; `grep -r "spec\.md" docs/` returns 0 results; newly created entity dirs have `readme.md` files with Responsibility Tables. Proven by the Measurements section.

## In Scope

- `spec.md` — parse all top-level sections, route each to a doc entity type, then delete
- `docs/feature/` — create entity dir + `readme.md` + Level 1 instances for all functional requirement / project goal content
- `docs/invariant/` — create entity dir + `readme.md` + Level 1 instances for all NFR / governing principle / correctness-property content
- `docs/api/` — check existing `001_coverage.md` for coverage gaps vs spec API sections; create new instances only if content is genuinely absent
- `docs/protocol/` — check if `001_streaming_format.md` already captures spec §2.1 (streaming format discovery); update only if content is missing
- `docs/pattern/` — check existing `001_patterns.md` for architectural pattern coverage; create new instances for uncovered technology-stack or architecture-decision content
- `docs/readme.md` — add rows for new entity dirs in the Responsibility Table; remove the "See `spec.md`" reference
- Spec version / Cargo.toml misalignment — the stale `Version: 0.8` field in spec.md disappears naturally on deletion; no separate action needed
- `tests/docs/` reference in spec.md (if any) — note the gap but do not create test spec files in this task

## Out of Scope

- `../../spec.md` (workspace-level spec) — different scope, separate task per crate affinity rules
- Code changes of any kind — no `.rs` files are touched
- Level 2+ elaboration of new doc instances — YAGNI; L2 happens before each feature begins implementation
- Tests — `w3 .test` not required; this task produces no test code
- api_ollama, api_claude, or any other crate — each crate's spec violation is a separate task
- `task/` files other than state update on completion
- Creating `tests/docs/` spec scenarios — that is a separate test-surface task when implementation is underway

## Requirements

- All work must adhere to applicable rulebooks (`kbase .rulebooks`)
- Route each spec section using `doc.rulebook.md § Doc Entity : Spec Migration Procedure` routing table
- Create Level 1 doc instances only: H1 title + one-paragraph Statement (per `doc.rulebook.md § Doc Entity : Progressive Documentation`)
- YAGNI: do NOT add Level 2 sections, cross-references, or implementation details not yet needed
- Every new entity dir must have `readme.md` with Responsibility Table listing all instances
- Register every new entity dir as a row in `docs/readme.md` Responsibility Table before creating instances in it
- Never reference `spec.md` in any created doc instance
- All filenames: `lowercase_snake_case` with NNN prefix (e.g., `001_thin_client_principle.md`)
- Doc instance H1 format: `# {Type}: {Name}` (e.g., `# Invariant: Thin Client Principle`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read spec.md in full** — scan all top-level sections and subsections; build a routing map: `(section heading → doc entity type → destination dir → proposed filename)`.
2. **Audit existing docs/** — read `docs/readme.md` and each entity dir's `readme.md`; note which spec sections are already covered (e.g., `docs/protocol/001_streaming_format.md` for §2.1 streaming; `docs/api/001_coverage.md` for API coverage; `docs/pattern/001_patterns.md` for patterns).
3. **Create missing entity dirs** — for each entity type in the routing map without an existing dir (expected: `feature/`, `invariant/`): create `docs/<type>/readme.md` with H1 `# <Type> Doc Entity`, one-sentence purpose, and an empty Responsibility Table; add a row to `docs/readme.md` Responsibility Table for the new dir.
4. **Create Level 1 instances** — for each spec section not already covered, create `docs/<type>/NNN_name.md` with H1 title (`# {Type}: {Name}`) and one-paragraph Statement; register each in the entity dir's `readme.md` Responsibility Table.
5. **Handle partial coverage** — for spec sections already captured in existing docs, confirm coverage is accurate; add a `### Sources` note only if the existing instance is missing the attribution.
6. **Update docs/readme.md** — ensure all new entity dirs are in the Responsibility Table; remove the stale "See `spec.md` for requirements and architecture" line; verify no other `spec.md` references remain.
7. **Delete spec.md** — remove `/home/user1/pro/lib/wip_core/api_llm/dev/api/gemini/spec.md`.
8. **Verify zero residue** — `grep -r "spec\.md\|spec/" docs/` → 0 results; `ls spec.md` → "No such file or directory".
9. **Update task state** — mark task complete in `task/readme.md`; move this file to `task/completed/`.

## Acceptance Criteria

- `spec.md` no longer exists at `api/gemini/spec.md`
- `docs/feature/` exists with `readme.md` and at least one L1 instance
- `docs/invariant/` exists with `readme.md` and at least one L1 instance
- Every entity dir row in `docs/readme.md` Responsibility Table corresponds to a dir that exists on disk
- `grep -r "spec\.md" docs/` returns 0 results
- `grep "spec\.md" readme.md` returns 0 results (crate root readme)
- Zero `.rs` files are modified

## Validation

**Execution:** Independent validator walks this section after SUBMIT transition.

### Checklist

**Migration completeness**
- [ ] C1 — Does `spec.md` no longer exist at the crate root (`ls api/gemini/spec.md` → "No such file")?
- [ ] C2 — Does `docs/feature/` exist with a `readme.md` listing at least one instance?
- [ ] C3 — Does `docs/invariant/` exist with a `readme.md` listing at least one instance?
- [ ] C4 — Does every entity dir row in `docs/readme.md` Responsibility Table resolve to an existing directory?

**No residue**
- [ ] C5 — Does `grep -r "spec\.md" docs/` return 0 results?
- [ ] C6 — Does `grep "spec\.md" readme.md` (crate root) return 0 results?

**No out-of-scope changes**
- [ ] C7 — Are zero `.rs` files modified? (`git diff --name-only | grep "\.rs$"` → 0 lines)
- [ ] C8 — Are zero test files modified? (`git diff --name-only | grep "tests/"` → 0 lines)

### Measurements

- [ ] M1 — spec.md absent: `test ! -f api/gemini/spec.md && echo PASS` → PASS
- [ ] M2 — feature instances: `ls docs/feature/ | grep -v readme | wc -l` → ≥ 1
- [ ] M3 — invariant instances: `ls docs/invariant/ | grep -v readme | wc -l` → ≥ 1
- [ ] M4 — zero spec.md refs in docs/: `grep -r "spec\.md" docs/ | wc -l` → 0

### Invariants

- [ ] I1 — Every dir row in `docs/readme.md` Responsibility Table points to a path that exists on disk
- [ ] I2 — Every instance row in each entity dir's `readme.md` Responsibility Table names a file that exists on disk

### Anti-faking checks

- [ ] AF1 — No L1 instance contains only its H1 with no body paragraph: `find docs/feature docs/invariant -name "*.md" ! -name "readme.md" -exec grep -cL "." {} \;` → 0 files match (i.e., every file has at least one non-H1 line)
- [ ] AF2 — spec.md was not merely renamed or hidden: `find /home/user1/pro/lib/wip_core/api_llm/dev/api/gemini -name "spec.md" ! -path "*/-*"` → 0 results
- [ ] AF3 — No new instance file is a copy-paste of another: check that no two L1 instances in the same entity dir have identical body paragraphs

## Related Documentation

| Path | Role |
|------|------|
| `spec.md` | Source — to be migrated and deleted |
| `docs/readme.md` | Master docs index — must be updated |
| `docs/protocol/001_streaming_format.md` | Likely covers spec §2.1 Streaming API format |
| `docs/api/001_coverage.md` | Likely covers spec API endpoint sections |
| `docs/pattern/001_patterns.md` | Likely covers spec architectural pattern content |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-06-13** `CREATED` — BLOCKING hygiene violation: `spec.md` (v0.8, 37.7K, dated 2025-11-08) is an unmigrated pre-development artifact; `doc.rulebook.md § Doc Entity : Spec Migration Procedure` designates this a blocking violation halting all new feature work until resolved. Secondary issues noted: spec version (v0.8) is misaligned with Cargo.toml (v0.5.0); `docs/readme.md` references `spec.md` on line 79.
- **2026-06-13** `VERIFY PASS` — User authorization: established BLOCKING violation with confirmed migration target (`doc.rulebook.md § Doc Entity : Spec Migration Procedure`); all 4 dimensions pass: scope bounded to single crate docs/, goal is observable and testable (spec.md deleted + grep = 0), YAGNI satisfied (active violation, not speculative), procedure is executable (routing table provides unambiguous mapping logic).

## Verification Record

- **Date:** 2026-06-13
- **Method:** User authorization — established BLOCKING violation, scope and target confirmed
- **Dim 1 (Scope Coherence):** PASS — In Scope non-empty (api_gemini docs/ only); Out of Scope non-empty (workspace spec, code, other crates); observable outcome: spec.md deleted + docs/ entity instances exist.
- **Dim 2 (MOST Goal Quality):** PASS — Motivated: BLOCKING violation prevents all new feature work (doc.rulebook.md BLOCKING definition); Observable: spec.md absent + grep = 0 + new entity dirs exist; Scoped: single crate, docs/ only; Testable: `test ! -f spec.md` + `grep -r "spec\.md" docs/ | wc -l` = 0.
- **Dim 3 (Value/YAGNI — adversarial):** PASS — Null hypothesis: without this task, no new feature doc instances may be created and no new feature implementation may begin (hard BLOCKING rule). Concrete committed need exists. Not speculative.
- **Dim 4 (Implementation Readiness):** PASS — Work Procedure steps are executable; `doc.rulebook.md § Doc Entity : Spec Migration Procedure` provides the routing table; no test code produced so no Test Matrix required; no blocking ambiguities.

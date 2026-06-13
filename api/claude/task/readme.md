# api_claude Task System

<!-- task_system_metadata
type: local
crate: api_claude
workspace: api_llm
-->

### Scope

- **Purpose**: Track all implementation tasks for the `api_claude` crate.
- **Responsibility**: One task per bounded unit of work; IDs are permanent and never retired.
- **Task Dir:** `task/` (committed, permanent)

## Tasks Index

| ID | Title | State | Priority | Advisability | Dir |
|----|-------|-------|----------|-------------|-----|
| 001 | [Implement doc test specs](completed/001_implement_doc_test_specs.md) | ✅ | 2 | 180 | `tests/` |
| 002 | [Implement operation test specs](completed/002_implement_operation_test_specs.md) | ✅ | 2 | 180 | `tests/` |
| 003 | [Fix test surface coverage gaps](completed/003_fix_test_surface_gaps.md) | ✅ | 2 | — | `tests/` |

## Global ID Registry

**Highest allocated ID:** 003

## Responsibility Table

| Path | Purpose |
|------|---------|
| `readme.md` | Task system root — Tasks Index and Global ID Registry |
| `decisions.md` | Q-NN decision registry; tracks open questions tasks may close |
| `unverified/` | New task files awaiting VERIFY gate |
| `verified/` | Task files that passed VERIFY; ready for execution |
| `completed/` | Task files fully executed and closed |

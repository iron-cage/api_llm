# api_llm Task System

<!-- task_system_metadata
type: workspace
workspace: api_llm
-->

### Scope

- **Purpose**: Track workspace-level implementation tasks that span multiple crates or govern cross-crate concerns.
- **Responsibility**: One task per bounded unit of work; IDs are permanent and never retired.
- **Task Dir:** `task/` (committed, permanent)

## Tasks Index

| ID | Title | State | Priority | Dir |
|----|-------|-------|----------|-----|
| 001 | [Create Test Surface for Normalized Doc Instances](unverified/001_create_test_surface_for_normalized_docs.md) | ❓ | 3 | `api/{openai,ollama,xai}/tests/docs/` |

## Global ID Registry

**Highest allocated ID:** 001

## Responsibility Table

| Path | Purpose |
|------|---------|
| `readme.md` | Task system root — Tasks Index and Global ID Registry |
| `decisions.md` | Q-NN decision registry; tracks open questions tasks may close |
| `procedure.md` | Task lifecycle procedure |
| `unverified/` | New task files awaiting VERIFY gate |
| `verified/` | Task files that passed VERIFY; ready for execution |
| `completed/` | Task files fully executed and closed |

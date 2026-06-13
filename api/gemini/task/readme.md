# api_gemini Task System

<!-- task_system_metadata
type: local
crate: api_gemini
workspace: api_llm
-->

### Scope

- **Purpose**: Track all implementation tasks for the `api_gemini` crate.
- **Responsibility**: One task per bounded unit of work; IDs are permanent and never retired.
- **Task Dir:** `task/` (committed, permanent)

## Tasks Index

| ID | Title | State | Priority | Advisability | Dir |
|----|-------|-------|----------|-------------|-----|
| 002 | [Remove fabricated Gemini API endpoints](verified/002_remove_fabricated_gemini_api_endpoints.md) | 🎯 | 4 | 2016 | `src/client/api_interfaces/` |
| 004 | [Remove mock data from BatchApi](verified/004_remove_batch_api_mock_data.md) | 🎯 | 4 | 2016 | `src/` |
| 001 | [Migrate spec.md to docs/ entity instances](verified/001_migrate_spec_to_docs.md) | 🎯 | 5 | 1440 | `docs/` |
| 003 | [Fix circuit breaker per-call state reset](verified/003_fix_circuit_breaker_per_call_reset.md) | 🎯 | 4 | 1120 | `src/internal/http/` |
| 006 | [Migrate execute_legacy callers to execute_with_optional_retries](verified/006_migrate_execute_legacy_callers.md) | 🎯 | 3 | 1008 | `src/client/api_interfaces/` |
| 005 | [Resolve clippy allow overrides in Cargo.toml](completed/005_resolve_clippy_allow_overrides.md) | ✅ | 3 | 432 | `src/` |
| 007 | [Implement client enterprise fields](unverified/007_implement_client_enterprise_fields.md) | ❓ | — | — | `src/client/` |
| 008 | [Implement WebSocket send and planned features](unverified/008_implement_websocket_send_and_features.md) | ❓ | — | — | `src/models/` |

## Global ID Registry

**Highest allocated ID:** 008

## Responsibility Table

| Path | Purpose |
|------|---------|
| `readme.md` | Task system root — Tasks Index and Global ID Registry |
| `decisions.md` | Q-NN decision registry; tracks open questions tasks may close |
| `unverified/` | New task files awaiting VERIFY gate |
| `verified/` | Task files that passed VERIFY; ready for execution |
| `completed/` | Task files fully executed and closed |

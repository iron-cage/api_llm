# api_huggingface Task System

<!-- task_system_metadata
type: local
crate: api_huggingface
workspace: api_llm
-->

### Scope

- **Purpose**: Track all implementation tasks for the `api_huggingface` crate.
- **Responsibility**: One task per bounded unit of work; IDs are permanent and never retired.
- **Task Dir:** `task/` (committed, permanent)

## Tasks Index

| Order | ID | Advisability | Value | Easiness | Safety | Priority | State | Executor | Dir | Task | Purpose |
|-------|----|-------------|-------|----------|--------|----------|-------|----------|-----|------|---------|
| 1 | [002](unverified/002_export_streaming_control_via_mod_interface.md) | 1134 | 7 | 9 | 9 | 2 | ❓ (Unverified) | ai | `src/` | [Export streaming_control via mod_interface](unverified/002_export_streaming_control_via_mod_interface.md) | Add streaming_control layer to mod_interface! block in lib.rs |
| 2 | [005](unverified/005_gate_simple_inference_integration_tests.md) | 1008 | 7 | 8 | 9 | 2 | ❓ (Unverified) | ai | `tests/` | [Gate inference_tests.rs integration tests at function level](unverified/005_gate_simple_inference_integration_tests.md) | Dissolve module-level cfg gate in inference_tests.rs; add per-function #[cfg(feature = "integration")] before each #[tokio::test] |
| 3 | [003](unverified/003_fix_url_path_inconsistency.md) | 896 | 8 | 8 | 7 | 2 | ❓ (Unverified) | ai | `src/` | [Fix URL path inconsistency in providers.rs](unverified/003_fix_url_path_inconsistency.md) | Replace absolute /v1/ paths in providers.rs with relative paths matching inference.rs |
| 4 | [001](unverified/001_consolidate_duplicate_chat_types.md) | 588 | 7 | 6 | 7 | 2 | ❓ (Unverified) | ai | `src/` | [Consolidate duplicate chat types](unverified/001_consolidate_duplicate_chat_types.md) | Remove duplicate ChatCompletionRequest/ChatMessage/ChatChoice/Usage defs from providers.rs |
| 5 | [006](unverified/006_add_secret_workspace_fallback.md) | 504 | 7 | 6 | 6 | 2 | ❓ (Unverified) | ai | `src/` | [Add secret workspace-secrets fallback](unverified/006_add_secret_workspace_fallback.md) | Add load_with_fallbacks() to secret.rs aligning with workspace pattern |
| 6 | [004](unverified/004_replace_wiremock_with_real_api.md) | 420 | 6 | 5 | 7 | 2 | ❓ (Unverified) | ai | `tests/` | [Replace wiremock with real API calls](unverified/004_replace_wiremock_with_real_api.md) | Replace wiremock in health_check_tests.rs with real integration tests |
| 7 | [007](unverified/007_test_suite_compliance.md) | 360 | 9 | 5 | 8 | 1 | ❓ (Unverified) | ai | `tests/`, `src/` | [Test suite compliance](unverified/007_test_suite_compliance.md) | Fix 8 violation categories: missing cfg gates, silent passes, helper duplication, wrong model, fn main(), inline src tests |

## Global ID Registry

**Highest allocated ID:** 007

## Responsibility Table

| Path | Purpose |
|------|---------|
| `readme.md` | Task system root — Tasks Index and Global ID Registry |
| `decisions.md` | Q-NN decision registry; tracks open questions tasks may close |
| `unverified/` | New task files awaiting VERIFY gate |
| `verified/` | Task files that passed VERIFY; ready for execution |
| `completed/` | Task files fully executed and closed |
| `cancelled/` | Cancelled task files |
| `actors/` | Actor registry (Promoted Entity Format) |
| `action_plan/` | Per-actor action plans combining task list + conditional actions + watch items |

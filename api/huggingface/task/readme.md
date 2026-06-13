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
| 1 | [003](verified/003_fix_url_path_inconsistency.md) | 896 | 8 | 8 | 7 | 2 | 🎯 (Verified) | ai | `src/` | [Fix URL path inconsistency in providers.rs and inference.rs](verified/003_fix_url_path_inconsistency.md) | Replace absolute path literals in providers.rs (/v1/chat/completions) and inference.rs lines 204, 233 (/models/) with relative paths; Url::join strips /v1/ from base URL when path starts with / |
| 2 | [008](verified/008_implement_doc_spec_test_coverage.md) | 648 | 6 | 6 | 9 | 2 | 🎯 (Verified) | ai | `tests/` | [Implement GWT spec tests for 28 doc entity scenarios](verified/008_implement_doc_spec_test_coverage.md) | Create tests/doc_spec_tests.rs with one named function per scenario (FE-01..04, AP-01..06, OP-01..06, IN-01..08, PT-01..04) |
| 3 | [001](verified/001_consolidate_duplicate_chat_types.md) | 588 | 7 | 6 | 7 | 2 | 🎯 (Verified) | ai | `src/` | [Consolidate duplicate chat types](verified/001_consolidate_duplicate_chat_types.md) | Remove duplicate ChatCompletionRequest/ChatMessage/ChatChoice/Usage defs from providers.rs |
| 4 | [006](unverified/006_add_secret_workspace_fallback.md) | 504 | 7 | 6 | 6 | 2 | ❓ (Unverified) | ai | `src/` | [Add secret workspace-secrets fallback](unverified/006_add_secret_workspace_fallback.md) | Add load_with_fallbacks() to secret.rs aligning with workspace pattern |
| 5 | [004](verified/004_replace_wiremock_with_real_api.md) | 420 | 6 | 5 | 7 | 2 | 🎯 (Verified) | ai | `tests/` | [Replace wiremock with real API calls](verified/004_replace_wiremock_with_real_api.md) | Replace wiremock in health_check_tests.rs with real integration tests |
| 6 | [007](unverified/007_test_suite_compliance.md) | 360 | 9 | 5 | 8 | 1 | ❓ (Unverified) | ai | `tests/`, `src/` | [Test suite compliance](unverified/007_test_suite_compliance.md) | Fix 6 active violation categories: missing cfg gates, silent passes, helper duplication, wrong model, inline src tests (V06 pre-resolved — target file deleted) |
| 7 | [009](verified/009_implement_collection_pitfall_spec_tests.md) | 432 | 6 | 6 | 9 | 2 | 🎯 (Verified) | ai | `tests/` | [Implement CL and PF GWT spec tests](verified/009_implement_collection_pitfall_spec_tests.md) | Add 9 named functions (test_cl_01..05, test_pf_01..04) to tests/doc_spec_tests.rs; PF-01/02/04 use todo!() pending task 003 |
| — | [002](cancelled/002_export_streaming_control_via_mod_interface.md) | — | — | — | — | — | ❌ (Cancelled) | — | — | [Export streaming_control via mod_interface](cancelled/002_export_streaming_control_via_mod_interface.md) | VERIFY FAIL — false violation; pub mod pattern is correct per docs/pattern/001_module_organization.md |
| — | [005](completed/005_gate_simple_inference_integration_tests.md) | — | — | — | — | 0 | ✅ (Completed) | ai | `tests/` | [Gate inference_tests.rs integration tests at function level](completed/005_gate_simple_inference_integration_tests.md) | Dissolve module-level cfg gate in inference_tests.rs; add per-function #[cfg(feature = "integration")] before each #[tokio::test] |

## Global ID Registry

**Highest allocated ID:** 009

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

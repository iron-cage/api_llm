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
| — | [001](completed/001_consolidate_duplicate_chat_types.md) | — | — | — | — | — | ✅ (Completed) | ai | `src/` | [Consolidate duplicate chat types](completed/001_consolidate_duplicate_chat_types.md) | Remove duplicate ChatCompletionRequest/ChatMessage/ChatChoice/Usage defs from providers.rs |
| — | [006](cancelled/006_add_secret_workspace_fallback.md) | — | — | — | — | — | ❌ (Cancelled) | — | — | [Add secret workspace-secrets fallback](cancelled/006_add_secret_workspace_fallback.md) | YAGNI — workspace secret pattern already available via workspace_tools |
| — | [004](completed/004_replace_wiremock_with_real_api.md) | — | — | — | — | — | ✅ (Completed) | ai | `tests/` | [Replace wiremock with real API calls](completed/004_replace_wiremock_with_real_api.md) | Replace wiremock in health_check_tests.rs with real integration tests |
| — | [007](completed/007_test_suite_compliance.md) | — | — | — | — | — | ✅ (Completed) | ai | `tests/`, `src/` | [Test suite compliance](completed/007_test_suite_compliance.md) | Fix 6 active violation categories: missing cfg gates, silent passes, helper duplication, wrong model, inline src tests |
| — | [002](cancelled/002_export_streaming_control_via_mod_interface.md) | — | — | — | — | — | ❌ (Cancelled) | — | — | [Export streaming_control via mod_interface](cancelled/002_export_streaming_control_via_mod_interface.md) | VERIFY FAIL — false violation; pub mod pattern is correct per docs/pattern/001_module_organization.md |
| — | [003](completed/003_fix_url_path_inconsistency.md) | — | — | — | — | — | ✅ (Completed) | ai | `src/` | [Fix URL path inconsistency in providers.rs and inference.rs](completed/003_fix_url_path_inconsistency.md) | Replace absolute path literals in providers.rs and inference.rs with relative paths |
| — | [005](completed/005_gate_simple_inference_integration_tests.md) | — | — | — | — | 0 | ✅ (Completed) | ai | `tests/` | [Gate inference_tests.rs integration tests at function level](completed/005_gate_simple_inference_integration_tests.md) | Dissolve module-level cfg gate in inference_tests.rs; add per-function #[cfg(feature = "integration")] before each #[tokio::test] |
| — | [008](completed/008_implement_doc_spec_test_coverage.md) | — | — | — | — | — | ✅ (Completed) | ai | `tests/` | [Implement GWT spec tests for 28 doc entity scenarios](completed/008_implement_doc_spec_test_coverage.md) | Create tests/doc_spec_tests.rs with one named function per scenario (FE-01..04, AP-01..06, OP-01..06, IN-01..08, PT-01..04) |
| — | [009](completed/009_implement_collection_pitfall_spec_tests.md) | — | — | — | — | — | ✅ (Completed) | ai | `tests/` | [Implement CL and PF GWT spec tests](completed/009_implement_collection_pitfall_spec_tests.md) | Add 9 named functions (test_cl_01..05, test_pf_01..04) to tests/doc_spec_tests.rs |
| — | [010](completed/010_expand_doc_spec_test_coverage.md) | — | — | — | — | — | ✅ (Completed) | ai | `tests/` | [Expand GWT spec test coverage — AP-07, FE-06, CL-06, CL-07](completed/010_expand_doc_spec_test_coverage.md) | Add 4 named functions to tests/doc_spec_tests.rs implementing the 4 scenarios added in the 2026-06-13 doc_tsk session |

## Global ID Registry

**Highest allocated ID:** 010

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
| `bug/` | Bug reports index; all resolved bugs archived in `bug/closed/` |

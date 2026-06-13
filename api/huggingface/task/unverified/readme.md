# Task Inbox (Unverified)

Holds task files awaiting the VERIFY gate. A task arrives here on creation and leaves when VERIFY PASS promotes it to `verified/`.

### Responsibility Table

| File | Purpose |
|------|---------|
| `readme.md` | This file — directory responsibility and lifecycle note |
| `001_consolidate_duplicate_chat_types.md` | Remove duplicate type defs from providers.rs |
| `002_export_streaming_control_via_mod_interface.md` | Add streaming_control to mod_interface! block |
| `003_fix_url_path_inconsistency.md` | Fix absolute /v1/ paths in providers.rs |
| `004_replace_wiremock_with_real_api.md` | Replace wiremock in health_check_tests.rs |
| `005_gate_simple_inference_integration_tests.md` | Add cfg integration gates to test functions |
| `006_add_secret_workspace_fallback.md` | Add load_with_fallbacks() to secret.rs |

*(Task files are listed in `../readme.md` Tasks Index.)*

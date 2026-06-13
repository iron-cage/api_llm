# Task Inbox (Unverified)

Holds task files awaiting the VERIFY gate. A task arrives here on creation and leaves when VERIFY PASS promotes it to `verified/`.

### Responsibility Table

| File | Purpose |
|------|---------|
| `readme.md` | This file — directory responsibility and lifecycle note |
| `001_consolidate_duplicate_chat_types.md` | Remove duplicate type defs from providers.rs |
| `003_fix_url_path_inconsistency.md` | Fix absolute URL path literals in providers.rs and inference.rs |
| `004_replace_wiremock_with_real_api.md` | Replace wiremock in health_check_tests.rs |
| `006_add_secret_workspace_fallback.md` | Add load_with_fallbacks() to secret.rs |
| `007_test_suite_compliance.md` | Fix 6 active test suite violation categories |
| `008_implement_doc_spec_test_coverage.md` | Implement 28 GWT spec test functions in doc_spec_tests.rs |

*(Task files are listed in `../readme.md` Tasks Index.)*

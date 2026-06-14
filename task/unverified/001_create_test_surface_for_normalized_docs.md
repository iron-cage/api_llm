# Task 001: Create Test Surface for Normalized Doc Instances

## Execution State

- **State:** ❓ (Unverified)
- **Executor:** AI
- **Closes:** null

## Goal

- **Motivated:** Three crates (api_openai, api_ollama, api_xai) each have 6 normalized doc instances in `docs/` but zero corresponding `tests/docs/` test specification files. The 4 other crates (api_claude, api_gemini, api_huggingface, api_openai_compatible) all have `tests/docs/` specs — this gap breaks cross-crate parity and leaves 18 doc instances without a test surface to verify their behavioral claims.
- **Observable:** Each of the 3 target crates contains a `tests/docs/` directory tree mirroring `docs/`, with one spec file per doc instance using the `# {Type} Spec: {Name}` H1 convention. Each spec file has an Overview Table and Given/When/Then scenarios. Corresponding test functions exist in `tests/` source files and pass at Level 3 verification.
- **Scoped:** 3 crates x 6 doc instances = 18 spec files + corresponding test implementations. No changes to docs/, no changes to src/, no changes to crates that already have test surfaces.
- **Testable:** `cargo nextest run --all-features -p api_openai`, `-p api_ollama`, `-p api_xai` each include the new test functions; all pass. `find api/{openai,ollama,xai}/tests/docs -name "*.md" | wc -l` returns 18.

## Scope

- **In Scope:**
  - Create `tests/docs/` directory trees in api_openai, api_ollama, api_xai
  - Write test specification files (Given/When/Then scenarios) for all 18 doc instances
  - Implement test functions in `tests/` source files matching each scenario
  - Follow the `# {Type} Spec: {Name}` H1 convention used by existing test specs
- **Out of Scope:**
  - Modifying any `docs/` content (already normalized)
  - Modifying any `src/` code
  - Adding test specs to crates that already have them (claude, gemini, huggingface, openai_compatible)
  - Creating or modifying task systems in individual crates

## Requirements

- Test spec files follow the format established in `api/openai_compatible/tests/docs/` and `api/claude/tests/docs/`
- Scenario IDs use the same prefix convention: `IN-NN` (invariant), `FT-NN` (feature), `AP-NN` (api), `PT-NN` (pattern), `OP-NN` (operation)
- All tests must be real-implementation tests — no mocking
- Integration tests gated with `#[cfg(feature = "integration")]` where they require API calls; these tests require provider API keys (`OPENAI_API_KEY`, `XAI_API_KEY`) or a running Ollama server
- Pure structural/serde tests (e.g., thin client invariant checks) need no feature gate and run without credentials
- Test function naming follows crate convention (descriptive names acceptable; `test_{prefix}_{nn}` naming optional)

## Work Procedure

1. For each target crate (api_openai, api_ollama, api_xai):
   a. Create the `tests/docs/` directory tree: `tests/docs/{api,feature,invariant,operation,pattern}/`
   b. Read the corresponding `docs/{type}/NNN_name.md` to extract behavioral claims
   c. Write `tests/docs/{type}/NN_name.md` spec file with Overview Table and Given/When/Then scenarios
   d. Implement test functions: add to existing `tests/*.rs` files when the file already covers the same domain (e.g., add thin-client serde checks to an existing wire/serialization test file); create a new `tests/doc_spec_*.rs` file only when no existing file covers the domain
   e. Run `cargo nextest run --all-features -p api_{crate}` to verify tests pass
2. Run Level 3 verification per crate: `RUSTFLAGS="-D warnings" cargo nextest run --all-features -p api_{crate} && RUSTDOCFLAGS="-D warnings" cargo test --doc --all-features -p api_{crate} && cargo clippy --all-targets --all-features -p api_{crate} -- -D warnings`

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|---------------|-------------------|-------------------|
| api_openai thin client invariant | Default ChatRequest with only model+messages | Serialized JSON has no extra keys; no implicit defaults injected |
| api_openai enterprise feature flags | Feature flag enabled, no explicit config | Zero enterprise behavior activated without builder call |
| api_openai endpoint coverage | Each API endpoint method | Method exists and maps to correct HTTP verb + path |
| api_openai async patterns | Async method signatures | All public methods are async; sync wrappers behind feature flag |
| api_openai semantic versioning | Version string in Cargo.toml | Follows semver; public API changes tracked |
| api_ollama thin client invariant | Default request serialization | Optional fields absent from JSON when None |
| api_ollama enterprise feature flags | Feature flag enabled, no explicit config | Zero enterprise behavior activated automatically |
| api_ollama endpoint coverage | Each API endpoint method | Method exists and maps to correct HTTP verb + path |
| api_ollama module organization | Public module structure | Modules follow `client_ext_*.rs` pattern; `mod_interface` used |
| api_ollama secret loading | Secret::load_with_fallbacks | Loads from env var or workspace secrets |
| api_xai thin client invariant | Default request serialization | Optional fields absent from JSON when None |
| api_xai enterprise feature flags | Feature flag enabled, no explicit config | Zero enterprise behavior activated automatically |
| api_xai endpoint coverage | Each API endpoint method | Method exists and maps to correct HTTP verb + path |
| api_xai module organization | Public module structure | Components, environment, error modules properly organized |
| api_xai secret loading | Secret::load_with_fallbacks("XAI_API_KEY") | Loads from env var or workspace secrets |

## Acceptance Criteria

- AC-001: Each of the 3 target crates has exactly 6 test spec files under `tests/docs/`, one per doc instance
- AC-002: Every spec file uses `# {Type} Spec: {Name}` H1 format matching the convention in api_claude and api_openai_compatible
- AC-003: Every spec file has an Overview Table with ID, Name, Category, and Status columns
- AC-004: Every scenario has Given/When/Then structure with specific, testable assertions
- AC-005: Corresponding test functions exist in `tests/` source files covering each spec scenario
- AC-006: All 3 crates pass Level 3 verification after test implementation
- AC-007: No changes to `docs/` content or `src/` code in any crate

## Related Documentation

- `api/openai/docs/invariant/001_thin_client_principle.md` — source for openai invariant spec
- `api/openai/docs/invariant/002_testing_standards.md` — source for openai testing invariant spec
- `api/openai/docs/api/001_endpoint_coverage.md` — source for openai api spec
- `api/openai/docs/feature/001_enterprise_reliability.md` — source for openai feature spec
- `api/openai/docs/pattern/001_async_patterns.md` — source for openai pattern spec
- `api/openai/docs/operation/001_semantic_versioning.md` — source for openai operation spec
- `api/ollama/docs/invariant/001_thin_client_principle.md` — source for ollama invariant spec
- `api/ollama/docs/invariant/002_testing_standards.md` — source for ollama testing invariant spec
- `api/ollama/docs/api/001_endpoint_coverage.md` — source for ollama api spec
- `api/ollama/docs/feature/001_enterprise_reliability.md` — source for ollama feature spec
- `api/ollama/docs/pattern/001_module_organization.md` — source for ollama pattern spec
- `api/ollama/docs/operation/001_secret_loading.md` — source for ollama operation spec
- `api/xai/docs/invariant/001_thin_client_principle.md` — source for xai invariant spec
- `api/xai/docs/invariant/002_testing_standards.md` — source for xai testing invariant spec
- `api/xai/docs/api/001_endpoint_coverage.md` — source for xai api spec
- `api/xai/docs/feature/001_enterprise_reliability.md` — source for xai feature spec
- `api/xai/docs/pattern/001_module_organization.md` — source for xai pattern spec
- `api/xai/docs/operation/001_secret_loading.md` — source for xai operation spec
- `api/openai_compatible/tests/docs/` — reference implementation for test spec format
- `api/claude/tests/docs/` — reference implementation for test spec format

## History

- **[2026-06-14]** `CREATED` — Create tests/docs/ test specifications and implementations for 18 doc instances across api_openai, api_ollama, and api_xai to close cross-crate test surface parity gap.
- **[2026-06-14]** `VERIFY-R1` — First MAAV round: Scope Coherence PASS, MOST Goal PASS, Value/YAGNI FAIL (adversarial agent cited openai_compatible having no test implementations — factually wrong, 36 tests exist with descriptive names; self-referential meta-tests removed), Implementation Readiness FAIL (missing credential prerequisites, ambiguous file placement). Fixed: added credential requirements, clarified file placement rule, removed meta-test rows, relaxed naming convention to descriptive names.
- **[2026-06-14]** `VERIFY-R2` — Second MAAV round: Scope Coherence PASS, MOST Goal PASS, Implementation Readiness PASS (all R1 fixes validated). Value/YAGNI FAIL (3/4): adversarial agent argues the 3 target crates already have 1,187 tests that verify behavioral claims; tests/docs/ adds documentation symmetry but no net new verification; no committed consumer process depends on these artifacts; effort is disproportionate to marginal gain. Task remains ❓ Unverified pending user decision on YAGNI override.

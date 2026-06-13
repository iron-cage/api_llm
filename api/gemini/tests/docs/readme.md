# Docs Test Surface

### Scope

- **Purpose**: Define the top-level structure of behavioral test specs derived from `docs/` entity instances in `api_gemini`.
- **Responsibility**: Each subdirectory mirrors one `docs/` entity type; each spec file maps one-to-one to a doc instance.
- **In Scope**: All test specs in `tests/docs/` — behavioral verification of invariants, API contracts, feature specs, operation procedures, patterns, and protocol specs.
- **Out of Scope**: Source-level unit tests in `tests/inc/`; workspace-level behavioral tests outside `api_gemini`.

### Overview Table

| Directory | Mirrors | Purpose | Status |
|-----------|---------|---------|--------|
| [`invariant/`](invariant/readme.md) | `docs/invariant/` | Behavioral compliance tests for non-negotiable design constraints | ✅ |
| [`feature/`](feature/readme.md) | `docs/feature/` | Configuration and activation tests for core and enterprise feature specs | ✅ |
| [`api/`](api/readme.md) | `docs/api/` | Coverage verification tests for all Gemini API endpoints | ✅ |
| [`operation/`](operation/readme.md) | `docs/operation/` | Step-by-step scenario tests for operational procedures | ✅ |
| [`pattern/`](pattern/readme.md) | `docs/pattern/` | Structural conformance tests for mandated code patterns | ✅ |
| [`protocol/`](protocol/readme.md) | `docs/protocol/` | Wire format compliance tests for streaming protocol specs | ✅ |

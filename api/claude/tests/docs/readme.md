# Docs Test Surface

### Scope

- **Purpose**: Define the top-level structure of behavioral test specs derived from `docs/` entity instances in `api_claude`.
- **Responsibility**: Each subdirectory mirrors one `docs/` entity type; each spec file maps one-to-one to a doc instance.
- **In Scope**: All test specs in `tests/docs/` — behavioral verification of invariants, API contracts, feature specs, operation procedures, and patterns.
- **Out of Scope**: Source-level unit tests in `tests/inc/`; workspace-level behavioral tests outside `api_claude`.

### Overview Table

| Directory | Mirrors | Purpose | Status |
|-----------|---------|---------|--------|
| [`invariant/`](invariant/readme.md) | `docs/invariant/` | Behavioral compliance tests for non-negotiable design constraints | ✅ |
| [`api/`](api/readme.md) | `docs/api/` | Coverage verification tests for all Anthropic API endpoints | ✅ |
| [`feature/`](feature/readme.md) | `docs/feature/` | Configuration and activation tests for enterprise reliability features | ✅ |
| [`operation/`](operation/readme.md) | `docs/operation/` | Step-by-step scenario tests for operational procedures | ✅ |
| [`pattern/`](pattern/readme.md) | `docs/pattern/` | Structural conformance tests for mandated code patterns | ✅ |

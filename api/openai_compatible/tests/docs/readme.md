# Docs Test Surface

### Scope

- **Purpose**: Define the top-level structure of behavioral test specs derived from `docs/` entity instances in `api_openai_compatible`.
- **Responsibility**: Each subdirectory mirrors one `docs/` entity type; each spec file maps one-to-one to a doc instance.
- **In Scope**: All test specs in `tests/docs/` — behavioral verification of invariants, API wire contracts, and feature behavioral requirements.
- **Out of Scope**: Source-level unit tests in `tests/`; workspace-level behavioral tests outside `api_openai_compatible`.

### Overview Table

| Directory | Mirrors | Purpose | Status |
|-----------|---------|---------|--------|
| [`invariant/`](invariant/readme.md) | `docs/invariant/` | Behavioral compliance tests for non-negotiable design constraints | ✅ |
| [`feature/`](feature/readme.md) | `docs/feature/` | Configuration and activation tests for opt-in feature behavioral requirements | ✅ |
| [`api/`](api/readme.md) | `docs/api/` | Wire contract verification tests for endpoint coverage and chat completion | ✅ |
| [`pattern/`](pattern/readme.md) | `docs/pattern/` | Module organization, generic client design, and feature-gated layer tests | ✅ |

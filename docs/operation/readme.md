# Operation Doc Entity

### Scope

- **Purpose**: Document workspace-level operational procedures for `api_llm`.
- **Responsibility**: Master file listing all operation doc instances with ID, name, and status.
- **In Scope**: Workspace-wide operational workflows — secret loading, dependency management, running tests.
- **Out of Scope**: Crate-specific operations (see `api/*/docs/operation/` in each crate).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Secret Loading](001_secret_loading.md) | Workspace-wide API key management and secret loading procedure | ✅ |

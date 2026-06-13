# docs

### Scope

- **Purpose**: Workspace-level documentation for the `api_llm` workspace.
- **Responsibility**: Master file listing all doc entities and instances governing the entire workspace.
- **In Scope**: Workspace-wide invariants, operational procedures, and governing principles applying to all crates.
- **Out of Scope**: Crate-specific documentation (see `api/*/docs/` in each crate).

### Responsibility Table

| Path | Responsibility |
|------|---------------|
| `readme.md` | Workspace docs master file — entity index |
| `invariant/` | Non-negotiable workspace-wide behavioral constraints |
| `operation/` | Workspace-level operational procedures |

### Collections

| Entity | Instances |
|--------|-----------|
| [invariant/](invariant/readme.md) | 001 Thin Client Principle, 002 Testing Standards |
| [operation/](operation/readme.md) | 001 Secret Loading |

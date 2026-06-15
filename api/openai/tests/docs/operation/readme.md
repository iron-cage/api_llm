# Operation Test Surface

### Scope

- **Purpose**: Define test cases that verify compliance with operational procedures documented in `docs/operation/` instances.
- **Responsibility**: Each spec file maps one-to-one to an operation doc instance; all release validation gates must have corresponding spec entries.
- **In Scope**: Compliance scenarios for `docs/operation/001_semantic_versioning.md`.
- **Out of Scope**: API wire contract tests (see `tests/docs/api/`); feature behavioral tests (see `tests/docs/feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 01 | [`01_semantic_versioning.md`](01_semantic_versioning.md) | Verify semver format, compilation, and documentation gates -- OP-01..OP-04 (4 scenarios) | ✅ |

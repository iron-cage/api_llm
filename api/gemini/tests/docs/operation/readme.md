# Operation Test Surface

### Scope

- **Purpose**: Define test cases that verify step-by-step operational procedures documented in `docs/operation/` instances.
- **Responsibility**: Each spec file maps one-to-one to an operation instance; all procedure steps and expected outcomes must have corresponding spec entries.
- **In Scope**: Compliance scenarios for `docs/operation/002_usage_examples.md`.
- **Out of Scope**: Source implementation behavior (see `tests/inc/`); pattern structural conformance (see `tests/docs/pattern/`); testing policy (see `tests/readme.md`); cookbook navigation (external resource, not design knowledge).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 02 | [`002_usage_examples.md`](002_usage_examples.md) | Verify usage example procedure steps for each API call pattern — OP-06..OP-14 (9 scenarios) | ✅ |

# Pattern Test Surface

### Scope

- **Purpose**: Define test cases that verify adherence to design patterns documented in `docs/pattern/` instances.
- **Responsibility**: Each spec file maps one-to-one to a pattern doc instance; all canonical async forms must have corresponding spec entries.
- **In Scope**: Compliance scenarios for `docs/pattern/001_async_patterns.md`.
- **Out of Scope**: API wire contract tests (see `tests/docs/api/`); enterprise feature tests (see `tests/docs/feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 01 | [`01_async_patterns.md`](01_async_patterns.md) | Verify typed async method signatures and streaming channel pattern -- PT-01..PT-03 (3 scenarios) | ⏳ |

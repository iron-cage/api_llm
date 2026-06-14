# Feature Test Surface

### Scope

- **Purpose**: Define test cases that verify behavioral compliance with feature requirements documented in `docs/feature/` instances.
- **Responsibility**: Each spec file maps one-to-one to a feature doc instance; all activation and isolation behaviors must have corresponding spec entries.
- **In Scope**: Compliance scenarios for `docs/feature/001_enterprise_reliability.md`.
- **Out of Scope**: API wire contract tests (see `tests/docs/api/`); invariant enforcement tests (see `tests/docs/invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 01 | [`01_enterprise_reliability.md`](01_enterprise_reliability.md) | Verify enterprise feature isolation, activation, and no-auto-behavior policy -- FT-01..FT-04 (4 scenarios) | ⏳ |

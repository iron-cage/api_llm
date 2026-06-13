# API Test Surface

### Scope

- **Purpose**: Define test cases that verify coverage of all Gemini API endpoints and advanced feature families documented in `docs/api/` instances.
- **Responsibility**: Each spec file maps one-to-one to an API coverage instance; every endpoint row must have a corresponding spec entry.
- **In Scope**: Compliance scenarios for `docs/api/001_coverage.md`.
- **Out of Scope**: Wire-format compliance (see `tests/docs/protocol/`); operation procedures (see `tests/docs/operation/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 01 | [`001_coverage.md`](001_coverage.md) | Verify all core endpoints and advanced API families are covered — AP-01..AP-12 (12 scenarios) | ✅ |

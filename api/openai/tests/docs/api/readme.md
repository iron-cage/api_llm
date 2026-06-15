# API Test Surface

### Scope

- **Purpose**: Define test cases that verify wire contract compliance with API endpoint coverage documented in `docs/api/` instances.
- **Responsibility**: Each spec file maps one-to-one to an API doc instance; all endpoint behavioral contracts must have corresponding spec entries.
- **In Scope**: Compliance scenarios for `docs/api/001_endpoint_coverage.md`.
- **Out of Scope**: Enterprise reliability tests (see `tests/docs/feature/`); operational procedure tests (see `tests/docs/operation/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 01 | [`01_endpoint_coverage.md`](01_endpoint_coverage.md) | Verify endpoint method signatures, feature gating, and error contract -- AP-01..AP-04 (4 scenarios) | ✅ |

# Invariant Test Surface

### Scope

- **Purpose**: Define test cases that verify behavioral compliance with non-negotiable design constraints in `docs/invariant/` instances.
- **Responsibility**: Each spec file maps one-to-one to an invariant instance; all behavioral assertions in an instance must have corresponding spec entries.
- **In Scope**: Compliance scenarios for `docs/invariant/001_thin_client_principle.md` and `docs/invariant/002_testing_standards.md`.
- **Out of Scope**: Source-level unit tests in `tests/`; API wire contract tests (see `tests/docs/api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 01 | [`01_thin_client_principle.md`](01_thin_client_principle.md) | Verify no automatic or implicit client behaviors exist -- IN-01..IN-03 (3 scenarios) | ✅ |
| 02 | [`02_testing_standards.md`](02_testing_standards.md) | Verify no-mock mandate and loud-failure requirement -- IN-01..IN-02 (2 scenarios) | ✅ |

# Invariant Doc Entity

### Scope

- **Purpose**: Document non-negotiable workspace-wide behavioral constraints governing every crate in `api_llm`.
- **Responsibility**: Master file listing all invariant instances with ID, name, and enforcement status.
- **In Scope**: Workspace-level governing principles that apply to all crates — thin client mandate, testing policy.
- **Out of Scope**: Crate-specific invariants (see `api/*/docs/invariant/` in each crate).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Thin Client Principle](001_thin_client_principle.md) | All API crates are stateless HTTP transports; no automatic or implicit behaviors workspace-wide | ✅ |
| 002 | [Testing Standards](002_testing_standards.md) | No-mock mandate and loud-failure requirement for all integration tests workspace-wide | ✅ |

# Invariant Doc Entity

### Scope

- **Purpose**: Document non-negotiable behavioral constraints governing every implementation decision in `api_claude`.
- **Responsibility**: Master file listing all invariant instances with ID, name, and enforcement status.
- **In Scope**: Design-level constraints applying universally across the crate — governing principles, test mandates, policy rules.
- **Out of Scope**: Feature-specific behaviors (see feature/), API contracts (see api/), implementation patterns (see pattern/).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Thin Client Principle](001_thin_client_principle.md) | No automatic or implicit behaviors; all client actions must be explicit | ✅ |
| 002 | [Testing Standards](002_testing_standards.md) | No-mock mandate and loud-failure requirement for all tests | ✅ |

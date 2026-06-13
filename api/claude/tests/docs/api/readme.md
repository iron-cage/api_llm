# API Test Surface

### Scope

- **Purpose**: Define test cases that verify Anthropic API endpoint coverage and feature-gating policy per `docs/api/` instances.
- **Responsibility**: Each spec file maps one-to-one to an API coverage instance; all listed endpoints must have corresponding spec entries.
- **In Scope**: Coverage scenarios for `docs/api/001_endpoint_coverage.md` — core endpoints, feature-gated endpoints, and policy verification.
- **Out of Scope**: Source-level unit tests in `tests/inc/`; enterprise feature tests (see `tests/docs/feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 01 | [`01_endpoint_coverage.md`](01_endpoint_coverage.md) | Verify all core and feature-gated API endpoints are callable with correct signatures — 12 scenarios | ✅ |

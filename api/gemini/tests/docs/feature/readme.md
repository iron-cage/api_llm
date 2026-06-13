# Feature Test Surface

### Scope

- **Purpose**: Define test cases that verify correct activation and behavior of features documented in `docs/feature/` instances.
- **Responsibility**: Each spec file maps one-to-one to a feature instance; activation constraints and behavioral contracts must have corresponding spec entries.
- **In Scope**: Compliance scenarios for `docs/feature/001_core_api.md` and `docs/feature/002_enterprise_reliability.md`.
- **Out of Scope**: Wire-format compliance (see `tests/docs/protocol/`); endpoint URL coverage (see `tests/docs/api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 01 | [`001_core_api.md`](001_core_api.md) | Verify core Gemini API endpoint behaviors and content capabilities — FT-01..FT-08 (8 scenarios) | ✅ |
| 02 | [`002_enterprise_reliability.md`](002_enterprise_reliability.md) | Verify enterprise feature activation contracts and dispatch behavior — FT-09..FT-16 (8 scenarios) | ✅ |

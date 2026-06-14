# Feature Test Surface

### Scope

- **Purpose**: Define test cases that verify behavioral requirements for opt-in features in `docs/feature/` instances.
- **Responsibility**: Each spec file maps one-to-one to a feature instance; all behavioral constraints in an instance must have corresponding spec entries.
- **In Scope**: Behavioral scenarios for `docs/feature/001_streaming.md` and `docs/feature/002_sync_api.md`.
- **Out of Scope**: Source-level unit tests in `tests/`; API wire contract tests (see `tests/docs/api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [`001_streaming.md`](001_streaming.md) | Verify SSE chunk wire types and streaming behavioral constraints — FT-01..FT-06 (6 scenarios) | ✅ |
| 002 | [`002_sync_api.md`](002_sync_api.md) | Verify SyncClient construction, URL routing, and blocking semantics — FT-07..FT-09 (3 scenarios) | ✅ |

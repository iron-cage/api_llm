# API Test Surface

### Scope

- **Purpose**: Define test cases that verify the wire contract for API endpoints documented in `docs/api/` instances.
- **Responsibility**: Each spec file maps one-to-one to an API doc instance; all request/response shape assertions must have corresponding spec entries.
- **In Scope**: Wire contract scenarios for all `docs/api/` instances.
- **Out of Scope**: Source-level unit tests in `tests/`; invariant compliance tests (see `tests/docs/invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [`001_endpoint_coverage.md`](001_endpoint_coverage.md) | Verify Client methods, environment headers, SyncClient — AP-01..AP-06 (6 scenarios) | ✅ |
| 002 | [`002_chat_completion.md`](002_chat_completion.md) | Verify `POST chat/completions` request/response wire shapes and optional-field omission — AP-01..AP-07 (7 scenarios) | ✅ |

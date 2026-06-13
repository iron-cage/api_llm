# Protocol Test Surface

### Scope

- **Purpose**: Define test cases that verify wire format compliance for streaming protocol specs documented in `docs/protocol/` instances.
- **Responsibility**: Each spec file maps one-to-one to a protocol instance; all message type, format, and parsing behavior must have corresponding spec entries.
- **In Scope**: Compliance scenarios for `docs/protocol/001_streaming_format.md`.
- **Out of Scope**: Client streaming control features (see `tests/docs/feature/`); endpoint coverage (see `tests/docs/api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 01 | [`001_streaming_format.md`](001_streaming_format.md) | Verify JSON array wire format, message types, and buffering strategy — PR-01..PR-08 (8 scenarios) | ✅ |

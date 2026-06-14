# Feature Doc Entity

### Scope

- **Purpose**: Document optional feature behavior specifications for `api_openai_compatible`.
- **Responsibility**: Master file listing all feature doc instances with ID, name, and status.
- **In Scope**: Optional feature activation requirements, configuration contracts, wire type additions.
- **Out of Scope**: Core HTTP client transport (always-on via `enabled`), invariant constraints (see `invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Streaming](001_streaming.md) | Server-Sent Events streaming support for chat completions | ✅ |
| 002 | [Sync API](002_sync_api.md) | Blocking synchronous wrapper around the async client | ✅ |

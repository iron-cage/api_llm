# API Doc Entity

### Scope

- **Purpose**: Document API contracts and endpoint coverage requirements for `api_openai_compatible`.
- **Responsibility**: Master file listing all API doc instances with ID, name, and status.
- **In Scope**: OpenAI-compatible endpoint contracts, required wire types, serialization invariants.
- **Out of Scope**: Provider-specific extensions, enterprise reliability features (see `feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Endpoint Coverage](001_endpoint_coverage.md) | Client methods, environment trait, wire types, error handling | ✅ |
| 002 | [Chat Completion](002_chat_completion.md) | Chat completion wire type detail — request, response, tool calling | ✅ |

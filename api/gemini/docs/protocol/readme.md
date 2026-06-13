# Protocol Doc Entity

### Scope

- **Purpose**: Specify the wire-level communication protocols used by the api_gemini crate.
- **Responsibility**: Master file listing all protocol spec instances with ID, name, and protocol scope.
- **In Scope**: Streaming wire format, HTTP headers, JSON array structure, buffering strategy.
- **Out of Scope**: Client-level streaming control features (see `feature/`), usage patterns (see `operation/`), investigation history (see `investigations/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [`001_streaming_format.md`](001_streaming_format.md) | JSON array streaming protocol specification | ✅ |

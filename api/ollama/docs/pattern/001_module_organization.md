# Pattern: Module Organization

### Scope

- **Purpose**: Document the module structure and extension patterns used in `api_ollama`.
- **Responsibility**: Crate maintainers; all new modules must follow these structural patterns.
- **In Scope**: Source layout, mod_interface usage, client extension pattern, feature module placement.
- **Out of Scope**: Test organization (see invariant/002), API contract structure (see api/).

### Abstract

`api_ollama` uses the `mod_interface` pattern for module declaration and a `client_ext_*.rs` extension file pattern for feature-specific client methods. Both patterns enforce separation of concerns and enable granular feature gating.

### mod_interface Layer Structure

All modules are declared in `src/lib.rs` via `mod_interface!` macro with feature-gated `layer` declarations. Each logical capability lives in its own module file, and feature flags activate the corresponding layer:

```
src/lib.rs          — mod_interface! root
src/client.rs       — core Client struct (always-on)
src/client_ext_*.rs — feature-specific client extensions (13 files)
src/components/     — shared request/response types
src/streaming.rs    — streaming response handling (feature: streaming)
src/retry_logic.rs  — retry with exponential backoff (feature: retry)
src/circuit_breaker.rs — circuit breaker pattern (feature: circuit_breaker)
```

### client_ext_*.rs Extension Pattern

Feature-specific client methods are organized into `client_ext_*.rs` files rather than one large client file. Each extension file adds methods to the `Client` struct for one feature domain (e.g., `client_ext_streaming.rs`, `client_ext_retry.rs`). This keeps the core client small and makes feature additions non-invasive.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `mod_interface!` root — all layer declarations |
| `src/client.rs` | Core Client struct with base HTTP methods |
| `src/client_ext_*.rs` | Feature-specific client method extensions (13 files) |

### Tests

| File | Relationship |
|------|--------------|
| `tests/` | Integration tests organized by feature domain, mirroring src/ structure |

# Pattern: Module Organization

### Scope

- **Purpose**: Define the `mod_interface` module structure pattern that all `api_huggingface` source modules must follow.
- **Responsibility**: All contributors; deviations require explicit code-review justification before merging.
- **In Scope**: Every `.rs` file and directory under `src/` — module declarations, public re-exports, submodule structure.
- **Out of Scope**: Test files in `tests/`, example files in `examples/`.

### Problem

Rust's default module system makes it easy to accidentally expose implementation details or create import paths that change without intent. Feature-gated modules scattered with `#[cfg(feature = "...")]` guards become inconsistent and hard to audit. The `api_huggingface` crate has subdirectory modules (`audio/`, `vision/`, `reliability/`, `performance/`, `cache/`, `token_counter/`, `config/`) that require consistent structure conventions.

### Solution

Every module uses the `mod private { }` + `crate::mod_interface!` structure from the wTools ecosystem. The `mod private { }` block contains all implementation details — everything inside is inaccessible outside the module unless explicitly re-exported via `crate::mod_interface! { }`. `lib.rs` declares all top-level modules as `layer` entries; optional modules are wrapped in `#[cfg(feature = "feature-name")]` on their `layer` declaration line. The `mod private { }` block is always **inline** — never a separate `private.rs` file or `private/` directory.

Subdirectory modules (`audio/`, `vision/`, `reliability/`, etc.) follow the same pattern: each subdirectory has a root module file (no `mod.rs`) with `mod private { }` + `mod_interface!` declarations, and submodules declared as `layer` entries.

### Applicability

Apply to every `.rs` file and directory under `src/`. Do not use this pattern in `tests/` or `examples/` — those use standard Rust module conventions.

### Consequences

**Benefits**: Public API surface is explicit and auditable. Feature gating is centralized. Accidental exposure is prevented. **Trade-offs**: Requires `mod_interface` macro overhead. Directory module structure (no `mod.rs`) may be unfamiliar.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | Top-level `mod_interface!` declaration — reference implementation with feature-gated `layer` entries |
| `src/client.rs` | Example of `mod private { }` + `mod_interface!` in a large single-file module |
| `src/audio/` | Example of subdirectory module with sub-layer declarations |
| `src/reliability/` | Example of feature-gated reliability module grouping |

### Tests

| File | Relationship |
|------|--------------|
| `tests/` | Verifies public API surface exposed by `mod_interface` re-exports compiles correctly |

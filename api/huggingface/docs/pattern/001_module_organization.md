# Pattern: Module Organization

### Scope

- **Purpose**: Define the `mod_interface` module structure pattern that all `api_huggingface` source modules must follow.
- **Responsibility**: mod_interface module structure — inline private block, mod_interface! re-exports, subdirectory conventions.
- **In Scope**: Every `.rs` file and directory under `src/` — module declarations, public re-exports, submodule structure.
- **Out of Scope**: Test files in `tests/`, example files in `examples/`.

### Problem

Rust's default module system makes it easy to accidentally expose implementation details or create import paths that change without intent. Feature-gated modules scattered with `#[cfg(feature = "...")]` guards become inconsistent and hard to audit. The `api_huggingface` crate has subdirectory modules (`audio/`, `vision/`, `reliability/`, `performance/`, `cache/`, `token_counter/`, `config/`) that require consistent structure conventions.

### Solution

Individual feature module files (`inference.rs`, `providers.rs`, `client.rs`, `error.rs`, etc.) use the `mod private { }` + `crate::mod_interface!` structure from the wTools ecosystem. The `mod private { }` block contains all implementation details — imports, type definitions, and impls — everything inside is inaccessible outside the module unless explicitly re-exported via `crate::mod_interface! { exposed use ... }`. The `mod private { }` block is always **inline** — never a separate `private.rs` file or `private/` directory.

`lib.rs` declares all top-level modules as `pub mod` entries; optional modules are preceded by `#[cfg(feature = "feature-name")]`. `lib.rs` itself has an empty `mod private {}` and uses `crate::mod_interface! { exposed use ... }` only for a small set of convenience crate-root re-exports.

Subdirectory modules (`audio/`, `vision/`, `reliability/`, etc.) each have a `mod.rs` root file that uses standard `pub mod` declarations for submodules. Some subdirectories (`components/`, `environment/`) additionally use `crate::mod_interface! { exposed use submod; }` to re-export submodule namespaces at the subdirectory level; others (`audio/`, `reliability/`) use direct `pub mod` and `pub use` for submodule access.

### Applicability

Apply to every `.rs` file and directory under `src/`. Do not use this pattern in `tests/` or `examples/` — those use standard Rust module conventions.

### Consequences

**Benefits**: Public API surface is explicit and auditable. Feature gating is centralized. Accidental exposure is prevented. **Trade-offs**: Requires `mod_interface` macro overhead in individual feature modules. Subdirectory `mod.rs` files have mixed conventions — some use `mod_interface!`, others use standard `pub mod`/`pub use`.

### APIs

| File | Relationship |
|------|--------------|
| `api/001_reference.md` | Defines the public API surface that this module pattern exposes |

### Features

| File | Relationship |
|------|--------------|
| `feature/001_enterprise_reliability.md` | Enterprise feature modules follow this pattern with feature-gated `pub mod` declarations |

### Invariants

| File | Relationship |
|------|--------------|
| `invariant/001_thin_client_principle.md` | Governs explicit API exposure — no accidental re-exports permitted |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | Crate root — empty `mod private {}`, `pub mod` with `#[cfg]` gates, sparse `crate::mod_interface!` re-exports |
| `src/client.rs` | Individual feature module — `mod private { }` + `crate::mod_interface!` pattern |
| `src/components/mod.rs` | Subdirectory mod.rs — `pub mod` submodules + `crate::mod_interface! { exposed use submod; }` |
| `src/audio/mod.rs` | Subdirectory mod.rs — `pub mod` + `pub use` (no `mod_interface!`) |

### Tests

| File | Relationship |
|------|--------------|
| `tests/client_tests.rs` | Verifies public API surface exposed by `mod_interface` re-exports compiles correctly |
| `tests/components_tests.rs` | Verifies shared component types are accessible via module re-exports |
| `tests/docs/pattern/01_module_organization.md` | GWT spec scenarios for this doc instance |

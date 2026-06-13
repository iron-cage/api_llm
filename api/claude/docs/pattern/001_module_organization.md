# Pattern: Module Organization

### Scope

- **Purpose**: Define the `mod_interface` module structure pattern that all `api_claude` source modules must follow.
- **Responsibility**: All contributors; deviations require explicit code-review justification before merging.
- **In Scope**: Every `.rs` file and directory under `src/` — module declarations, public re-exports, submodule structure.
- **Out of Scope**: Test files in `tests/`, example files in `examples/`, benchmark harnesses in `benches/`.

### Problem

Rust's default module system (`pub use`, `mod module;`) makes it easy to accidentally expose implementation details or create import paths that change without intent. Feature-gated modules declared with `#[cfg(feature = "...")]` scattered across files become inconsistent and hard to audit. Without a consistent structure, the public API surface drifts as new developers add code.

### Solution

Every module uses the `mod private { }` + `crate::mod_interface!` structure from the wTools ecosystem. The `mod private { }` block contains all implementation details (types, structs, impl blocks, helper functions) — everything inside is inaccessible outside the module unless explicitly re-exported. Re-exports are declared in the companion `crate::mod_interface! { }` block: `exposed use TypeName` makes the type accessible at crate root; `orphan use TypeName` makes it accessible only to the parent module.

`lib.rs` declares all top-level modules as `layer` entries inside `crate::mod_interface!`. Always-on modules appear without a `#[cfg]` guard; optional modules are wrapped in `#[cfg(feature = "feature-name")]` on their `layer` declaration line. No traditional `mod module_name;` declarations are used — `mod_interface` handles all module registration. The `mod private { }` block is always **inline** — never a separate `private.rs` file or `private/` directory.

When a module grows beyond a single file it becomes a directory: `src/client.rs` → `src/client/` with `src/client.rs` staying as the module root (no `mod.rs`); submodules declared via `mod_interface` layer declarations inside `client.rs`.

### Applicability

Apply this pattern to every `.rs` file and directory under `src/`. This includes always-on modules, feature-gated enterprise modules, and multi-file modules. Do not use this pattern in `tests/`, `examples/`, or `benches/` — those use standard Rust module conventions.

### Consequences

**Benefits**: The public API surface is explicit and auditable at a glance via `mod_interface!` blocks. Feature gating is centralized in `lib.rs` layer declarations. Accidental exposure is prevented because everything in `mod private { }` is inaccessible by default. **Trade-offs**: Requires the wTools `mod_interface` macro and its compilation overhead. Developers unfamiliar with the pattern need orientation before contributing. Deviations (e.g., `pub mod` instead of `mod_interface` re-export) are immediately visible as inconsistencies during code review.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | Top-level `mod_interface!` declaration — reference implementation of the layer pattern |
| `src/client.rs` | Example of `mod private { }` + submodule structure in a multi-file module |
| `src/secret.rs` | Example of single-file `mod private { }` + `mod_interface!` re-exports |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/pattern/01_module_organization.md` | Behavioral spec — 6 scenarios verifying mod_interface layer pattern and private namespace conformance |
| `tests/tests.rs` | Verifies the public API surface exposed by `mod_interface` re-exports compiles correctly |

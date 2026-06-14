# Pattern: Module Organization

### Scope

- **Purpose**: Define the `mod_interface` module structure pattern that all `api_xai` source modules must follow.
- **Responsibility**: Documents the Module Organization pattern — problem context, solution structure, applicability conditions, and consequences.
- **In Scope**: Every `.rs` file and directory under `src/` — module declarations, public re-exports, submodule structure.
- **Out of Scope**: Test files in `tests/`, example files in `examples/`.

### Problem

Rust's default module system makes it easy to accidentally expose implementation details or create import paths that change without intent. Feature-gated modules declared with `#[cfg(feature = "...")]` scattered across files become inconsistent and hard to audit. Without a consistent structure, the public API surface drifts as new developers add code.

### Solution

Every module uses the `mod private { }` + `crate::mod_interface!` structure from the wTools ecosystem. The `mod private { }` block contains all implementation details — everything inside is inaccessible outside the module unless explicitly re-exported. Re-exports are declared in the companion `crate::mod_interface! { }` block: `exposed use TypeName` makes the type accessible at crate root; `orphan use TypeName` makes it accessible only to the parent module.

`lib.rs` declares all top-level modules as `layer` entries inside `crate::mod_interface!`. Always-on modules appear without a `#[cfg]` guard; optional modules are wrapped in `#[cfg(feature = "feature-name")]` on their `layer` declaration line. The `mod private { }` block is always **inline** — never a separate `private.rs` file or `private/` directory.

### Applicability

Apply to every `.rs` file and directory under `src/`. This includes always-on modules (client, environment, error, secret, chat, models), feature-gated enterprise modules (retry, circuit_breaker, etc.), and multi-file modules. Do not use this pattern in `tests/` or `examples/` — those use standard Rust module conventions.

### Consequences

**Benefits**: The public API surface is explicit and auditable at a glance via `mod_interface!` blocks. Feature gating is centralized in `lib.rs` layer declarations. Accidental exposure is prevented because everything in `mod private { }` is inaccessible by default. **Trade-offs**: Requires the wTools `mod_interface` macro and its compilation overhead. Developers unfamiliar with the pattern need orientation before contributing.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | Top-level `mod_interface!` declaration — reference implementation of the layer pattern with feature-gated `layer` entries |
| `src/client.rs` | Example of `mod private { }` + `mod_interface!` structure in the main client module |
| `src/secret.rs` | Example of single-file `mod private { }` + `mod_interface!` re-exports |

### Tests

| File | Relationship |
|------|--------------|
| `tests/components_tests.rs` | Verifies component types are re-exported correctly by mod_interface |
| `tests/environment_tests.rs` | Verifies environment struct construction via mod_interface re-exports |
| `tests/error_tests.rs` | Verifies error type re-export via mod_interface |

# Pattern Spec: Module Organization

Spec scenarios for `docs/pattern/001_module_organization.md`. Verifies that the `mod_interface` module structure pattern is consistently applied across `src/`.

## PT-01: Individual feature modules use mod private block

**Given:** any individual feature module file in `src/` (e.g., `inference.rs`, `providers.rs`, `client.rs`, `error.rs`) — excluding `lib.rs` and subdirectory `mod.rs` files
**When:** the file structure is inspected
**Then:** all type definitions, `impl` blocks, and imports appear inside a `mod private { ... }` block; no type or `impl` is defined at module root level outside that block

## PT-02: Feature module public surface defined via mod_interface macro

**Given:** any individual feature module file in `src/` (e.g., `inference.rs`, `providers.rs`) that exposes public symbols
**When:** the `crate::mod_interface! { }` invocation at the end of the file is inspected
**Then:** only symbols listed in the `mod_interface!` block are re-exported; symbols inside `mod private { }` that are not listed are not accessible to external callers

## PT-03: No private.rs files or private/ directories in src/

**Given:** the `src/` directory tree
**When:** the filesystem is searched with `find src/ -name "private.rs" -o -name "private"`
**Then:** zero results are returned; no `private.rs` file and no directory named `private/` exist anywhere under `src/`

## PT-04: Optional pub mod declarations are feature-gated in lib.rs

**Given:** `src/lib.rs` `pub mod` declarations corresponding to optional Cargo features
**When:** a `pub mod` entry for an optional feature module is inspected
**Then:** the `pub mod` declaration is preceded by `#[cfg(feature = "feature-name")]`; compiling without that feature excludes the module from the build

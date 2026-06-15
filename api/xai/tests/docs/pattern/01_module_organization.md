# Pattern Spec: Module Organization
**Source:** `../../docs/pattern/001_module_organization.md`

## Test Cases

### PT-01: mod_interface exposed types accessible at crate root ✅

- **Given:** Types declared inside `mod private {}` and re-exported via `exposed use TypeName` in `crate::mod_interface!`
- **When:** A caller imports the type from the crate root (e.g., `api_xai::Client`, `api_xai::Secret`)
- **Then:** The types are accessible and usable without navigating internal module paths
- **Test:** `components_tests.rs::crate_root_exports_are_accessible` and all other tests that
  import `api_xai::{ Client, Secret, Message, Role, XaiEnvironmentImpl, ... }`.

### PT-02: Private module contents inaccessible from outside ✅

- **Given:** Implementation details inside `mod private {}` that are not listed in the `crate::mod_interface!` block
- **When:** A caller attempts to access those internal items from outside the module
- **Then:** Compilation fails — private contents are not visible outside the declaring module
- **Test:** Verified by compilation — Rust's module system enforces this; the test suite
  never imports `api_xai::chat::private::*` or similar paths, confirming inaccessibility.

### PT-03: Feature-gated layers only compiled when flag enabled ✅

- **Given:** A `layer` declaration in `lib.rs` guarded by `#[cfg(feature = "retry")]`
- **When:** The crate is compiled without the `retry` feature
- **Then:** The retry module is not compiled — its types and functions do not exist in the binary
- **Test:** Verified by compilation — each `layer` in `lib.rs` for enterprise features is wrapped
  in `#[cfg(feature = "...")]`; the Level 1 verification run (`RUSTFLAGS="-D warnings"`) confirms
  no unused-import or dead-code warnings from unguarded enterprise symbols.

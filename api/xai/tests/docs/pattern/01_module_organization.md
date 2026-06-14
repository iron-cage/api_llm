# Pattern Spec: Module Organization
**Source:** `../../docs/pattern/001_module_organization.md`

## Test Cases

### PT-01: mod_interface exposed types accessible at crate root

- **Given:** Types declared inside `mod private {}` and re-exported via `exposed use TypeName` in `crate::mod_interface!`
- **When:** A caller imports the type from the crate root (e.g., `api_xai::Client`, `api_xai::Secret`)
- **Then:** The types are accessible and usable without navigating internal module paths

### PT-02: Private module contents inaccessible from outside

- **Given:** Implementation details inside `mod private {}` that are not listed in the `crate::mod_interface!` block
- **When:** A caller attempts to access those internal items from outside the module
- **Then:** Compilation fails — private contents are not visible outside the declaring module

### PT-03: Feature-gated layers only compiled when flag enabled

- **Given:** A `layer` declaration in `lib.rs` guarded by `#[cfg(feature = "retry")]`
- **When:** The crate is compiled without the `retry` feature
- **Then:** The retry module is not compiled — its types and functions do not exist in the binary

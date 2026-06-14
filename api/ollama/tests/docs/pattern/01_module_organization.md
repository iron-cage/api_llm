# Pattern Spec: Module Organization

**Source:** `../../docs/pattern/001_module_organization.md`

### PT-01: mod_interface types re-exported at crate root

- **Given:** A type declared inside a `mod private {}` block within a module using `mod_interface!`
- **When:** A downstream crate or test imports the type via `api_ollama::<TypeName>`
- **Then:** The type is accessible at the crate root — mod_interface re-exports it correctly

### PT-02: Private module contents not accessible externally

- **Given:** An item declared inside `mod private {}` without `pub` visibility
- **When:** External code attempts to access `api_ollama::private::<item>` or `api_ollama::<module>::private::<item>`
- **Then:** The access fails at compile time — private internals are not leaked

### PT-03: client_ext methods gated by corresponding feature flag

- **Given:** A method defined in `client_ext_retry.rs` (gated by `retry` feature)
- **When:** The crate is compiled without the `retry` feature flag
- **Then:** The method is not available on the `Client` type — calling it is a compile error

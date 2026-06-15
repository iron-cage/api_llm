# Pattern Spec: Module Organization

**Source:** `../../docs/pattern/001_module_organization.md`

### PT-01: mod_interface types re-exported at crate root ✅

- **Given:** A type declared inside a `mod private {}` block within a module using `mod_interface!`
- **When:** A downstream crate or test imports the type via `api_ollama::<TypeName>`
- **Then:** The type is accessible at the crate root — mod_interface re-exports it correctly
- **Test:** `core_functionality_tests.rs::crate_root_exports_are_accessible` and all other tests
  that import `api_ollama::{ OllamaClient, ChatRequest, ChatMessage, GenerateRequest, ... }`.

### PT-02: Private module contents not accessible externally ✅

- **Given:** An item declared inside `mod private {}` without `pub` visibility
- **When:** External code attempts to access `api_ollama::private::<item>` or `api_ollama::<module>::private::<item>`
- **Then:** The access fails at compile time — private internals are not leaked
- **Test:** Verified by compilation — Rust's visibility rules enforce this; the test suite has
  never imported `api_ollama::chat::private::*` or similar paths, confirming inaccessibility.

### PT-03: client_ext methods gated by corresponding feature flag ✅

- **Given:** A method defined in `client_ext_retry.rs` (gated by `retry` feature)
- **When:** The crate is compiled without the `retry` feature flag
- **Then:** The method is not available on the `Client` type — calling it is a compile error
- **Test:** Verified by compilation — `client_ext_retry.rs` is only included when
  `#[cfg(feature = "retry")]` is active. All per-feature client extension files follow this pattern.

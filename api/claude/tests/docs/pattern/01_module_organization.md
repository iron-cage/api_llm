# Pattern Spec: Module Organization

**Source:** [`docs/pattern/001_module_organization.md`](../../docs/pattern/001_module_organization.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| PT-01 | lib.rs uses mod_interface! layer declarations | layer pattern | ✅ |
| PT-02 | No mod.rs files exist in module directories | submodule structure | ✅ |
| PT-03 | mod private { } block present in source modules | private namespace | ✅ |
| PT-04 | No private.rs or private/ directory in src/ | inline private | ✅ |
| PT-05 | Optional modules use #[cfg(feature)] on layer line | feature-gating | ✅ |
| PT-06 | exposed use re-exports are visible externally; orphan use items are not | re-export visibility | ✅ |

---

### PT-01: lib.rs uses mod_interface! layer declarations

- **Given:** `src/lib.rs` in the `api_claude` crate
- **When:** Its contents are inspected for module declarations
- **Then:** All top-level module registrations appear as `layer` entries inside a `crate::mod_interface! { }` block; no bare `mod module_name;` declarations appear outside the `mod_interface!` invocation; the pattern is used consistently for all modules

---

### PT-02: No mod.rs files exist in module directories

- **Given:** Module directories under `src/` (e.g., `src/client/`, `src/error/`)
- **When:** Their file contents are enumerated
- **Then:** No `mod.rs` file exists in any module directory; the module root file is the same-named `.rs` file in the parent directory (`src/client.rs` is root for `src/client/`); `mod_interface` handles submodule registration without `mod.rs`

---

### PT-03: mod private { } block present in source modules

- **Given:** Source module files under `src/` that contain implementation code
- **When:** Their structure is inspected
- **Then:** Implementation details (types, structs, impl blocks, helper functions) are contained inside a `mod private { }` block; the block is present in each non-trivial module file; public API surface is exposed only via the companion `crate::mod_interface! { }` re-export block

---

### PT-04: No private.rs or private/ directory in src/

- **Given:** The entire `src/` directory tree
- **When:** Its file and directory listing is inspected
- **Then:** No file named `private.rs` exists anywhere under `src/`; no directory named `private/` exists anywhere under `src/`; all private namespace usage is via inline `mod private { }` blocks within each module file

---

### PT-05: Optional modules use #[cfg(feature)] on layer line

- **Given:** Feature-gated modules declared in `src/lib.rs`
- **When:** Their `layer` declarations in the `mod_interface!` block are inspected
- **Then:** Each optional module's `layer` line carries a `#[cfg(feature = "feature-name")]` attribute; always-on modules have no `#[cfg]` guard on their `layer` line; no alternative gating mechanism (e.g., conditional `use`, inline `if cfg!`) is used for module-level feature gating

---

### PT-06: exposed use re-exports are visible externally; orphan use items are not

- **Given:** A module's `crate::mod_interface! { }` block containing both `exposed use TypeA;` and `use TypeB;` (orphan use, not exposed)
- **When:** The crate's public API is inspected from an external caller (e.g., a test file or dependent crate)
- **Then:** `TypeA` is accessible via the crate's public path; `TypeB` is not accessible from outside the module; the distinction is enforced by `mod_interface` re-export semantics, not by `pub` vs private visibility on the original type

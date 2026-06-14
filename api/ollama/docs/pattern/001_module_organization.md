# Pattern: Module Organization

### Scope

- **Purpose**: Documents the Module Organization pattern — source layout and module declaration conventions for api_ollama.
- **Responsibility**: Specifies the mod_interface layer structure and client_ext_*.rs extension file pattern for all api_ollama modules.
- **In Scope**: Source layout, mod_interface usage, client extension pattern, feature module placement.
- **Out of Scope**: Test organization (see invariant/002), API contract structure (see api/).

### Problem

`api_ollama` supports 47 feature flags. Placing all feature-specific client methods in a single `client.rs` and declaring all modules in one flat `lib.rs` block makes feature additions invasive — every new flag requires edits to the core files, which grow without bound. A flat module pile collapses all capability dimensions into one unstructured list.

### Solution

Two complementary patterns divide responsibility:

- **mod_interface layer declarations** — all modules are declared in `src/lib.rs` via `mod_interface!` with feature-gated `layer` entries. Each logical capability lives in its own module file. The root file is a pure declarative index.
- **client_ext_*.rs extension files** — feature-specific client methods are split into dedicated extension files (one per feature domain, 13 files total). Each extension file adds methods to the `Client` struct for one capability area only. The core `client.rs` remains small and feature-independent.

### Applicability

Apply when adding any new enterprise reliability feature or endpoint group to api_ollama. Each addition requires: a new `layer` entry in `src/lib.rs`, a new `client_ext_<feature>.rs` for client methods, and a corresponding module file in `src/<feature>.rs` or `src/<feature>/`.

### Consequences

Feature additions require no changes to the core client. The root module file stays declarative and readable. The cost is a higher file count (core + 13 extension files + matching module files) and the need to navigate multiple files to understand the full client surface.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `mod_interface!` root — all layer declarations |
| `src/client.rs` | Core Client struct with base HTTP methods |
| `src/client_ext_*.rs` | Feature-specific client method extensions (13 files) |

### Tests

| File | Relationship |
|------|--------------|
| `tests/core_client_api_tests.rs` | Verifies client API surface is correctly exposed via mod_interface |
| `tests/builder_construction_tests.rs` | Verifies builder patterns work across feature-gated extensions |

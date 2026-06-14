# Pattern: Module Organization

### Scope

- **Purpose**: Documents the module structure conventions and generic client architecture of `api_openai_compatible`.
- **Responsibility**: Specifies the `mod_interface` layer structure, `Client<E>` generic parameterization, and feature-gated module layout.
- **In Scope**: Module declarations in `lib.rs`, the `mod private {}` + `crate::mod_interface!` structure in each source file, feature flag gating of module layers, `Client<E>` generic over `OpenAiCompatEnvironment`.
- **Out of Scope**: Wire type field-level documentation (see `api/`), streaming event format (see `feature/001_streaming.md`), test organization (see `invariant/002`).

### Problem

A shared HTTP wire-protocol layer must be reusable across multiple provider crates (api_xai, and future crates) without each provider re-implementing request/response serialization, environment configuration, or error handling. The shared layer must expose a stable public API surface while keeping implementation details private, and must allow downstream crates to control which capabilities are compiled via feature flags.

### Solution

Three patterns divide the problem:

**mod_interface layer declarations** — `src/lib.rs` declares all modules as `layer` entries inside `crate::mod_interface!`. Each module file uses `mod private {}` for implementation and `crate::mod_interface!` for re-exports. The `enabled` feature gates the entire public surface; `streaming` and `sync_api` gate individual layers. Four unconditional layers exist (`error`, `components`, `environment`, `client`) plus one feature-gated layer (`sync_client`, behind `sync_api`).

**Client<E> generic over OpenAiCompatEnvironment** — The `Client` is parameterized over a trait `E: OpenAiCompatEnvironment` so each downstream crate supplies its own environment (API key source, base URL, timeout) without modifying the shared client. The trait requires `api_key()`, `base_url()`, `timeout()` and provides a default `headers()` implementation. `OpenAiCompatEnvironmentImpl` is the built-in implementor with builder methods.

**Feature-gated module layers** — Four Cargo features control compilation: `enabled` (master switch for all public types), `streaming` (SSE chunk types), `sync_api` (blocking `SyncClient<E>`), `integration` (live API tests). `full` enables all four. `default = ["full"]`. Each optional layer appears with a `#[cfg(feature = "...")]` guard on its `layer` declaration in `lib.rs`.

### Applicability

Apply to every `.rs` file under `src/`. New modules must use the `mod private {}` + `crate::mod_interface!` structure. New optional capabilities must have a dedicated Cargo feature flag and a guarded `layer` entry in `lib.rs`. Downstream crates implement `OpenAiCompatEnvironment` for their own environment type and construct `Client<MyEnv>`.

### Consequences

The public API surface is explicit and auditable via `mod_interface!` blocks. Feature gating is centralized in five `layer` lines. Downstream crates add provider support by implementing one trait — no forks, no copy-paste. The cost is the `mod_interface` macro dependency and the learning curve for the `mod private {}` pattern.

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | Top-level `mod_interface!` — 5 layer declarations with feature guards |
| `src/client.rs` | `Client<E>` generic struct — reference implementation of `mod private {}` pattern |
| `src/environment.rs` | `OpenAiCompatEnvironment` trait and `OpenAiCompatEnvironmentImpl` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/environment_test.rs` | Verifies `OpenAiCompatEnvironmentImpl` construction and builder methods |
| `tests/wire_test.rs` | Verifies types are re-exported correctly through mod_interface layers |

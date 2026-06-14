# Pattern Spec: Module Organization

**Source:** [`docs/pattern/001_module_organization.md`](../../../docs/pattern/001_module_organization.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| PT-01 | mod_interface re-exports all public types through enabled feature | mod-interface-layers | ✅ |
| PT-02 | Client<E> accepts any OpenAiCompatEnvironment implementor | generic-client | ✅ |
| PT-03 | Feature-gated layers compile only when their flag is active | feature-gating | ✅ |

---

### PT-01: mod_interface re-exports all public types through enabled feature

- **Given:** The `enabled` feature is active
- **When:** All public types from the 5 mod_interface layers (error, components, environment, client, sync_client) are referenced
- **Then:** `OpenAiCompatError`, `ChatCompletionRequest`, `ChatCompletionResponse`, `Message`, `Role`, `Usage`, `OpenAiCompatEnvironment`, `OpenAiCompatEnvironmentImpl`, and `Client` are importable from the crate root; no direct `mod private` access is possible

---

### PT-02: Client<E> accepts any OpenAiCompatEnvironment implementor

- **Given:** A custom struct implementing `OpenAiCompatEnvironment` with `api_key()`, `base_url()`, and `timeout()` returning valid values
- **When:** `Client::build(custom_env)` is called
- **Then:** Returns `Ok(Client<CustomEnv>)` — the generic parameter accepts any trait implementor, not just `OpenAiCompatEnvironmentImpl`

---

### PT-03: Feature-gated layers compile only when their flag is active

- **Given:** The `enabled` feature is active but `streaming` and `sync_api` are not
- **When:** Code attempts to reference `ChatCompletionChunk` (streaming) or `SyncClient` (sync_api)
- **Then:** The types are absent from the crate's public API; enabling `streaming` makes `ChatCompletionChunk`, `ChunkChoice`, and `Delta` available; enabling `sync_api` makes `SyncClient` available

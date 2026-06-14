# Invariant Spec: Thin Client Principle

**Source:** [`docs/invariant/001_thin_client_principle.md`](../../../docs/invariant/001_thin_client_principle.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| IN-01 | Optional fields absent from serialized JSON when None | no-implicit-defaults | ⏳ |
| IN-02 | Enterprise feature module not compiled without its feature flag | feature-gate-isolation | ⏳ |
| IN-03 | No automatic request headers beyond what environment provides | no-implicit-mutation | ⏳ |

---

### IN-01: Optional fields absent from serialized JSON when None

- **Given:** A `ChatRequest` with only `model` and one user message set; all other fields (`temperature`, `max_tokens`, `top_p`, `frequency_penalty`, `presence_penalty`, `stream`, `tools`) are `None`
- **When:** The request is serialized with `serde_json::to_string`
- **Then:** The resulting JSON contains exactly the `model` and `messages` keys; no optional field key appears in the output; the client injects no implicit defaults on behalf of the caller

---

### IN-02: Enterprise feature module not compiled without its feature flag

- **Given:** The crate is compiled with `--no-default-features --features enabled` (no enterprise feature flags active)
- **When:** A test attempts to reference a type from an enterprise module (e.g., `retry_logic`, `circuit_breaker`)
- **Then:** The module is absent from the compiled binary; `#[cfg(feature = "retry")]` gates prevent compilation of enterprise code when the flag is not set

---

### IN-03: No automatic request headers beyond what environment provides

- **Given:** An `OpenaiEnvironmentImpl` constructed with a known API key and no custom headers
- **When:** `env.headers()` is called
- **Then:** The returned `HeaderMap` contains only the `Authorization` (`Bearer <key>`) and `Content-Type` (`application/json`) headers; no additional headers are injected by the client layer beyond what the environment provides

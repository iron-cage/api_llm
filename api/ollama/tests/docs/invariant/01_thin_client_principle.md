# Invariant Spec: Thin Client Principle

**Source:** `../../docs/invariant/001_thin_client_principle.md`

### IN-01: Optional fields absent from serialized JSON when None ✅

- **Given:** A `ChatRequest` with `options`, `format`, and `template` fields set to `None`
- **When:** The request is serialized to JSON via `serde_json::to_value`
- **Then:** The serialized JSON object does not contain keys `options`, `format`, or `template`
- **Test:** `core_functionality_tests.rs::chat_request_optional_fields_absent_when_none`,
  `generate_request_optional_fields_absent_when_none`

### IN-02: Enterprise module not compiled without feature flag ✅

- **Given:** The crate is compiled with only the `enabled` feature (no enterprise flags)
- **When:** A caller attempts to reference a symbol from an enterprise module (e.g., `retry_logic`)
- **Then:** The symbol is absent from the compiled crate — the module is not included
- **Test:** Verified by compilation — each enterprise field in `OllamaClient` is guarded by
  `#[cfg(feature = "...")]`; building without the flag excludes the module entirely.

### IN-03: Streaming not activated without explicit stream parameter ✅

- **Given:** A `ChatRequest` with `stream` field set to `None` or `false`
- **When:** The request is sent to the Ollama server
- **Then:** The server returns a single JSON response, not a stream of NDJSON chunks
- **Test:** `core_functionality_tests.rs::chat_request_no_stream_flag_when_not_set`

# Invariant Spec: Thin Client Principle
**Source:** `../../docs/invariant/001_thin_client_principle.md`

## Test Cases

### IN-01: Optional fields absent from serialized JSON when None ✅

- **Given:** A ChatCompletionRequest with only required fields set (model, messages) and all optional fields left as None
- **When:** The request is serialized to JSON via serde
- **Then:** The serialized JSON contains no keys for the None-valued optional fields (temperature, top_p, tools, etc.)
- **Test:** `components_tests.rs::chat_request_omits_none_fields`,
  `chat_request_with_only_required_fields_omits_all_optional_keys`

### IN-02: Enterprise feature module not compiled without its feature flag ✅

- **Given:** The crate is compiled with only the `enabled` feature (no enterprise flags)
- **When:** A caller attempts to use an enterprise type (e.g., retry config, circuit breaker)
- **Then:** The type does not exist at the crate root — compilation fails if referenced
- **Test:** Verified by compilation — every enterprise module in `lib.rs` is guarded by
  `#[cfg(feature = "...")]`; omitting the flag removes the module from compilation.

### IN-03: No automatic token pre-computation on requests ✅

- **Given:** A client built with `count_tokens` feature enabled
- **When:** A chat completion request is sent without calling `count_tokens()` first
- **Then:** The request is sent as-is with no token count field injected or pre-computed
- **Test:** `components_tests.rs::chat_request_payload_contains_no_token_count_field`

### IN-04: No implicit caching on repeated identical requests ✅

- **Given:** A client built with `caching` feature enabled but no cache configured
- **When:** Two identical chat completion requests are sent sequentially
- **Then:** Both requests hit the real API — no cached response is returned without explicit cache setup
- **Test:** Verified by `request_caching_tests.rs` — caching requires explicit cache construction
  and configuration; the client struct holds `cache: Option<LruCache<...>>` which defaults to `None`,
  and the send path checks `if let Some(cache) = &self.cache` before any lookup.

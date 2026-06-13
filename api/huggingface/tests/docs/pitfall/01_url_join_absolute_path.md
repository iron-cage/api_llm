# Pitfall Spec: URL Join Absolute Path

Spec scenarios for `docs/pitfall/001_url_join_absolute_path.md`. Verifies that endpoint path construction uses relative paths to avoid `Url::join` stripping the base URL prefix.

### PF-01: providers.rs uses relative path for Router API endpoint

- **Given:** `src/providers.rs` endpoint path string for the chat completions route
- **When:** the path string literal is inspected
- **Then:** the path is `"chat/completions"` with no leading slash — not `"/v1/chat/completions"` or `"/chat/completions"`

### PF-02: inference.rs uses relative path for legacy model endpoint

- **Given:** `src/inference.rs` `create_with_options()` method — the one method that still uses `Url::join` with a model-specific endpoint path
- **When:** the format string at the `Url::join` call site inside `create_with_options()` is inspected
- **Then:** the path string is `"models/{model_id}"` with no leading slash — not `"/models/{id}"` or `"/v1/models/{id}"`; `create()`, `create_with_parameters()`, and `create_stream()` use `"chat/completions"` on the Router API and are not subject to this pitfall

### PF-03: base URL carries the version prefix

- **Given:** the HuggingFace environment base URL configuration
- **When:** the base URL used for `Url::join` is inspected
- **Then:** the base URL ends with `/v1/` — ensuring relative paths compose to the correct versioned endpoint

### PF-04: no leading-slash path literals in Url::join callers

- **Given:** `src/providers.rs` and `src/inference.rs` — the source files that use `Url::join` for endpoint path construction
- **When:** a search for `"/` (quote followed by slash) in path string arguments is performed in those two files
- **Then:** no leading-slash path literals are found in path strings passed to `Url::join`; `src/models.rs` and `src/embeddings.rs` use absolute base URLs via `format!()` and bypass `Url::join` entirely, making them exempt from this pitfall

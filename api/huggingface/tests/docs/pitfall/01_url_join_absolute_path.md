# Pitfall Spec: URL Join Absolute Path

Spec scenarios for `docs/pitfall/001_url_join_absolute_path.md`. Verifies that endpoint path construction uses relative paths to avoid `Url::join` stripping the base URL prefix.

### PF-01: providers.rs uses relative path for Router API endpoint

- **Given:** `src/providers.rs` endpoint path string for the chat completions route
- **When:** the path string literal is inspected
- **Then:** the path is `"chat/completions"` with no leading slash — not `"/v1/chat/completions"` or `"/chat/completions"`

### PF-02: inference.rs uses relative path for model endpoint

- **Given:** `src/inference.rs` format string constructing the model-specific endpoint path
- **When:** the format string at the inference endpoint construction sites is inspected
- **Then:** the path format is `"models/{id}"` with no leading slash — not `"/models/{id}"` or `"/v1/models/{id}"`

### PF-03: base URL carries the version prefix

- **Given:** the HuggingFace environment base URL configuration
- **When:** the base URL used for `Url::join` is inspected
- **Then:** the base URL ends with `/v1/` — ensuring relative paths compose to the correct versioned endpoint

### PF-04: no leading-slash path literals in endpoint construction

- **Given:** all source files under `src/` that construct endpoint paths
- **When:** a search for `"/` (quote followed by slash) in path string arguments is performed
- **Then:** no leading-slash path literals are found in any `Url::join`, `format!`, or string concatenation call constructing an API endpoint path

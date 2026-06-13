# Invariant Spec: Thin Client Principle

Spec scenarios for `docs/invariant/001_thin_client_principle.md`. Verifies that the client has no implicit behaviors and maps directly to HuggingFace endpoints.

### IN-01: Enterprise feature absent when feature flag disabled

- **Given:** `api_huggingface` compiled without the `rate_limiting` feature flag
- **When:** the compiled binary is inspected for rate-limiting logic
- **Then:** no rate limiting is applied to outgoing requests; the `rate_limiting` module is not compiled in; requests proceed immediately without throttling

### IN-02: No automatic retry without explicit configuration

- **Given:** a `Client` constructed with no retry configuration and no `retry-logic` feature
- **When:** an inference request encounters a transient error (e.g., HTTP 503)
- **Then:** the error is returned to the caller immediately with no automatic retry; the error variant matches the HTTP failure category

### IN-03: One client method maps to at most one HTTP request

- **Given:** a `Client` with no enterprise features enabled and a valid API key
- **When:** `client.inference().create(prompt, model)` is called once
- **Then:** exactly one HTTP request is sent to the HuggingFace endpoint; no background requests, prefetches, or side-channel calls are made

### IN-04: Explicit model selection reaches API unchanged

- **Given:** a specific model identifier passed to `client.inference().create(prompt, model)`
- **When:** the outgoing HTTP request is inspected
- **Then:** the model identifier in the request body exactly matches the identifier provided by the caller; no substitution, aliasing, or fallback occurs

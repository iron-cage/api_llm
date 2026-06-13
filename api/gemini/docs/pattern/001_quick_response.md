# Pattern: Quick Response

### Scope

- **Purpose**: Define the structural pattern for single-call text generation that delegates all error handling to the caller.
- **Responsibility**: Applied wherever the call site can handle `Result` directly and needs minimal client-side boilerplate.
- **In Scope**: Single-turn, non-conversational generate_content calls where upstream error handling is acceptable.
- **Out of Scope**: Multi-turn conversations, calls requiring transparent error absorption, rate-limited batch workloads (see `002_error_resilient.md` and `003_batch_processing.md`).

### Problem

Simple text generation requires minimal configuration but provides no protection against errors, timeouts, or rate limits. Callers need a single-call path with sane defaults that delegates error handling upstream.

### Solution

Create a thin wrapper that constructs a `GenerateContentRequest` with defaults, calls `generate_content`, and extracts text from the first candidate's first part. All errors propagate to the caller as `Result`. See `docs/operation/002_usage_examples.md` for the step-by-step request construction procedure.

### Applicability

Use when:
- The call site can handle `Result` directly
- Single-turn, non-conversational generation is sufficient
- The caller controls retry and fallback behavior

Avoid when network failures or rate limits must be absorbed transparently at the call site.

### Consequences

- Minimal setup overhead; no boilerplate beyond the request struct
- No retry protection — transient errors propagate directly to the caller
- No fallback — the caller decides how to handle failure cases

### Sources

| File | Relationship |
|------|-------------|
| `src/models/api/content_generation/api_impl.rs` | `generate_content()` underlying this pattern |

### Tests

| File | Relationship |
|------|-------------|
| `tests/inc/messages_api_test.rs` | Tests exercising the Quick Response path |

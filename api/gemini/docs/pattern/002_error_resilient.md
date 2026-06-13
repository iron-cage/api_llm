# Pattern: Error-Resilient

### Scope

- **Purpose**: Define the structural pattern for generation calls where all failure modes must degrade to a human-readable fallback string rather than propagating an error.
- **Responsibility**: Applied wherever the caller cannot handle `Result` — typically UI rendering or display code expecting a `String`.
- **In Scope**: Generation calls inside UI-layer code, background formatting jobs, or any context where error propagation reaches an end user.
- **Out of Scope**: Library-internal code where error transparency is required; batch workloads where distinct failure accounting is needed.

### Problem

User-facing features cannot surface raw API errors. A generation failure must degrade gracefully to a human-readable placeholder rather than propagating an error or crashing.

### Solution

Wrap the generation call in a match that maps specific error variants (`RateLimitError`, `TimeoutError`, general failure) to distinct user-readable fallback strings. Client construction failure is handled before any network call is attempted. The function signature returns `String`, not `Result<String, Error>` — the error signal is absorbed at this boundary. See `docs/operation/002_usage_examples.md` for the underlying request construction procedure.

### Applicability

Use when:
- The caller cannot handle `Result` (for example, UI rendering code expecting `String`)
- Specific error conditions have distinct user-facing messages
- Latency spikes from rate limiting must be absorbed transparently

Avoid when:
- Transparency about error type is required by callers
- Callers need to distinguish between error variants for retry logic

### Consequences

- The caller always receives a string — no panics reach the UI layer
- Error signal is absorbed — callers cannot distinguish authentication failure from timeout
- Fallback messages must be meaningful to end users without exposing implementation details

### Sources

| File | Relationship |
|------|-------------|
| `src/models/api/content_generation/api_impl.rs` | `generate_content()` underlying this pattern |
| `src/error.rs` | Error variants matched in the fallback arms |

### Tests

| File | Relationship |
|------|-------------|
| `tests/inc/messages_api_test.rs` | Tests exercising the Error-Resilient path |

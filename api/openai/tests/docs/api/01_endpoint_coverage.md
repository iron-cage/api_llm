# API Spec: Endpoint Coverage

**Source:** [`docs/api/001_endpoint_coverage.md`](../../../docs/api/001_endpoint_coverage.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| AP-01 | Chat completion accepts typed request and returns typed response | chat-method-signature | ✅ |
| AP-02 | Model listing returns typed response with non-empty models array | models-endpoint | ✅ |
| AP-03 | Feature-gated endpoint not accessible when feature disabled | feature-gate-enforcement | ✅ |
| AP-04 | All API methods return error_tools::Error on authentication failure | error-contract | ✅ |

---

### AP-01: Chat completion accepts typed request and returns typed response

- **Given:** A valid `ChatRequest` struct with `model = "gpt-4o-mini"` and `messages` containing one user message
- **When:** `client.chat().create(request).await` is called against the live OpenAI API
- **Then:** The method returns `Result<ChatResponse, error_tools::Error>`; the response contains a non-empty `choices` array; `choices[0].message.content` is a non-empty string; `usage` fields (`prompt_tokens`, `completion_tokens`, `total_tokens`) are all greater than zero

---

### AP-02: Model listing returns typed response with non-empty models array

- **Given:** A valid authenticated client
- **When:** `client.models().list().await` is called against the live OpenAI API
- **Then:** The method returns a typed list response; the `data` array is non-empty; each entry has a non-empty `id` field; at least one model ID contains `"gpt"`

---

### AP-03: Feature-gated endpoint not accessible when feature disabled

- **Given:** The crate is compiled with `--no-default-features --features enabled` (the `websocket` feature flag is not active)
- **When:** Compilation is attempted with code that calls `client.realtime()`
- **Then:** Compilation fails because the `realtime()` accessor is gated behind `#[cfg(feature = "websocket")]`; the WebSocket module is excluded from the binary

---

### AP-04: All API methods return error_tools::Error on authentication failure

- **Given:** A client constructed with an invalid API key (`"sk-invalid-key-for-testing"`)
- **When:** `client.chat().create(request).await` is called
- **Then:** The method returns `Err(error_tools::Error)` containing an authentication failure indication (HTTP 401); the error is not a panic or silent fallback

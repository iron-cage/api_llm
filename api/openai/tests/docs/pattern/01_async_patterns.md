# Pattern Spec: Async Patterns

**Source:** [`docs/pattern/001_async_patterns.md`](../../../docs/pattern/001_async_patterns.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| PT-01 | Chat completion is async and accepts typed request struct | typed-create | ✅ |
| PT-02 | Method return type is Result with typed response | typed-result | ✅ |
| PT-03 | Streaming method returns receiver of typed events | typed-streaming | ✅ |

---

### PT-01: Chat completion is async and accepts typed request struct

- **Given:** A `ChatRequest` struct populated with `model` and `messages` fields (not a `serde_json::Value`)
- **When:** `client.chat().create(request).await` is called
- **Then:** The method compiles and executes as an `async fn`; the request parameter is a concrete typed struct, not `serde_json::Value`; the compiler enforces field types at the call site

---

### PT-02: Method return type is Result with typed response

- **Given:** A valid authenticated client and a well-formed `ChatRequest`
- **When:** `client.chat().create(request).await` returns successfully
- **Then:** The return type is `Result<ChatResponse, error_tools::Error>` (not `Result<serde_json::Value, _>`); `response.choices`, `response.usage`, and `response.model` are typed fields accessible without JSON parsing

---

### PT-03: Streaming method returns receiver of typed events

- **Given:** A `ChatRequest` with `stream: Some(true)` and the `streaming` feature enabled
- **When:** The streaming chat method is invoked
- **Then:** The method returns a stream or receiver of typed `ChatCompletionChunk` events (not raw bytes or untyped JSON); each chunk has typed `choices` with `delta` containing optional `role`, `content`, and `tool_calls` fields

# Feature Spec: Sync API

**Source:** [`docs/feature/002_sync_api.md`](../../../docs/feature/002_sync_api.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| FT-07 | SyncClient::new() with valid client succeeds | construction | ✅ |
| FT-08 | SyncClient::post() with fake base URL returns Err from real HTTP | url-routing | ✅ |
| FT-09 | SyncClient::post() blocks and returns Ok on valid response (integration) | blocking-post | ✅ |

---

### FT-07: SyncClient::new() with valid client succeeds

- **Given:** A valid `OpenAiCompatEnvironmentImpl` constructed with a non-empty API key; an async `Client<E>` built from it; no active tokio runtime on the calling thread
- **When:** `SyncClient::new(client)` is called
- **Then:** Returns `Ok(SyncClient)`; the `SyncClient` owns an internal `Arc<tokio::runtime::Runtime>`; no error is returned; construction does not require external runtime infrastructure

---

### FT-08: SyncClient::post() with fake base URL returns Err from real HTTP

- **Given:** An `OpenAiCompatEnvironmentImpl` with `base_url` set to `"http://127.0.0.1:1/"` (nothing listening) and a non-empty API key; a `SyncClient` built from it
- **When:** `sync_client.post::<_, serde_json::Value>("models", &serde_json::json!({}))` is called
- **Then:** Returns `Err(_)` — a network or connection error is propagated; the post call routes to the custom base URL, confirming URL routing is respected; the error is not silently swallowed

---

### FT-09: SyncClient::post() blocks and returns Ok on valid response (integration)

- **Given:** The `integration` Cargo feature is enabled; a real API key is available; a `SyncClient` wrapping a `Client<OpenAiCompatEnvironmentImpl>` pointed at the real OpenAI endpoint
- **When:** `sync_client.post::<_, ChatCompletionResponse>("chat/completions", &minimal_request)` is called with a well-formed minimal `ChatCompletionRequest`
- **Then:** Returns `Ok(ChatCompletionResponse)` with at least one entry in `choices`; the calling thread was blocked until the response arrived; no async runtime is visible to the caller

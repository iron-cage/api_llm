# API Spec: Endpoint Coverage

**Source:** [`docs/api/001_endpoint_coverage.md`](../../../docs/api/001_endpoint_coverage.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| AP-01 | Client::build constructs from environment | client-construction | ✅ |
| AP-02 | Client::post sends JSON body and deserializes typed response | post-method | ✅ |
| AP-03 | Client::get sends GET and deserializes typed response | get-method | ✅ |
| AP-04 | Non-2xx response propagates as Api error | error-handling | ✅ |
| AP-05 | OpenAiCompatEnvironment::headers produces Authorization and Content-Type | environment-headers | ✅ |
| AP-06 | SyncClient wraps async client with blocking post | sync-wrapper | ✅ |

---

### AP-01: Client::build constructs from environment

- **Given:** An `OpenAiCompatEnvironmentImpl` constructed with a valid API key
- **When:** `Client::build(env)` is called
- **Then:** Returns `Ok(Client)` with timeout and connection pool configured from the environment

---

### AP-02: Client::post sends JSON body and deserializes typed response

- **Given:** A `Client<E>` built from a valid environment, and a `ChatCompletionRequest` with `model` and `messages`
- **When:** `client.post::<ChatCompletionRequest, ChatCompletionResponse>("chat/completions", &request).await`
- **Then:** The request is serialized as JSON, sent to `base_url + "chat/completions"` with Authorization and Content-Type headers, and the JSON response is deserialized into `ChatCompletionResponse`

---

### AP-03: Client::get sends GET and deserializes typed response

- **Given:** A `Client<E>` built from a valid environment
- **When:** `client.get::<O>("models").await` is called
- **Then:** A GET request is sent to `base_url + "models"` with Authorization and Content-Type headers; the JSON response is deserialized into the target type `O`

---

### AP-04: Non-2xx response propagates as Api error

- **Given:** A `Client<E>` built with a base URL pointing to an endpoint that returns HTTP 401 with body `"Unauthorized"`
- **When:** `client.post::<ChatCompletionRequest, ChatCompletionResponse>("chat/completions", &request).await` is called
- **Then:** Returns `Err` whose Display output contains `"Unauthorized"` — the non-2xx body is propagated as an `OpenAiCompatError::Api` variant through the internal `handle_response` path

---

### AP-05: OpenAiCompatEnvironment::headers produces Authorization and Content-Type

- **Given:** An `OpenAiCompatEnvironmentImpl` constructed with API key `"sk-test123"`
- **When:** `env.headers()` is called
- **Then:** Returns a `HeaderMap` containing exactly `Authorization: Bearer sk-test123` and `Content-Type: application/json`

---

### AP-06: SyncClient wraps async client with blocking post

- **Given:** A `Client<E>` built from a valid environment
- **When:** `SyncClient::new(client)` is called, then `sync_client.post("chat/completions", &request)`
- **Then:** The POST is executed synchronously via a dedicated tokio runtime; the response is the same typed result as the async version

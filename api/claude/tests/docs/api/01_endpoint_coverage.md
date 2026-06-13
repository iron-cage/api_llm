# API Spec: Endpoint Coverage

**Source:** [`docs/api/001_endpoint_coverage.md`](../../docs/api/001_endpoint_coverage.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| AP-01 | create_message() callable with correct path | core endpoint | ✅ |
| AP-02 | count_message_tokens() callable with correct path | core endpoint | ✅ |
| AP-03 | create_messages_batch() callable with correct path | core endpoint | ✅ |
| AP-04 | retrieve_batch() callable with correct path | core endpoint | ✅ |
| AP-05 | list_batches() callable with correct path | core endpoint | ✅ |
| AP-06 | cancel_batch() callable with correct path | core endpoint | ✅ |
| AP-07 | create_message_stream() only under streaming feature | feature-gated | ✅ |
| AP-08 | create_embedding() returns NotImplemented | not-available | ✅ |
| AP-09 | count_message_tokens() absent without count-tokens feature | feature-gated | ✅ |
| AP-10 | batch methods absent without batch-processing feature | feature-gated | ✅ |
| AP-11 | invalid credentials return authentication error | error path | ✅ |
| AP-12 | create_embeddings_batch() returns NotImplemented | not-available | ✅ |

---

### AP-01: create_message() callable with correct path

- **Given:** A `Client` with a valid secret
- **When:** `client.create_message(request)` is called with a well-formed `CreateMessageRequest`
- **Then:** The method exists on `Client` and issues a `POST` to `/v1/messages`; it returns a `Result<CreateMessageResponse, _>`; the response type contains the expected fields (`id`, `content`, `model`, `usage`)

---

### AP-02: count_message_tokens() callable with correct path

- **Given:** A `Client` with a valid secret
- **When:** `client.count_message_tokens(request)` is called with a well-formed request
- **Then:** The method exists on `Client` and issues a `POST` to `/v1/messages/count_tokens`; it returns a `Result<CountTokensResponse, _>`; the response type contains a token count field

---

### AP-03: create_messages_batch() callable with correct path

- **Given:** A `Client` with a valid secret
- **When:** `client.create_messages_batch(request)` is called with a well-formed batch request
- **Then:** The method exists on `Client` and issues a `POST` to `/v1/messages/batches`; it returns a `Result<BatchResponse, _>`; the response contains a batch `id`

---

### AP-04: retrieve_batch() callable with correct path

- **Given:** A `Client` with a valid secret and a known batch ID string
- **When:** `client.retrieve_batch(batch_id)` is called
- **Then:** The method exists on `Client` and issues a `GET` to `/v1/messages/batches/{id}`; it returns a `Result<BatchResponse, _>`; the response contains processing status information

---

### AP-05: list_batches() callable with correct path

- **Given:** A `Client` with a valid secret
- **When:** `client.list_batches(params)` is called with optional pagination parameters
- **Then:** The method exists on `Client` and issues a `GET` to `/v1/messages/batches`; it returns a `Result<ListBatchesResponse, _>`; the response contains a list of batch entries

---

### AP-06: cancel_batch() callable with correct path

- **Given:** A `Client` with a valid secret and a known batch ID string
- **When:** `client.cancel_batch(batch_id)` is called
- **Then:** The method exists on `Client` and issues a `POST` to `/v1/messages/batches/{id}/cancel`; it returns a `Result<BatchResponse, _>`; the response reflects the cancelled batch state

---

### AP-07: create_message_stream() only under streaming feature

- **Given:** The `streaming` Cargo feature is enabled at compile time
- **When:** `client.create_message_stream(request)` is called with a well-formed request
- **Then:** The method exists and returns an SSE stream type; the method is absent (does not compile) when the `streaming` feature is disabled; no streaming capability is available without the feature flag

---

### AP-08: create_embedding() returns NotImplemented

- **Given:** A `Client` with a valid secret; the `embeddings` Cargo feature may or may not be enabled
- **When:** `client.create_embedding(request)` is called
- **Then:** The method returns `Err(...)` containing a `NotImplemented` error variant; the error message clearly states that Anthropic does not expose an embeddings API endpoint; no HTTP request is made to any external endpoint

---

### AP-09: count_message_tokens() absent without count-tokens feature

- **Given:** The crate is compiled with the `count-tokens` Cargo feature disabled
- **When:** The crate's public API surface is inspected at compile time
- **Then:** `Client::count_message_tokens()` does not exist in the compiled crate; any code referencing the method fails to compile; no token-counting capability is exposed without the feature flag

---

### AP-10: batch methods absent without batch-processing feature

- **Given:** The crate is compiled with the `batch-processing` Cargo feature disabled
- **When:** The crate's public API surface is inspected at compile time
- **Then:** `Client::create_messages_batch()`, `Client::retrieve_batch()`, `Client::list_batches()`, and `Client::cancel_batch()` do not exist in the compiled crate; any code referencing these methods fails to compile; no batch processing capability is exposed without the feature flag

---

### AP-11: invalid credentials return authentication error

- **Given:** A `Client` constructed with a syntactically valid but rejected API key (e.g., a revoked key)
- **When:** `client.create_message(request)` is called against the real Anthropic API
- **Then:** Returns `Err(...)` containing an authentication error variant; the HTTP status 401 is reflected in the error; no partial or default response is returned

---

### AP-12: create_embeddings_batch() returns NotImplemented

- **Given:** A `Client` with a valid secret; the `embeddings` Cargo feature may or may not be enabled
- **When:** `client.create_embeddings_batch(requests)` is called
- **Then:** The method returns `Err(...)` containing a `NotImplemented` error variant; the error message clearly states that Anthropic does not expose an embeddings API endpoint; no HTTP request is made to any external endpoint

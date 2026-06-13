# Feature Spec: Core API

**Source:** [`docs/feature/001_core_api.md`](../../docs/feature/001_core_api.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| FT-01 | List Models endpoint reachable | core endpoint | ✅ |
| FT-02 | Get Model endpoint returns model details | core endpoint | ✅ |
| FT-03 | Generate Content endpoint returns candidates | core endpoint | ✅ |
| FT-04 | Stream Generate Content uses JSON array format | streaming | ✅ |
| FT-05 | Embed Content endpoint returns embedding values | core endpoint | ✅ |
| FT-06 | Count Tokens endpoint returns token count | core endpoint | ✅ |
| FT-07 | Multimodal content accepted in generate_content | content capability | ✅ |
| FT-08 | Error variants cover all failure categories | error handling | ✅ |

---

### FT-01: List Models endpoint reachable

- **Given:** A `Client` with a valid API key and the `enabled` feature compiled in
- **When:** `client.models().list().await` is called
- **Then:** A successful `ListModelsResponse` is returned containing at least one `Model`; the request targets `GET /v1beta/models` with the API key as a query parameter (`?key=...`), not as a Bearer header

---

### FT-02: Get Model endpoint returns model details

- **Given:** A `Client` with a valid API key; a known model identifier such as `"models/gemini-2.5-flash"`
- **When:** `client.models().by_name("models/gemini-2.5-flash").get().await` is called
- **Then:** A `Model` struct is returned with non-empty `name`, `supported_generation_methods`, and `input_token_limit` fields; the request targets `GET /v1beta/models/{model}`

---

### FT-03: Generate Content endpoint returns candidates

- **Given:** A `Client` with a valid API key; a `GenerateContentRequest` with one user `Content` containing a text `Part`
- **When:** `client.models().by_name(model).generate_content(&request).await` is called
- **Then:** A `GenerateContentResponse` is returned with at least one candidate; `candidates[0].content.parts[0].text` is non-empty; the request targets `POST /v1beta/models/{model}:generateContent`

---

### FT-04: Stream Generate Content uses JSON array format

- **Given:** A `Client` with the `streaming` feature compiled in; a valid `GenerateContentRequest`
- **When:** `stream_generate_content()` is called
- **Then:** The HTTP response body is parsed as a JSON array (`Vec<GenerateContentResponse>`), not as SSE events; the `Accept` request header is `application/json`; the full response is buffered before yielding stream elements; the request targets `POST /v1beta/models/{model}:streamGenerateContent`

---

### FT-05: Embed Content endpoint returns embedding values

- **Given:** A `Client` with a valid API key; an `EmbedContentRequest` with a text `Part` and `task_type: Some("RETRIEVAL_DOCUMENT")`
- **When:** `client.models().by_name("models/gemini-embedding-001").embed_content(&request).await` is called
- **Then:** An `EmbedContentResponse` is returned with `embedding.values` as a non-empty `Vec<f32>`; the request targets `POST /v1beta/models/{model}:embedContent`

---

### FT-06: Count Tokens endpoint returns token count

- **Given:** A `Client` with a valid API key; a `CountTokensRequest` with at least one content part
- **When:** `client.models().by_name(model).count_tokens(&request).await` is called
- **Then:** A `CountTokensResponse` is returned with `total_tokens` greater than zero; the request targets `POST /v1beta/models/{model}:countTokens`

---

### FT-07: Multimodal content accepted in generate_content

- **Given:** A `Client` with a valid API key; a `GenerateContentRequest` whose `Content` includes a `Part` with `inline_data: Some(Blob { mime_type: "image/jpeg", data: base64_string })`
- **When:** `generate_content()` is called with the multimodal request
- **Then:** The API accepts the request and returns a `GenerateContentResponse` with candidates describing the image content; no client-side rejection or transformation of the `inline_data` part occurs

---

### FT-08: Error variants cover all failure categories

- **Given:** Various error conditions: invalid API key, non-existent model name, malformed request, network timeout
- **When:** The corresponding API methods are called under each error condition
- **Then:** Each condition maps to a distinct `Error` variant — `AuthenticationError` for invalid key, `ApiError` for non-2xx HTTP response, `NetworkError` for connection failure, `InvalidArgument` for invalid parameter values; no error condition returns `Ok(...)` with a default value

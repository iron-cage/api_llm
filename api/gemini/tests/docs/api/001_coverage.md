# API Spec: Coverage

**Source:** [`docs/api/001_coverage.md`](../../docs/api/001_coverage.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| AP-01 | List Models endpoint has async and sync variants | core endpoint | ✅ |
| AP-02 | Get Model endpoint has async and sync variants | core endpoint | ✅ |
| AP-03 | Generate Content has async and sync variants | core endpoint | ✅ |
| AP-04 | Stream Generate Content has async variant only | core endpoint | ✅ |
| AP-05 | Embed Content has async and sync variants | core endpoint | ✅ |
| AP-06 | Batch Embed Contents has async and sync variants | core endpoint | ✅ |
| AP-07 | Count Tokens has async and sync variants | core endpoint | ✅ |
| AP-08 | Cached Content endpoint is implemented | core endpoint | ✅ |
| AP-09 | Google Search Grounding is implemented | advanced API | ✅ |
| AP-10 | Enhanced Function Calling is implemented | advanced API | ✅ |
| AP-11 | Code Execution tool is implemented | advanced API | ✅ |
| AP-12 | All operations return typed Result with Error enum | error handling | ✅ |

---

### AP-01: List Models endpoint has async and sync variants

- **Given:** A `Client` with a valid API key; the `enabled` feature compiled in
- **When:** `client.models().list().await` is called (async) and — with `sync_api` feature — `client.models().list()` is called synchronously
- **Then:** Both variants return a `ListModelsResponse` containing a non-empty `models` list; the async variant uses `await`; the sync variant blocks the calling thread; both target `GET /v1beta/models`

---

### AP-02: Get Model endpoint has async and sync variants

- **Given:** A `Client` with a valid API key; a known model identifier
- **When:** `client.models().by_name(id).get().await` is called (async) and the sync equivalent is called (with `sync_api` feature)
- **Then:** Both variants return a `Model` struct with the expected name and capabilities; the endpoint is `GET /v1beta/models/{model}`

---

### AP-03: Generate Content has async and sync variants

- **Given:** A `Client` with a valid API key; a `GenerateContentRequest` with at least one user content part
- **When:** `generate_content().await` is called (async) and — with `sync_api` feature — `generate_content()` is called synchronously
- **Then:** Both variants return a `GenerateContentResponse` with non-empty candidates; the endpoint is `POST /v1beta/models/{model}:generateContent`

---

### AP-04: Stream Generate Content has async variant only

- **Given:** A `Client` with the `streaming` feature compiled in; a valid `GenerateContentRequest`
- **When:** `stream_generate_content()` is called
- **Then:** A stream of `GenerateContentResponse` objects is yielded; no synchronous variant exists for this endpoint; the endpoint is `POST /v1beta/models/{model}:streamGenerateContent`

---

### AP-05: Embed Content has async and sync variants

- **Given:** A `Client` with a valid API key and an `EmbedContentRequest`
- **When:** `embed_content().await` (async) and the sync equivalent (with `sync_api`) are called
- **Then:** Both variants return an `EmbedContentResponse` with a non-empty `embedding.values` vector; the endpoint is `POST /v1beta/models/{model}:embedContent`

---

### AP-06: Batch Embed Contents has async and sync variants

- **Given:** A `Client` with a valid API key; a `BatchEmbedContentsRequest` with multiple requests
- **When:** `batch_embed_contents().await` (async) and the sync equivalent are called
- **Then:** Both variants return a `BatchEmbedContentsResponse` with one embedding per input request; the endpoint is `POST /v1beta/models/{model}:batchEmbedContents`

---

### AP-07: Count Tokens has async and sync variants

- **Given:** A `Client` with a valid API key; a `CountTokensRequest` with content parts
- **When:** `count_tokens().await` (async) and the sync equivalent are called
- **Then:** Both variants return a `CountTokensResponse` with `total_tokens` greater than zero; the endpoint is `POST /v1beta/models/{model}:countTokens`

---

### AP-08: Cached Content endpoint is implemented

- **Given:** A `Client` with a valid API key and the `enabled` feature
- **When:** `client.cached_content().create(&request).await` is called with a valid `CreateCachedContentRequest`
- **Then:** A cached content resource is created and a name identifier is returned; the endpoint is `POST /v1beta/cachedContents`; the cache name can be referenced in subsequent `GenerateContentRequest` via `cached_content` field

---

### AP-09: Google Search Grounding is implemented

- **Given:** A `Client` with a valid API key; a `GenerateContentRequest` with `Tool { google_search_retrieval: Some(GoogleSearchTool { .. }) }` in the `tools` field
- **When:** `generate_content()` is called
- **Then:** The API response includes `grounding_metadata` with source URLs; the search retrieval tool is passed transparently to the API without client-side transformation; no client-side web search is performed

---

### AP-10: Enhanced Function Calling is implemented

- **Given:** A `Client` with a valid API key; a `GenerateContentRequest` with `tools` containing a `FunctionDeclaration` and a `tool_config` with mode `AUTO`, `ANY`, or `NONE`
- **When:** `generate_content()` is called
- **Then:** The response may contain `function_call` parts requesting function execution; the tool config mode is forwarded to the API unchanged; no client-side function execution occurs

---

### AP-11: Code Execution tool is implemented

- **Given:** A `Client` with a valid API key; a `GenerateContentRequest` with `Tool { code_execution_tool: Some(CodeExecutionTool { .. }) }` in the `tools` field
- **When:** `generate_content()` is called
- **Then:** The response may contain `executable_code` parts (generated Python code) and `code_execution_result` parts (execution output); the code execution tool config is forwarded to the API; no client-side code execution occurs

---

### AP-12: All operations return typed Result with Error enum

- **Given:** Any core or advanced API method call under any error condition (auth failure, 4xx/5xx, network timeout, deserialization failure)
- **When:** The method returns
- **Then:** The return type is `Result<T, api_gemini::Error>` where `T` is the typed response struct; error conditions map to specific `Error` variants — `AuthenticationError`, `ApiError`, `NetworkError`, `SerializationError`; no operation returns a generic `Box<dyn Error>` from the public API surface

# Operation Spec: Usage Examples

**Source:** [`docs/operation/002_usage_examples.md`](../../docs/operation/002_usage_examples.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| OP-06 | Text generation extracts from candidates[0] | procedure step | ✅ |
| OP-07 | Multi-turn conversation builds alternating role history | procedure step | ✅ |
| OP-08 | Vision requires base64-encoded inline_data Part | procedure step | ✅ |
| OP-09 | Function calling uses FunctionDeclaration with JSON schema | procedure step | ✅ |
| OP-10 | System instructions set via system_instruction field | procedure step | ✅ |
| OP-11 | Embeddings use text-embedding-004 and return Vec<f32> | procedure step | ✅ |
| OP-12 | Sync API requires sync_api feature and sync_builder | procedure step | ✅ |
| OP-13 | Safety settings add 15-17 seconds to response time | timing note | ✅ |
| OP-14 | All operations return typed Result; no state modified | expected outcome | ✅ |

---

### OP-06: Text generation extracts from candidates[0]

- **Given:** A `GenerateContentRequest` with a user `Content` containing a single text `Part`
- **When:** `generate_content()` is called and a successful response is received
- **Then:** Generated text is extracted from `response.candidates[0].content.parts[0].text`; an optional `GenerationConfig` may be set with `temperature` (0.0–2.0), `top_k`, `top_p`, and `max_output_tokens` fields

---

### OP-07: Multi-turn conversation builds alternating role history

- **Given:** A conversation with prior user and model turns
- **When:** A `GenerateContentRequest` is constructed for the next user turn
- **Then:** The `contents` field contains a `Vec<Content>` with alternating `"user"` and `"model"` roles representing the full conversation history; the model continues the conversation from the final user turn; the response `Content` is appended to history for subsequent turns

---

### OP-08: Vision requires base64-encoded inline_data Part

- **Given:** An image file available to the caller
- **When:** A `GenerateContentRequest` for multimodal analysis is constructed
- **Then:** The image bytes are base64-encoded; the `Part` carries `inline_data: Some(Blob { mime_type: "image/jpeg", data: base64_string })`; the image `Part` and a text `Part` are both included in the same `Content`; no file upload step is needed for inline data

---

### OP-09: Function calling uses FunctionDeclaration with JSON schema

- **Given:** A local function available to be called by the model
- **When:** A `GenerateContentRequest` with function calling tools is constructed and `generate_content()` is called
- **Then:** A `FunctionDeclaration` with name, description, and JSON schema for parameters is defined; it is wrapped in `Tool { function_declarations: Some(vec![declaration]), .. }` and included in `tools`; the response may contain `function_call` parts; the function result is returned in the next turn as a `function_response` Part

---

### OP-10: System instructions set via system_instruction field

- **Given:** A `GenerateContentRequest` that requires a specific model persona or behavior constraint
- **When:** The request is constructed with a system instruction
- **Then:** A `SystemInstruction { role: "system", parts: vec![Part { text: instruction }] }` is created; it is set as `system_instruction: Some(instruction)` on the `GenerateContentRequest`; the instruction persists for the entire request; no separate API call is required to set system instructions

---

### OP-11: Embeddings use text-embedding-004 and return Vec<f32>

- **Given:** Text content to be embedded for semantic search or comparison
- **When:** `embed_content()` is called on the text-embedding-004 model
- **Then:** An `EmbedContentRequest` is constructed with `content: Content { parts: [text_part], role: "user" }` and `task_type: Some("RETRIEVAL_DOCUMENT")`; the response `embedding.values` is a `Vec<f32>` with 768 dimensions for the text-embedding-004 model

---

### OP-12: Sync API requires sync_api feature and sync_builder

- **Given:** A use case requiring synchronous (blocking) API calls
- **When:** The synchronous client is constructed and used
- **Then:** The `sync_api` Cargo feature must be enabled; the client is built via `Client::sync_builder().api_key(key).timeout(duration).build()`; the same API methods are called without `.await`; the internal async runtime is managed by the sync wrapper transparently

---

### OP-13: Safety settings add 15-17 seconds to response time

- **Given:** A `GenerateContentRequest` with `safety_settings: Some(vec![...])` containing `SafetySetting` entries
- **When:** The request is sent and a response is received
- **Then:** Response time is approximately 15-17 seconds due to safety analysis on Google's servers; test-level timeouts for safety requests must be at least 25 seconds; the four available categories are Harassment, HateSpeech, SexuallyExplicit, and DangerousContent

---

### OP-14: All operations return typed Result; no state modified

- **Given:** Any API operation (generate content, embed, count tokens, list models)
- **When:** The operation completes successfully or fails
- **Then:** The return value is a typed `Result<Response, api_gemini::Error>` — success contains the typed response struct; failure contains an explicit actionable `Error` variant; no local system state is modified by any operation (except cached content creation, which can be reversed via `delete()`)

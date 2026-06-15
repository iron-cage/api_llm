# API Spec: Endpoint Coverage
**Source:** `../../docs/api/001_endpoint_coverage.md`

## Test Cases

### AP-01: Chat completion returns typed response ✅

- **Given:** An authenticated client with a valid XAI_API_KEY
- **When:** `client.chat().create()` is called with a ChatCompletionRequest containing model and messages
- **Then:** A `ChatCompletionResponse` is returned with a non-empty `id` and at least one choice containing message content
- **Test:** `integration_chat.rs::chat_completion_basic_request_succeeds`

### AP-02: Model listing returns at least one Grok model ✅

- **Given:** An authenticated client with a valid XAI_API_KEY
- **When:** `client.models().list()` is called
- **Then:** The response contains at least one model whose `id` contains "grok"
- **Test:** `integration_models.rs::models_list_returns_valid_structure`, `models_list_contains_grok_3`

### AP-03: Get model by ID returns metadata ✅

- **Given:** An authenticated client and a known model ID (e.g., one from the list response)
- **When:** `client.models().get(model_id)` is called with that ID
- **Then:** The response contains the model metadata with `id` matching the requested ID
- **Test:** `integration_models.rs::model_get_grok_3_returns_details`

### AP-04: Authentication failure returns error_tools::Error ✅

- **Given:** A client constructed with an invalid API key (e.g., "xai-invalid-key")
- **When:** Any API method (`chat().create()`, `models().list()`) is called
- **Then:** The method returns `Err(error_tools::Error)` — not a panic, not an HTTP 200 with error body
- **Test:** `integration_chat.rs::chat_completion_invalid_model_returns_error`

### AP-05: Streaming chat returns SSE event stream ✅

- **Given:** An authenticated client with `streaming` feature enabled
- **When:** `client.chat().create_stream()` is called with a valid ChatCompletionRequest
- **Then:** The response is an SSE stream yielding one or more chunk events, each containing delta content
- **Test:** `integration_streaming.rs::streaming_chat_completion_delivers_chunks`

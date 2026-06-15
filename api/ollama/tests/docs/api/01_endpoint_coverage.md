# API Spec: Endpoint Coverage

**Source:** `../../docs/api/001_endpoint_coverage.md`

### AP-01: Chat completion returns typed ChatResponse ✅

- **Given:** A valid `ChatRequest` with a model name and at least one message
- **When:** The chat completion method is called against `/api/chat`
- **Then:** The method returns `Ok(ChatResponse)` containing a response message with non-empty content
- **Test:** `api_comprehensive_tests.rs::test_integration_simple_chat`

### AP-02: Model listing returns available models ✅

- **Given:** A connected client with at least one model pulled locally
- **When:** The model listing method is called against `/api/tags`
- **Then:** The method returns a typed response containing a non-empty list of model entries
- **Test:** `api_comprehensive_tests.rs::test_integration_list_models`

### AP-03: Text generation returns typed GenerateResponse ✅

- **Given:** A valid `GenerateRequest` with a model name and a prompt string
- **When:** The text generation method is called against `/api/generate`
- **Then:** The method returns `Ok(GenerateResponse)` with a non-empty `response` field
- **Test:** `api_comprehensive_tests.rs::test_integration_simple_generation`

### AP-04: API methods return error on connection failure ✅

- **Given:** A client configured to connect to an unreachable host (e.g., `http://localhost:1`)
- **When:** Any API method (chat, generate, tags, show) is called
- **Then:** The method returns `Err(error_tools::Error)` — it does not panic or hang
- **Test:** `error_handling_tests.rs::test_chat_network_error`, `test_generate_network_error`, `test_list_models_network_error`, `test_model_info_network_error`

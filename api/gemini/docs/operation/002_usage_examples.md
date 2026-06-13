# Operation: Usage Examples

### Scope

- **Purpose**: Describe common api_gemini operation patterns and their required inputs/outputs
- **Responsibility**: Document each major API usage pattern as an abstract procedure with steps
- **In Scope**: API call patterns, required request fields, expected response shapes, async and sync variants
- **Out of Scope**: Complete runnable code (see `examples/`), enterprise feature configuration, protocol wire format

### Prerequisites

- api_gemini crate with appropriate feature flags enabled (`enabled`, `full`, or individual features)
- Gemini API key loaded via `GEMINI_API_KEY` environment variable or workspace secret (`secret/-secrets.sh`)
- Async runtime (tokio) for async operations; `sync_api` feature enabled for synchronous operations

### Procedure Steps

Each pattern follows the same structure: configure the client, construct a typed request, call the appropriate method, extract the result from the typed response.

#### Text Generation
1. Construct `GenerateContentRequest` with one `Content` (role: `"user"`) containing a `Part` with the prompt text
2. Optionally set `GenerationConfig` fields: `temperature` (0.0–2.0), `top_k`, `top_p`, `max_output_tokens`
3. Call `client.models().by_name(model).generate_content(&request).await`
4. Extract text from `response.candidates[0].content.parts[0].text`

#### Multi-turn Conversations
1. Build `Vec<Content>` alternating roles: `"user"` → `"model"` → `"user"` (conversation history)
2. Set `GenerateContentRequest { contents: history, .. }`
3. Call `generate_content` — model continues the conversation from the final user turn
4. Append the response `Content` to history for subsequent turns

#### Vision (Multimodal)
1. Read image bytes and base64-encode them
2. Create a `Part` with `inline_data: Blob { mime_type: "image/jpeg", data: base64_string }`
3. Include the image `Part` alongside a text `Part` in the same `Content`
4. Call `generate_content` — model analyzes the image in the context of the text prompt

#### Function Calling
1. Define `FunctionDeclaration` with name, description, and JSON schema for parameters
2. Wrap in `Tool { function_declarations: Some(vec![declaration]), .. }`
3. Include `tools: Some(tools)` in `GenerateContentRequest`
4. Call `generate_content` — response may include `function_call` parts requesting function execution
5. Execute the function locally and return the result as a `function_response` Part in the next turn

#### Google Search Grounding
1. Create `Tool { google_search_retrieval: Some(GoogleSearchTool { config: None }), .. }`
2. Include the tool in `GenerateContentRequest`
3. Call `generate_content` — model queries Google Search and incorporates results with citations
4. Access `response.grounding_metadata.grounding_chunks` for source URLs and attribution

#### System Instructions
1. Create `SystemInstruction { role: "system", parts: vec![Part { text: instruction }] }`
2. Set `system_instruction: Some(instruction)` in `GenerateContentRequest`
3. The instruction persists for the entire request, shaping model behavior and persona

#### Code Execution
1. Create `CodeExecutionTool { config: Some(CodeExecutionConfig { timeout: Some(30), enable_network: Some(false) }) }`
2. Wrap in `Tool { code_execution_tool: Some(tool), .. }`
3. Call `generate_content` — model may generate and execute Python code within the request
4. Response includes `executable_code` parts (generated code) and `code_execution_result` parts (output)

#### Embeddings
1. Create `EmbedContentRequest { content: Content { parts: [text_part], role: "user" }, task_type: Some("RETRIEVAL_DOCUMENT") }`
2. Call `client.models().by_name("models/gemini-embedding-001").embed_content(&request).await`
3. Access `response.embedding.values` — a `Vec<f32>` of embedding dimensions (768 for gemini-embedding-001)

#### Model Information
- List all available models: `client.models().list().await` → `ListModelsResponse { models: Vec<Model> }`
- Get specific model details: `client.models().get("models/gemini-2.5-flash").await` → `Model` with `input_token_limit`, `supported_generation_methods`, etc.

#### Synchronous API
1. Enable the `sync_api` cargo feature
2. Build client with `Client::sync_builder().api_key(key).timeout(duration).build()`
3. Call the same methods without `.await` — the runtime is managed internally

#### Safety Settings
1. Create `Vec<SafetySetting>` with `HarmCategory` variant and `HarmBlockThreshold` variant for each category
2. Set `safety_settings: Some(safety_settings)` in `GenerateContentRequest`
3. Note: safety settings processing adds approximately 15-17 seconds to response time (vs ~0.5s for plain text)
4. Available categories: Harassment, HateSpeech, SexuallyExplicit, DangerousContent

#### Server-side Cached Content
1. Create `CreateCachedContentRequest` with model, conversation context, and `ttl: Some("3600s")`
2. Call `client.cached_content().create(&request).await` → returns cache with a `name` identifier
3. Reference the cache name in subsequent requests via `cached_content: Some(cache_name)` in `GenerateContentRequest`
4. Reduces token costs for requests that share large context (system prompts, document content)

### Expected Outcome

Each operation returns a typed `Result<Response, api_gemini::Error>`. Successful responses contain the generated content, embeddings, or model metadata depending on the operation. Errors are explicit and actionable (authentication failure, rate limit, timeout, deserialization).

### Rollback Procedure

Not applicable — all operations are stateless requests to the Gemini API. No local state is modified, except cached content creation (which can be deleted via `client.cached_content().delete(name).await`).

### Sources

| File | Relationship |
|------|-------------|
| `src/models/api/content_generation/api_impl.rs` | Core generate_content implementation |
| `src/client/` | Client construction and secret loading |
| `examples/` | Runnable example programs for each operation pattern |

### Tests

| File | Relationship |
|------|-------------|
| `tests/inc/messages_api_test.rs` | Text generation and conversation tests |
| `tests/inc/vision_support_test.rs` | Multimodal and vision tests |
| `tests/inc/embeddings_test.rs` | Embedding operation tests |
| `tests/inc/tool_calling_test.rs` | Function calling tests |
| `tests/inc/sync_api_test.rs` | Synchronous API tests |
| `tests/inc/system_instructions_test.rs` | System instruction tests |
| `tests/inc/sync_cached_content_test.rs` | Server-side caching tests |

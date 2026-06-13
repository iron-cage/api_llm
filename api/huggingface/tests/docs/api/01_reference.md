# API Spec: API Reference

Spec scenarios for `docs/api/001_reference.md`. Verifies that the documented public API contract is accurate and testable.

### AP-01: Inference create returns generated text

- **Given:** a valid `HUGGINGFACE_API_KEY` and a Router API model (`meta-llama/Llama-3.3-70B-Instruct`)
- **When:** `client.inference().create("What is 2+2?", model)` is awaited
- **Then:** `InferenceResponse` is returned with `generated_text` containing a non-empty `String`

### AP-02: Embeddings create returns float vector

- **Given:** a valid API key and an embedding model (`BAAI/bge-large-en-v1.5`)
- **When:** `client.embeddings().create("hello world", model)` is awaited
- **Then:** `EmbeddingResponse` is returned with `embeddings[0]` as a `Vec<f32>` of length ≥ 1

### AP-03: Similarity returns value in range [-1.0, 1.0]

- **Given:** a valid API key, two identical text strings, and an embedding model
- **When:** `client.embeddings().similarity(text, text, model)` is awaited
- **Then:** the returned `f32` value is in the range `[-1.0, 1.0]` and is ≥ `0.99` for identical texts

### AP-04: Streaming create returns sequential chunks

- **Given:** a valid API key and a Router API generation model (`meta-llama/Llama-3.3-70B-Instruct`)
- **When:** `client.inference().create_stream(prompt, model, params)` is awaited and the returned `Receiver<Result<String>>` is polled via `rx.recv().await`
- **Then:** at least one non-empty `String` chunk is received before the channel yields `None`; each `Ok(text)` variant is a non-empty string fragment from the streamed response

### AP-05: Invalid API key returns error without panic

- **Given:** a `Client` constructed with an invalid API key (e.g., `"hf_invalid"`)
- **When:** `client.inference().create(prompt, model)` is awaited
- **Then:** a `HuggingFaceError::Authentication` or `HuggingFaceError::Http` variant is returned; the process does not panic

### AP-06: Model management get returns info or ModelUnavailable

- **Given:** a valid API key and a known model identifier
- **When:** `client.models().get(model)` is awaited
- **Then:** either model metadata is returned successfully, or `HuggingFaceError::ModelUnavailable` is returned; no panic occurs

### AP-07: Chat completion returns assistant reply

- **Given:** a valid API key and a Router API chat model (`meta-llama/Llama-3.3-70B-Instruct`)
- **When:** `client.providers().chat_completion(model, messages, None, None, None)` is awaited with `messages` containing a single `ChatMessage` with `role = "user"` and `content = "What is 2+2?"`
- **Then:** a `ChatCompletionResponse` is returned with `choices[0].message.content` containing a non-empty `String`; no panic occurs

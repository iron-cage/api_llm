# Protocol: Streaming Format

### Scope

- **Purpose**: Specify the wire protocol used by the Gemini streaming endpoint
- **Responsibility**: Document message format, structure, and parsing strategy for `:streamGenerateContent`
- **In Scope**: HTTP response format, JSON array structure, message types, buffering strategy
- **Out of Scope**: Client-level streaming control features (pause/resume/cancel), SSE format (not used by Gemini)

### Abstract

The Gemini `:streamGenerateContent` endpoint returns a complete JSON array of `GenerateContentResponse` objects rather than a Server-Sent Events stream. The client buffers the full response before yielding individual array elements as stream chunks to the caller.

HTTP response headers carry `Content-Type: application/json`. Requests must send `Accept: application/json` (not `text/event-stream`).

See `../investigations/001_streaming_format.md` for the debugging investigation that discovered this format.

### Message Structure

The HTTP response body is a top-level JSON array. Each element is a complete `GenerateContentResponse` object. The array may contain one or more elements depending on response length and chunking decisions by the Gemini backend. This contrasts with SSE format (`data: {...}\n\n`), which Gemini does not use for this endpoint.

| Layer | Description |
|-------|-------------|
| HTTP response body | Top-level JSON array — zero or more `GenerateContentResponse` objects |
| Array element type | `GenerateContentResponse` object |
| Element count | One or more, determined by Gemini backend |
| Excluded format | SSE (`data: {...}\n\n`) — not used by this endpoint |
| Excluded format | NDJSON (one JSON object per line) — not used by this endpoint |

### Message Types

#### Partial Response

Intermediate array elements carry content parts without a terminal `finishReason`.

Present fields:
- `candidates[].content.parts[].text` — incremental generated text
- `usageMetadata.promptTokenCount` — prompt token count (constant across chunks)
- `usageMetadata.candidatesTokenCount` — tokens generated so far

Absent or null fields: `candidates[].finishReason`, `candidates[].safetyRatings`

#### Final Response

The last array element carries a terminal `finishReason` and complete usage metadata.

Present fields (all partial fields plus):
- `candidates[].finishReason` — terminal reason; `"STOP"` for normal completion
- `candidates[].safetyRatings` — per-category safety evaluation results
- `usageMetadata.totalTokenCount` — total tokens consumed by the request

### Version Compatibility

- Observed behavior: API `v1beta`, Gemini models as of 2025-10-12
- Gemini does not document SSE for `:streamGenerateContent` — verify format by inspecting raw HTTP responses if behavior changes after API updates
- If Google changes the response format in a future API version, update parsing logic in `src/streaming/client_impl.rs`
- The `Accept: application/json` request header is required; `text/event-stream` will not produce the correct response
- The `batch_operations` endpoint uses a different format; this spec applies only to `:streamGenerateContent`

### Sources

| File | Relationship |
|------|-------------|
| `src/streaming/client_impl.rs` | JSON array buffering implementation |
| `../investigations/001_streaming_format.md` | Full debugging investigation record |

### Tests

| File | Relationship |
|------|-------------|
| `tests/inc/streaming_test.rs` | Integration tests validating streaming behavior |
| `tests/inc/comprehensive_integration_test.rs` | Streaming format documentation in test comments |

# Protocol Spec: Streaming Format

**Source:** [`docs/protocol/001_streaming_format.md`](../../docs/protocol/001_streaming_format.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| PR-01 | Streaming response is a JSON array, not SSE | wire format | ✅ |
| PR-02 | Request sends Accept: application/json header | request header | ✅ |
| PR-03 | Full response is buffered before yielding stream elements | buffering | ✅ |
| PR-04 | Partial response elements lack finishReason | partial message | ✅ |
| PR-05 | Final response element carries finishReason STOP | final message | ✅ |
| PR-06 | Final response includes safetyRatings and totalTokenCount | final message | ✅ |
| PR-07 | Content-Type response header is application/json | response header | ✅ |
| PR-08 | SSE text/event-stream format is not used by this endpoint | format exclusion | ✅ |

---

### PR-01: Streaming response is a JSON array, not SSE

- **Given:** A `stream_generate_content()` call to `POST /v1beta/models/{model}:streamGenerateContent`
- **When:** The HTTP response body is received
- **Then:** The body is a complete JSON array `[GenerateContentResponse, ...]` — a top-level array with zero or more `GenerateContentResponse` objects; the format is not Server-Sent Events (`data: {...}\n\n`); the format is not NDJSON (one JSON object per line)

---

### PR-02: Request sends Accept: application/json header

- **Given:** A streaming request to `:streamGenerateContent`
- **When:** The HTTP request headers are inspected
- **Then:** The request includes `Accept: application/json`; it does not include `Accept: text/event-stream`; sending the wrong `Accept` header will not produce the correct JSON array response format

---

### PR-03: Full response is buffered before yielding stream elements

- **Given:** A `stream_generate_content()` call returning a multi-element JSON array
- **When:** The client processes the HTTP response
- **Then:** The entire HTTP response body is read and buffered before any `GenerateContentResponse` elements are yielded to the caller; there is no first-chunk latency improvement — the caller waits for the complete response; individual array elements are yielded from the parsed `Vec<GenerateContentResponse>` after buffering completes

---

### PR-04: Partial response elements lack finishReason

- **Given:** A streaming response array with more than one element
- **When:** A non-final (intermediate) array element is inspected
- **Then:** `candidates[].finishReason` is absent or null on partial elements; `candidates[].safetyRatings` is absent or null on partial elements; the element carries `candidates[].content.parts[].text` with incremental generated text and `usageMetadata.promptTokenCount` and `usageMetadata.candidatesTokenCount`

---

### PR-05: Final response element carries finishReason STOP

- **Given:** A streaming response array representing a normally completed generation
- **When:** The last element of the array is inspected
- **Then:** `candidates[0].finishReason` equals `"STOP"` for normal completion; the final element is the last entry in the JSON array; earlier elements do not carry `finishReason`

---

### PR-06: Final response includes safetyRatings and totalTokenCount

- **Given:** The last element of a streaming response array
- **When:** Its fields are inspected
- **Then:** `candidates[].safetyRatings` is present with per-category safety evaluation results; `usageMetadata.totalTokenCount` is present with the total tokens consumed by the request; these fields are absent on all non-final elements

---

### PR-07: Content-Type response header is application/json

- **Given:** An HTTP response from `POST /v1beta/models/{model}:streamGenerateContent`
- **When:** The response headers are inspected
- **Then:** The `Content-Type` response header is `application/json`; it is not `text/event-stream`; the parsing strategy (full buffer then `serde_json::from_str::<Vec<GenerateContentResponse>>`) is consistent with this content type

---

### PR-08: SSE text/event-stream format is not used by this endpoint

- **Given:** A developer familiar with other LLM streaming APIs that use SSE
- **When:** The Gemini `:streamGenerateContent` endpoint response is processed
- **Then:** SSE parsing logic (splitting on `data: ` prefix and `\n\n` separators) will fail to produce valid output from this endpoint; the correct parsing strategy is to buffer the full response and parse it as a JSON array; this endpoint-specific behavior is documented to prevent incorrect SSE-based implementations

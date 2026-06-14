# Feature Spec: Streaming

**Source:** [`docs/feature/001_streaming.md`](../../../docs/feature/001_streaming.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| FT-01 | First chunk carries Delta::role field | first-chunk-role | ✅ |
| FT-02 | Final chunk has ChunkChoice::finish_reason set | final-chunk-finish-reason | ✅ |
| FT-03 | Intermediate chunk has no finish_reason and no role | intermediate-chunk | ✅ |
| FT-04 | Delta::default() produces valid empty delta | default-state | ✅ |
| FT-05 | Optional Delta fields absent from JSON when None | skip-serializing | ✅ |
| FT-06 | ChatCompletionChunk round-trips through serde | round-trip | ✅ |

---

### FT-01: First chunk carries Delta::role field

- **Given:** A JSON string representing the first SSE chunk in a streaming response, where `choices[0].delta.role = "assistant"` and `choices[0].finish_reason` is absent
- **When:** Deserialized with `serde_json::from_str::<ChatCompletionChunk>`
- **Then:** `chunk.choices[0].delta.role` equals `Some("assistant")`; `chunk.choices[0].finish_reason` is `None`

---

### FT-02: Final chunk has ChunkChoice::finish_reason set

- **Given:** A JSON string representing the final SSE chunk, where `choices[0].finish_reason = "stop"` and `choices[0].delta` contains no `role` or `content` fields
- **When:** Deserialized with `serde_json::from_str::<ChatCompletionChunk>`
- **Then:** `chunk.choices[0].finish_reason` equals `Some("stop")`; `chunk.choices[0].delta.role` is `None`

---

### FT-03: Intermediate chunk has no finish_reason and no role

- **Given:** A JSON string representing an intermediate SSE chunk, where `choices[0].delta.content = " world"`, `choices[0].delta` has no `role` key, and `choices[0].finish_reason` is absent
- **When:** Deserialized with `serde_json::from_str::<ChatCompletionChunk>`
- **Then:** `chunk.choices[0].delta.content` equals `Some(" world")`; `chunk.choices[0].delta.role` is `None`; `chunk.choices[0].finish_reason` is `None`

---

### FT-04: Delta::default() produces valid empty delta

- **Given:** No input; the `Default` trait implementation on `Delta`
- **When:** `Delta::default()` is called
- **Then:** `delta.role` is `None`; `delta.content` is `None`; `delta.tool_calls` is `None`; the struct is valid and `serde_json::to_string(&delta)` succeeds

---

### FT-05: Optional Delta fields absent from JSON when None

- **Given:** A `Delta` with `content: Some("world")` and `role: None`, `tool_calls: None`
- **When:** Serialized with `serde_json::to_string`
- **Then:** The JSON contains the `content` key with value `"world"`; the JSON does not contain `role` or `tool_calls` keys; `skip_serializing_if = "Option::is_none"` is active for all optional fields

---

### FT-06: ChatCompletionChunk round-trips through serde

- **Given:** A `ChatCompletionChunk` with `id`, `object`, `created`, `model` set and one `ChunkChoice` with `delta.content = Some("text")`
- **When:** Serialized with `serde_json::to_string` then deserialized back with `serde_json::from_str::<ChatCompletionChunk>`
- **Then:** The deserialized chunk equals the original struct; all fields survive the round-trip without loss or mutation

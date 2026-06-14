# API Spec: Chat Completion

**Source:** [`docs/api/002_chat_completion.md`](../../../docs/api/002_chat_completion.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| AP-01 | Minimal request serializes without optional fields | optional-field-omission | ✅ |
| AP-02 | Full response deserializes from JSON fixture | response-deserialization | ✅ |
| AP-03 | Role variants serialize to correct wire strings | role-serialization | ✅ |
| AP-04 | FunctionCall::arguments preserved as raw JSON string | tool-arguments-string | ✅ |
| AP-05 | Response with unknown JSON fields deserializes without error | forward-compat | ✅ |
| AP-06 | ToolCall in response deserializes with tool_type = "function" | tool-type-field | ✅ |
| AP-07 | Choice finish_reason "tool_calls" deserializes correctly | finish-reason-tool-calls | ✅ |

---

### AP-01: Minimal request serializes without optional fields

- **Given:** A `ChatCompletionRequest` built with only `model = "gpt-4o"` and `messages` containing a single user message; all optional fields (`temperature`, `max_tokens`, `top_p`, `frequency_penalty`, `presence_penalty`, `stream`, `tools`) are `None`
- **When:** Serialized with `serde_json::to_string`
- **Then:** The JSON contains exactly the `model` and `messages` keys; none of the optional field keys appear in the serialized output; the serialized length is minimal

---

### AP-02: Full response deserializes from JSON fixture

- **Given:** A valid JSON string matching the `ChatCompletionResponse` schema: `id`, `object = "chat.completion"`, `created` (u64), `model`, `choices` (array with one entry containing `index`, `message`, `finish_reason`), and `usage` (`prompt_tokens`, `completion_tokens`, `total_tokens`)
- **When:** Deserialized with `serde_json::from_str::<ChatCompletionResponse>`
- **Then:** All fields populate correctly; `choices[0].message.content` matches the fixture value; `usage.total_tokens` equals `prompt_tokens + completion_tokens`

---

### AP-03: Role variants serialize to correct wire strings

- **Given:** Four `Message` structs, each using a distinct `Role` variant (`Role::System`, `Role::User`, `Role::Assistant`, `Role::Tool`) with identical content
- **When:** Each is serialized with `serde_json::to_string`
- **Then:** `Role::System` serializes as `"system"`; `Role::User` serializes as `"user"`; `Role::Assistant` serializes as `"assistant"`; `Role::Tool` serializes as `"tool"`

---

### AP-04: FunctionCall::arguments preserved as raw JSON string

- **Given:** A JSON string for a `Message` in the assistant role where `tool_calls[0].function.arguments = "{\"location\":\"Paris\"}"` (a JSON-encoded string value, not a nested object)
- **When:** Deserialized with `serde_json::from_str::<Message>`
- **Then:** `message.tool_calls.unwrap()[0].function.arguments` is the raw `String` `"{\"location\":\"Paris\"}"`, not a parsed `serde_json::Value`; callers must re-parse it explicitly

---

### AP-05: Response with unknown JSON fields deserializes without error

- **Given:** A `ChatCompletionResponse` JSON string containing an extra unknown top-level key, such as `"system_fingerprint": "fp_abc123"`, alongside all required fields
- **When:** Deserialized with `serde_json::from_str::<ChatCompletionResponse>`
- **Then:** Deserialization succeeds and returns `Ok`; all known fields are populated correctly; the unknown field is silently ignored (forward compatibility)

---

### AP-06: ToolCall in response deserializes with tool_type = "function"

- **Given:** A JSON string for a `ToolCall` object with `"id": "call_abc"`, `"type": "function"`, and a `"function"` sub-object containing `name` and `arguments`
- **When:** Deserialized with `serde_json::from_str::<ToolCall>`
- **Then:** `tool_call.tool_type` equals `"function"`; `tool_call.id` equals `"call_abc"`; `tool_call.function.name` and `tool_call.function.arguments` are populated correctly

---

### AP-07: Choice finish_reason "tool_calls" deserializes correctly

- **Given:** A JSON string for a `ChatCompletionResponse` where `choices[0].finish_reason = "tool_calls"` and `choices[0].message.tool_calls` is a non-empty array
- **When:** Deserialized with `serde_json::from_str::<ChatCompletionResponse>`
- **Then:** `choices[0].finish_reason` equals `Some("tool_calls")`; `choices[0].message.tool_calls` is `Some(_)` with at least one entry; the response is valid

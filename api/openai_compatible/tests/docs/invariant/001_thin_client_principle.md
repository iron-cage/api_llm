# Invariant Spec: Thin Client Principle

**Source:** [`docs/invariant/001_thin_client_principle.md`](../../../docs/invariant/001_thin_client_principle.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| IN-01 | Optional fields absent from request JSON when None | no-implicit-defaults | ✅ |
| IN-02 | Streaming not activated without explicit stream: true | no-auto-streaming | ✅ |
| IN-03 | Headers come exclusively from environment.headers() | no-implicit-mutation | ✅ |
| IN-04 | Timeout returned unchanged — no clamping or auto-override | no-auto-modification | ✅ |

---

### IN-01: Optional fields absent from request JSON when None

- **Given:** A `ChatCompletionRequest` with only `model` and one user `Message` set; all other fields (`temperature`, `max_tokens`, `top_p`, `frequency_penalty`, `presence_penalty`, `stream`, `tools`) are `None`
- **When:** The request is serialized with `serde_json::to_string`
- **Then:** The resulting JSON contains exactly the `model` and `messages` keys; no optional field key appears anywhere in the JSON — the client injects no implicit defaults on behalf of the caller

---

### IN-02: Streaming not activated without explicit stream: true

- **Given:** A `ChatCompletionRequest` with `stream` set to `None` (the default state when the field is omitted from the builder)
- **When:** The request is serialized with `serde_json::to_string`
- **Then:** The JSON does not contain a `stream` key; the server will apply its own default; the client never activates streaming behavior without an explicit `stream: Some(true)` from the caller

---

### IN-03: Headers come exclusively from environment.headers()

- **Given:** An `OpenAiCompatEnvironmentImpl` constructed with a known API key
- **When:** `env.headers()` is called
- **Then:** The returned `HeaderMap` contains exactly `Authorization` (`Bearer <key>`) and `Content-Type` (`application/json`); no additional headers are injected by the client layer beyond what the environment provides

---

### IN-04: Timeout returned unchanged — no clamping or auto-override

- **Given:** An `OpenAiCompatEnvironmentImpl` built with `.with_timeout(Duration::from_secs(300))`
- **When:** `env.timeout()` is called
- **Then:** The returned `Duration` equals `Duration::from_secs(300)` exactly; the client layer does not clamp, modify, or override the caller-supplied timeout value

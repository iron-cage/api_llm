# Invariant Spec: Thin Client Principle

**Source:** [`docs/invariant/001_thin_client_principle.md`](../../docs/invariant/001_thin_client_principle.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| IN-01 | Client::new() activates no enterprise features | default state | ✅ |
| IN-02 | No auto-retry without explicit RetryConfig | prohibited behavior | ✅ |
| IN-03 | No implicit caching without explicit cache config | prohibited behavior | ✅ |
| IN-04 | generate_content() issues exactly one HTTP request | direct mapping | ✅ |
| IN-05 | Errors propagate without silent swallowing | transparent error | ✅ |
| IN-06 | No auto rate-limiting without explicit RateLimitingConfig | prohibited behavior | ✅ |

---

### IN-01: Client::new() activates no enterprise features

- **Given:** A `Client` constructed via `Client::new()` with a valid API key and no enterprise configuration
- **When:** Any API method is called on the client
- **Then:** No retry logic executes on failure; no caching layer intercepts the request; no rate limiting throttles the call; the raw result from the Gemini API is returned directly to the caller without any client-side transformation

---

### IN-02: No auto-retry without explicit RetryConfig

- **Given:** A `Client` constructed without enterprise configuration; the Gemini API endpoint returns an error response
- **When:** `generate_content()` is called and the API returns an error
- **Then:** The error is returned to the caller immediately without any retry attempt; the HTTP client makes exactly one outbound request per `generate_content()` call

---

### IN-03: No implicit caching without explicit cache config

- **Given:** A `Client` constructed without enterprise configuration; the same `GenerateContentRequest` is sent twice consecutively
- **When:** `generate_content()` is called twice with identical request parameters
- **Then:** Two distinct HTTP requests are made to the Gemini API; no cached response is returned for the second call; both results come from live API responses

---

### IN-04: generate_content() issues exactly one HTTP request

- **Given:** A `Client` with a valid API key and no enterprise config
- **When:** `generate_content()` is called once with a valid request
- **Then:** Exactly one HTTP POST is issued to `POST /v1beta/models/{model}:generateContent`; no preflight, no parallel, and no follow-up requests are made by the client itself; the method returns after the single response is received

---

### IN-05: Errors propagate without silent swallowing

- **Given:** A `Client` with a valid API key; the Gemini API returns an HTTP error status (4xx or 5xx)
- **When:** `generate_content()` is called
- **Then:** The method returns `Err(...)` containing the API error details; the error is not logged-and-suppressed, not converted to a default value, and not silently retried; the caller receives the full error information

---

### IN-06: No auto rate-limiting without explicit RateLimitingConfig

- **Given:** A `Client` constructed via `Client::new()` with no enterprise config; the `rate_limiting` Cargo feature may or may not be compiled in
- **When:** Multiple `generate_content()` calls are made in rapid succession
- **Then:** No rate limiting is applied; each call is dispatched immediately without any throttling or delay introduced by the client; rate limiting only activates when the caller explicitly provides a `RateLimitingConfig` via the builder

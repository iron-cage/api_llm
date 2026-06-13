# Invariant Spec: Thin Client Principle

**Source:** [`docs/invariant/001_thin_client_principle.md`](../../docs/invariant/001_thin_client_principle.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| IN-01 | Client::new() activates no enterprise features | default state | ✅ |
| IN-02 | No auto-retry without explicit RetryConfig | prohibited behavior | ✅ |
| IN-03 | No implicit caching without CacheControl | prohibited behavior | ✅ |
| IN-04 | create_message() issues exactly one HTTP request | direct mapping | ✅ |
| IN-05 | Errors propagate without silent swallowing | transparent error | ✅ |
| IN-06 | No auto rate-limiting without explicit RateLimiterConfig | prohibited behavior | ✅ |

---

### IN-01: Client::new() activates no enterprise features

- **Given:** A `Client` constructed via `Client::new(secret)` with a valid secret and no enterprise config passed
- **When:** Any API method is called on the client
- **Then:** No retry logic executes on failure; no caching layer intercepts the request; the raw result from the API is returned directly to the caller without automatic transformation

---

### IN-02: No auto-retry without explicit RetryConfig

- **Given:** A `Client` constructed without enterprise configuration; the API endpoint returns an error response
- **When:** `create_message()` is called and the network or API returns an error
- **Then:** The error is returned to the caller immediately without any retry attempt; the HTTP client makes exactly one outbound request per `create_message()` call

---

### IN-03: No implicit caching without CacheControl

- **Given:** A `Client` constructed without enterprise configuration; the same message request is sent twice consecutively
- **When:** `create_message()` is called twice with identical request parameters
- **Then:** Two distinct HTTP requests are made to the API endpoint; no cached response is returned for the second call; both results come from live API responses

---

### IN-04: create_message() issues exactly one HTTP request

- **Given:** A `Client` with a valid secret and no enterprise config
- **When:** `create_message()` is called once with a valid request
- **Then:** Exactly one HTTP POST is issued to `/v1/messages`; no preflight, no parallel, no follow-up requests are made by the client itself; the method returns after the single response is received

---

### IN-05: Errors propagate without silent swallowing

- **Given:** A `Client` with a valid secret; the API returns an HTTP error status (4xx or 5xx)
- **When:** `create_message()` is called
- **Then:** The method returns `Err(...)` containing the API error details; the error is not logged-and-suppressed, not converted to a default value, and not silently retried; the caller receives the full error information

---

### IN-06: No auto rate-limiting without explicit RateLimiterConfig

- **Given:** A `Client` constructed via `Client::new(secret)` with no enterprise config; the `rate-limiting` Cargo feature may or may not be enabled at compile time
- **When:** Multiple `create_message()` calls are made in rapid succession
- **Then:** No rate limiting is applied; each call is dispatched immediately without any throttling or delay introduced by the client; rate limiting only activates when the caller explicitly provides a `RateLimiterConfig` via an enterprise config builder

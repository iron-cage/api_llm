# Pattern: Async Patterns

### Scope

- **Purpose**: Documents the Async Patterns pattern — standardized async method signatures for OpenAI API endpoint accessors.
- **Responsibility**: Specifies the preferred async implementation form for all endpoint accessor methods in api_openai.
- **In Scope**: Async function signatures, return types, streaming channel patterns, error propagation conventions.
- **Out of Scope**: Sync API wrappers, runtime selection, executor configuration, retry and circuit-breaker behaviors.

### Problem

Endpoint methods in api_openai use two conflicting async signatures: fully-typed request/response structs and untyped `serde_json::Value`. The untyped form defers shape errors to runtime, produces weaker compiler feedback, and cannot express API contract expectations. The inconsistency makes the boundary between a caller's intent and an endpoint's contract ambiguous.

### Solution

All async endpoint methods use native `async fn` with fully typed request and response structs. Each method maps one-to-one to a single API endpoint. Errors propagate directly to the caller as `Result` — no automatic retries, no implicit transformation. Streaming methods return a bounded receiver channel of typed events. `serde_json::Value` signatures are acceptable only for endpoints lacking a defined schema; they are migrated as schemas are added.

Three canonical forms:

- **Create/Update** — accepts a typed request struct, returns a typed response
- **Retrieve/List** — accepts a path identifier or optional typed query struct, returns a typed response or typed list response
- **Stream** — accepts a typed request struct, returns a bounded receiver of typed streaming events

### Applicability

Apply to every endpoint method in api_openai. New endpoints must use typed structs from the outset. Existing untyped methods are migrated in order of call frequency — high-frequency endpoints first, specialized features last.

### Consequences

Native `async fn` avoids the heap allocation overhead of `#[async_trait]` and produces clearer compiler error messages. Typed request and response structs surface shape mismatches at compile time. Bounded streaming channels prevent unbounded memory growth when a consumer is slower than the producer.

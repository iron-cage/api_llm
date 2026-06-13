# Pattern: Batch Processing

### Scope

- **Purpose**: Define the structural pattern for processing many independent prompts within per-minute API rate limits by pacing requests sequentially with an inter-request delay.
- **Responsibility**: Applied wherever throughput is bounded by API rate limits rather than compute capacity.
- **In Scope**: Bulk prompt evaluation, dataset annotation, report generation — any workload with many independent, order-independent prompts.
- **Out of Scope**: Workloads requiring all-or-nothing semantics; workloads with high quota allocation that can run fully concurrent.

### Problem

Processing many independent prompts concurrently exhausts per-minute rate limits and produces intermittent failures. Sequential processing with pacing is more reliable, though slower overall.

### Solution

Process prompts sequentially in a loop, collecting results into an output vector. Insert a short delay between requests to stay within rate limits. Individual failures produce a sentinel value and do not abort the batch. See `docs/operation/002_usage_examples.md` for the underlying per-request construction procedure.

### Applicability

Use when:
- Throughput is bounded by API rate limits rather than compute capacity
- Partial results are acceptable (some items may produce sentinel values on failure)
- Requests are independent with no ordering dependency between prompts

Avoid when:
- Full parallelism is viable (high quota allocation, no rate pressure)
- All-or-nothing batch semantics are required (partial failure is unacceptable)

### Consequences

- Predictable, steady rate limit consumption
- Overall throughput reduced by inter-request delay
- Silent individual failures — failed items produce sentinel strings rather than errors in the result vector

### Sources

| File | Relationship |
|------|-------------|
| `src/models/api/content_generation/api_impl.rs` | `generate_content()` called per loop iteration |

### Tests

| File | Relationship |
|------|-------------|
| `examples/` | Runnable examples demonstrating the Batch Processing pattern |

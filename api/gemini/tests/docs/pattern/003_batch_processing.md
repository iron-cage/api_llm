# Pattern Spec: Batch Processing

**Source:** [`docs/pattern/003_batch_processing.md`](../../docs/pattern/003_batch_processing.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| PT-05 | Batch Processing pattern paces requests with inter-request delay | batch processing | ✅ |
| PT-06 | Batch Processing pattern continues on individual item failure | batch processing | ✅ |

---

### PT-05: Batch Processing pattern paces requests with inter-request delay

- **Given:** A list of independent prompts to process using the Batch Processing pattern
- **When:** The batch loop executes
- **Then:** Prompts are processed sequentially (not concurrently); a short delay is inserted between requests to stay within per-minute rate limits; the pattern is applicable when throughput is bounded by API rate limits and partial results are acceptable

---

### PT-06: Batch Processing pattern continues on individual item failure

- **Given:** A batch of prompts where some items fail during processing
- **When:** A `generate_content()` call fails for a specific prompt
- **Then:** The failure produces a sentinel value in the result vector rather than aborting the entire batch; the batch loop continues to the next item; the caller receives a complete result vector with sentinel values for failed items; all-or-nothing semantics are not provided by this pattern

# Gemini Streaming API Format Discovery

- **Date**: 2025-10-12
- **Issue**: Streaming test failure - zero chunks received
- **Root Cause**: Incorrect assumption about Gemini API streaming format
- **Resolution**: Fixed by switching from SSE parser to JSON array buffering

---

### Problem Statement

The streaming integration test `integration_test_streaming_real_api` was consistently failing with the error:
```
No chunks received from streaming API - streaming must return at least one chunk
```

This occurred despite:
- Valid API key authentication
- Successful HTTP connection to `:streamGenerateContent` endpoint
- Correct request format
- 200 OK response from Gemini API

### Investigation Process

#### Initial Hypothesis

The implementation assumed Gemini's streaming endpoint returned Server-Sent Events (SSE) format, based on:
- Common industry practice for streaming APIs
- Endpoint name containing "stream"
- Similar APIs (OpenAI, Anthropic) using SSE

#### Debug Evidence

Adding debug logging to the streaming parser revealed:

```
DEBUG: Received chunk #1: is_final=Some(false), has_candidates=false, has_error=Some("Failed to parse NDJSON line: invalid type: map, expected a sequence at line 1 column 1. Raw line: [{")
DEBUG: Received chunk #2: is_final=Some(false), has_candidates=false, has_error=Some("Failed to parse NDJSON line: invalid type: string \"candidates\", expected struct GenerateContentResponse at line 1 column 12. Raw line: \"candidates\": [")
...
DEBUG: Total chunks received: 59, Total content parts: 0
```

**Key Insight**: All 59 line-by-line parse attempts failed. The SSE parser was attempting to parse individual lines of a formatted JSON array.

#### Actual Format Discovery

Manual inspection of raw HTTP response showed:

```json
[
  {
    "candidates": [{"content": {"parts": [{"text": "1\n2\n"}]}, "index": 0}],
    "usageMetadata": {"promptTokenCount": 14, "candidatesTokenCount": 3, "totalTokenCount": 17}
  },
  {
    "candidates": [{"content": {"parts": [{"text": "\n3\n4\n5"}]}, "finishReason": "STOP", "index": 0}],
    "usageMetadata": {"promptTokenCount": 14, "candidatesTokenCount": 8, "totalTokenCount": 22}
  }
]
```

**Critical Discovery**: Gemini returns a complete JSON array, NOT Server-Sent Events.

### Root Cause Analysis

The implementation at `src/models/api/content_generation/api_impl.rs` (refactored 2025-10-18) used:

```rust
use eventsource_stream::Eventsource;

response
  .bytes_stream()
  .eventsource()  // ← Wrong: Expects SSE format
  .map
  (
    |event_result|
    {
      match event_result
      {
        Ok( event ) =>
        {
          match event.data.as_str()
          {
            // Parse SSE data field...
          }
        }
      }
    }
  )
```

**Why It Failed:**
1. `eventsource-stream` expects format: `data: {...}\n\n`
2. Gemini sends: `[{...}, {...}]` with formatting newlines
3. SSE parser sees opening bracket `[{` as invalid SSE syntax
4. Every line is rejected → zero chunks parsed

### Solution

#### Implementation Changes

1. **Removed SSE Parser:**
   - Removed `eventsource-stream` dependency usage
   - Removed line-by-line parsing logic

2. **Added JSON Array Buffering:**
   ```rust
   use async_stream::stream;

   async_stream::stream!
   {
     let bytes_result = response.bytes().await;  // Buffer complete response

     match bytes_result
     {
       Ok( bytes ) =>
       {
         let text = String::from_utf8_lossy( &bytes );

         // Parse as JSON array
         match serde_json::from_str::< Vec< GenerateContentResponse > >( &text )
         {
           Ok( responses ) =>
           {
             // Emit each array element as stream chunk
             for api_response in responses.into_iter()
             {
               yield Ok( streaming_response );
             }
           }
         }
       }
     }
   }
   ```

3. **Header Correction:**
   ```rust
   .header("Accept", "application/json")  // was: "text/event-stream"
   ```

4. **Added Dependency:**
   ```toml
   async-stream = { workspace = true }
   ```

#### Test Results

After fix:
```
DEBUG: Received chunk #1: is_final=Some(false), has_candidates=true, has_error=None
DEBUG: Chunk has 1 candidates
DEBUG: Candidate has 1 parts
DEBUG: Found text part: "1"
DEBUG: Received chunk #2: is_final=Some(true), has_candidates=true, has_error=None
DEBUG: Chunk has 1 candidates
DEBUG: Candidate has 1 parts
DEBUG: Found text part: "\n2\n3\n4\n5"
DEBUG: Total chunks received: 3, Total content parts: 2
test integration_test_streaming_real_api ... ok
```

**Success**: All 287 tests passing, including streaming integration test.

### Lessons Learned

#### Don't Assume Standard Formats

**Incorrect Assumption**: "Streaming endpoints use SSE format"
**Reality**: Each API has its own format choices

**Action**: Always verify actual format through:
- Official API documentation
- Raw HTTP response inspection
- Debug logging of parse failures

#### Test Against Real APIs

**Why It Matters**: Mock tests would have hidden this issue indefinitely.

Our no-mocking policy caught this because:
1. Test made real HTTP call to Gemini
2. Real response revealed format mismatch
3. Debug output showed exact failure mode

#### Document Format Discoveries

**Critical Knowledge**: API format details are not always documented clearly.

**Preservation**: This discovery is now documented in:
1. Test file: `tests/inc/comprehensive_integration_test.rs` (60+ lines of doc comments)
2. Implementation: `src/models/api/content_generation/api_impl.rs` (50+ lines of doc comments)
3. Protocol spec: `docs/protocol/001_streaming_format.md`
4. This document: Historical investigation record

### Performance Considerations

#### Trade-offs of Array Buffering

**Downside:**
- Must wait for complete response before first chunk
- Higher memory usage (full response in memory)

**Upside:**
- Simple, robust implementation
- Matches actual API behavior
- Gemini responses are typically small (<1MB) and fast (<1 second)

**Decision**: Buffering is acceptable for Gemini's typical response characteristics.

#### Alternative Approaches Considered

1. **True Streaming Parser**: Parse JSON array incrementally
   - ✅ Lower latency to first chunk
   - ❌ Complex implementation
   - ❌ Requires custom JSON array streaming parser
   - ❌ Not worth complexity for <1 second responses

2. **Chunked Reading**: Read in chunks and attempt parse
   - ✅ Some memory savings
   - ❌ Must handle incomplete JSON
   - ❌ Still requires full buffer for parse
   - ❌ Adds complexity without benefit

**Conclusion**: Full buffering is the right choice for this use case.

### Files Modified

1. `Cargo.toml`: Added `async-stream` dependency
2. `src/models/api/content_generation/api_impl.rs`: Rewrote `process_streaming_response()` (59 lines changed)
3. `tests/inc/comprehensive_integration_test.rs`: Added comprehensive documentation (60+ lines)

### References

- **Test Documentation**: `tests/inc/comprehensive_integration_test.rs:168-227`
- **Implementation**: `src/models/api/content_generation/api_impl.rs`
- **Protocol Spec**: `../protocol/001_streaming_format.md`
- **Bug Fix Date**: 2025-10-12
- **Level 3 Tests**: All passing (287 tests, 0 failures)

### Keywords for Future Searches

- Gemini streaming format
- `:streamGenerateContent` endpoint
- JSON array vs SSE
- eventsource-stream incompatibility
- streaming API debugging
- zero chunks received
- buffered streaming implementation

---

**Status**: Resolved
**Impact**: High (broke all streaming functionality)
**Complexity**: Medium (format mismatch, not logic error)
**Fix Verification**: All 287 level::3 tests passing

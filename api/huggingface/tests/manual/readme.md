# Manual Testing Plan - api_huggingface

## Overview

This crate has **comprehensive automated test coverage** (221 tests passing). Manual testing is **optional** and primarily useful for:
- Verifying live API behavior with real HuggingFace accounts
- Testing interactive examples with human interaction
- Exploratory testing of edge cases

## Automated Test Coverage

**Status**: ✅ **Complete** (221/221 tests passing)

```bash
# Run all automated tests
RUSTFLAGS="-D warnings" cargo nextest run --all-features
```

All core functionality (Chat, Embeddings, Models, Streaming, Retry, Function Calling) is fully covered by automated tests.

## Manual Testing (Optional)

### When Manual Testing Is Useful

1. **Interactive Examples**: Testing chat interfaces with human interaction
2. **Live API Verification**: Confirming behavior against live HuggingFace API
3. **Rate Limiting**: Observing actual rate limit responses
4. **Context Retention**: Verifying multi-turn conversation memory

### Prerequisites

```bash
# Set API key (choose one method)
export HUGGINGFACE_API_KEY="hf_..."

# OR use workspace secret file
# Automatically loaded from: ../../secret/-secrets.sh
```

**Note**: Router API requires valid HuggingFace API key. Examples use:
- **Endpoint**: `https://router.huggingface.co/v1/chat/completions`
- **Default Model**: `moonshotai/Kimi-K2-Instruct-0905:groq`

## Test Case 1: Context Retention and Math Reasoning

**Objective**: Verify multi-turn conversation context is maintained.

### Manual Test Procedure

**Steps**:
1. Start interactive chat:
   ```bash
   cargo run --example interactive_chat --features="full"
   ```

2. Tell AI: `x=13`
3. Ask: `x*3?`
4. Expected response: `39` (or explanation containing 39)

**Success Criteria**:
- AI remembers that x=13 from previous message
- AI correctly calculates 13 * 3 = 39
- Response contains the number 39

**Failure Indicators**:
- AI responds with wrong number
- AI doesnt remember x value
- Error responses (check API key validity)

### Automated Test Script

```bash
# Run automated math reasoning test
./tests/manual/run_math_test.sh
```

This script:
1. Builds the interactive_chat example
2. Feeds input sequence: "x=13" → "x*3?" → "quit"
3. Verifies output contains "39"
4. Exits with status 0 on success, 1 on failure

## Test Case 2: Multi-Step Context

**Objective**: Verify context persists across multiple exchanges.

**Steps**:
1. Run interactive chat
2. Say: `I have 5 apples`
3. Say: `I buy 3 more`
4. Ask: `How many do I have now?`
5. Expected: `8` (or explanation containing 8)

## Test Case 3: Function Calling

**Objective**: Verify tool/function calling integration.

**Steps**:
1. Run function calling example (if exists) or use test:
   ```bash
   cargo test --test function_calling_tests test_function_calling_basic --features="full" -- --ignored --nocapture
   ```

2. Verify tool definitions are sent correctly
3. Verify tool_choice parameter works (auto, none, required, specific)

**Note**: This is covered by automated tests, manual verification only needed for debugging.

## Test Case 4: Error Handling

**Objective**: Verify graceful handling of API errors.

**Steps**:
1. Use invalid API key:
   ```bash
   export HUGGINGFACE_API_KEY="invalid"
   cargo run --example chat --features="full"
   ```

2. Expected: Clear error message about authentication failure

3. Use valid key with malformed request:
   ```bash
   # Trigger validation error (if applicable)
   ```

## Common Issues & Troubleshooting

### Issue: 404 Not Found

**Cause**: Model not available or requires Pro access
**Solution**: Use Router API with supported models (default: Kimi-K2-Instruct)

### Issue: 401 Unauthorized

**Cause**: Invalid or missing API key
**Solution**:
```bash
# Verify key is set
echo $HUGGINGFACE_API_KEY

# Or check workspace secret file
cat ../../secret/-secrets.sh
```

### Issue: 429 Rate Limited

**Cause**: API rate limits exceeded
**Solution**: Wait and retry, or use explicit retry config:
```rust
let retry_config = ExplicitRetryConfig::conservative();
client.post_with_explicit_retry( url, payload, &retry_config ).await?;
```

## Testing Commands Reference

```bash
# Interactive chat (multi-turn with context)
cargo run --example interactive_chat --features="full"

# Multi-turn conversation (predefined sequence)
cargo run --example multi_turn_conversation --features="full"

# Basic single-turn chat
cargo run --example chat --features="full"

# Automated math test
./tests/manual/run_math_test.sh

# Run single ignored test manually
cargo test --test function_calling_tests test_function_calling_basic --features="full" -- --ignored --nocapture
```

## Test Coverage Summary

| Category | Automated | Manual |
|----------|-----------|--------|
| Chat Completions | ✅ | Optional |
| Text Generation | ✅ | Optional |
| Embeddings | ✅ | Optional |
| Model Management | ✅ | Optional |
| Streaming | ✅ | Optional |
| Function Calling | ✅ | Optional |
| Error Handling | ✅ | Optional |
| Retry Logic | ✅ | Optional |
| Sync API | ✅ | Optional |
| CURL Diagnostics | ✅ | Optional |

**Conclusion**: Manual testing is **not required** for verification. All functionality is covered by automated tests. Manual tests are useful only for exploratory testing and debugging live API interactions.

## Known Limitations

1. **Model Capability**: Math reasoning depends on model used (Kimi-K2 handles it well)
2. **Context Window**: Default examples retain last 10 exchanges
3. **Router API Models**: Limited to models supported by Router API

## References

- **Test Plan Details**: `test_chat_examples.md` (legacy, see above for current plan)
- **Automated Tests**: `../function_calling_tests.rs`, `../providers_api_tests.rs`
- **Examples**: `../../examples/interactive_chat.rs`, `../../examples/chat.rs`

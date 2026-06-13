# Manual Test Plan: api_gemini

## Purpose

Describes manual testing procedures requiring human judgment or real Gemini API credentials that cannot be fully automated.

## Automated Test Coverage

The following are validated by `cargo nextest run --all-features` (with `GEMINI_API_KEY` set):

| Test File | Coverage Area |
|-----------|---------------|
| integration_tests.rs | Core API: list models, get model, content generation, embeddings |
| comprehensive_integration_tests.rs | Multi-feature integration: streaming, retry, circuit breaker, rate limiting, caching |
| audio_processing_tests.rs | Audio input processing, transcription patterns |
| code_execution_tests.rs | Python code generation and execution via Gemini |
| system_instructions_tests.rs | System prompt configuration and behavior consistency |
| websocket_streaming_tests.rs | WebSocket bidirectional streaming |
| streaming_control_tests.rs | Stream pause, resume, cancel, buffer management |
| dynamic_configuration_tests.rs | Hot-reload configuration changes |
| structured_logging_tests.rs | Structured log output (requires `logging` feature) |
| health_checks_tests.rs | Endpoint health status checks |
| failover_tests.rs | Multi-endpoint failover behavior |
| count_tokens_tests.rs | Token counting across content types |
| model_comparison_tests.rs | A/B model comparison patterns |
| example_validation_tests.rs | Example code API usage correctness |
| api_key_failure_tests.rs | Loud failure when API key is missing |

## Manual Test Scenarios

### 1. Interactive Streaming

**Trigger**: Before releasing a new version with streaming changes.

**Steps**:
1. Build the interactive example: `cargo run --example gemini_api_interactive --features streaming`
2. Enter a multi-turn conversation (3+ turns)
3. Verify text streams in real-time (character-by-character, not all at once)
4. Verify conversation context is preserved across turns

**Pass criteria**: Responses appear incrementally; no buffering delays.

### 2. Cached Content Interactive Session

**Trigger**: Before releasing caching feature changes.

**Steps**:
1. Run: `cargo run --example gemini_api_cached_interactive --features streaming`
2. Send 3+ messages to prime the cache
3. Verify subsequent responses reference cached context

**Pass criteria**: Cache hit logs appear; response times for cached calls are shorter.

### 3. API Key Security Audit

**Trigger**: Before any public release.

**Steps**:
1. Enable debug logging: `RUST_LOG=debug cargo nextest run --all-features 2>&1 | grep -i "key\|secret\|token\|auth"`
2. Review all log lines for credential exposure
3. Run `cargo run --example gemini_error_handling` and inspect all error output

**Pass criteria**: No API key values appear in any log output or error messages.

### 4. Rate Limit Recovery

**Trigger**: When modifying rate limiting or retry logic.

**Steps**:
1. Configure a tight rate limit in tests
2. Observe retry behavior in logs
3. Verify exponential backoff timing (not fixed delays)

**Pass criteria**: Retries use increasing delays; eventual success on valid requests.

## Setup

```bash
# Source credentials
source secret/-secrets.sh

# Run all automated tests
cargo nextest run --all-features

# Run a specific manual example
cargo run --example gemini_api_interactive --features streaming
```

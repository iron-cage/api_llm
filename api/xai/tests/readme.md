# Tests

## Purpose

Comprehensive test suite for the X.AI Grok API client, validating functionality, integration scenarios, reliability features, and API compliance.

## Organization Principles

- **Domain-Based Organization**: Tests organized by functionality (what) not methodology (how)
- **Flat Structure**: All test files at top level for simplicity
- **Clear Naming**: Test files named after the functionality they test
- **Real API Testing**: Integration tests use real X.AI API (requires API key)
- **Feature Gating**: Tests requiring specific features use `#[cfg(feature = "...")]`

## Responsibility Table

### Core Infrastructure

| File | Responsibility | Coverage |
|------|----------------|----------|
| `readme.md` | Document test suite organization | Structure, patterns, execution guidance |
| `inc/` | Test infrastructure and helpers | Shared utilities, test setup (see inc/mod.rs, inc/test_helpers.rs) |
| `manual/` | Manual testing procedures | Human-verified functionality |

### Integration Tests

| File | Responsibility | Feature Coverage |
|------|----------------|------------------|
| `integration_chat.rs` | Test chat completion integration | End-to-end chat workflows, multi-turn conversations |
| `integration_models.rs` | Test model listing integration | Model discovery, model details retrieval |
| `integration_streaming.rs` | Test SSE streaming integration | Server-sent events, stream handling |
| `integration_tool_calling.rs` | Test tool calling integration | Function definition, execution, responses |

### Reliability & Resilience Tests

| File | Responsibility | Feature Coverage |
|------|----------------|------------------|
| `circuit_breaker_tests.rs` | Test circuit breaker functionality | State transitions, failure thresholds, recovery |
| `failover_tests.rs` | Test endpoint failover | Automatic failover, endpoint rotation |

### Component Tests

| File | Responsibility | Feature Coverage |
|------|----------------|------------------|
| `components_tests.rs` | Test core components | Client initialization, configuration, builders |
| `enhanced_tools_tests.rs` | Test enhanced tool calling features | Parallel execution, tool helpers |
| `environment_tests.rs` | Test environment management | Secret loading, configuration management |
| `error_tests.rs` | Test error handling | Error types, error propagation, recovery |
| `secret_tests.rs` | Test secret management | Secret loading, validation, workspace integration |

## Test Execution

### Run All Tests

```bash
# Run all tests (requires XAI_API_KEY for integration tests)
cargo test --all-features
```

### Run Specific Test File

```bash
# Run specific test file
cargo test --test integration_chat --all-features
cargo test --test circuit_breaker_tests --all-features
```

### Run Without Integration Tests

```bash
# Skip integration tests (no API key needed)
cargo test --all-features --lib
```

### Run Integration Tests Only

```bash
# Requires XAI_API_KEY environment variable
export XAI_API_KEY="your-api-key-here"
cargo test --test integration_ --all-features
```

## Prerequisites for Integration Tests

Integration tests require:
1. **X.AI API Key**: Set `XAI_API_KEY` environment variable
2. **Network Connection**: Tests make real API calls to X.AI
3. **API Rate Limits**: Tests respect X.AI rate limits (may be slow)

**Setting API Key:**
```bash
export XAI_API_KEY="your-api-key-here"
```

## Test Organization Philosophy

### Why Domain-Based (Not Methodology-Based)?

✅ **GOOD - Domain-based:**
```
integration_chat.rs          # Tests chat functionality
circuit_breaker_tests.rs     # Tests circuit breaker reliability
```

❌ **BAD - Methodology-based:**
```
unit/                        # Don't organize by test type
integration/
e2e/
```

**Rationale:** Domain-based organization makes tests easier to find ("where are chat tests?") compared to methodology-based ("is this unit or integration?").

## Known Limitations

- **Integration tests require API key**: Cannot run without valid XAI_API_KEY
- **Rate limiting**: X.AI API has rate limits; tests may be slow
- **Network dependency**: Integration tests require internet connection
- **Model availability**: Tests assume grok-2-1212 model is available

## Related Documentation

- See `../docs/invariant/` for behavioral invariants and requirements
- See `../examples/` for usage examples
- See source code (`../src/`) for implementation details

# api_gemini Testing Guide

This document explains the testing philosophy, policies, and organization structure for the `api_gemini` crate.

## 🚫 NO MOCKUP TESTS POLICY

This crate follows a **strict no-mockup policy** for all testing:

### Core Principles

- **Real Integration Tests Only**: All API functionality is tested against the actual Gemini API
- **No Mock Servers**: Tests use real HTTP calls to Google's production endpoints
- **No Mock Objects**: No synthetic test doubles or stub implementations
- **No Test Ignoring**: Tests are never marked with `#[ignore]` or conditionally skipped
- **No Silent Skipping**: Tests NEVER silently skip when API keys are unavailable
- **Missing API Key = Test Failure**: When `GEMINI_API_KEY` is unavailable, tests MUST fail explicitly (not skip, not pass)
- **Explicit Failures Only**: All authentication errors and missing keys result in test failures, never graceful skips
- **Confidence in Reality**: Tests validate actual production behavior, not simulated responses

### Rationale

**Why we don't use mocks:**
1. **Hide Integration Failures**: Mocks can hide real-world integration issues
2. **Miss API Changes**: Real API changes aren't caught by mock tests
3. **False Confidence**: Passing mock tests don't guarantee production functionality
4. **Edge Case Blind Spots**: Real services have edge cases that mocks don't simulate
5. **Maintenance Overhead**: Mock data becomes stale and requires constant updates

**Why we don't silently skip tests:**
1. **Masks Configuration Issues**: Silent skips hide missing API keys in CI/CD
2. **False Green Builds**: Tests appear to pass when they didn't actually run
3. **Deployment Risks**: Code reaches production without proper integration validation
4. **Debug Difficulty**: Developers don't know if tests ran or were skipped

**Benefits of real API testing with explicit failures:**
1. **Production Confidence**: Tests prove the client works with the actual service
2. **Immediate Feedback**: API changes and configuration issues detected immediately
3. **Real Error Handling**: Tests encounter actual error conditions
4. **Performance Insights**: Tests reveal actual latency and timing characteristics
5. **Authentication Validation**: Tests verify actual API key and auth flows
6. **CI/CD Integrity**: Build failures force proper configuration before deployment

## 📁 Test Organization Structure

```
tests/
├── readme.md                          # This file - testing guide and policies
├── tests.rs                          # Main test module coordinator
├── api_key_failure_test.rs           # API key validation and error handling
├── integration_tests.rs              # Core real API integration tests
├── comprehensive_integration_tests.rs # Extended real API testing scenarios
├── count_tokens_tests.rs             # Real count tokens API functionality
├── sync_api_tests.rs                 # Synchronous API wrapper tests
├── example_validation_test.rs        # Documentation example validation
├── batch_operations_tests.rs         # Batch processing functionality
├── audio_processing_tests.rs         # Audio content processing tests
└── structured_logging_tests.rs       # Logging and diagnostics tests
```

## 🎯 Test Categories

### 1. Integration Tests (`integration_tests.rs`)
**Purpose**: Core API functionality validation
**Type**: Real API calls
**Requirements**: Valid `GEMINI_API_KEY`

- List models and get model details
- Single-turn content generation
- Multi-turn conversations
- Multimodal content (text + images)
- Function calling and tool use
- Text embeddings
- Safety settings
- Error handling for invalid inputs

### 2. Comprehensive Integration Tests (`comprehensive_integration_tests.rs`)
**Purpose**: Extended real-world scenarios
**Type**: Real API calls
**Requirements**: Valid `GEMINI_API_KEY`

- Advanced feature combinations
- Edge cases and error conditions
- Performance and timeout scenarios
- Large request handling
- Concurrent operation testing

### 3. Count Tokens Tests (`count_tokens_tests.rs`)
**Purpose**: Token counting functionality
**Type**: Real API calls
**Requirements**: Valid `GEMINI_API_KEY`

- Simple text token counting
- Multimodal content token counting
- Conversation context token counting
- Different model token counting
- Error handling for invalid requests

### 4. Synchronous API Tests (`sync_api_tests.rs`)
**Purpose**: Blocking wrapper validation
**Type**: Real API calls (via sync wrapper)
**Requirements**: Valid `GEMINI_API_KEY`

- Sync client construction
- Thread safety validation
- Runtime management
- Performance overhead measurement

### 5. Unit Tests (embedded in implementation files)
**Purpose**: Internal logic validation
**Type**: Pure unit tests (no API calls)
**Requirements**: None

- Builder pattern validation
- Error type construction
- Data structure serialization
- Configuration parsing

### 6. Example Validation Tests (`example_validation_test.rs`)
**Purpose**: Documentation example verification
**Type**: Real API calls
**Requirements**: Valid `GEMINI_API_KEY`

- Ensure README examples work
- Validate API patterns in docs
- Verify example code compiles and runs

## 🔧 Running Tests

### Run All Tests (Default - Requires API Key)
```bash
cargo test
```

### Run Only Unit Tests (No API Key Required)
```bash
cargo test --no-default-features
```

### Run Specific Test Categories
```bash
# Core integration tests
cargo test --test integration_tests

# Count tokens functionality
cargo test --test count_tokens_tests

# Synchronous API tests
cargo test --test sync_api_tests

# Example validation
cargo test --test example_validation_test
```

### Debug Individual Tests
```bash
# Run with output capture disabled
cargo test test_generate_content_simple -- --nocapture

# Run specific test with features
cargo test --features logging test_structured_logging
```

## 🔑 API Key Requirements

### How Tests Load API Keys

Tests use `Client::new()` which relies on **workspace_tools** to load the API key in this order:

1. **Workspace Secrets File** (Primary): `secret/-secrets.sh`
   - Uses workspace_tools 0.6.0 to locate the workspace root
   - **Important**: workspace_tools 0.6.0 uses `secret/` (visible directory, NO dot prefix)
   - Follows the [Secret Directory Policy](../../../secret/readme.md)

2. **Environment Variable** (Fallback): `GEMINI_API_KEY`
   - Standard environment variable
   - Used if workspace secrets unavailable

### Setup Options

**Option 1: Workspace Secrets File (Recommended)**
```bash
# workspace_tools 0.6.0 uses secret/ directory (visible, NO dot prefix)
echo 'export GEMINI_API_KEY="your-key-here"' >> secret/-secrets.sh
chmod 600 secret/-secrets.sh
```

**Option 2: Environment Variable**
```bash
export GEMINI_API_KEY="your-key-here"
```

### Test Behavior and Error Messages

- **With Valid API Key**: All tests run and validate against real API

- **Without API Key**: Integration tests FAIL EXPLICITLY with detailed error messages showing:
  - Workspace secrets path tried (`secret/-secrets.sh`)
  - Specific error from workspace_tools (e.g., "key not found or file unreadable")
  - Environment variable status (e.g., "not set or empty")
  - Clear setup instructions with exact commands

- **Invalid API Key**: Tests FAIL with authentication errors (this is correct and expected behavior)

- **Silent Skips Prohibited**: Tests never silently skip - all missing keys result in explicit test failures with actionable error messages listing all paths tried

## ⚠️ Important Testing Insights

### API Response Timing (Real-World Data)

Based on actual API testing, different request types have significantly different response times:

- **Simple text generation**: ~0.5 seconds (fast)
- **Safety settings requests**: ~15-17 seconds (slow due to content analysis)
- **Function calling**: ~2-4 seconds (moderate)
- **Multimodal requests**: ~3-8 seconds (varies by image complexity)

### Test Timeout Strategy

Tests use appropriate timeouts based on actual API behavior:

```rust
// Safety settings require longer timeouts
let result = tokio::time::timeout
(
  Duration::from_secs( 25 ), // Accommodate safety processing
  client.models().by_name( "gemini-1.5-pro-latest" )
    .generate_content( &safety_request )
).await;
```

### Common Pitfalls to Avoid

❌ **Don't do this:**
- Silent test skipping on failures or missing keys
- Graceful skips when API keys are unavailable
- Generic short timeouts for all request types
- Environment variable race conditions in parallel tests
- Assuming all API calls have same performance characteristics
- Using `.expect()` to hide authentication errors

✅ **Do this instead:**
- Explicit test failures with actionable error messages when keys are missing
- Let authentication errors propagate as test failures (NOT skips)
- Request-type-specific timeouts
- Proper test isolation
- Clear panic messages that indicate missing API key configuration

## 🏗️ Test Development Guidelines

### When Adding New Tests

1. **Follow the no-mock policy**: Use real API calls for all functionality tests
2. **Never skip on missing keys**: Tests MUST fail explicitly when API keys are missing (use `.expect()` with clear messages)
3. **No graceful skip patterns**: Don't use `match` patterns that return `Ok(())` on authentication failures
4. **Use appropriate timeouts**: Different request types need different timeout values
5. **Test error conditions**: Validate error handling with real API error responses
6. **Document test purpose**: Include clear comments about what each test validates
7. **Explicit failure messages**: Use `.expect("GEMINI_API_KEY not found...")` for clear error reporting

### Test Naming Conventions

- `integration_test_*`: Real API integration tests
- `test_*_real_api`: Explicit real API testing
- `test_*_error_handling`: Error condition validation
- `test_*_authentication_*`: Auth-related testing

### Error Handling Patterns

**✅ CORRECT: Explicit failure on missing API key**
```rust
// Helper function that fails explicitly
fn create_test_client() -> Client
{
  Client::new().expect( "GEMINI_API_KEY not found - integration tests require valid API key" )
}

#[ tokio::test ]
async fn test_generate_content() -> Result< (), Box< dyn std::error::Error > >
{
  let client = create_test_client(); // Panics with clear message if key missing

  let request = GenerateContentRequest { /* ... */ };
  let response = client.models().by_name( "gemini-1.5-pro" ).generate_content( &request ).await?;

  assert!( !response.candidates.is_empty() );
  Ok( () )
}
```

**❌ WRONG: Graceful skip pattern (DO NOT USE)**
```rust
// This is PROHIBITED - never do this
let client = match create_test_client()
{
  Ok( client ) => client,
  Err( _ ) =>
  {
    println!( "⏸️  Skipping test - no API key available" );
    return Ok( () ); // WRONG - this hides missing configuration
  }
};
```

## 📊 Test Metrics

### Current Test Coverage

- **Total Tests**: 67 test functions across 10 test files
- **Integration Tests**: 54 real API endpoint tests
- **Unit Tests**: 13 component isolation tests
- **Success Rate**: 100% (when API key is available)

### Test Categories Breakdown

| Category | Count | Type | API Required |
|----------|-------|------|--------------|
| Core Integration | 13 | Real API | Yes |
| Comprehensive Integration | 10 | Real API | Yes |
| Count Tokens | 8 | Real API | Yes |
| Sync API | 7 | Real API | Yes |
| Batch Operations | 10 | Real API | Yes |
| Example Validation | 14 | Real API | Yes |
| Audio Processing | 5 | Real API | Yes |
| Unit Tests | 13 | Pure Unit | No |

## 🔄 Continuous Integration

### CI/CD Strategy

**IMPORTANT**: Tests themselves NEVER gracefully skip. The CI pipeline must decide whether to run integration tests or not.

```bash
# CI pipeline example - decision happens at CI level, NOT in tests
if [ -z "$GEMINI_API_KEY" ]; then
    # Explicitly skip integration tests at CI level
    cargo test --no-default-features
    echo "⚠️ Running unit tests only - no GEMINI_API_KEY configured"
    echo "⚠️ Integration tests were NOT run - configure key for full validation"
else
    # Run full test suite including integration tests
    cargo test
    echo "✅ All tests passed including integration tests"
fi
```

**Key Points**:
- Tests themselves use `.expect()` and fail when keys are missing
- The CI pipeline decides whether to run integration tests or not via `--no-default-features`
- This ensures developers are always aware when integration tests didn't run
- No false confidence from silently skipped tests

### External Service Dependencies

Integration tests require real API access:
- All integration tests make real API calls to Google Gemini API
- Timeouts are set appropriately for each request type
- Authentication failures result in test failures (not skips)
- Clear error messages indicate missing API key configuration

## 📚 References

- [Main README Testing Section](../readme.md#testing)
- [Specification Testing Strategy](../spec.md#testing-strategy-implemented)
- [Google Gemini API Documentation](https://ai.google.dev/api/rest)

---

**Last Updated**: 2025-01-28
**Maintainer**: Development Team
**Policy Version**: 1.0
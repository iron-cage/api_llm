# Testing Framework and Policies

## ⚠️ CRITICAL POLICY: NO MOCKING ALLOWED ⚠️

### Zero Tolerance Mock Policy

This test suite enforces a **strict NO MOCKING ALLOWED policy** for all integration tests. This is a non-negotiable architectural decision that ensures:

- ✅ **Authentic API Testing**: Tests validate actual Anthropic API behavior
- ✅ **Real Network Conditions**: Tests expose genuine connectivity issues
- ✅ **Actual Error Handling**: Tests verify real API error responses
- ✅ **Production Fidelity**: Test environment mirrors production conditions

### 🚫 Prohibited Practices

The following practices are **absolutely forbidden** in integration tests:

- **Fake API Keys**: Any `sk-ant-test-*` patterns or similar fake credentials
- **Mock Servers**: HTTP mocking libraries, test servers, or request interceptors
- **Hardcoded Responses**: JSON responses mimicking API format for testing
- **Simulated Errors**: Artificial error injection that bypasses real API errors
- **Silent Failures**: Tests that pass when APIs are unavailable
- **Graceful Fallbacks**: Tests that skip when credentials/network unavailable

### ✅ Required Practices

All integration tests **must**:

- Use `Client::from_workspace()` for real credential loading
- Include `#[cfg(feature = "integration")]` feature gating
- Document strict failure policy in file headers
- Fail immediately and loudly when issues occur
- Make actual HTTP requests to Anthropic API endpoints
- Validate real API response structures

## 🏗️ Test Organization Structure

### Directory Layout

```
tests/
├── readme.md                                  # This file - policies and organisation
├── tests.rs                                   # Main test entry point with module includes
├── docs/                                      # Test surface specs (one entity per docs/ collection)
│   ├── api/                                   # AP- scenarios (12): endpoint coverage
│   ├── feature/                               # FT- scenarios (12): enterprise reliability
│   ├── invariant/                             # IN- scenarios (12): thin-client + testing standards
│   ├── operation/                             # OP- scenarios (15): secret loading
│   └── pattern/                               # PT- scenarios (6): module organisation
├── manual/                                    # Manual testing plans and procedures
├── -default_topic/                            # Temporary working directory (gitignored)
└── inc/                                       # 52 test modules — 576 tests (469 unit, 107 integration)
    ├── mod.rs                                 # Module aggregator (re-exports all test modules)
    ├── authentication_test.rs                 # Authentication and credential tests
    ├── batch_messages_test.rs                 # Batch Messages API tests
    ├── circuit_breaker_test.rs                # Circuit breaker pattern tests
    ├── comprehensive_integration_test.rs      # Full end-to-end integration tests
    ├── compression_test.rs                    # Compression feature tests (FT-12)
    ├── content_generation_refactor_test.rs    # Content generation refactored API
    ├── content_generation_test.rs             # Content generation core tests
    ├── core_client_test.rs                    # Core client lifecycle tests
    ├── curl_diagnostics_test.rs               # Curl diagnostic output tests
    ├── dynamic_config_test.rs                 # Dynamic configuration tests
    ├── embeddings_test.rs                     # Embeddings API tests
    ├── endpoint_coverage_test.rs              # AP- spec: API endpoint coverage
    ├── enhanced_function_calling_test.rs      # Enhanced function calling tests
    ├── enhanced_model_details_test.rs         # Enhanced model detail tests
    ├── enhanced_retry_logic_test.rs           # Enhanced retry strategy tests
    ├── enterprise_configuration_test.rs       # Enterprise configuration tests
    ├── enterprise_quota_test.rs               # Enterprise quota management tests
    ├── enterprise_reliability_test.rs         # FT- spec: enterprise reliability
    ├── error_handling_integration_test.rs     # Real-API error handling tests
    ├── error_handling_test.rs                 # Error handling and recovery tests
    ├── example_model_validation_test.rs       # Example model name validation
    ├── examples_validation_test.rs            # Examples compilation validation
    ├── failover_test.rs                       # Failover mechanism tests
    ├── fallback_behavior_integration_test.rs  # Fallback behaviour integration tests
    ├── general_diagnostics_test.rs            # General diagnostics tests
    ├── health_checks_test.rs                  # Health check mechanism tests
    ├── input_validation_test.rs               # Input validation tests
    ├── messages_api_test.rs                   # Messages API integration tests
    ├── model_management_test.rs               # Model management tests
    ├── module_organization_test.rs            # PT- spec: module organisation
    ├── operation_test_specs.rs                # OP- spec: secret loading operations
    ├── performance_monitoring_test.rs         # Performance monitoring tests
    ├── performance_test.rs                    # Performance and timing tests
    ├── prompt_caching_tests.rs                # Prompt caching tests
    ├── rate_limiting_test.rs                  # Rate limiting behaviour tests
    ├── request_caching_test.rs                # Response caching tests
    ├── retry_logic_test.rs                    # Retry mechanism tests
    ├── simple_integration_test.rs             # Minimal real-API smoke tests
    ├── spec_verification_integration_test.rs  # Spec alignment verification
    ├── streaming_control_test.rs              # Streaming control tests
    ├── streaming_test.rs                      # Streaming API tests
    ├── structured_logging_test.rs             # Structured logging tests
    ├── sync_api_test.rs                       # Synchronous API wrapper tests
    ├── sync_cached_content_test.rs            # Sync cached content tests
    ├── sync_streaming_test.rs                 # Sync streaming tests
    ├── system_instructions_test.rs            # System instructions tests
    ├── testing_standards_test.rs              # IN- spec: testing standards (IN-07..12)
    ├── thin_client_principle_test.rs          # IN- spec: thin client principle (IN-01..06)
    ├── token_counting_test.rs                 # Token counting tests
    ├── token_validation_test.rs               # Token validation tests
    ├── tool_calling_test.rs                   # Tool calling functionality tests
    └── vision_support_test.rs                 # Vision and image analysis tests
```

### Test Categories

#### 1. Unit Tests (Limited Scope)
- **Location**: `tests/inc/` directory
- **Purpose**: Isolated component testing without external API calls
- **Mocking**: **ABSOLUTELY PROHIBITED** — zero tolerance per No-Mock Mandate
- **Scope**: Individual functions, data structures, validation logic
- **API Requirements**: No API keys needed

#### 2. Integration Tests (NO MOCKING ALLOWED)
- **Location**: `tests/inc/*_test.rs`
- **Purpose**: End-to-end API integration validation
- **Mocking**: **ABSOLUTELY PROHIBITED**
- **Scope**: Real API calls, full request/response cycles
- **API Requirements**: Valid `ANTHROPIC_API_KEY` **mandatory**

#### 3. Feature-Specific Tests
- **Tool Calling**: `#[cfg(feature = "tools")]`
- **Vision Support**: `#[cfg(feature = "vision")]`
- **Streaming**: `#[cfg(feature = "streaming")]`
- **Authentication**: `#[cfg(feature = "authentication")]`

## 🔐 Credential Management

### 🚨 STRICT FAILURE POLICY: Tests MUST Fail When Credentials Unavailable

**Integration tests NEVER skip silently - they FAIL EXPLICITLY with detailed error messages.**

This crate enforces 4 critical requirements:

1. ✅ **MUST FAIL if token unavailable** - Tests fail explicitly, never skip
2. ✅ **MUST use workspace_tools for loading** - All secret loading via workspace_tools
3. ✅ **MUST list all tried paths in errors** - Error messages show every path attempted
4. ✅ **MUST document in tests/readme.md** - This file documents all requirements

### How Tests Load API Keys (workspace_tools)

Tests use `Client::from_workspace()` which relies on **workspace_tools** to load the API key in this order:

1. **Workspace Secrets File** (Primary): `<workspace_root>/secret/-secrets.sh`
   - Uses workspace_tools 0.6.0 to auto-discover workspace root (searches for `Cargo.toml`)
   - Looks for `secret/-secrets.sh` at workspace root (NO dot prefix)
   - Follows the [Secret Directory Policy](../../../secret/readme.md)

2. **Environment Variable** (Fallback): `ANTHROPIC_API_KEY`
   - Standard environment variable
   - Used if workspace secrets unavailable

### Required Environment Setup

Integration tests require valid Anthropic API credentials through one of:

1. **Workspace Secrets** (Primary - Recommended): `secret/-secrets.sh`
   ```bash
   # At workspace root: /home/user/pro/lib/api_llm/secret/-secrets.sh
   export ANTHROPIC_API_KEY="sk-ant-api03-your-actual-key-here"
   ```

2. **Environment Variable** (Fallback):
   ```bash
   export ANTHROPIC_API_KEY="sk-ant-api03-your-actual-key-here"
   ```

3. **Runtime Loading** (Direct):
   ```rust
   let client = Client::from_workspace()
       .expect( "INTEGRATION : Must have valid API key" );
   ```

### Credential Validation

All integration tests must:
- Load credentials using approved methods
- Validate API key format (`sk-ant-` prefix)
- Fail immediately if credentials unavailable
- Never proceed with invalid or missing credentials

### Test Behavior and Error Messages

**With Valid API Key**: All tests run and validate against real Anthropic API

**Without API Key**: Integration tests FAIL EXPLICITLY with detailed error messages showing:
- **Workspace secrets path tried**: Actual path attempted (e.g., `/home/user/pro/lib/api_llm/secret/-secrets.sh`)
- **Specific error from workspace_tools**: Exact error (e.g., "key not found or file unreadable")
- **Environment variable status**: Whether `ANTHROPIC_API_KEY` was set (e.g., "not set or empty")
- **Clear setup instructions**: Exact commands to fix the issue
- **All attempted paths**: Complete list of every location checked

**Invalid API Key**: Tests FAIL with authentication errors (correct and expected behavior)

**Silent Skips Prohibited**: Tests NEVER silently skip - all missing keys result in explicit test failures with actionable error messages listing all paths tried

### Example Error Message

```
❌ INTEGRATION TEST FAILURE: No valid ANTHROPIC_API_KEY found!

🔍 Attempted to load API key from:
  1. Workspace secrets: /home/user/pro/lib/api_llm/secret/-secrets.sh
     ❌ Error: File not found or key not present in file
  2. Environment variable: ANTHROPIC_API_KEY
     ❌ Error: Not set or empty

💡 To fix:
  Option 1: Create workspace secrets file
    echo 'export ANTHROPIC_API_KEY="sk-ant-api03-YOUR-KEY"' > secret/-secrets.sh
    chmod 600 secret/-secrets.sh

  Option 2: Set environment variable
    export ANTHROPIC_API_KEY="sk-ant-api03-YOUR-KEY"

🚫 Integration tests CANNOT be silently skipped - this failure is intentional
📚 See: tests/readme.md for complete credential management documentation
```

## 🧪 Test Execution

### Running Tests

```bash
# Unit tests only (no API key required — excludes integration feature)
cargo nextest run --no-default-features --features enabled,streaming,authentication,content-generation,model-management,error-handling,tools,vision,embeddings,curl-diagnostics,general-diagnostics,sync-api,retry-logic,circuit-breaker,rate-limiting,failover,health-checks,batch-processing,count-tokens,request-caching,streaming-control,compression,enterprise-quota,dynamic-config,model-comparison,request-templates,buffered-streaming,input-validation,enhanced-function-calling

# Integration tests (requires valid ANTHROPIC_API_KEY)
cargo nextest run --all-features

# Complete test suite (canonical verification)
w3 .test l::3
```

### Test Failure Expectations

Integration tests **should fail** when:
- ✅ `ANTHROPIC_API_KEY` not available → **Expected failure**
- ✅ Network connectivity issues → **Expected failure**
- ✅ API authentication problems → **Expected failure**
- ✅ API endpoint errors → **Expected failure**
- ✅ Invalid request parameters → **Expected failure**

Integration tests **should pass** when:
- ✅ Valid API credentials available
- ✅ Network connectivity stable
- ✅ API endpoints responding normally
- ✅ Request parameters valid

## 📋 Test Writing Guidelines

### Integration Test Template

```rust
//! [Feature Name] Integration Tests - STRICT FAILURE POLICY
//!
//! MANDATORY INTEGRATION TEST REQUIREMENTS:
//! - These tests use REAL Anthropic API endpoints - NO MOCKING ALLOWED
//! - Tests MUST FAIL IMMEDIATELY if API secrets are not available
//! - Tests MUST FAIL IMMEDIATELY on network connectivity issues
//! - Tests MUST FAIL IMMEDIATELY on API authentication failures
//! - Tests MUST FAIL IMMEDIATELY on any API endpoint errors
//! - NO SILENT PASSES allowed when problems occur
//!
//! Run with: cargo test --features integration
//! Requires: Valid `ANTHROPIC_API_KEY` in environment or ../../secret/-secrets.sh

#[ allow( unused_imports ) ]
use super::*;

#[ tokio::test ]
#[ cfg(feature = "integration") ]
async fn integration_real_api_test( )
{
    let client = the_module::Client::from_workspace()
        .expect( "INTEGRATION : Must have valid API key for testing" );

    // Real API call with actual request
    let request = the_module::CreateMessageRequest {
        model: "claude-3-5-haiku-20241022".to_string( ),
        max_tokens : 10,
        messages : vec![the_module::Message::user("Test".to_string( ))],
        // ... other fields
    };

    let response = client.create_message(request).await
        .expect( "INTEGRATION : Real API call must succeed" );

    // Validate real API response structure
    assert!( !response.id.is_empty(  ), "Real API must return message ID");
    assert_eq!( response.type, "message" );
    assert!( !response.content.is_empty(  ), "Real API must return content");

    println!("✅ Integration test passed with real API response");
}
```

### Unit Test Guidelines

```rust
#[test]
fn test_request_validation( )
{
    // Unit test for isolated validation logic
    let request = CreateMessageRequest {
        model : String::new(), // Invalid
        max_tokens : 0,        // Invalid
        messages : vec![],     // Invalid
        // ...
    };

    // Test validation without API calls
    assert!( request.validate(  ).is_err( ));
}
```

## 🔍 Quality Assurance

### Code Review Checklist

- [ ] No mock usage in integration tests
- [ ] Real API credentials used (`Client::from_workspace()`)
- [ ] Proper failure policy documentation
- [ ] Feature gating applied correctly
- [ ] Error handling validates real API responses
- [ ] Tests fail appropriately when credentials unavailable

### Automated Scanning

CI/CD pipeline scans for prohibited patterns:
- `sk-ant-test-*` fake API keys
- Mock server implementations
- Hardcoded JSON responses
- Graceful test skipping

### Policy Violations

Any mock usage discovery triggers:
1. **Immediate remediation required**
2. **Test conversion to real API calls**
3. **Code review for compliance**
4. **Documentation updates**

## 🚀 Best Practices

### Do ✅
- Use real Anthropic API endpoints exclusively
- Load credentials through approved workspace methods
- Fail fast and loud when issues occur
- Document strict failure policies
- Validate authentic API response structures
- Feature-gate tests appropriately

### Don't ❌
- Use fake or test API keys
- Implement mock servers or interceptors
- Create hardcoded JSON responses
- Allow graceful fallbacks in integration tests
- Skip tests when credentials unavailable
- Simulate error conditions artificially

## 📞 Support

For questions about testing policies or implementation:
- Review documentation: `docs/`
- Check examples: `examples/`
- Verify credential setup: `../../secret/-secrets.sh`

---

**Remember: The NO MOCKING ALLOWED policy ensures our tests validate real-world behavior and catch actual integration issues before production deployment.**
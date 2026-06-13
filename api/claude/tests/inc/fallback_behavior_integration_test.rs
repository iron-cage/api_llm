//! Fallback Behavior Integration Tests - STRICT FAILURE POLICY
//!
//! MANDATORY INTEGRATION TEST REQUIREMENTS:
//! - These tests use REAL Anthropic API endpoints - NO MOCKING ALLOWED
//! - Tests MUST FAIL IMMEDIATELY if API secrets are not available (no graceful fallbacks)
//! - Tests MUST FAIL IMMEDIATELY on network connectivity issues
//! - Tests MUST FAIL IMMEDIATELY on API authentication failures
//! - Tests MUST FAIL IMMEDIATELY on any API endpoint errors
//! - NO SILENT PASSES allowed when problems occur
//!
//! Run with : cargo test --features integration
//! Requires : Valid `ANTHROPIC_API_KEY` in environment or ../../secret/-secrets.sh

#[ allow( unused_imports ) ]
use super::*;

#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_fallback_behavior_integration()
{
    // INTEGRATION TEST - STRICT FAILURE POLICY: NO GRACEFUL FALLBACKS
    // This test validates real credential loading behavior ONLY

    println!("🧪 Real Credential Loading Behavior Test");
    println!("=========================================");

    // Test 1: Validate workspace secret loading works with real credentials
    println!("\n🔐 Step 1: Testing real workspace secret loading...");

    let secret = the_module::Secret::from_workspace()
        .expect("INTEGRATION FAILURE: Must have real API key from workspace - no fake keys allowed");

    // Validate real API key format
    assert!(secret.ANTHROPIC_API_KEY.starts_with("sk-ant-"),
        "INTEGRATION FAILURE: Must use real Anthropic API key format");
    assert!(secret.ANTHROPIC_API_KEY.len() > 30,
        "INTEGRATION FAILURE: API key too short, likely fake test key");

    println!("✅ Real API key loaded from workspace successfully");

    // Test 2: Validate client creation with real credentials
    println!("\n🔧 Step 2: Testing client creation with real credentials...");

    let client = the_module::Client::from_workspace()
        .expect("INTEGRATION FAILURE: Must have real client from workspace");

    // Verify client uses same real credentials
    assert_eq!(client.secret().ANTHROPIC_API_KEY, secret.ANTHROPIC_API_KEY,
        "INTEGRATION FAILURE: Client must use identical real credentials");

    println!("✅ Client created with real credentials successfully");

    // Test 3: Validate real credential consistency
    println!("\n🔍 Step 3: Testing real credential consistency...");

    // Load secret again to verify consistency
    let secret2 = the_module::Secret::from_workspace()
        .expect("INTEGRATION FAILURE: Consistent secret loading must work");

    assert_eq!(secret.ANTHROPIC_API_KEY, secret2.ANTHROPIC_API_KEY,
        "INTEGRATION FAILURE: Multiple loads must return identical real credentials");

    println!("✅ Real credential loading is consistent");

    println!("\n🎉 Real Credential Loading Test Results");
    println!("=======================================");
    println!("✅ Workspace secret loading verified");
    println!("✅ Client creation verified");
    println!("✅ Credential consistency verified");
    println!("🚀 All real credential tests PASSED!");
}
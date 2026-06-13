//! All test.

pub use api_gemini as the_module;

// Include integration tests that run by default with real API calls
mod integration_tests;

// Include comprehensive integration tests that use ONLY real API calls
mod comprehensive_integration_tests;

// Include count tokens tests (implementation complete)
mod count_tokens_tests;

// Include enhanced retry logic tests
mod enhanced_retry_logic_tests;

// Include enhanced circuit breaker tests
mod enhanced_circuit_breaker_tests;

// Include enhanced rate limiting tests
mod enhanced_rate_limiting_tests;


//! Anthropic API client implementation
//!
//! ## 🏛️ Governing Principle : "Thin Client, Rich API"
//!
//! The `Client` implementation strictly adheres to the "Thin Client, Rich API" principle:
//!
//! - **🎯 Direct API Mapping**: One-to-one correspondence with Anthropic's API endpoints
//! - **⚙️ Explicit Configuration**: All enterprise features require explicit configuration
//! - **🔍 Transparent Operations**: All operations expose their internal behavior
//! - **🚀 Transport Reliability**: Focus on robust HTTP transport, not business logic
//!
//! ## Enterprise Features (Explicit Configuration Required)
//!
//! - **Rate Limiting**: `with_rate_limiter()` - Configurable token bucket
//! - **Circuit Breaker**: `with_circuit_breaker()` - Fault tolerance pattern
//! - **Retry Logic**: `with_retry_config()` - Exponential backoff configuration
//!
//! All enterprise features are:
//! - ✅ Feature-gated and disabled by default
//! - ✅ Explicitly configured through builder methods
//! - ✅ Transparent in their operation
//! - ✅ Zero overhead when disabled

mod private {}

crate::mod_interface!
{
  layer system_instructions;
  layer types;
  layer implementation;
  layer explicit_retry;
}

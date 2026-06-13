#![ doc( html_root_url = "https://docs.rs/api_claude/latest/api_claude/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

// Strategic clippy configuration for comprehensive API client
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::must_use_candidate)]

//! Anthropic API client for Rust
//!
//! This crate provides a comprehensive HTTP client for interacting with Anthropic's Claude API.
//! It handles authentication, request/response serialization, and streaming support.
//!
//! # Governing Principle : "Thin Client, Rich API"
//!
//! This library follows the principle of **"Thin Client, Rich API"** - exposing all
//! server-side functionality transparently while maintaining zero client-side intelligence
//! or **automatic** behaviors.
//!
//! **Key Distinction**: The principle prohibits **automatic/implicit** behaviors but explicitly
//! **allows and encourages** **explicit/configurable** enterprise reliability features.
//!
//! ## Core Principles
//!
//! - **API Transparency**: One-to-one mapping with Anthropic Claude API endpoints
//! - **Zero Automatic Behavior**: No implicit decision-making or magic thresholds
//! - **Explicit Control**: Developer decides when, how, and why operations occur
//! - **Information vs Action**: Clear separation between data retrieval and state changes
//! - **Configurable Reliability**: Enterprise features available through explicit configuration
//!
//! ## Enterprise Reliability Features
//!
//! The following enterprise reliability features are **explicitly allowed** when implemented
//! with explicit configuration and transparent operation:
//!
//! - **Configurable Retry Logic**: Exponential backoff with explicit configuration
//! - **Circuit Breaker Pattern**: Failure threshold management with transparent state
//! - **Rate Limiting**: Request throttling with explicit rate configuration
//! - **Failover Support**: Multi-endpoint configuration and automatic switching
//! - **Health Checks**: Periodic endpoint health verification and monitoring
//!
//! ## State Management Policy
//!
//! **✅ ALLOWED: Runtime-Stateful, Process-Stateless**
//! - Connection pools, circuit breaker state, rate limiting buckets
//! - Retry logic state, failover state, health check state
//! - Runtime state that dies with the process
//! - No persistent storage or cross-process state
//!
//! **❌ PROHIBITED: Process-Persistent State**
//! - File storage, databases, configuration accumulation
//! - State that survives process restarts
//!
//! **Implementation Requirements**:
//! - Feature gating behind cargo features (`retry`, `circuit_breaker`, `rate_limiting`, `failover`, `health_checks`)
//! - Explicit configuration required (no automatic enabling)
//! - Transparent method naming (e.g., `execute_with_retries()`, `execute_with_circuit_breaker()`)
//! - Zero overhead when features disabled
//!
//! # Examples
//!
//! ```no_run
//! use api_claude::{ Client, Secret, CreateMessageRequest, Message, Role, Content };
//!
//! # async fn example() -> Result< (), Box< dyn std::error::Error > > {
//! // Create a client
//! let secret = Secret::new( "sk-ant-api03-your-key-here".to_string() )?;
//! let client = Client::new( secret );
//!
//! // Create a simple message
//! let request = CreateMessageRequest::builder()
//!   .model( "claude-sonnet-4-6".to_string() )
//!   .max_tokens( 1000 )
//!   .messages( vec![
//!     Message {
//!       role : Role::User,
//!       content : vec![ Content::Text {
//!         r#type : "text".to_string(),
//!         text : "Hello, Claude! How are you?".to_string()
//!       } ],
//!       cache_control : None,
//!     }
//!   ] )
//!   .build();
//!
//! // Send the request
//! let response = client.create_message( request ).await?;
//! println!( "Claude responded : {:?}", response.content );
//! # Ok( () )
//! # }
//! ```

#[ cfg( feature = "enabled" ) ]
use mod_interface::mod_interface;

mod private {}

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{
  #[ cfg( feature = "authentication" ) ]
  layer authentication;
  #[ cfg( feature = "batch-processing" ) ]
  layer batch;
  #[ cfg( feature = "circuit-breaker" ) ]
  layer circuit_breaker;
  #[ cfg( feature = "compression" ) ]
  layer compression;
  layer client;
  #[ cfg( feature = "content-generation" ) ]
  layer content_generation;
  #[ cfg( feature = "curl-diagnostics" ) ]
  layer curl_diagnostics;
  #[ cfg( feature = "dynamic-config" ) ]
  layer dynamic_config;
  layer enterprise_config;
  #[ cfg( feature = "enterprise-quota" ) ]
  layer enterprise_quota;
  layer environment;
  #[ cfg( feature = "failover" ) ]
  layer failover;
  #[ cfg( feature = "general-diagnostics" ) ]
  layer general_diagnostics;
  #[ cfg( feature = "health-checks" ) ]
  layer health_checks;
  #[ cfg( feature = "error-handling" ) ]
  layer error;
  layer secret;
  layer messages;
  #[ cfg( feature = "model-management" ) ]
  layer model_management;
  #[ cfg( feature = "rate-limiting" ) ]
  layer rate_limiting;
  #[ cfg( feature = "request-caching" ) ]
  layer request_caching;
  #[ cfg( feature = "retry-logic" ) ]
  layer retry_logic;
  #[ cfg( feature = "streaming" ) ]
  layer streaming;
  #[ cfg( feature = "streaming-control" ) ]
  layer streaming_control;
  #[ cfg( feature = "sync-api" ) ]
  layer sync_api;
  #[ cfg( feature = "model-comparison" ) ]
  layer model_comparison;
  #[ cfg( feature = "request-templates" ) ]
  layer request_templates;
  #[ cfg( feature = "buffered-streaming" ) ]
  layer buffered_streaming;
  #[ cfg( feature = "input-validation" ) ]
  layer input_validation;
  #[ cfg( feature = "enhanced-function-calling" ) ]
  layer enhanced_function_calling;
}

/// Serde-related exports.
#[ cfg( feature = "enabled" ) ]
pub mod ser
{
  pub use serde::
  {
    Serialize,
    Deserialize,
  };
  pub use serde_with::*;
}

/// Error-related exports.
#[ cfg( feature = "enabled" ) ]
pub mod error_tools
{
  pub use::error_tools::*;
}
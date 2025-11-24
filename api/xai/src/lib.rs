#![ doc( html_root_url = "https://docs.rs/api_xai/latest/api_xai/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

// Strategic clippy configuration for comprehensive API client
#![ allow( clippy::missing_inline_in_public_items ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::must_use_candidate ) ]

//! X.AI Grok API client for Rust
//!
//! This crate provides a comprehensive HTTP client for interacting with X.AI's Grok API.
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
//! - **API Transparency**: One-to-one mapping with X.AI Grok API endpoints
//! - **Zero Automatic Behavior**: No implicit decision-making or magic thresholds
//! - **Explicit Control**: Developer decides when, how, and why operations occur
//! - **Information vs Action**: Clear separation between data retrieval and state changes
//! - **Configurable Reliability**: Enterprise features available through explicit configuration
//!
//! ## `OpenAI` Compatibility
//!
//! The X.AI Grok API is `OpenAI`-compatible, using the same REST endpoint patterns and
//! request/response formats. This allows for easy migration from `OpenAI` to X.AI with minimal
//! code changes.
//!
//! ## Enterprise Reliability Features
//!
//! The following enterprise reliability features are **explicitly allowed** when implemented
//! with explicit configuration and transparent operation:
//!
//! - **Configurable Retry Logic**: Exponential backoff with explicit configuration (feature : `retry`)
//! - **Circuit Breaker Pattern**: Failure threshold management with transparent state (feature : `circuit_breaker`)
//! - **Rate Limiting**: Request throttling with explicit rate configuration (feature : `rate_limiting`)
//! - **Failover Support**: Multi-endpoint configuration and automatic switching (feature : `failover`)
//! - **Health Checks**: Periodic endpoint health verification and monitoring (feature : `health_checks`)
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
//! # Secret Management with `workspace_tools`
//!
//! This crate follows wTools ecosystem conventions by prioritizing `workspace_tools`
//! for secret management over environment variables.
//!
//! ## Recommended : Automatic Fallback Chain
//!
//! The `Secret::load_with_fallbacks()` method tries multiple sources in priority order:
//!
//! 1. **Workspace secrets** (`-secrets.sh`) - primary workspace pattern
//! 2. **Alternative files** (`secrets.sh`, `.env`) - workspace alternatives
//! 3. **Environment variable** - fallback for CI/deployment
//!
//! ## Setup Instructions
//!
//! **Option 1: Workspace Secrets (Recommended)**
//!
//! Create `./secret/-secrets.sh` in your workspace root:
//!
//! ```bash
//! #!/bin/bash
//! export XAI_API_KEY="xai-your-key-here"
//! ```
//!
//! The `workspace_tools` fallback chain searches:
//! 1. `./secret/{filename}` (local workspace)
//! 2. `$PRO/secret/{filename}` (PRO workspace)
//! 3. `$HOME/secret/{filename}` (home directory)
//! 4. Environment variable `$XAI_API_KEY`
//!
//! **Option 2: Environment Variable (CI/Deployment)**
//!
//! ```bash
//! export XAI_API_KEY="xai-your-key-here"
//! ```
//!
//! ## Usage
//!
//! ```no_run
//! use api_xai::Secret;
//!
//! // Recommended : tries all sources (workspace-first)
//! let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
//!
//! // Explicit : load from workspace only
//! let secret = Secret::load_from_workspace( "XAI_API_KEY", "-secrets.sh" )?;
//!
//! // Explicit : load from environment only
//! let secret = Secret::load_from_env( "XAI_API_KEY" )?;
//! # Ok::<(), Box< dyn std::error::Error > >(())
//! ```
//!
//! # Examples
//!
//! ```no_run
//! use api_xai::{ Client, Secret, XaiEnvironmentImpl, ChatCompletionRequest, Message, ClientApiAccessors };
//!
//! # async fn example() -> Result< (), Box< dyn std::error::Error > > {
//! // Create a client
//! let secret = Secret::new( "xai-your-key-here".to_string() )?;
//! let env = XaiEnvironmentImpl::new( secret )?;
//! let client = Client::build( env )?;
//!
//! // Create a chat request using the Former builder
//! let request = ChatCompletionRequest::former()
//!   .model( "grok-2-1212".to_string() )
//!   .messages( vec![ Message::user( "Hello, Grok! How are you?" ) ] )
//!   .form();
//!
//! // Send the request
//! let response = client.chat().create( request ).await?;
//! println!( "Grok responded : {:?}", response.choices[ 0 ].message.content );
//! # Ok( () )
//! # }
//! ```

#[ cfg( feature = "enabled" ) ]
use mod_interface::mod_interface;

mod private {}

#[ cfg( feature = "enabled" ) ]
pub use client_api_accessors::ClientApiAccessors;

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{
  /// Error types and result handling for XAI API operations.
  layer error;

  /// Secret management for API keys using `workspace_tools`.
  layer secret;

  /// Environment configuration and HTTP client setup.
  layer environment;

  /// Core HTTP client for XAI API requests.
  layer client;

  /// Trait-based API accessors for chat and models endpoints.
  layer client_api_accessors;

  /// Chat completion request and response types.
  layer chat;

  /// Model listing and details endpoints.
  layer models;

  /// Shared component types (messages, tools, etc).
  layer components;

  /// Circuit breaker pattern for failure management.
  #[ cfg( feature = "circuit_breaker" ) ]
  layer circuit_breaker;

  /// Enhanced retry logic with exponential backoff.
  #[ cfg( feature = "retry" ) ]
  layer enhanced_retry;

  /// Rate limiting with token bucket algorithm.
  #[ cfg( feature = "rate_limiting" ) ]
  layer rate_limiting;

  /// Failover support with multi-endpoint rotation.
  #[ cfg( feature = "failover" ) ]
  layer failover;

  /// Enhanced function calling with parallel execution.
  #[ cfg( feature = "enhanced_tools" ) ]
  layer enhanced_tools;

  /// Health check endpoints for production readiness.
  #[ cfg( feature = "health_checks" ) ]
  layer health_checks;

  /// Structured logging with tracing integration.
  #[ cfg( feature = "structured_logging" ) ]
  layer structured_logging;

  /// Token counting using tiktoken library.
  #[ cfg( feature = "count_tokens" ) ]
  layer count_tokens;

  /// Response caching with LRU eviction policy.
  #[ cfg( feature = "caching" ) ]
  layer caching;

  /// Request parameter validation and error detection.
  #[ cfg( feature = "input_validation" ) ]
  layer input_validation;

  /// CURL command generation for debugging.
  #[ cfg( feature = "curl_diagnostics" ) ]
  layer curl_diagnostics;

  /// Parallel request orchestration for batch processing.
  #[ cfg( feature = "batch_operations" ) ]
  layer batch_operations;

  /// Prometheus metrics collection and export.
  #[ cfg( feature = "performance_metrics" ) ]
  layer performance_metrics;

  /// Synchronous blocking wrappers for async API.
  #[ cfg( feature = "sync_api" ) ]
  layer sync_api;
}

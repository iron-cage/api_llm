// src/lib.rs
//! This is a library for interacting with the `OpenAI` API.
//! It provides a client for various `OpenAI` services, including
//! assistants, chat, embeddings, files, fine-tuning, images, models,
//! moderations, realtime, responses, and vector stores.
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
//! - **API Transparency**: One-to-one mapping with `OpenAI` API endpoints
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
//! This design ensures predictable behavior, explicit control, and transparency
//! for developers using the library.


#[ cfg( feature = "enabled" ) ]
use mod_interface::mod_interface;

#[ cfg( feature = "enabled" ) ]
mod private {}

// Re-export ClientApiAccessors at crate root for convenience
#[ cfg( feature = "enabled" ) ]
pub use client_api_accessors::ClientApiAccessors;

// Client extension modules (impl blocks for Client)
#[ cfg( feature = "enabled" ) ]
mod client_ext_builder;
#[ cfg( feature = "enabled" ) ]
mod client_ext_request_core;
#[ cfg( feature = "enabled" ) ]
mod client_ext_http_basic;
#[ cfg( feature = "enabled" ) ]
mod client_ext_http_stream;

#[ cfg( feature = "enabled" ) ]
crate ::mod_interface!
{
  // API endpoint modules
  layer admin;
  layer assistants;
  #[ cfg( feature = "audio" ) ]
  layer audio;
  layer chat;
  layer embeddings;
  layer files;
  layer fine_tuning;
  layer images;
  layer models;
  #[ cfg( feature = "moderation" ) ]
  layer moderations;
  #[ cfg( feature = "websocket" ) ]
  layer realtime;
  layer responses;
  layer uploads;
  layer vector_stores;

  // Core functionality modules
  layer advanced_auth;
  layer builder_enhancements;
  layer client;
  layer client_api_accessors;
  layer components;
  layer connection_manager;
  // Temporarily disabled due to compilation errors
  layer enhanced_batch_operations;
  layer enhanced_client;
  layer enhanced_client_builder;
  layer enhanced_client_performance;
  #[ cfg( feature = "batching" ) ]
  layer enhanced_embeddings;
  layer curl_generation;
  layer diagnostics;
  layer dynamic_configuration;

  // Feature-gated enhanced modules
  #[ cfg( feature = "circuit_breaker" ) ]
  layer enhanced_circuit_breaker;
  #[ cfg( feature = "rate_limiting" ) ]
  layer enhanced_rate_limiting;
  #[ cfg( feature = "retry" ) ]
  layer enhanced_retry;
  #[ cfg( feature = "enterprise" ) ]
  layer enterprise;

  layer environment;
  layer error;

  #[ cfg( feature = "failover" ) ]
  layer failover;
  #[ cfg( feature = "health_checks" ) ]
  layer health_checks;

  layer metrics_framework;
  layer model_deployment;
  layer model_tuning;

  #[ cfg( feature = "model_comparison" ) ]
  layer model_comparison;
  #[ cfg( feature = "request_templates" ) ]
  layer request_templates;
  #[ cfg( feature = "buffered_streaming" ) ]
  layer buffered_streaming;

  #[ cfg( all( feature = "caching", feature = "compression" ) ) ]
  layer performance_cache;

  layer performance_monitoring;
  layer platform_specific;

  #[ cfg( feature = "input_validation" ) ]
  layer input_validation;
  #[ cfg( feature = "input_validation" ) ]
  layer request_validation;

  #[ cfg( feature = "batching" ) ]
  layer request_batching;

  layer request_cache;
  layer request_cache_enhanced;

  #[ cfg( feature = "caching" ) ]
  layer response_cache;

  layer secret;
  #[ cfg( feature = "streaming_control" ) ]
  layer streaming_control;
  layer streaming_performance_enhanced;
  layer sync;
  #[ cfg( feature = "websocket" ) ]
  layer websocket_reliability_enhanced;
  layer websocket_streaming;

  exposed use admin;
  exposed use advanced_auth;
  exposed use builder_enhancements;
  exposed use client_api_accessors;
  exposed use enhanced_batch_operations;
  exposed use enhanced_client;
  exposed use enhanced_client_builder;
  exposed use enhanced_client_performance;
  // Temporarily disabled due to compilation errors
  #[ cfg( feature = "batching" ) ]
  exposed use enhanced_embeddings;
  exposed use components;
  exposed use connection_manager;
  exposed use curl_generation;
  exposed use diagnostics;
  exposed use dynamic_configuration;

  // Feature-gated exposed modules
  #[ cfg( feature = "circuit_breaker" ) ]
  exposed use enhanced_circuit_breaker;
  #[ cfg( feature = "rate_limiting" ) ]
  exposed use enhanced_rate_limiting;
  #[ cfg( feature = "retry" ) ]
  exposed use enhanced_retry;
  #[ cfg( feature = "enterprise" ) ]
  exposed use enterprise;

  exposed use environment;

  #[ cfg( feature = "failover" ) ]
  exposed use failover;
  #[ cfg( feature = "health_checks" ) ]
  exposed use health_checks;

  exposed use metrics_framework;
  exposed use model_deployment;
  exposed use model_tuning;

  #[ cfg( feature = "model_comparison" ) ]
  exposed use model_comparison;
  #[ cfg( feature = "request_templates" ) ]
  exposed use request_templates;
  #[ cfg( feature = "buffered_streaming" ) ]
  exposed use buffered_streaming;

  #[ cfg( all( feature = "caching", feature = "compression" ) ) ]
  exposed use performance_cache;

  exposed use performance_monitoring;
  exposed use platform_specific;

  #[ cfg( feature = "batching" ) ]
  exposed use request_batching;

  exposed use request_cache;
  exposed use request_cache_enhanced;

  #[ cfg( feature = "caching" ) ]
  exposed use response_cache;

  exposed use secret;
  #[ cfg( feature = "streaming_control" ) ]
  exposed use streaming_control;
  exposed use streaming_performance_enhanced;
  exposed use sync;
  exposed use uploads;
  #[ cfg( feature = "websocket" ) ]
  exposed use websocket_reliability_enhanced;
  exposed use websocket_streaming;
  exposed use error;
  exposed use client;
}

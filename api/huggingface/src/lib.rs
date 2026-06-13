// src/lib.rs
//! `HuggingFace` API client for Rust
//!
//! This library provides a comprehensive HTTP client for interacting with
//! `HuggingFace's` Inference API. It handles authentication, request/response serialization,
//! streaming support, and opt-in reliability features for production deployments.
//!
//! # Design Philosophy : "Thin Client, Rich API"
//!
//! This library provides comprehensive opt-in features with explicit developer control.
//! No automatic behaviors occur without explicit configuration.
//!
//! ## Opt-In Enterprise Features
//!
//! All features require explicit cargo feature flags and developer configuration:
//!
//! - **Circuit Breaker**: Opt-in failure detection with explicit configuration (developer must enable and configure)
//! - **Rate Limiting**: Token bucket rate limiting (developer must explicitly call `rate_limiter.acquire().await`)
//! - **Failover**: Multi-endpoint failover (developer configures and controls failover strategy)
//! - **Health Checks**: Background endpoint health monitoring (developer must explicitly enable and configure)
//! - **Caching**: Intelligent content caching (developer must explicitly enable and use cache)
//! - **Performance Metrics**: Request tracking (developer must explicitly enable metrics collection)
//! - **Dynamic Configuration**: Runtime config updates (developer must explicitly set up watchers)
//!
//! ## Why Explicit Control Matters
//!
//! - **Predictability**: Features only activate when explicitly configured
//! - **Transparency**: No hidden behaviors or automatic decision-making
//! - **Flexibility**: Choose which features to use (if any)
//! - **Performance**: Zero overhead from unused features
//!
//! ## Implemented APIs
//!
//! **Core Functionality**:
//! - Text Generation (Inference API) - `/models/{model_id}` endpoint
//! - Embeddings (Feature Extraction) - Embedding generation and similarity
//! - Model Information - Model metadata and status checking
//! - Inference Providers - Pro plan models via chat completions
//! - Function Calling - Tool/function calling support for chat completions
//! - Streaming - Server-sent events for real-time responses
//!
//! **Planned Expansion**:
//! - Computer Vision APIs : Image classification, object detection, text-to-image, etc.
//! - Audio APIs : ASR, audio classification, text-to-speech
//! - Multimodal APIs : Document QA, visual QA, video-text-to-text
//!
//! ## Historical Context
//!
//! **2024-10-19**: Architecture decision to add opt-in enterprise reliability features.
//! All features require explicit cargo feature flags and developer configuration
//! to maintain the "Thin Client, Rich API" governing principle.

#![ cfg_attr( feature = "enabled", deny( missing_docs ) ) ]
#![ cfg_attr( not( feature = "enabled" ), allow( unused ) ) ]

use mod_interface::mod_interface;

mod private {}

// Core modules (always available)
pub mod error;
pub mod components;
pub mod validation;
pub mod diagnostics;

// Token counting (available with client feature)
#[ cfg( feature = "client" ) ]
pub mod token_counter;

// Content caching (available with client feature)
#[ cfg( feature = "client" ) ]
pub mod cache;

// Performance metrics (available with client feature)
#[ cfg( feature = "client" ) ]
pub mod performance;

// Enterprise reliability features — sub-features: circuit-breaker, rate-limiting, failover, health-checks
#[ cfg( feature = "reliability" ) ]
pub mod reliability;

// Configuration management
#[ cfg( feature = "reliability" ) ]
pub mod config;

// Client module (available with client feature)
#[ cfg( feature = "client" ) ]
pub mod client;

// Environment and secret management
#[ cfg( feature = "env-config" ) ]
pub mod environment;
pub mod secret;

// API endpoint modules (feature-gated)
#[ cfg( feature = "inference" ) ]
pub mod inference;
#[ cfg( feature = "embeddings" ) ]
pub mod embeddings;
#[ cfg( feature = "models" ) ]
pub mod models;
#[ cfg( feature = "inference" ) ]
pub mod providers;

// Vision API module (available with vision feature)
#[ cfg( feature = "vision" ) ]
pub mod vision;

// Audio API module (available with audio feature)
#[ cfg( feature = "audio" ) ]
pub mod audio;

// Sync API module (available with sync feature)
#[ cfg( feature = "sync" ) ]
pub mod sync;

// Streaming control module (available with streaming-control feature; requires inference-streaming)
#[ cfg( feature = "streaming-control" ) ]
pub mod streaming_control;

crate::mod_interface!
{
  // Re-export key types at crate root for convenience
  #[ cfg( feature = "client" ) ]
  exposed use client::Client;
  exposed use secret::Secret;
  #[ cfg( feature = "env-config" ) ]
  exposed use environment::HuggingFaceEnvironmentImpl;
  #[ cfg( feature = "client" ) ]
  exposed use client::ExplicitRetryConfig;
}
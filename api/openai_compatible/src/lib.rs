//! Shared `OpenAI` wire-protocol HTTP layer.
//!
//! Provides a provider-neutral implementation of the `OpenAI` REST protocol
//! suitable for any OpenAI-compatible API endpoint. Extracted from `api_xai`
//! and `api_openai` to eliminate infrastructure duplication.
//!
//! # Features
//!
//! - `enabled` — activates all public types and the HTTP client
//! - `streaming` — Server-Sent Events streaming support
//! - `sync_api` — blocking wrappers around the async client
//! - `integration` — real-API integration tests (requires live credentials)
//! - `full` — enables `enabled`, `streaming`, and `sync_api`
//!
//! # Architecture
//!
//! Follows the "Thin Client, Rich API" principle: every method maps to exactly
//! one API endpoint, zero automatic decision-making, explicit control over all
//! operations. See workspace `spec.md` for governing principles.

#[ cfg( feature = "enabled" ) ]
use mod_interface::mod_interface;

mod private {}

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{
  /// Error type and result alias.
  layer error;

  /// Wire types for chat completion requests, responses, and streaming.
  layer components;

  /// Environment configuration trait and default implementation.
  layer environment;

  /// Async HTTP client.
  layer client;

  /// Blocking wrapper around the async client.
  #[ cfg( feature = "sync_api" ) ]
  layer sync_client;
}

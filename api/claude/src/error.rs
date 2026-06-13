//! Error handling for the Anthropic API client
//!
//! This module provides comprehensive error handling including error classification,
//! recovery strategies, contextual information, and actionable error messages.
//! All error types support structured error recovery and debugging capabilities.

mod private {}

crate::mod_interface!
{
  layer core;
  #[ cfg( feature = "error-handling" ) ]
  layer enhanced;
  #[ cfg( feature = "error-handling" ) ]
  layer enhanced_services;
}

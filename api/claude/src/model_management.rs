//! Model management functionality for Anthropic API
//!
//! This module provides comprehensive model management capabilities including
//! model listing, information retrieval, capability detection, selection logic,
//! and performance optimization through caching.

mod private {}

#[ cfg( feature = "model-management" ) ]
crate::mod_interface!
{
  layer core;
  layer manager;
  layer enhanced;
  layer enhanced_impls;
}

#[ cfg( not( feature = "model-management" ) ) ]
crate::mod_interface!
{
  // Empty when model-management feature is disabled
}

//! Model Deployment Integration Tests for Gemini API Client
//!
//! These tests verify model deployment capabilities including:
//! - Deployment workflow management and lifecycle operations
//! - Configuration validation and builder patterns
//! - State monitoring and change notifications
//! - Scaling and resource management
//! - Deployment strategies (Rolling, Blue-Green, Canary)
//! - Error handling and cleanup operations
//!
//! All tests use real API tokens and make actual deployment operations where possible.

// Import shared test utilities from common module
mod common;
#[ cfg( feature = "integration" ) ]
use common::create_integration_client;

use api_gemini::models::model_deployment::*;
use std::time::Duration;
use std::collections::HashMap;

#[ cfg( feature = "integration" ) ]
mod integration;
mod unit;

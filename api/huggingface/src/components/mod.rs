//! This module defines shared data structures and components used across various
//! `HuggingFace` API groups. It includes common types for requests, responses,
//! and specific components like inference, embeddings, and model-related structures.
//!
//! # Component Organization
//! 
//! Components are logically organized into the following groups:
//! 
//! ## Core Components
//! Foundation components used across all API endpoints:
//! - [`models`] - `HuggingFace` model definitions  
//! - [`wire_types`] - Wire-format types — error response, metadata, task type
//! - [`input`] - Common input handling
//! - [`output`] - Common output handling
//!
//! ## Endpoint Components
//! 
//! ### Text Generation
//! - [`inference_shared`] - Text generation and inference components
//! 
//! ### Embeddings
//! - [`embeddings`] - Text embeddings
//! 
//! ### Tools
//! - [`tools`] - Tool definitions for function calling

mod private
{
}

// === CORE COMPONENTS ===
pub mod wire_types;
pub mod input;
pub mod models;
pub mod output;

// === TEXT GENERATION ===
pub mod inference_shared;

// === EMBEDDINGS ===
pub mod embeddings;

// === TOOLS ===
pub mod tools;

crate::mod_interface!
{
  exposed use wire_types;
  exposed use embeddings;
  exposed use inference_shared;
  exposed use input;
  exposed use models;
  exposed use output;
  exposed use tools;
}
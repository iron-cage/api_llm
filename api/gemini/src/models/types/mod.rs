//! Type definitions for the Gemini API.
//!
//! This module organizes all request/response and configuration types
//! into focused submodules for better maintainability.

pub mod core;
pub mod generation;
pub mod embedding;
pub mod file;
pub mod token;
pub mod cache;
pub mod content;
pub mod streaming;
pub mod chat;
pub mod search;
pub mod function;
pub mod code_execution;
pub mod tuning;

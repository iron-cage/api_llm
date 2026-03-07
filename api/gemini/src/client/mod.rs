//! Client module for interacting with the Gemini API.
//!
//! This module provides both asynchronous and synchronous clients for accessing
//! Google's Gemini AI models. The client supports content generation, embeddings,
//! file operations, cached content, and more.

mod core;
mod builder;
mod config;
mod api_interfaces;
mod api_accessors;
mod dynamic_config;
mod sync;

mod private
{
  // Re-export all types from submodules
  pub use super::core::Client;
  pub use super::builder::ClientBuilder;
  pub use super::config::ClientConfig;
  #[ cfg( feature = "dynamic_configuration" ) ]
  pub use super::config::ConfigWatchHandle;
  pub use super::api_interfaces::ModelsApi;
  #[ allow( unused_imports ) ]  // Used as return types but not re-exported
  pub use super::api_interfaces::{ TunedModelsApi, FilesApi };
  pub use super::api_interfaces::CachedContentApi;
  pub use super::sync::{
    SyncClientBuilder, SyncClient, SyncModelsApi,
    SyncModelApi, SyncCachedContentApi,
  };

  #[ cfg( feature = "chat" ) ]
  pub use super::api_interfaces::{
    ChatApi, ConversationBuilder, ConversationSummary,
  };
}

::mod_interface::mod_interface!
{
  exposed use private::Client;
  exposed use private::ClientBuilder;
  exposed use private::ClientConfig;
  exposed use private::ModelsApi;
  exposed use private::CachedContentApi;
  exposed use private::SyncClientBuilder;
  exposed use private::SyncClient;
  exposed use private::SyncModelsApi;
  exposed use private::SyncModelApi;
  exposed use private::SyncCachedContentApi;

  #[ cfg( feature = "chat" ) ]
  exposed use private::ChatApi;
  #[ cfg( feature = "chat" ) ]
  exposed use private::ConversationBuilder;
  #[ cfg( feature = "chat" ) ]
  exposed use private::ConversationSummary;
}

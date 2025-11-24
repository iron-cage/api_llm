//! API implementation for interacting with Gemini models.
//!
//! This module provides the core API interface for model discovery, content generation,
//! and embedding operations. It is organized into focused submodules:
//!
//! - `models`: Model metadata and discovery
//! - `content_generation`: Text generation, conversation handling, streaming
//! - `embeddings`: Vector embeddings for semantic operations

/// API handle for interacting with a specific model.
///
/// This handle provides model-specific operations and is bound to a particular
/// Gemini model. It offers a convenient way to perform multiple operations
/// with the same model without having to specify the model ID repeatedly.
///
/// The handle is lightweight and can be cloned or used across multiple
/// async operations. All operations are performed with the model specified
/// when the handle was created.
///
/// # Examples
///
/// ```rust,no_run
/// # use api_gemini::client::Client;
/// # #[ tokio::main ]
/// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
/// let client = Client::new()?;
/// let models_api = client.models();
/// let model = models_api.by_name( "gemini-2.5-flash" );
///
/// // Get model information
/// let info = model.get().await?;
/// println!( "Model : {} v{}", info.display_name.as_deref().unwrap_or( "N/A" ), info.version.as_deref().unwrap_or( "N/A" ) );
///
/// // Use the model for content generation
/// let request = api_gemini::GenerateContentRequest {
///   contents : vec![ api_gemini::Content {
///     parts : vec![ api_gemini::Part {
///       text : Some( "What is Rust?".to_string() ),
///       ..Default::default()
///     } ],
///     role : "user".to_string(),
///   } ],
///   ..Default::default()
/// };
///
/// let response = model.generate_content( &request ).await?;
/// if let Some( candidate ) = response.candidates.first() {
///   let content = &candidate.content;
///   if let Some( part ) = content.parts.first() {
///     if let Some( text ) = &part.text {
///       println!( "Response : {}", text );
///     }
///   }
/// }
/// # Ok( () )
/// # }
/// ```
#[ derive( Debug, Clone ) ]
pub struct ModelApi< 'a >
{
  pub( crate ) client : &'a crate::client::Client,
  pub( crate ) model_id : String,
}

// Submodule declarations
mod models;
mod content_generation;
mod embeddings;

// Re-export builders from submodules
pub use content_generation::GenerationRequestBuilder;
pub use embeddings::{ EmbeddingRequestBuilder, BatchEmbeddingRequestBuilder };

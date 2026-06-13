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

impl ModelApi< '_ >
{
  /// Guard used by all API methods: returns InvalidArgument when model_id is empty or whitespace.
  ///
  /// Root cause of previous bug: `by_name("")` accepted empty strings, producing a malformed
  /// URL (`/v1beta/models/:generateContent`) whose HTTP 404 error lacked actionable context.
  /// Pitfall: always validate model_id before URL construction — the URL builder does not
  /// reject empty path segments on its own.
  pub( crate ) fn validate_model_id( &self ) -> Result< (), crate::error::Error >
  {
    if self.model_id.trim().is_empty()
    {
      return Err( crate::error::Error::InvalidArgument(
        "Model ID cannot be empty. Pass a valid model name such as \"gemini-2.5-flash\" \
         or \"models/gemini-embedding-001\" to by_name().".to_string()
      ) );
    }
    Ok( () )
  }
}

// Submodule declarations
mod models;
mod content_generation;
mod embeddings;

// Re-export builders from submodules
pub use content_generation::GenerationRequestBuilder;
pub use embeddings::{ EmbeddingRequestBuilder, BatchEmbeddingRequestBuilder };

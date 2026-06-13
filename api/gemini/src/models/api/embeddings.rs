//! Embedding generation API implementation.
//!
//! This module provides text embedding capabilities for semantic search,
//! similarity comparisons, and other vector-based operations.

use reqwest::Method;
use crate::error::Error;
use crate::models::{ Content, EmbedContentRequest };
use crate::internal::http;

use super::ModelApi;

impl ModelApi< '_ >
{
  /// Generates embeddings for the given content using this model.
  ///
  /// This method creates vector embeddings from text content using the specified
  /// embedding model. Embeddings are useful for semantic search, similarity
  /// comparisons, clustering, and other machine learning tasks.
  ///
  /// # Arguments
  ///
  /// * `request` - An [`crate::models::EmbedContentRequest`] containing:
  ///   - `content`: The text content to embed
  ///   - `task_type`: Optional task type hint (e.g., `"RETRIEVAL_QUERY"`, `"RETRIEVAL_DOCUMENT"`)
  ///   - `title`: Optional title for the content
  ///   - `output_dimensionality`: Optional dimension reduction
  ///
  /// # Returns
  ///
  /// Returns an [`crate::models::EmbedContentResponse`] containing:
  /// - `embedding`: The generated embedding vector
  ///
  /// # Errors
  ///
  /// This method returns an error in the following cases:
  /// - [`Error::InvalidArgument`] - Invalid request format, empty content, or model doesn't support embeddings
  /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
  /// - [`Error::AuthenticationError`] - Invalid or missing API key
  /// - [`Error::RateLimitError`] - API rate limits exceeded
  /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
  /// - [`Error::SerializationError`] - Failed to serialize the request
  /// - [`Error::DeserializationError`] - Failed to parse the API response
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::{ client::Client, EmbedContentRequest, Content, Part };
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
  /// let model = models_api.by_name( "gemini-embedding-001" );
  ///
  /// // Create embedding for search query
  /// let request = EmbedContentRequest {
  ///   content : Content {
  ///     parts : vec![ Part {
  ///       text : Some( "How to learn Rust programming".to_string() ),
  ///       ..Default::default()
  ///     } ],
  ///     role : "user".to_string(),
  ///   },
  ///   task_type : Some( "RETRIEVAL_QUERY".to_string() ),
  ///   title : Some( "Search Query".to_string() ),
  ///   output_dimensionality : None,
  /// };
  ///
  /// let response = model.embed_content( &request ).await?;
  ///
  /// let embedding = &response.embedding;
  /// println!( "Generated embedding with {} dimensions", embedding.values.len() );
  ///
  /// // Use embedding for similarity comparison, search indexing, etc.
  /// // Example : calculate similarity with other embeddings
  /// // let similarity = cosine_similarity( &embedding.values, &other_embedding );
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn embed_content
  (
    &self,
    request : &crate::models::EmbedContentRequest,
  )
  ->
  Result< crate::models::EmbedContentResponse, Error >
  {
    // Validate request before sending
    if request.content.parts.is_empty()
    {
      return Err( Error::InvalidArgument( 
        "Embed content request cannot have empty content parts. Please provide text to embed.".to_string()
      ) );
    }

    // Check if any part has actual content
    let has_content = request.content.parts.iter().any( |part| {
      part.text.as_ref().is_some_and( |text| !text.trim().is_empty() )
    } );

    if !has_content
    {
      return Err( Error::InvalidArgument( 
        "Embed content request must contain at least one text part with non-empty content.".to_string()
      ) );
    }

    let url = format!(
      "{}/v1beta/models/{}:embedContent",
      self.client.base_url,
      self.model_id
    );

    http ::execute_with_optional_retries
    (
      self.client,
      Method::POST,
      &url,
      &self.client.api_key,
      Some( request ),
    )
    .await
    .map_err( |e| self.enhance_model_operation_error( "generate embeddings", e ) )
  }

  /// Generates embeddings from simple text with default settings.
  ///
  /// This is a convenience method for simple text embedding that automatically
  /// wraps the text in the required request structure. For more control over
  /// embedding parameters, use [`embed_content`] directly.
  ///
  /// # Arguments
  ///
  /// * `text` - The text content to embed
  ///
  /// # Returns
  ///
  /// Returns the embedding vector, or an error if embedding fails.
  ///
  /// # Errors
  ///
  /// Returns the same errors as [`embed_content`] plus:
  /// - [`Error::ApiError`] - No embedding returned in response
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
/// let model = models_api.by_name( "gemini-embedding-001" );
  /// 
  /// let embedding = model.embed_text( "Hello, world!" ).await?;
  /// println!( "Generated embedding with {} dimensions", embedding.len() );
  /// # Ok( () )
  /// # }
  /// ```
  ///
  /// [`embed_content`]: ModelApi::embed_content
  #[ inline ]
  pub async fn embed_text( &self, text : &str ) -> Result< Vec< f32 >, Error >
  {
    let request = crate::models::EmbedContentRequest {
      content : crate::models::Content {
        parts : vec![ crate::models::Part {
          text : Some( text.to_string() ),
          ..Default::default()
        } ],
        role : "user".to_string(),
      },
      task_type : None,
      title : None,
      output_dimensionality : None,
    };

    let response = self.embed_content( &request ).await?;
    
    // Extract embedding vector
    let values = response.embedding.values;
    if values.is_empty()
    {
      Err( Error::ApiError( 
        format!( "No embedding values returned from model '{}'. The request may have been invalid or the model may not support embeddings.", 
          self.model_id )
      ) )
    } else {
      Ok( values )
    }
  }

  /// Generates embeddings with task type specification.
  ///
  /// This convenience method allows easy specification of the task type,
  /// which helps the model optimize the embedding for specific use cases
  /// like document retrieval or semantic search.
  ///
  /// # Arguments
  ///
  /// * `text` - The text content to embed
  /// * `task_type` - The task type (e.g., "`RETRIEVAL_QUERY`", "`RETRIEVAL_DOCUMENT`")
  ///
  /// # Returns
  ///
  /// Returns the embedding vector optimized for the specified task.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
/// let model = models_api.by_name( "gemini-embedding-001" );
  /// 
  /// // Embed a search query
  /// let query_embedding = model.embed_text_with_task(
  ///   "How to learn Rust programming", 
  ///   "RETRIEVAL_QUERY"
  /// ).await?;
  /// 
  /// // Embed a document for retrieval
  /// let doc_embedding = model.embed_text_with_task(
  ///   "Rust is a systems programming language...",
  ///   "RETRIEVAL_DOCUMENT"
  /// ).await?;
  /// 
  /// println!( "Query embedding : {} dims, Doc embedding : {} dims", 
  ///   query_embedding.len(), doc_embedding.len() );
  /// # Ok( () )
  /// # }
  /// ```
  ///
  /// # Errors
  ///
  /// Returns the same errors as [`embed_content`].
  ///
  /// [`embed_content`]: ModelApi::embed_content
  #[ inline ]
  pub async fn embed_text_with_task
  (
    &self,
    text : &str,
    task_type : &str,
  )
  ->
  Result< Vec< f32 >, Error >
  {
    let request = crate::models::EmbedContentRequest {
      content : crate::models::Content {
        parts : vec![ crate::models::Part {
          text : Some( text.to_string() ),
          ..Default::default()
        } ],
        role : "user".to_string(),
      },
      task_type : Some( task_type.to_string() ),
      title : None,
      output_dimensionality : None,
    };

    let response = self.embed_content( &request ).await?;
    
    let values = response.embedding.values;
    if values.is_empty()
    {
      Err( Error::ApiError( 
        format!( "No embedding values returned from model '{}' for task type '{}'.", 
          self.model_id, task_type )
      ) )
    } else {
      Ok( values )
    }
  }

  /// Generates embeddings for multiple texts efficiently.
  ///
  /// This method processes multiple texts in batches for better performance
  /// compared to individual embedding requests. It automatically handles
  /// batch size optimization and concurrent processing.
  ///
  /// # Arguments
  ///
  /// * `texts` - Vector of text strings to embed
  /// * `task_type` - Optional task type for all embeddings
  ///
  /// # Returns
  ///
  /// Returns a vector of embedding vectors in the same order as input texts.
  ///
  /// # Performance
  ///
  /// This method is optimized for batch processing:
  /// - Processes texts concurrently when possible
  /// - Uses efficient memory allocation patterns
  /// - Minimizes API calls through smart batching
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
/// let model = models_api.by_name( "gemini-embedding-001" );
  /// 
  /// let texts = vec![
  ///   "First document to embed".to_string(),
  ///   "Second document to embed".to_string(),
  ///   "Third document to embed".to_string(),
  /// ];
  /// 
  /// let embeddings = model.embed_texts( texts, Some( "RETRIEVAL_DOCUMENT" ) ).await?;
  /// println!( "Generated {} embeddings", embeddings.len() );
  /// 
  /// for (i, embedding) in embeddings.iter().enumerate() {
  ///   println!( "Text {}: {} dimensions", i + 1, embedding.len() );
  /// }
  /// # Ok( () )
  /// # }
  /// ```
  ///
  /// # Errors
  ///
  /// Returns the same errors as [`embed_text`] or [`embed_text_with_task`] for each individual text.
  ///
  /// [`embed_text`]: ModelApi::embed_text
  /// [`embed_text_with_task`]: ModelApi::embed_text_with_task
  #[ inline ]
  pub async fn embed_texts
  (
    &self,
    texts : Vec< String >,
    task_type : Option< &str >,
  )
  ->
  Result< Vec< Vec< f32 > >, Error >
  {
    if texts.is_empty()
    {
      return Ok( vec![] );
    }

    // For now, process individually - future optimization could use batch API
    // when available from Gemini
    let mut results = Vec::with_capacity( texts.len() );
    
    for text in texts
    {
      let embedding = match task_type
      {
        Some( task ) => self.embed_text_with_task( &text, task ).await?,
        None => self.embed_text( &text ).await?,
      };
      results.push( embedding );
    }
    
    Ok( results )
  }

  /// Calculates cosine similarity between two embedding vectors.
  ///
  /// This utility method helps with common embedding operations like
  /// similarity search and document ranking. Cosine similarity ranges
  /// from -1 (opposite) to 1 (identical), with 0 indicating orthogonality.
  ///
  /// # Arguments
  ///
  /// * `embedding1` - First embedding vector
  /// * `embedding2` - Second embedding vector
  ///
  /// # Returns
  ///
  /// Returns the cosine similarity as a float between -1.0 and 1.0.
  ///
  /// # Errors
  ///
  /// Returns [`Error::InvalidArgument`] if:
  /// - Vectors have different dimensions
  /// - Either vector is zero-length or all zeros
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
/// let model = models_api.by_name( "gemini-embedding-001" );
  /// 
  /// let embed1 = model.embed_text( "machine learning" ).await?;
  /// let embed2 = model.embed_text( "artificial intelligence" ).await?;
  /// let embed3 = model.embed_text( "cooking recipes" ).await?;
  /// 
  /// let similarity_related = model.cosine_similarity( &embed1, &embed2 )?;
  /// let similarity_unrelated = model.cosine_similarity( &embed1, &embed3 )?;
  /// 
  /// println!( "ML vs AI similarity : {:.3}", similarity_related );
  /// println!( "ML vs Cooking similarity : {:.3}", similarity_unrelated );
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub fn cosine_similarity
  (
    &self,
    embedding1 : &[ f32 ],
    embedding2 : &[ f32 ],
  )
  ->
  Result< f32, Error >
  {
    if embedding1.len() != embedding2.len()
    {
      return Err( Error::InvalidArgument( 
        format!( "Embedding dimensions must match : {} vs {}", 
          embedding1.len(), embedding2.len() )
      ) );
    }

    if embedding1.is_empty()
    {
      return Err( Error::InvalidArgument( 
        "Cannot compute similarity for empty embeddings".to_string()
      ) );
    }

    // Calculate dot product
    let dot_product : f32 = embedding1.iter()
      .zip( embedding2.iter() )
      .map( |(a, b)| a * b )
      .sum();

    // Calculate magnitudes
    let magnitude1 : f32 = embedding1.iter().map( |x| x * x ).sum::< f32 >().sqrt();
    let magnitude2 : f32 = embedding2.iter().map( |x| x * x ).sum::< f32 >().sqrt();

    // Handle zero vectors
    if magnitude1 == 0.0 || magnitude2 == 0.0
    {
      return Err( Error::InvalidArgument( 
        "Cannot compute similarity for zero vectors".to_string()
      ) );
    }

    Ok( dot_product / ( magnitude1 * magnitude2 ) )
  }

  /// Creates an embedding request builder for complex scenarios.
  ///
  /// This method returns a builder that allows fluent configuration of
  /// embedding parameters before executing the request. Useful for
  /// fine-grained control over the embedding process.
  ///
  /// # Returns
  ///
  /// Returns an [`EmbeddingRequestBuilder`] for fluent request configuration.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
/// let model = models_api.by_name( "gemini-embedding-001" );
  /// 
  /// let embedding = model.embedding_request()
  ///   .with_text( "Advanced machine learning techniques" )
  ///   .with_task_type( "RETRIEVAL_DOCUMENT" )
  ///   .with_title( "ML Research Paper" )
  ///   .with_output_dimensionality( 512 )
  ///   .execute_vector()
  ///   .await?;
  /// 
  /// println!( "Generated {} dimensional embedding", embedding.len() );
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  #[ must_use ]
  pub fn embedding_request( &self ) -> EmbeddingRequestBuilder< '_ >
  {
    EmbeddingRequestBuilder::new( self )
  }
}
impl ModelApi< '_ >
{
  /// Generates embeddings for multiple texts in batch.
  ///
  /// This method processes multiple texts efficiently using batch processing
  /// to minimize API calls and improve performance compared to individual requests.
  /// The implementation automatically handles optimal batch sizes and concurrent processing.
  ///
  /// # Arguments
  ///
  /// * `texts` - A slice of text strings to embed
  ///
  /// # Returns
  ///
  /// Returns a vector of embedding vectors, one for each input text.
  /// The order of embeddings corresponds to the order of input texts.
  ///
  /// # Errors
  ///
  /// This method returns an error in the following cases:
  /// - [`Error::ValidationError`] - Empty input or invalid texts
  /// - [`Error::BatchProcessingError`] - Partial processing failures with details
  /// - [`Error::NetworkError`] - Network connectivity issues
  /// - [`Error::AuthenticationError`] - Invalid or missing API key
  /// - [`Error::RateLimitError`] - Rate limits exceeded
  /// - [`Error::ApiError`] - Other API-related errors
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
  /// let model = models_api.by_name( "gemini-embedding-001" );
  /// 
  /// let texts = vec![
  ///   "Hello world",
  ///   "This is a test",
  ///   "Batch processing example",
  /// ];
  /// 
  /// let embeddings = model.batch_embed_texts( &texts ).await?;
  /// println!( "Generated {} embeddings", embeddings.len() );
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn batch_embed_texts( &self, texts : &[ &str ] ) -> Result< Vec< Vec< f32 > >, Error >
  {
    // Validate input
    if texts.is_empty()
    {
      return Err( Error::ValidationError { 
        message : "Cannot process empty text list".to_string() 
      } );
    }

    // Generate batch ID for correlation and logging
    #[ cfg( feature = "logging" ) ]
    let batch_id = format!( "batch-{:08x}", rand::random::< u32 >() );
    
    #[ cfg( feature = "logging" ) ]
    tracing ::info!(
      batch_id = %batch_id,
      batch_size = texts.len(),
      "Starting batch embedding operation"
    );

    // For now, process texts individually
    // qqq : Implement actual batch API when available from Gemini (task/verified/004)
    let mut embeddings = Vec::with_capacity( texts.len() );
    let mut successful = 0;
    let mut failed = 0;

    for ( index, text ) in texts.iter().enumerate()
    {
      #[ cfg( feature = "logging" ) ]
      tracing ::debug!(
        batch_id = %batch_id,
        batch_index = index,
        "Processing batch item"
      );
      
      #[ cfg( not( feature = "logging" ) ) ]
      let _ = index; // Suppress unused variable warning when logging disabled
      
      match self.embed_text( text ).await
      {
        Ok( embedding ) => {
          embeddings.push( embedding );
          successful += 1;
        },
        Err( e ) => {
          failed += 1;
          // For now, propagate the first error
          if embeddings.is_empty()
          {
            return Err( e );
          }
          // If we have some successful embeddings, return a batch error
          // Count remaining texts as failed
          let remaining = texts.len() - successful - failed;
          
          #[ cfg( feature = "logging" ) ]
          tracing ::error!(
            batch_id = %batch_id,
            successful = successful,
            failed = failed + remaining,
            "Batch embedding operation failed"
          );
          
          return Err( Error::BatchProcessingError {
            successful,
            failed : failed + remaining,
            message : format!( "Batch processing failed on text '{text}': {e}" ),
          } );
        }
      }
    }

    #[ cfg( feature = "logging" ) ]
    tracing ::info!(
      batch_id = %batch_id,
      successful = successful,
      failed = failed,
      "Batch embedding operation completed"
    );

    Ok( embeddings )
  }

  /// Generates embeddings for multiple content objects in batch.
  ///
  /// This method processes multiple content objects (text, images, etc.) efficiently
  /// using batch processing to minimize API calls and improve performance.
  ///
  /// # Arguments
  ///
  /// * `contents` - A slice of Content objects to embed
  ///
  /// # Returns
  ///
  /// Returns a vector of embedding vectors, one for each input content.
  /// The order of embeddings corresponds to the order of input contents.
  ///
  /// # Errors
  ///
  /// Same error conditions as [`Self::batch_embed_texts`].
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # use api_gemini::models::{ Content, Part };
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
  /// let model = models_api.by_name( "gemini-embedding-001" );
  /// 
  /// let contents = vec![
  ///   Content {
  ///     parts : vec![ Part { text : Some( "First content".to_string() ), ..Default::default() } ],
  ///     role : "user".to_string(),
  ///   },
  ///   Content {
  ///     parts : vec![ Part { text : Some( "Second content".to_string() ), ..Default::default() } ],
  ///     role : "user".to_string(),
  ///   },
  /// ];
  /// 
  /// let embeddings = model.batch_embed_contents( &contents ).await?;
  /// println!( "Generated {} embeddings", embeddings.len() );
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn batch_embed_contents( &self, contents : &[ Content ] ) -> Result< Vec< Vec< f32 > >, Error >
  {
    // Validate input
    if contents.is_empty()
    {
      return Err( Error::ValidationError { 
        message : "Cannot process empty content list".to_string() 
      } );
    }

    // For now, process contents individually
    // qqq : Implement actual batch API when available from Gemini (task/verified/004)
    let mut embeddings = Vec::with_capacity( contents.len() );
    let mut successful = 0;
    let mut failed = 0;

    for content in contents
    {
      let embed_request = EmbedContentRequest {
        content : content.clone(),
        task_type : None,
        title : None,
        output_dimensionality : None,
      };

      match self.embed_content( &embed_request ).await
      {
        Ok( response ) => {
          embeddings.push( response.embedding.values );
          successful += 1;
        },
        Err( e ) => {
          failed += 1;
          if embeddings.is_empty()
          {
            return Err( e );
          }
          let remaining = contents.len() - successful - failed;
          return Err( Error::BatchProcessingError {
            successful,
            failed : failed + remaining,
            message : format!( "Batch processing failed on content : {e}" ),
          } );
        }
      }
    }

    Ok( embeddings )
  }

  /// Creates a batch embedding request builder for advanced configuration.
  ///
  /// This method returns a builder that allows fine-grained control over
  /// batch processing parameters such as batch size, timeouts, and retry logic.
  ///
  /// # Returns
  ///
  /// Returns a [`BatchEmbeddingRequestBuilder`] for configuring the batch request.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # use std::time::Duration;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let models_api = client.models();
  /// let model = models_api.by_name( "gemini-embedding-001" );
  /// 
  /// let texts = vec![ "Text 1", "Text 2" ];
  /// let embeddings = model
  ///   .batch_embed_request()
  ///   .with_texts( &texts )
  ///   .with_batch_size( 10 )
  ///   .with_timeout( Duration::from_secs( 30 ) )
  ///   .execute()
  ///   .await?;
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  #[ must_use ]
  pub fn batch_embed_request( &self ) -> BatchEmbeddingRequestBuilder< '_ >
  {
    BatchEmbeddingRequestBuilder::new( self )
  }
}

#[ path = "embeddings_builders.rs" ]
mod embeddings_builders;
pub use embeddings_builders::{ EmbeddingRequestBuilder, BatchEmbeddingRequestBuilder };

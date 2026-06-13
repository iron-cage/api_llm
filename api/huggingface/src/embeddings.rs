//! Embedding generation and feature extraction operations for `HuggingFace` API.

mod private
{
use crate::
{
  client::Client,
  components::
  {
  embeddings::{ EmbeddingRequest, EmbeddingResponse, EmbeddingOptions },
  // common::TaskType,
  },
  error::Result,
  validation::{ validate_input_text, validate_model_identifier, validate_batch_inputs },
};

#[ cfg( feature = "env-config" ) ]
use crate::environment::{ HuggingFaceEnvironment, EnvironmentInterface };

/// `HuggingFace` Inference provider base URL for embedding/feature-extraction endpoints.
/// Uses the `hf-inference` provider via the Router (not the `/v1/chat/completions` path).
const HF_SERVERLESS_API_BASE : &str = "https://router.huggingface.co/hf-inference";

/// API group for `HuggingFace` embedding operations
#[ derive( Debug ) ]
pub struct Embeddings< E >
where
  E : Clone,
{
  client : Client< E >,
}

#[ cfg( feature = "env-config" ) ]
impl< E > Embeddings< E >
where
  E : HuggingFaceEnvironment + EnvironmentInterface + Send + Sync + 'static + Clone,
{
  /// Create a new Embeddings API group
  #[ inline ]
  #[ must_use ]
  pub fn new( client : &Client< E > ) -> Self
  {
  Self
  {
      client : client.clone(),
  }
  }
  
  /// Create an embedding for a single text
  ///
  /// # Arguments
  /// - `input`: Input text to embed
  /// - `model`: Model identifier (e.g., "sentence-transformers/all-MiniLM-L6-v2")
  ///
  /// # Errors
  /// Returns error if the request fails or response is invalid
  #[ inline ]
  pub async fn create( 
  &self, 
  input : impl Into< String >, 
  model : impl AsRef< str >
  ) -> Result< EmbeddingResponse >
  {
  let input_text = input.into();
  let model_id = model.as_ref();
  
  // Validate input parameters
  validate_input_text( &input_text )?;
  validate_model_identifier( model_id )?;
  
  // Wrap single text in array so response is [[f1,...]] matching EmbeddingResponse::Single
  let request = EmbeddingRequest::new_batch( vec![ input_text ] );
  let url = format!( "{HF_SERVERLESS_API_BASE}/models/{model_id}" );

  self.client.post( &url, &request ).await
  }

  /// Create embeddings for multiple texts
  ///
  /// # Arguments
  /// - `inputs`: Input texts to embed
  /// - `model`: Model identifier
  ///
  /// # Errors
  /// Returns error if the request fails or response is invalid
  #[ inline ]
  pub async fn create_batch( 
  &self, 
  inputs : Vec< String >, 
  model : impl AsRef< str >
  ) -> Result< EmbeddingResponse >
  {
  let model_id = model.as_ref();
  
  // Validate input parameters
  validate_batch_inputs( &inputs )?;
  validate_model_identifier( model_id )?;
  
  let request = EmbeddingRequest::new_batch( inputs );
  let url = format!( "{HF_SERVERLESS_API_BASE}/models/{model_id}" );

  self.client.post( &url, &request ).await
  }

  /// Create an embedding with custom options
  ///
  /// # Arguments
  /// - `input`: Input text to embed
  /// - `model`: Model identifier
  /// - `options`: Embedding options (normalization, pooling, etc.)
  ///
  /// # Errors
  /// Returns error if the request fails or response is invalid
  #[ inline ]
  pub async fn create_with_options( 
  &self, 
  input : impl Into< String >, 
  model : impl AsRef< str >,
  options : EmbeddingOptions
  ) -> Result< EmbeddingResponse >
  {
  let input_text = input.into();
  let model_id = model.as_ref();
  
  // Validate input parameters
  validate_input_text( &input_text )?;
  validate_model_identifier( model_id )?;
  
  let request = EmbeddingRequest::new_batch( vec![ input_text ] ).with_options( options );
  let url = format!( "{HF_SERVERLESS_API_BASE}/models/{model_id}" );

  self.client.post( &url, &request ).await
  }

  /// Create embeddings using the feature extraction pipeline
  ///
  /// # Arguments
  /// - `inputs`: Input texts to embed
  /// - `model`: Model identifier
  ///
  /// # Errors
  /// Returns error if the request fails or response is invalid
  #[ inline ]
  pub async fn feature_extraction( 
  &self, 
  inputs : Vec< String >, 
  model : impl AsRef< str >
  ) -> Result< Vec< Vec< f32 > > >
  {
  let model_id = model.as_ref();
  
  // Validate input parameters
  validate_batch_inputs( &inputs )?;
  validate_model_identifier( model_id )?;
  
  let request = serde_json::json!
  ({
      "inputs": inputs,
      "parameters": {
  "task": "feature-extraction"
      }
  });

  let url = format!( "{HF_SERVERLESS_API_BASE}/models/{model_id}" );

  self.client.post( &url, &request ).await
  }

  /// Get similarity between two texts using embeddings
  ///
  /// # Arguments
  /// - `text1`: First text to compare
  /// - `text2`: Second text to compare
  /// - `model`: Embedding model identifier
  ///
  /// # Returns
  /// Cosine similarity score between -1.0 and 1.0
  ///
  /// # Errors
  /// Returns error if the request fails or similarity calculation fails
  #[ inline ]
  pub async fn similarity( 
  &self, 
  text1 : impl Into< String >, 
  text2 : impl Into< String >,
  model : impl AsRef< str >
  ) -> Result< f32 >
  {
  let first_text = text1.into();
  let second_text = text2.into();
  let model_id = model.as_ref();
  
  // Validate input parameters
  validate_input_text( &first_text )?;
  validate_input_text( &second_text )?;
  validate_model_identifier( model_id )?;
  
  let text_inputs = vec![ first_text, second_text ];
  let embeddings : Vec< Vec< f32 > > = self.feature_extraction( text_inputs, model_id ).await?;
  
  if embeddings.len() != 2
  {
      return Err( crate::error::HuggingFaceError::Generic( 
  "Expected exactly 2 embeddings for similarity calculation".to_string() 
      ) );
  }
  
  let first_embedding = &embeddings[ 0 ];
  let second_embedding = &embeddings[ 1 ];
  
  let similarity = cosine_similarity( first_embedding, second_embedding )?;
  Ok( similarity )
  }
}

// Basic implementation for when env-config is not available
#[ cfg( not( feature = "env-config" ) ) ]
impl< E > Embeddings< E >
where
  E : Clone,
{
  /// Create a new Embeddings API group
  #[ inline ]
  #[ must_use ]
  pub fn new( client : &Client< E > ) -> Self
  {
  Self
  {
      client : (*client).clone(),
  }
  }
}

/// Calculate cosine similarity between two vectors
#[ inline ]
fn cosine_similarity( a : &[ f32 ], b : &[ f32 ] ) -> Result< f32 >
{
  if a.len() != b.len()
  {
  return Err( crate::error::HuggingFaceError::InvalidArgument( 
      "Vectors must have the same dimension".to_string() 
  ) );
  }
  
  let dot_product : f32 = a.iter().zip( b.iter() ).map( | ( x, y ) | x * y ).sum();
  let magnitude_a : f32 = a.iter().map( | x | x * x ).sum::< f32 >().sqrt();
  let magnitude_b : f32 = b.iter().map( | x | x * x ).sum::< f32 >().sqrt();
  
  if magnitude_a == 0.0 || magnitude_b == 0.0
  {
  return Err( crate::error::HuggingFaceError::Generic( 
      "Cannot compute similarity with zero magnitude vector".to_string() 
  ) );
  }
  
  // Clamp to [-1.0, 1.0]: floating-point rounding can yield values slightly outside
  // this range for nearly-identical vectors, violating the cosine similarity invariant.
  Ok( ( dot_product / ( magnitude_a * magnitude_b ) ).clamp( -1.0, 1.0 ) )
}

} // end mod private

crate::mod_interface!
{
  exposed use 
  {
  private::Embeddings,
  };
}
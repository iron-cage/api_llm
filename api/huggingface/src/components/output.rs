//! Output handling and response processing for `HuggingFace` API.

use serde::{ Deserialize, Serialize };
use super::wire_types::ResponseMetadata;

/// Base response structure for `HuggingFace` inference
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct InferenceOutput
{
  /// Generated text
  pub generated_text : String,
  
  /// Input tokens (if available)
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub input_tokens : Option< u32 >,
  
  /// Generated tokens (if available)
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub generated_tokens : Option< u32 >,
  
  /// Response metadata
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub metadata : Option< ResponseMetadata >,
}

/// Batch inference response containing multiple outputs
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct BatchInferenceOutput
{
  /// List of inference outputs
  pub outputs : Vec< InferenceOutput >,
  
  /// Response metadata
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub metadata : Option< ResponseMetadata >,
}

/// Streaming token response
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct StreamToken
{
  /// Token text
  pub token : String,
  
  /// Token id
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub id : Option< u32 >,
  
  /// Log probability
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub logprob : Option< f32 >,
  
  /// Whether this is the final token
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub is_final : Option< bool >,
}

/// Embedding output for feature extraction
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct EmbeddingOutput
{
  /// Embedding vector
  pub embedding : Vec< f32 >,
  
  /// Input text length
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub input_length : Option< u32 >,
  
  /// Response metadata
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub metadata : Option< ResponseMetadata >,
}

/// Batch embedding response
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct BatchEmbeddingOutput
{
  /// List of embeddings
  pub embeddings : Vec< EmbeddingOutput >,
  
  /// Response metadata
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub metadata : Option< ResponseMetadata >,
}
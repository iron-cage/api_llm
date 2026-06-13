//! Wire-format types for `HuggingFace` API — error response, response metadata, task type.

use serde::{ Deserialize, Serialize };

/// Base error response from `HuggingFace` API
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ErrorResponse
{
  /// Error message
  pub error : String,
  
  /// Optional error type
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub error_type : Option< String >,
  
  /// Optional status code
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub status_code : Option< u16 >,
}

/// Common response metadata
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ResponseMetadata
{
  /// Model used for the request
  pub model : String,
  
  /// Compute time in seconds
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub compute_time : Option< f64 >,
  
  /// Load time in seconds
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub load_time : Option< f64 >,
}

/// Task type for `HuggingFace` inference
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( rename_all = "lowercase" ) ]
pub enum TaskType
{
  /// Text generation task
  #[ serde( rename = "text-generation" ) ]
  TextGeneration,
  
  /// Feature extraction (embeddings)
  #[ serde( rename = "feature-extraction" ) ]
  FeatureExtraction,
  
  /// Conversational task
  Conversational,
  
  /// Question answering
  #[ serde( rename = "question-answering" ) ]
  QuestionAnswering,
  
  /// Summarization
  Summarization,
  
  /// Translation
  Translation,
}

// No Default implementation - explicit task type selection required
//! Content structure types for the Gemini API.

use serde::{ Deserialize, Serialize };
use super::file::VideoMetadata;

/// Content in a conversation.
#[ derive( Debug, Clone, Serialize, Deserialize, Default ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct Content
{
  /// Parts that make up the content.
  /// Optional to handle cases where API returns content without parts.
  #[ serde( default ) ]
  pub parts : Vec< Part >,
  /// Role of the content creator.
  /// Optional to handle cases where API returns content without role (e.g., finishReason=MAX_TOKENS with empty content).
  #[ serde( default ) ]
  pub role : String,
}

/// A part of content.
#[ derive( Debug, Clone, Serialize, Deserialize, Default ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct Part
{
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Text content.
  pub text : Option< String >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Inline data blob.
  pub inline_data : Option< Blob >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Function call made by the model.
  pub function_call : Option< FunctionCall >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Response to a function call.
  pub function_response : Option< FunctionResponse >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// File data for multimedia content
  pub file_data : Option< FileData >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Video metadata for video content
  pub video_metadata : Option< VideoMetadata >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Whether this part is an internal thinking step (gemini-2.5+ thinking models).
  /// Thinking parts should not be included in the user-visible response.
  pub thought : Option< bool >,
}

/// Binary data with MIME type.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct Blob
{
  /// MIME type of the data.
  pub mime_type : String,
  /// Base64-encoded data.
  pub data : String,
}

/// File data for multimedia content
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct FileData
{
  /// File URI or identifier
  pub uri : Option< String >,
  /// MIME type of the file
  pub mime_type : Option< String >,
}


/// A function call made by the model.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct FunctionCall
{
  /// The name of the function.
  pub name : String,
  /// Arguments for the function call.
  pub args : serde_json::Value,
}

/// Response to a function call.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct FunctionResponse
{
  /// The name of the function.
  pub name : String,
  /// The function's response data.
  pub response : serde_json::Value,
}

/// A response candidate from the model.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct Candidate
{
  /// Generated content.
  pub content : Content,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Reason why generation stopped.
  pub finish_reason : Option< String >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Safety ratings for the content.
  pub safety_ratings : Option< Vec< SafetyRating > >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Citation information.
  pub citation_metadata : Option< CitationMetadata >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Number of tokens in this candidate.
  pub token_count : Option< i32 >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Index of this candidate.
  pub index : Option< i32 >,
}

/// Safety rating for content.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct SafetyRating
{
  /// The safety category.
  pub category : String,
  /// Probability level of the category.
  pub probability : String,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Whether content was blocked.
  pub blocked : Option< bool >,
}

/// Citation information for generated content.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct CitationMetadata
{
  /// List of citation sources.
  pub citation_sources : Vec< CitationSource >,
}

/// A source citation.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct CitationSource
{
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// Start index in the generated text.
  pub start_index : Option< i32 >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// End index in the generated text.
  pub end_index : Option< i32 >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// URI of the source.
  pub uri : Option< String >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  /// License of the source.
  pub license : Option< String >,
}

/// System instruction with structured content.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( rename_all = "camelCase" ) ]
pub struct SystemInstruction
{
  /// Role of the system instruction (typically "system").
  pub role : String,

  /// Parts containing the system instruction content.
  pub parts : Vec< Part >,
}

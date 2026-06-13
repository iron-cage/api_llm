//! Vision API Types
//!
//! Data structures for vision API requests and responses.

use serde::{ Deserialize, Serialize };

/// Image input for vision tasks
///
/// Supports multiple input formats for flexibility.
#[ derive( Debug, Clone ) ]
pub enum ImageInput
{
  /// Raw image bytes
  Bytes( Vec< u8 > ),

  /// Base64-encoded image
  Base64( String ),

  /// URL to image
  Url( String ),
}

impl ImageInput
{
  /// Create image input from raw bytes
  #[ inline ]
  #[ must_use ]
  pub fn from_bytes( bytes : Vec< u8 > ) -> Self
  {
  Self::Bytes( bytes )
  }

  /// Create image input from base64 string
  #[ inline ]
  #[ must_use ]
  pub fn from_base64( data : impl Into< String > ) -> Self
  {
  Self::Base64( data.into() )
  }

  /// Create image input from URL
  #[ inline ]
  #[ must_use ]
  pub fn from_url( url : impl Into< String > ) -> Self
  {
  Self::Url( url.into() )
  }

  /// Convert to base64 for API transmission
  #[ inline ]
  #[ must_use ]
  pub fn to_base64( &self ) -> String
  {
  match self
  {
      Self::Bytes( bytes ) => base64_encode( bytes ),
      Self::Base64( data ) => data.clone(),
      Self::Url( url ) => url.clone(), // URLs sent as-is
  }
  }
}

/// Encode bytes to base64
fn base64_encode( bytes : &[ u8 ] ) -> String
{
  use base64::{ Engine, engine::general_purpose };
  general_purpose::STANDARD.encode( bytes )
}

/// Image classification result
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ClassificationResult
{
  /// Predicted label
  pub label : String,

  /// Confidence score (0.0 - 1.0)
  pub score : f64,
}

/// Object detection bounding box
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct BoundingBox
{
  /// X coordinate of top-left corner
  pub xmin : f64,

  /// Y coordinate of top-left corner
  pub ymin : f64,

  /// X coordinate of bottom-right corner
  pub xmax : f64,

  /// Y coordinate of bottom-right corner
  pub ymax : f64,
}

/// Object detection result
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct DetectionResult
{
  /// Detected object label
  pub label : String,

  /// Confidence score (0.0 - 1.0)
  pub score : f64,

  /// Bounding box coordinates
  #[ serde( rename = "box" ) ]
  pub box_coords : BoundingBox,
}

/// Image captioning result
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct CaptionResult
{
  /// Generated caption text
  pub generated_text : String,
}

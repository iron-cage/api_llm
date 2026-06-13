//! Audio API Types
//!
//! Data structures for audio API requests and responses.

use serde::{ Deserialize, Serialize };

/// Audio input for audio processing tasks
///
/// Supports multiple input formats for flexibility.
#[ derive( Debug, Clone ) ]
pub enum AudioInput
{
  /// Raw audio bytes (WAV, MP3, FLAC, etc.)
  Bytes( Vec< u8 > ),

  /// Base64-encoded audio
  Base64( String ),

  /// URL to audio file
  Url( String ),
}

impl AudioInput
{
  /// Create audio input from raw bytes
  #[ inline ]
  #[ must_use ]
  pub fn from_bytes( bytes : Vec< u8 > ) -> Self
  {
  Self::Bytes( bytes )
  }

  /// Create audio input from base64 string
  #[ inline ]
  #[ must_use ]
  pub fn from_base64( data : impl Into< String > ) -> Self
  {
  Self::Base64( data.into() )
  }

  /// Create audio input from URL
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

/// Automatic Speech Recognition (ASR) result
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct TranscriptionResult
{
  /// Transcribed text
  pub text : String,
}

/// Audio classification result
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct AudioClassificationResult
{
  /// Predicted label
  pub label : String,

  /// Confidence score (0.0 - 1.0)
  pub score : f64,
}

/// Text-to-Speech generation result
#[ derive( Debug, Clone ) ]
pub struct SpeechGenerationResult
{
  /// Generated audio data
  pub audio_data : Vec< u8 >,

  /// Sample rate (Hz)
  pub sample_rate : Option< u32 >,

  /// Audio format (e.g., "wav", "mp3")
  pub format : Option< String >,
}

/// Audio-to-audio transformation result
#[ derive( Debug, Clone ) ]
pub struct AudioTransformResult
{
  /// Transformed audio data
  pub audio_data : Vec< u8 >,

  /// Sample rate (Hz)
  pub sample_rate : Option< u32 >,

  /// Audio format
  pub format : Option< String >,
}

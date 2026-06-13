//! Input handling and validation for `HuggingFace` API requests.

use serde::{ Deserialize, Serialize };
use std::collections::HashMap;
use crate::
{
  error::{ HuggingFaceError, Result },
  validation::
  {
  validate_temperature,
  validate_max_new_tokens,
  validate_top_p,
  validate_repetition_penalty,
  validate_stop_sequences,
  },
};

/// Binary input for classification endpoints (vision and audio).
///
/// Sends raw binary data (base64-encoded) or a URL string to classification models.
/// Used by both vision and audio classification to avoid duplicating the same type.
#[ derive( Debug, Serialize ) ]
pub( crate ) struct BinaryClassificationInput
{
  /// Input data — base64-encoded bytes or a URL
  pub( crate ) inputs : String,
}

/// Base parameters for `HuggingFace` inference requests
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct InferenceParameters
{
  /// Temperature for sampling (0.0 to 2.0)
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub temperature : Option< f32 >,
  
  /// Maximum number of tokens to generate
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub max_new_tokens : Option< u32 >,
  
  /// Top-p sampling parameter
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub top_p : Option< f32 >,
  
  /// Top-k sampling parameter
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub top_k : Option< u32 >,
  
  /// Repetition penalty
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub repetition_penalty : Option< f32 >,
  
  /// Whether to return full text or just new tokens
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub return_full_text : Option< bool >,
  
  /// Stop sequences
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub stop : Option< Vec< String > >,
  
  /// Whether to stream the response
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub stream : Option< bool >,
  
  /// Additional model-specific parameters
  #[ serde( flatten ) ]
  pub additional : HashMap< String, serde_json::Value >,
}

impl Default for InferenceParameters
{
  #[ inline ]
  fn default() -> Self
  {
  Self::recommended()
  }
}

impl InferenceParameters
{
  /// Create new inference parameters with HuggingFace-recommended values.
  ///
  /// # Governing Principle Compliance
  ///
  /// This provides HuggingFace-recommended parameters without making them implicit defaults.
  /// Developers must explicitly choose to use these recommended values.
  #[ inline ]
  #[ must_use ]
  pub fn recommended() -> Self
  {
  Self
  {
      temperature : Some( 0.7 ),        // Good balance for most text generation
      max_new_tokens : Some( 512 ),     // Reasonable response length
      top_p : Some( 0.9 ),              // Nucleus sampling for quality
      top_k : None,                     // Let model decide diverse sampling
      repetition_penalty : Some( 1.1 ), // Mild repetition avoidance
      return_full_text : Some( false ),  // Only return generated text
      stop : None,                      // No specific stop sequences
      stream : Some( false ),           // Non-streaming by default
      additional : HashMap::new(),
  }
  }

  /// Create new inference parameters (convenience wrapper)
  ///
  /// # Compatibility
  ///
  /// This method provides backward compatibility by delegating to `recommended()`.
  /// For explicit control, use `recommended()`, `empty()`, or `conservative()`.
  #[ inline ]
  #[ must_use ]
  pub fn new() -> Self
  {
  Self::recommended()
  }

  /// Create empty inference parameters requiring explicit configuration
  ///
  /// # Governing Principle Compliance
  ///
  /// This requires explicit configuration for all parameters, providing full transparency
  /// and control over inference behavior.
  #[ inline ]
  #[ must_use ]
  pub fn empty() -> Self
  {
  Self
  {
      temperature : None,
      max_new_tokens : None,
      top_p : None,
      top_k : None,
      repetition_penalty : None,
      return_full_text : None,
      stop : None,
      stream : None,
      additional : HashMap::new(),
  }
  }

  /// Create conservative parameters for production use
  #[ inline ]
  #[ must_use ]
  pub fn conservative() -> Self
  {
  Self
  {
      temperature : Some( 0.3 ),        // Lower temperature for more predictable output
      max_new_tokens : Some( 256 ),     // Shorter responses for efficiency
      top_p : Some( 0.8 ),              // More focused sampling
      top_k : Some( 50 ),               // Limited vocabulary for consistency
      repetition_penalty : Some( 1.2 ), // Stronger repetition avoidance
      return_full_text : Some( false ),
      stop : None,
      stream : Some( false ),
      additional : HashMap::new(),
  }
  }
  
  /// Set temperature
  #[ inline ]
  #[ must_use ]
  pub fn with_temperature( mut self, temperature : f32 ) -> Self
  {
  self.temperature = Some( temperature );
  self
  }
  
  /// Set max new tokens
  #[ inline ]
  #[ must_use ]
  pub fn with_max_new_tokens( mut self, max_tokens : u32 ) -> Self
  {
  self.max_new_tokens = Some( max_tokens );
  self
  }
  
  /// Set top-p
  #[ inline ]
  #[ must_use ]
  pub fn with_top_p( mut self, top_p : f32 ) -> Self
  {
  self.top_p = Some( top_p );
  self
  }
  
  /// Set top-k
  #[ inline ]
  #[ must_use ]
  pub fn with_top_k( mut self, top_k : u32 ) -> Self
  {
  self.top_k = Some( top_k );
  self
  }
  
  /// Set repetition penalty
  #[ inline ]
  #[ must_use ]
  pub fn with_repetition_penalty( mut self, penalty : f32 ) -> Self
  {
  self.repetition_penalty = Some( penalty );
  self
  }
  
  /// Set return full text flag
  #[ inline ]
  #[ must_use ]
  pub fn with_return_full_text( mut self, return_full : bool ) -> Self
  {
  self.return_full_text = Some( return_full );
  self
  }
  
  /// Enable streaming
  #[ inline ]
  #[ must_use ]
  pub fn with_streaming( mut self, stream : bool ) -> Self
  {
  self.stream = Some( stream );
  self
  }
  
  /// Set stop sequences
  #[ inline ]
  #[ must_use ]
  pub fn with_stop_sequences( mut self, stop : Vec< String > ) -> Self
  {
  self.stop = Some( stop );
  self
  }

  /// Validate all parameters
  ///
  /// # Errors
  /// Returns validation error if any parameters are invalid
  #[ inline ]
  pub fn validate( &self ) -> Result< () >
  {
  let mut errors = Vec::new();

  // Validate temperature
  if let Some( temperature ) = self.temperature
  {
      if let Err( e ) = validate_temperature( temperature )
      {
  errors.push( e.to_string() );
      }
  }

  // Validate max_new_tokens
  if let Some( max_tokens ) = self.max_new_tokens
  {
      if let Err( e ) = validate_max_new_tokens( max_tokens )
      {
  errors.push( e.to_string() );
      }
  }

  // Validate top_p
  if let Some( top_p ) = self.top_p
  {
      if let Err( e ) = validate_top_p( top_p )
      {
  errors.push( e.to_string() );
      }
  }

  // Validate repetition_penalty
  if let Some( penalty ) = self.repetition_penalty
  {
      if let Err( e ) = validate_repetition_penalty( penalty )
      {
  errors.push( e.to_string() );
      }
  }

  // Validate stop sequences
  if let Some( ref stop_sequences ) = self.stop
  {
      if let Err( e ) = validate_stop_sequences( stop_sequences )
      {
  errors.push( e.to_string() );
      }
  }

  // If there are errors, combine them into a single validation error
  if !errors.is_empty()
  {
      return Err( HuggingFaceError::Validation(
  format!( "Parameter validation failed : {}", errors.join( "; " ) )
      ) );
  }

  Ok( () )
  }
}
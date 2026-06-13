//! Input validation utilities for API requests
//!
//! This module provides validation functions to ensure API requests are well-formed
//! and contain valid data before sending them to the Gemini API.

use crate::models::*;

/// Validation error types for input validation.
#[ derive( Debug, Clone ) ]
pub enum ValidationError
{
  /// Required field is missing or empty.
  RequiredFieldMissing {
    /// The name of the field that is missing
    field : String,
    /// Context where the validation occurred
    context : String
  },

  /// Field value is invalid.
  InvalidFieldValue {
    /// The name of the field with invalid value
    field : String,
    /// The invalid value that was provided
    value : String,
    /// Reason why the value is invalid
    reason : String
  },

  /// Field value is out of acceptable range.
  ValueOutOfRange {
    /// The name of the field with out-of-range value
    field : String,
    /// The value that is out of range
    value : f64,
    /// Minimum allowed value (if any)
    min : Option< f64 >,
    /// Maximum allowed value (if any)
    max : Option< f64 >
  },

  /// Collection is empty when it should contain items.
  EmptyCollection {
    /// The name of the collection field
    field : String,
    /// Context where the validation occurred
    context : String
  },

  /// Collection exceeds maximum allowed size.
  CollectionTooLarge {
    /// The name of the collection field
    field : String,
    /// Current size of the collection
    size : usize,
    /// Maximum allowed size
    max : usize
  },
}

impl core::fmt::Display for ValidationError
{
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    match self
    {
      ValidationError::RequiredFieldMissing { field, context } =>
        write!( f, "Required field '{field}' is missing or empty in {context}" ),

      ValidationError::InvalidFieldValue { field, value, reason } =>
        write!( f, "Invalid value '{value}' for field '{field}': {reason}" ),

      ValidationError::ValueOutOfRange { field, value, min, max } =>
      {
        let range_str = match ( min, max )
        {
          ( Some( min ), Some( max ) ) => format!( "between {min} and {max}" ),
          ( Some( min ), None ) => format!( "at least {min}" ),
          ( None, Some( max ) ) => format!( "at most {max}" ),
          ( None, None ) => "within valid range".to_string(),
        };
        write!( f, "Value {value} for field '{field}' is not {range_str}" )
      }

      ValidationError::EmptyCollection { field, context } =>
        write!( f, "Collection '{field}' cannot be empty in {context}" ),

      ValidationError::CollectionTooLarge { field, size, max } =>
        write!( f, "Collection '{field}' has {size} items, exceeding maximum of {max}" ),
    }
  }
}

impl core::error::Error for ValidationError
{
}

/// Maximum number of requests in a batch token counting operation.
const MAX_BATCH_TOKEN_REQUESTS: usize = 100;

/// Maximum number of allowed function names in function calling config.
const MAX_ALLOWED_FUNCTION_NAMES: usize = 100;

/// Maximum number of tuning examples in a training dataset.
const MAX_TUNING_EXAMPLES: usize = 10000;

/// Maximum timeout for code execution in seconds.
const MAX_CODE_EXECUTION_TIMEOUT: i32 = 300;

/// Validate a model name.
///
/// # Arguments
///
/// * `model_name` - The model name to validate
///
/// # Returns
///
/// Returns `Ok(())` if the model name is valid, or a validation error.
fn validate_model_name( model_name : &str ) -> Result< (), ValidationError >
{
  if model_name.trim().is_empty()
  {
    return Err( ValidationError::RequiredFieldMissing {
      field : "model_name".to_string(),
      context : "model validation".to_string(),
    } );
  }

  // Check for obviously invalid characters
  if model_name.contains( '\n' ) || model_name.contains( '\r' ) || model_name.contains( '\0' )
  {
    return Err( ValidationError::InvalidFieldValue {
      field : "model_name".to_string(),
      value : model_name.to_string(),
      reason : "Model name contains invalid characters (newlines or null bytes)".to_string(),
    } );
  }

  Ok( () )
}

/// Validate content structure.
///
/// # Arguments
///
/// * `content` - The content to validate
///
/// # Returns
///
/// Returns `Ok(())` if the content is valid, or a validation error.
fn validate_content( content : &Content ) -> Result< (), ValidationError >
{
  if content.parts.is_empty()
  {
    return Err( ValidationError::EmptyCollection {
      field : "parts".to_string(),
      context : "Content".to_string(),
    } );
  }

  // Validate each part
  for ( i, part ) in content.parts.iter().enumerate()
  {
    validate_part( part )
      .map_err( |e| ValidationError::InvalidFieldValue {
        field : format!( "parts[{}]", i ),
        value : "Part".to_string(),
        reason : e.to_string(),
      } )?;
  }

  Ok( () )
}

/// Validate a content part.
///
/// # Arguments
///
/// * `part` - The part to validate
///
/// # Returns
///
/// Returns `Ok(())` if the part is valid, or a validation error.
fn validate_part( part : &Part ) -> Result< (), ValidationError >
{
  let has_text = part.text.as_ref().is_some_and( |t| !t.trim().is_empty() );
  let has_inline_data = part.inline_data.is_some();
  let has_function_call = part.function_call.is_some();
  let has_function_response = part.function_response.is_some();

  let content_count = [ has_text, has_inline_data, has_function_call, has_function_response ]
    .iter()
    .filter( |&&x| x )
    .count();

  if content_count == 0
  {
    return Err( ValidationError::RequiredFieldMissing {
      field : "content".to_string(),
      context : "Part must have at least one content type (text, inline_data, function_call, or function_response)".to_string(),
    } );
  }

  Ok( () )
}

// Module declarations
mod tokens;
mod config;
mod tuning;
mod content;

// Re-export all public functions
pub use tokens::*;
pub use config::*;
pub use tuning::*;
pub use content::*;

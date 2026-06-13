//! Token and model discovery validation functions

use super::*;

/// Validate a batch count tokens request.
///
/// # Arguments
///
/// * `request` - The batch count tokens request to validate
///
/// # Returns
///
/// Returns `Ok(())` if the request is valid, or a validation error.
///
/// # Errors
///
/// Returns `ValidationError` if the request is invalid, such as:
/// - Empty requests collection
/// - Individual request validation failures
#[ inline ]
pub fn validate_batch_count_tokens_request( request : &BatchCountTokensRequest ) -> Result< (), ValidationError >
{
  if request.requests.is_empty()
  {
    return Err( ValidationError::EmptyCollection {
      field : "requests".to_string(),
      context : "BatchCountTokensRequest".to_string(),
    } );
  }

  if request.requests.len() > MAX_BATCH_TOKEN_REQUESTS
  {
    return Err( ValidationError::CollectionTooLarge {
      field : "requests".to_string(),
      size : request.requests.len(),
      max : MAX_BATCH_TOKEN_REQUESTS,
    } );
  }

  // Validate each individual request
  for ( i, count_request ) in request.requests.iter().enumerate()
  {
    validate_count_tokens_request( count_request )
      .map_err( |e| ValidationError::InvalidFieldValue {
        field : format!( "requests[{i}]" ),
        value : "CountTokensRequest".to_string(),
        reason : e.to_string(),
      } )?;
  }

  Ok( () )
}

/// Validate a count tokens request.
///
/// # Arguments
///
/// * `request` - The count tokens request to validate
///
/// # Returns
///
/// Returns `Ok(())` if the request is valid, or a validation error.
///
/// # Errors
///
/// Returns `ValidationError` if the request is invalid, such as:
/// - Empty contents collection
/// - Individual content validation failures
#[ inline ]
pub fn validate_count_tokens_request( request : &CountTokensRequest ) -> Result< (), ValidationError >
{
  if request.contents.is_empty()
  {
    return Err( ValidationError::EmptyCollection {
      field : "contents".to_string(),
      context : "CountTokensRequest".to_string(),
    } );
  }

  // Validate each content item
  for ( i, content ) in request.contents.iter().enumerate()
  {
    validate_content( content )
      .map_err( |e| ValidationError::InvalidFieldValue {
        field : format!( "contents[{i}]" ),
        value : "Content".to_string(),
        reason : e.to_string(),
      } )?;
  }

  Ok( () )
}

/// Validate an analyze tokens request.
///
/// # Arguments
///
/// * `request` - The analyze tokens request to validate
///
/// # Returns
///
/// Returns `Ok(())` if the request is valid, or a validation error.
pub fn validate_analyze_tokens_request( request : &AnalyzeTokensRequest ) -> Result< (), ValidationError >
{
  if request.contents.is_empty()
  {
    return Err( ValidationError::EmptyCollection {
      field : "contents".to_string(),
      context : "AnalyzeTokensRequest".to_string(),
    } );
  }

  // Validate each content item
  for ( i, content ) in request.contents.iter().enumerate()
  {
    validate_content( content )
      .map_err( |e| ValidationError::InvalidFieldValue {
        field : format!( "contents[{i}]" ),
        value : "Content".to_string(),
        reason : e.to_string(),
      } )?;
  }

  Ok( () )
}

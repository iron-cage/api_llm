//! Request validation functionality for `HuggingFace` API

mod private
{
use crate::error::{ HuggingFaceError, Result };

/// Maximum allowed input text length (characters)
pub const MAX_INPUT_LENGTH : usize = 50000;

/// Maximum allowed batch size for batch operations
pub const MAX_BATCH_SIZE : usize = 1000;

/// Maximum allowed number of stop sequences
pub const MAX_STOP_SEQUENCES : usize = 10;

/// Maximum allowed model identifier length
pub const MAX_MODEL_ID_LENGTH : usize = 200;

/// Maximum allowed tokens to generate
pub const MAX_NEW_TOKENS : u32 = 8192;

/// Validate input text for API requests
///
/// # Arguments
/// - `input`: The input text to validate
///
/// # Errors
/// Returns validation error if:
/// - Input is empty
/// - Input exceeds maximum length
/// - Input contains invalid characters
#[ inline ]
pub fn validate_input_text( input : &str ) -> Result< () >
{
  if input.is_empty()
  {
  return Err( HuggingFaceError::Validation(
      "Input text cannot be empty".to_string()
  ) );
  }

  if input.len() > MAX_INPUT_LENGTH
  {
  return Err( HuggingFaceError::Validation(
      format!(
  "Input text is too long ({} characters). Maximum allowed : {} characters",
  input.len(),
  MAX_INPUT_LENGTH
      )
  ) );
  }

  // Reject control characters that are not common whitespace.
  // '\n', '\r', '\t' are allowed; other control chars (including ASCII NUL, BEL, etc.) are not.
  if !input.chars().all( | c | !c.is_control() || c == '\n' || c == '\r' || c == '\t' )
  {
  return Err( HuggingFaceError::Validation(
      "Input text contains invalid control characters".to_string()
  ) );
  }

  Ok( () )
}

/// Validate model identifier
///
/// # Arguments
/// - `model_id`: The model identifier to validate
///
/// # Errors
/// Returns validation error if model identifier is invalid
#[ inline ]
pub fn validate_model_identifier( model_id : &str ) -> Result< () >
{
  if model_id.is_empty()
  {
  return Err( HuggingFaceError::Validation(
      "Model identifier cannot be empty".to_string()
  ) );
  }

  if model_id.trim() != model_id
  {
  return Err( HuggingFaceError::Validation(
      "Model identifier cannot have leading or trailing whitespace".to_string()
  ) );
  }

  if model_id.len() > MAX_MODEL_ID_LENGTH
  {
  return Err( HuggingFaceError::Validation(
      format!(
  "Model identifier is too long ({} characters). Maximum allowed : {} characters",
  model_id.len(),
  MAX_MODEL_ID_LENGTH
      )
  ) );
  }

  // Check for invalid characters
  if model_id.contains( '\n' ) || model_id.contains( '\r' ) || model_id.contains( '\t' )
  {
  return Err( HuggingFaceError::Validation(
      "Model identifier cannot contain newlines, carriage returns, or tabs".to_string()
  ) );
  }

  // Model IDs shouldn't have double slashes or leading/trailing slashes
  if model_id.starts_with( '/' ) || model_id.ends_with( '/' ) || model_id.contains( "//" )
  {
  return Err( HuggingFaceError::Validation(
      "Model identifier cannot start/end with slash or contain double slashes".to_string()
  ) );
  }

  // Check for spaces in model identifier (HuggingFace uses hyphens and slashes)
  if model_id.contains( ' ' )
  {
  return Err( HuggingFaceError::Validation(
      "Model identifier cannot contain spaces".to_string()
  ) );
  }

  Ok( () )
}

/// Validate batch inputs
///
/// # Arguments
/// - `inputs`: The batch of input texts to validate
///
/// # Errors
/// Returns validation error if batch is invalid
#[ inline ]
pub fn validate_batch_inputs( inputs : &[ String ] ) -> Result< () >
{
  if inputs.is_empty()
  {
  return Err( HuggingFaceError::Validation(
      "Batch inputs cannot be empty".to_string()
  ) );
  }

  if inputs.len() > MAX_BATCH_SIZE
  {
  return Err( HuggingFaceError::Validation(
      format!(
  "Too many batch inputs ({}). Maximum allowed : {}",
  inputs.len(),
  MAX_BATCH_SIZE
      )
  ) );
  }

  // Validate each individual input
  for ( index, input ) in inputs.iter().enumerate()
  {
  if let Err( e ) = validate_input_text( input )
  {
      return Err( HuggingFaceError::Validation(
  format!( "Invalid input at index {index}: {e}" )
      ) );
  }
  }

  Ok( () )
}

/// Validate temperature parameter
///
/// # Arguments
/// - `temperature`: Temperature value to validate
///
/// # Errors
/// Returns validation error if temperature is out of valid range
#[ inline ]
pub fn validate_temperature( temperature : f32 ) -> Result< () >
{
  if !( 0.0..=2.0 ).contains( &temperature )
  {
  return Err( HuggingFaceError::Validation(
      format!( "Temperature must be between 0.0 and 2.0, got : {temperature}" )
  ) );
  }
  
  if temperature.is_nan() || temperature.is_infinite()
  {
  return Err( HuggingFaceError::Validation(
      format!( "Temperature must be a valid number, got : {temperature}" )
  ) );
  }

  Ok( () )
}

/// Validate `max_new_tokens` parameter
///
/// # Arguments  
/// - `max_tokens`: Maximum tokens value to validate
///
/// # Errors
/// Returns validation error if `max_tokens` is invalid
#[ inline ]
pub fn validate_max_new_tokens( max_tokens : u32 ) -> Result< () >
{
  if max_tokens == 0
  {
  return Err( HuggingFaceError::Validation(
      "max_new_tokens must be greater than 0".to_string()
  ) );
  }

  if max_tokens > MAX_NEW_TOKENS
  {
  return Err( HuggingFaceError::Validation(
      format!( "max_new_tokens is too large ({max_tokens}). Maximum allowed : {MAX_NEW_TOKENS}" )
  ) );
  }

  Ok( () )
}

/// Validate `top_p` parameter
///
/// # Arguments
/// - `top_p`: Top-p value to validate
///
/// # Errors
/// Returns validation error if `top_p` is out of valid range
#[ inline ]
pub fn validate_top_p( top_p : f32 ) -> Result< () >
{
  if !( 0.0..=1.0 ).contains( &top_p )
  {
  return Err( HuggingFaceError::Validation(
      format!( "top_p must be between 0.0 and 1.0, got : {top_p}" )
  ) );
  }

  if top_p.is_nan() || top_p.is_infinite()
  {
  return Err( HuggingFaceError::Validation(
      format!( "top_p must be a valid number, got : {top_p}" )
  ) );
  }

  Ok( () )
}

/// Validate `repetition_penalty` parameter
///
/// # Arguments
/// - `penalty`: Repetition penalty value to validate
///
/// # Errors
/// Returns validation error if penalty is invalid
#[ inline ]
pub fn validate_repetition_penalty( penalty : f32 ) -> Result< () >
{
  if penalty <= 0.0
  {
  return Err( HuggingFaceError::Validation(
      format!( "repetition_penalty must be positive, got : {penalty}" )
  ) );
  }

  if penalty > 10.0
  {
  return Err( HuggingFaceError::Validation(
      format!( "repetition_penalty is too high ({penalty}). Maximum recommended : 10.0" )
  ) );
  }

  if penalty.is_nan() || penalty.is_infinite()
  {
  return Err( HuggingFaceError::Validation(
      format!( "repetition_penalty must be a valid number, got : {penalty}" )
  ) );
  }

  Ok( () )
}

/// Validate stop sequences
///
/// # Arguments
/// - `stop_sequences`: Stop sequences to validate
///
/// # Errors
/// Returns validation error if stop sequences are invalid
#[ inline ]
pub fn validate_stop_sequences( stop_sequences : &[ String ] ) -> Result< () >
{
  if stop_sequences.len() > MAX_STOP_SEQUENCES
  {
  return Err( HuggingFaceError::Validation(
      format!(
  "Too many stop sequences ({}). Maximum allowed : {}",
  stop_sequences.len(),
  MAX_STOP_SEQUENCES
      )
  ) );
  }

  for ( index, stop ) in stop_sequences.iter().enumerate()
  {
  if stop.is_empty()
  {
      return Err( HuggingFaceError::Validation(
  format!( "Stop sequence at index {index} cannot be empty" )
      ) );
  }

  if stop.len() > 100
  {
      return Err( HuggingFaceError::Validation(
  format!(
          "Stop sequence at index {} is too long ({}). Maximum : 100 characters",
          index,
          stop.len()
  )
      ) );
  }
  }

  Ok( () )
}

/// Validate `top_k` parameter
///
/// # Arguments
/// - `top_k`: Top-k value to validate
///
/// # Errors
/// Returns validation error if `top_k` is invalid
#[ inline ]
pub fn validate_top_k( top_k : u32 ) -> Result< () >
{
  if top_k == 0
  {
  return Err( HuggingFaceError::Validation(
      "top_k must be greater than 0".to_string()
  ) );
  }

  if top_k > 1000
  {
  return Err( HuggingFaceError::Validation(
      format!( "top_k is too large ({top_k}). Maximum recommended : 1000" )
  ) );
  }

  Ok( () )
}

/// Validate frequency penalty parameter
///
/// # Arguments
/// - `penalty`: Frequency penalty value to validate
///
/// # Errors
/// Returns validation error if penalty is out of valid range
#[ inline ]
pub fn validate_frequency_penalty( penalty : f32 ) -> Result< () >
{
  if !( -2.0..=2.0 ).contains( &penalty )
  {
  return Err( HuggingFaceError::Validation(
      format!( "frequency_penalty must be between -2.0 and 2.0, got : {penalty}" )
  ) );
  }

  if penalty.is_nan() || penalty.is_infinite()
  {
  return Err( HuggingFaceError::Validation(
      format!( "frequency_penalty must be a valid number, got : {penalty}" )
  ) );
  }

  Ok( () )
}

/// Validate presence penalty parameter
///
/// # Arguments
/// - `penalty`: Presence penalty value to validate
///
/// # Errors
/// Returns validation error if penalty is out of valid range
#[ inline ]
pub fn validate_presence_penalty( penalty : f32 ) -> Result< () >
{
  if !( -2.0..=2.0 ).contains( &penalty )
  {
  return Err( HuggingFaceError::Validation(
      format!( "presence_penalty must be between -2.0 and 2.0, got : {penalty}" )
  ) );
  }

  if penalty.is_nan() || penalty.is_infinite()
  {
  return Err( HuggingFaceError::Validation(
      format!( "presence_penalty must be a valid number, got : {penalty}" )
  ) );
  }

  Ok( () )
}

/// Validate message role
///
/// # Arguments
/// - `role`: Message role to validate
///
/// # Errors
/// Returns validation error if role is invalid
#[ inline ]
pub fn validate_message_role( role : &str ) -> Result< () >
{
  match role
  {
  "system" | "user" | "assistant" | "tool" | "function" => Ok( () ),
  _ => Err( HuggingFaceError::Validation(
      format!( "Invalid message role : {role}. Must be one of : system, user, assistant, tool, function" )
  ) ),
  }
}

/// Validate message content
///
/// # Arguments
/// - `content`: Message content to validate
///
/// # Errors
/// Returns validation error if content is invalid
#[ inline ]
pub fn validate_message_content( content : &str ) -> Result< () >
{
  if content.is_empty()
  {
  return Err( HuggingFaceError::Validation(
      "Message content cannot be empty".to_string()
  ) );
  }

  if content.len() > MAX_INPUT_LENGTH
  {
  return Err( HuggingFaceError::Validation(
      format!(
  "Message content is too long ({} characters). Maximum allowed : {} characters",
  content.len(),
  MAX_INPUT_LENGTH
      )
  ) );
  }

  Ok( () )
}

/// Validate tool choice parameter
///
/// # Arguments
/// - `tool_choice`: Tool choice value to validate
///
/// # Errors
/// Returns validation error if tool choice is invalid
#[ inline ]
pub fn validate_tool_choice( tool_choice : &str ) -> Result< () >
{
  match tool_choice
  {
  "auto" | "none" | "required" => Ok( () ),
  _ =>
  {
      // Also accept specific tool names (not just the predefined values)
      if tool_choice.is_empty()
      {
  Err( HuggingFaceError::Validation(
          "tool_choice cannot be empty".to_string()
  ) )
      }
      else
      {
  Ok( () )
      }
  }
  }
}

/// Maximum allowed image size in bytes (10 MB)
pub const MAX_IMAGE_SIZE_BYTES : usize = 10 * 1024 * 1024;

/// Maximum allowed audio size in bytes (25 MB)
pub const MAX_AUDIO_SIZE_BYTES : usize = 25 * 1024 * 1024;

/// Validate image data size
///
/// # Arguments
/// - `data`: Image data bytes
///
/// # Errors
/// Returns validation error if image is too large
#[ inline ]
pub fn validate_image_size( data : &[ u8 ] ) -> Result< () >
{
  if data.is_empty()
  {
  return Err( HuggingFaceError::Validation(
      "Image data cannot be empty".to_string()
  ) );
  }

  if data.len() > MAX_IMAGE_SIZE_BYTES
  {
  return Err( HuggingFaceError::Validation(
      format!(
  "Image data is too large ({} bytes). Maximum allowed : {} bytes",
  data.len(),
  MAX_IMAGE_SIZE_BYTES
      )
  ) );
  }

  Ok( () )
}

/// Validate audio data size
///
/// # Arguments
/// - `data`: Audio data bytes
///
/// # Errors
/// Returns validation error if audio is too large
#[ inline ]
pub fn validate_audio_size( data : &[ u8 ] ) -> Result< () >
{
  if data.is_empty()
  {
  return Err( HuggingFaceError::Validation(
      "Audio data cannot be empty".to_string()
  ) );
  }

  if data.len() > MAX_AUDIO_SIZE_BYTES
  {
  return Err( HuggingFaceError::Validation(
      format!(
  "Audio data is too large ({} bytes). Maximum allowed : {} bytes",
  data.len(),
  MAX_AUDIO_SIZE_BYTES
      )
  ) );
  }

  Ok( () )
}

/// Validate URL format
///
/// # Arguments
/// - `url`: URL string to validate
///
/// # Errors
/// Returns validation error if URL is invalid
#[ inline ]
pub fn validate_url( url : &str ) -> Result< () >
{
  if url.is_empty()
  {
  return Err( HuggingFaceError::Validation(
      "URL cannot be empty".to_string()
  ) );
  }

  if !url.starts_with( "http://" ) && !url.starts_with( "https://" )
  {
  return Err( HuggingFaceError::Validation(
      format!( "URL must start with http:// or https://, got : {url}" )
  ) );
  }

  if url.len() > 2048
  {
  return Err( HuggingFaceError::Validation(
      format!( "URL is too long ({} characters). Maximum allowed : 2048 characters", url.len() )
  ) );
  }

  Ok( () )
}
} // end mod private

crate::mod_interface!
{
  exposed use private::
  {
  MAX_INPUT_LENGTH,
  MAX_BATCH_SIZE,
  MAX_STOP_SEQUENCES,
  MAX_MODEL_ID_LENGTH,
  MAX_NEW_TOKENS,
  MAX_IMAGE_SIZE_BYTES,
  MAX_AUDIO_SIZE_BYTES,
  validate_input_text,
  validate_model_identifier,
  validate_batch_inputs,
  validate_temperature,
  validate_max_new_tokens,
  validate_top_p,
  validate_repetition_penalty,
  validate_stop_sequences,
  validate_top_k,
  validate_frequency_penalty,
  validate_presence_penalty,
  validate_message_role,
  validate_message_content,
  validate_tool_choice,
  validate_image_size,
  validate_audio_size,
  validate_url,
  };
}
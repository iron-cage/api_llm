//! Tests for request validation functionality

use api_huggingface::
{
  components::input::InferenceParameters,
  error::HuggingFaceError,
};

/// Test `InferenceParameters` validation ranges
#[ test ]
fn test_inference_parameters_temperature_validation()
{
  // Valid temperature ranges
  let valid_params = InferenceParameters::new()
  .with_temperature( 0.1 )
  .validate();
  assert!( valid_params.is_ok() );

  let valid_params = InferenceParameters::new()
  .with_temperature( 1.0 )
  .validate();
  assert!( valid_params.is_ok() );

  let valid_params = InferenceParameters::new()
  .with_temperature( 2.0 )
  .validate();
  assert!( valid_params.is_ok() );

  // Invalid temperature ranges
  let invalid_params = InferenceParameters::new()
  .with_temperature( -0.1 )
  .validate();
  assert!( invalid_params.is_err() );
  if let Err( HuggingFaceError::Validation( msg ) ) = invalid_params
  {
  assert!( msg.to_lowercase().contains( "temperature" ) );
  assert!( msg.contains( "0.0" ) );
  assert!( msg.contains( "2.0" ) );
  }
  else
  {
  panic!( "Expected validation error for negative temperature" );
  }

  let invalid_params = InferenceParameters::new()
  .with_temperature( 2.1 )
  .validate();
  assert!( invalid_params.is_err() );
  if let Err( HuggingFaceError::Validation( msg ) ) = invalid_params
  {
  assert!( msg.to_lowercase().contains( "temperature" ) );
  }
  else
  {
  panic!( "Expected validation error for high temperature" );
  }
}

/// Test `max_new_tokens` validation
#[ test ]
fn test_inference_parameters_max_tokens_validation()
{
  // Valid max_new_tokens
  let valid_params = InferenceParameters::new()
  .with_max_new_tokens( 1 )
  .validate();
  assert!( valid_params.is_ok() );

  let valid_params = InferenceParameters::new()
  .with_max_new_tokens( 4096 )
  .validate();
  assert!( valid_params.is_ok() );

  // Invalid max_new_tokens (zero)
  let invalid_params = InferenceParameters::new()
  .with_max_new_tokens( 0 )
  .validate();
  assert!( invalid_params.is_err() );
  if let Err( HuggingFaceError::Validation( msg ) ) = invalid_params
  {
  assert!( msg.contains( "max_new_tokens" ) );
  assert!( msg.contains( "greater than 0" ) );
  }
  else
  {
  panic!( "Expected validation error for zero max_new_tokens" );
  }

  // Invalid max_new_tokens (too large)
  let invalid_params = InferenceParameters::new()
  .with_max_new_tokens( 10000 )
  .validate();
  assert!( invalid_params.is_err() );
  if let Err( HuggingFaceError::Validation( msg ) ) = invalid_params
  {
  assert!( msg.contains( "max_new_tokens" ) );
  assert!( msg.contains( "8192" ) );
  }
  else
  {
  panic!( "Expected validation error for excessive max_new_tokens" );
  }
}

/// Test `top_p` validation
#[ test ]
fn test_inference_parameters_top_p_validation()
{
  // Valid top_p values
  let valid_params = InferenceParameters::new()
  .with_top_p( 0.0 )
  .validate();
  assert!( valid_params.is_ok() );

  let valid_params = InferenceParameters::new()
  .with_top_p( 1.0 )
  .validate();
  assert!( valid_params.is_ok() );

  // Invalid top_p values
  let invalid_params = InferenceParameters::new()
  .with_top_p( -0.1 )
  .validate();
  assert!( invalid_params.is_err() );

  let invalid_params = InferenceParameters::new()
  .with_top_p( 1.1 )
  .validate();
  assert!( invalid_params.is_err() );
  if let Err( HuggingFaceError::Validation( msg ) ) = invalid_params
  {
  assert!( msg.to_lowercase().contains( "top_p" ) );
  assert!( msg.contains( "0.0" ) );
  assert!( msg.contains( "1.0" ) );
  }
  else
  {
  panic!( "Expected validation error for invalid top_p" );
  }
}

/// Test input text validation
#[ test ]
fn test_input_text_validation()
{
  use api_huggingface::validation::validate_input_text;

  // Valid input text
  let valid_text = "Hello, world!";
  assert!( validate_input_text( valid_text ).is_ok() );

  let medium_text = "a".repeat( 1000 );
  assert!( validate_input_text( &medium_text ).is_ok() );

  // Empty input should be invalid
  let empty_text = "";
  let result = validate_input_text( empty_text );
  assert!( result.is_err() );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "empty" ) );
  }
  else
  {
  panic!( "Expected validation error for empty input" );
  }

  // Extremely long input should be invalid
  let long_text = "a".repeat( 100_000 );
  let result = validate_input_text( &long_text );
  assert!( result.is_err() );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "too long" ) );
  assert!( msg.contains( "50000" ) );
  }
  else
  {
  panic!( "Expected validation error for excessively long input" );
  }

  // Non-UTF8 sequences should be handled gracefully
  // (This test ensures we handle encoding properly)
  let unicode_text = "Hello 🌍! 你好世界 مرحبا بالعالم";
  assert!( validate_input_text( unicode_text ).is_ok() );
}

/// Test model identifier validation
#[ test ]
fn test_model_identifier_validation()
{
  use api_huggingface::validation::validate_model_identifier;

  // Valid model identifiers
  assert!( validate_model_identifier( "gpt2" ).is_ok() );
  assert!( validate_model_identifier( "meta-llama/Llama-2-7b-hf" ).is_ok() );
  assert!( validate_model_identifier( "microsoft/DialoGPT-medium" ).is_ok() );
  assert!( validate_model_identifier( "sentence-transformers/all-MiniLM-L6-v2" ).is_ok() );

  // Invalid model identifiers
  let long_model = "a".repeat( 300 );
  let invalid_models = vec![
  "",                           // empty
  " ",                          // whitespace only
  "model with spaces",          // spaces in name
  "model\nwith\nnewlines",     // newlines
  "/leading-slash",             // leading slash
  "trailing-slash/",            // trailing slash
  "double//slash",              // double slash
  &long_model,                  // too long
  ];

  for invalid_model in invalid_models
  {
  let result = validate_model_identifier( invalid_model );
  assert!( result.is_err(), "Model '{invalid_model}' should be invalid" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
      assert!( msg.to_lowercase().contains( "model" ) );
  }
  else
  {
      panic!( "Expected validation error for model '{invalid_model}'" );
  }
  }
}

/// Test batch input validation
#[ test ]
fn test_batch_input_validation()
{
  use api_huggingface::validation::validate_batch_inputs;

  // Valid batch inputs
  let valid_batch = vec![ "Hello".to_string(), "World".to_string() ];
  assert!( validate_batch_inputs( &valid_batch ).is_ok() );

  // Empty batch should be invalid
  let empty_batch : Vec< String > = vec![];
  let result = validate_batch_inputs( &empty_batch );
  assert!( result.is_err() );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "empty" ) );
  }
  else
  {
  panic!( "Expected validation error for empty batch" );
  }

  // Too many inputs should be invalid
  let large_batch : Vec< String > = ( 0..1001 ).map( | i | format!( "input_{i}" ) ).collect();
  let result = validate_batch_inputs( &large_batch );
  assert!( result.is_err() );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.to_lowercase().contains( "too many" ) );
  assert!( msg.contains( "1000" ) );
  }
  else
  {
  panic!( "Expected validation error for excessive batch size" );
  }

  // Batch with invalid individual inputs should fail
  let invalid_batch = vec![ "Valid input".to_string(), String::new() ];
  let result = validate_batch_inputs( &invalid_batch );
  assert!( result.is_err() );
}

/// Test stop sequences validation
#[ test ]
fn test_stop_sequences_validation()
{
  // Valid stop sequences
  let valid_params = InferenceParameters::new()
  .with_stop_sequences( vec![ "\n".to_string(), "END".to_string() ] )
  .validate();
  assert!( valid_params.is_ok() );

  // Too many stop sequences should be invalid
  let many_stops : Vec< String > = ( 0..20 ).map( | i | format!( "stop_{i}" ) ).collect();
  let invalid_params = InferenceParameters::new()
  .with_stop_sequences( many_stops )
  .validate();
  assert!( invalid_params.is_err() );
  if let Err( HuggingFaceError::Validation( msg ) ) = invalid_params
  {
  assert!( msg.contains( "stop" ) );
  assert!( msg.contains( "10" ) );
  }
  else
  {
  panic!( "Expected validation error for too many stop sequences" );
  }

  // Empty stop sequences should be invalid
  let empty_stops = vec![ String::new() ];
  let invalid_params = InferenceParameters::new()
  .with_stop_sequences( empty_stops )
  .validate();
  assert!( invalid_params.is_err() );
}

/// Test comprehensive parameter validation
#[ test ]
fn test_multiple_validation_errors()
{
  // Test that multiple validation errors are reported
  let invalid_params = InferenceParameters::new()
  .with_temperature( -1.0 )           // Invalid temperature
  .with_max_new_tokens( 0 )           // Invalid max tokens
  .with_top_p( 2.0 )                  // Invalid top_p
  .validate();
  
  assert!( invalid_params.is_err() );
  if let Err( HuggingFaceError::Validation( msg ) ) = invalid_params
  {
  // Should contain multiple error messages
  assert!( msg.to_lowercase().contains( "temperature" ) );
  assert!( msg.to_lowercase().contains( "max_new_tokens" ) );
  assert!( msg.to_lowercase().contains( "top_p" ) );
  }
  else
  {
  panic!( "Expected validation error with multiple issues" );
  }
}

/// Test default parameters are valid
#[ test ]
fn test_default_parameters_valid()
{
  let default_params = InferenceParameters::default();
  assert!( default_params.validate().is_ok() );

  let new_params = InferenceParameters::new();
  assert!( new_params.validate().is_ok() );
}

/// Reproducing test for bug: ASCII control characters bypassed `validate_input_text`.
///
/// Root Cause: the guard `!input.is_ascii() &&` caused the control-char scan to be
/// skipped entirely for ASCII-only strings, allowing NUL, BEL, and similar chars through.
/// Fix: removed `!input.is_ascii() &&` so all strings are scanned.
///
/// Why Not Caught: existing tests only verified Unicode and length limits; no test
/// exercised ASCII control characters.
///
/// Pitfall: `&str` in Rust is always valid UTF-8, but that does NOT mean it is free of
/// ASCII control characters.  Validators must inspect character values, not just encoding.
#[ test ]
fn test_validate_input_text_control_chars()
{
  use api_huggingface::validation::validate_input_text;

  // ASCII control characters must be rejected
  let ctrl_nul = "\x00";
  assert!( validate_input_text( ctrl_nul ).is_err(), "NUL byte must be rejected" );

  let ctrl_bel = "\x07";
  assert!( validate_input_text( ctrl_bel ).is_err(), "BEL must be rejected" );

  let ctrl_esc = "\x1B";
  assert!( validate_input_text( ctrl_esc ).is_err(), "ESC must be rejected" );

  // Control char embedded in ASCII text — must also be rejected
  let embedded = "hello\x01world";
  assert!( validate_input_text( embedded ).is_err(), "Embedded ASCII control char must be rejected" );

  // These whitespace control chars ARE explicitly allowed
  assert!( validate_input_text( "line\nbreak" ).is_ok(), "\\n must be accepted" );
  assert!( validate_input_text( "tab\there" ).is_ok(), "\\t must be accepted" );
  assert!( validate_input_text( "cr\rchar" ).is_ok(), "\\r must be accepted" );

  // Normal text still valid
  assert!( validate_input_text( "hello world" ).is_ok() );
  assert!( validate_input_text( "unicode: 你好 🌍" ).is_ok() );
}

/// Boundary-value tests for `validate_input_text` length limit.
#[ test ]
fn test_validate_input_text_boundary_length()
{
  use api_huggingface::validation::{ validate_input_text, MAX_INPUT_LENGTH };

  // Exactly at limit: must pass
  let at_limit = "a".repeat( MAX_INPUT_LENGTH );
  assert!(
  validate_input_text( &at_limit ).is_ok(),
  "Text at exactly MAX_INPUT_LENGTH must be valid"
  );

  // One over the limit: must fail
  let over_limit = "a".repeat( MAX_INPUT_LENGTH + 1 );
  let result = validate_input_text( &over_limit );
  assert!( result.is_err(), "Text exceeding MAX_INPUT_LENGTH must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "too long" ) );
  }
  else
  {
  panic!( "Expected Validation error for text over length limit" );
  }
}

/// Boundary-value tests for `validate_model_identifier` length and tab character.
#[ test ]
fn test_validate_model_identifier_boundary_and_tab()
{
  use api_huggingface::validation::{ validate_model_identifier, MAX_MODEL_ID_LENGTH };

  // Exactly at limit: must pass
  let at_limit = "a".repeat( MAX_MODEL_ID_LENGTH );
  assert!(
  validate_model_identifier( &at_limit ).is_ok(),
  "Model ID at exactly MAX_MODEL_ID_LENGTH must be valid"
  );

  // One over the limit: must fail
  let over_limit = "a".repeat( MAX_MODEL_ID_LENGTH + 1 );
  assert!(
  validate_model_identifier( &over_limit ).is_err(),
  "Model ID exceeding MAX_MODEL_ID_LENGTH must be rejected"
  );

  // Tab character is explicitly rejected
  let with_tab = "org/model\tname";
  assert!(
  validate_model_identifier( with_tab ).is_err(),
  "Tab in model ID must be rejected"
  );

  // Colon is valid (used for provider-specific models, e.g., "model:provider")
  assert!(
  validate_model_identifier( "moonshotai/Kimi-K2-Instruct-0905:groq" ).is_ok(),
  "Colon in model ID must be accepted"
  );

  // Leading/trailing space must be rejected
  assert!( validate_model_identifier( " org/model" ).is_err(), "Leading space must be rejected" );
  assert!( validate_model_identifier( "org/model " ).is_err(), "Trailing space must be rejected" );
}

/// Boundary-value tests for `validate_batch_inputs`.
#[ test ]
fn test_validate_batch_inputs_boundary()
{
  use api_huggingface::validation::{ validate_batch_inputs, MAX_BATCH_SIZE };

  // Exactly at limit: must pass
  let at_limit : Vec< String > = ( 0..MAX_BATCH_SIZE ).map( | i | format!( "text_{i}" ) ).collect();
  assert!(
  validate_batch_inputs( &at_limit ).is_ok(),
  "Batch at exactly MAX_BATCH_SIZE must be valid"
  );

  // One over the limit: must fail
  let over_limit : Vec< String > = ( 0..=MAX_BATCH_SIZE ).map( | i | format!( "text_{i}" ) ).collect();
  let result = validate_batch_inputs( &over_limit );
  assert!( result.is_err(), "Batch exceeding MAX_BATCH_SIZE must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.to_lowercase().contains( "too many" ) );
  }
  else
  {
  panic!( "Expected Validation error for oversized batch" );
  }
}

/// Boundary-value tests for `validate_max_new_tokens`.
#[ test ]
fn test_validate_max_new_tokens_boundary()
{
  use api_huggingface::validation::{ validate_max_new_tokens, MAX_NEW_TOKENS };

  // Exactly at limit: must pass
  assert!(
  validate_max_new_tokens( MAX_NEW_TOKENS ).is_ok(),
  "max_new_tokens at exactly MAX_NEW_TOKENS must be valid"
  );

  // One over: must fail
  let result = validate_max_new_tokens( MAX_NEW_TOKENS + 1 );
  assert!( result.is_err(), "max_new_tokens exceeding MAX_NEW_TOKENS must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "max_new_tokens" ) );
  }
  else
  {
  panic!( "Expected Validation error for excessive max_new_tokens" );
  }
}

/// NaN and Infinity handling in float validators.
#[ test ]
fn test_validate_temperature_special_floats()
{
  use api_huggingface::validation::validate_temperature;

  // NaN and Infinity must be rejected (range check catches them before NaN check)
  assert!( validate_temperature( f32::NAN ).is_err(), "NaN temperature must be rejected" );
  assert!( validate_temperature( f32::INFINITY ).is_err(), "INFINITY temperature must be rejected" );
  assert!( validate_temperature( f32::NEG_INFINITY ).is_err(), "NEG_INFINITY temperature must be rejected" );

  // Exact boundaries must pass
  assert!( validate_temperature( 0.0 ).is_ok(), "temperature 0.0 must be valid" );
  assert!( validate_temperature( 2.0 ).is_ok(), "temperature 2.0 must be valid" );
}

/// NaN and Infinity handling in `top_p` validator.
#[ test ]
fn test_validate_top_p_special_floats()
{
  use api_huggingface::validation::validate_top_p;

  assert!( validate_top_p( f32::NAN ).is_err(), "NaN top_p must be rejected" );
  assert!( validate_top_p( f32::INFINITY ).is_err(), "INFINITY top_p must be rejected" );
  assert!( validate_top_p( f32::NEG_INFINITY ).is_err(), "NEG_INFINITY top_p must be rejected" );

  // Exact boundaries must pass
  assert!( validate_top_p( 0.0 ).is_ok(), "top_p 0.0 must be valid" );
  assert!( validate_top_p( 1.0 ).is_ok(), "top_p 1.0 must be valid" );
}

/// Full coverage for `validate_top_k` (previously completely untested).
#[ test ]
fn test_validate_top_k()
{
  use api_huggingface::validation::validate_top_k;

  // 0 is invalid
  let result = validate_top_k( 0 );
  assert!( result.is_err(), "top_k=0 must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "top_k" ) );
  assert!( msg.contains( "greater than 0" ) );
  }
  else
  {
  panic!( "Expected Validation error for top_k=0" );
  }

  // Valid range
  assert!( validate_top_k( 1 ).is_ok(), "top_k=1 (minimum) must be valid" );
  assert!( validate_top_k( 50 ).is_ok(), "top_k=50 must be valid" );
  assert!( validate_top_k( 1000 ).is_ok(), "top_k=1000 (boundary) must be valid" );

  // Over limit
  let result = validate_top_k( 1001 );
  assert!( result.is_err(), "top_k=1001 must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "top_k" ) );
  assert!( msg.contains( "1000" ) );
  }
  else
  {
  panic!( "Expected Validation error for top_k over 1000" );
  }
}

/// Full coverage for `validate_frequency_penalty` (previously completely untested).
#[ test ]
fn test_validate_frequency_penalty()
{
  use api_huggingface::validation::validate_frequency_penalty;

  // Valid range: -2.0..=2.0
  assert!( validate_frequency_penalty( -2.0 ).is_ok(), "frequency_penalty=-2.0 (boundary) must be valid" );
  assert!( validate_frequency_penalty( 0.0 ).is_ok(), "frequency_penalty=0.0 must be valid" );
  assert!( validate_frequency_penalty( 2.0 ).is_ok(), "frequency_penalty=2.0 (boundary) must be valid" );

  // Out of range
  let result = validate_frequency_penalty( -2.1 );
  assert!( result.is_err(), "frequency_penalty=-2.1 must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.to_lowercase().contains( "frequency_penalty" ) );
  }
  else
  {
  panic!( "Expected Validation error for frequency_penalty below -2.0" );
  }

  let result = validate_frequency_penalty( 2.1 );
  assert!( result.is_err(), "frequency_penalty=2.1 must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.to_lowercase().contains( "frequency_penalty" ) );
  }
  else
  {
  panic!( "Expected Validation error for frequency_penalty above 2.0" );
  }

  // NaN and Infinity
  assert!( validate_frequency_penalty( f32::NAN ).is_err(), "NaN frequency_penalty must be rejected" );
  assert!( validate_frequency_penalty( f32::INFINITY ).is_err(), "INFINITY frequency_penalty must be rejected" );
}

/// Full coverage for `validate_presence_penalty` (previously completely untested).
#[ test ]
fn test_validate_presence_penalty()
{
  use api_huggingface::validation::validate_presence_penalty;

  // Valid range: -2.0..=2.0
  assert!( validate_presence_penalty( -2.0 ).is_ok(), "presence_penalty=-2.0 (boundary) must be valid" );
  assert!( validate_presence_penalty( 0.0 ).is_ok(), "presence_penalty=0.0 must be valid" );
  assert!( validate_presence_penalty( 2.0 ).is_ok(), "presence_penalty=2.0 (boundary) must be valid" );

  // Out of range
  assert!( validate_presence_penalty( -2.1 ).is_err(), "presence_penalty=-2.1 must be rejected" );
  assert!( validate_presence_penalty( 2.1 ).is_err(), "presence_penalty=2.1 must be rejected" );

  // NaN and Infinity
  assert!( validate_presence_penalty( f32::NAN ).is_err(), "NaN presence_penalty must be rejected" );
  assert!( validate_presence_penalty( f32::NEG_INFINITY ).is_err(), "NEG_INFINITY presence_penalty must be rejected" );
}

/// Full coverage for `validate_repetition_penalty` (previously untested).
#[ test ]
fn test_validate_repetition_penalty()
{
  use api_huggingface::validation::validate_repetition_penalty;

  // Must be positive
  let result = validate_repetition_penalty( 0.0 );
  assert!( result.is_err(), "repetition_penalty=0.0 must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.to_lowercase().contains( "repetition_penalty" ) );
  assert!( msg.to_lowercase().contains( "positive" ) );
  }
  else
  {
  panic!( "Expected Validation error for non-positive repetition_penalty" );
  }

  assert!( validate_repetition_penalty( -0.1 ).is_err(), "Negative repetition_penalty must be rejected" );

  // Valid range: (0.0, 10.0]
  assert!( validate_repetition_penalty( 0.1 ).is_ok(), "repetition_penalty=0.1 must be valid" );
  assert!( validate_repetition_penalty( 1.0 ).is_ok(), "repetition_penalty=1.0 must be valid" );
  assert!( validate_repetition_penalty( 10.0 ).is_ok(), "repetition_penalty=10.0 (boundary) must be valid" );

  // Over limit
  let result = validate_repetition_penalty( 10.1 );
  assert!( result.is_err(), "repetition_penalty=10.1 must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.to_lowercase().contains( "repetition_penalty" ) );
  }
  else
  {
  panic!( "Expected Validation error for excessive repetition_penalty" );
  }

  // NaN and Infinity
  assert!( validate_repetition_penalty( f32::NAN ).is_err(), "NaN repetition_penalty must be rejected" );
  assert!( validate_repetition_penalty( f32::INFINITY ).is_err(), "INFINITY repetition_penalty must be rejected" );
}

/// Full coverage for `validate_message_role` (previously completely untested).
#[ test ]
fn test_validate_message_role()
{
  use api_huggingface::validation::validate_message_role;

  // All valid roles
  assert!( validate_message_role( "system" ).is_ok() );
  assert!( validate_message_role( "user" ).is_ok() );
  assert!( validate_message_role( "assistant" ).is_ok() );
  assert!( validate_message_role( "tool" ).is_ok() );
  assert!( validate_message_role( "function" ).is_ok() );

  // Invalid roles
  let invalid_roles = [ "bot", "ai", "human", "System", "USER", "", " user" ];
  for role in invalid_roles
  {
  let result = validate_message_role( role );
  assert!(
  result.is_err(),
  "Role '{role}' should be invalid"
  );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
      assert!( msg.to_lowercase().contains( "role" ) || msg.to_lowercase().contains( "invalid" ) );
  }
  else
  {
      panic!( "Expected Validation error for role '{role}'" );
  }
  }
}

/// Full coverage for `validate_message_content` (previously completely untested).
#[ test ]
fn test_validate_message_content()
{
  use api_huggingface::validation::{ validate_message_content, MAX_INPUT_LENGTH };

  // Empty content must fail
  let result = validate_message_content( "" );
  assert!( result.is_err(), "Empty message content must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "empty" ) );
  }
  else
  {
  panic!( "Expected Validation error for empty message content" );
  }

  // Valid content
  assert!( validate_message_content( "Hello, how are you?" ).is_ok() );
  assert!( validate_message_content( "Multi\nline\ncontent" ).is_ok() );

  // At exactly max length: must pass
  let at_limit = "a".repeat( MAX_INPUT_LENGTH );
  assert!( validate_message_content( &at_limit ).is_ok(), "Message content at limit must be valid" );

  // Over max length: must fail
  let over_limit = "a".repeat( MAX_INPUT_LENGTH + 1 );
  let result = validate_message_content( &over_limit );
  assert!( result.is_err(), "Message content over limit must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "too long" ) );
  }
  else
  {
  panic!( "Expected Validation error for excessive message content" );
  }
}

/// Full coverage for `validate_tool_choice` (previously completely untested).
#[ test ]
fn test_validate_tool_choice()
{
  use api_huggingface::validation::validate_tool_choice;

  // Standard values
  assert!( validate_tool_choice( "auto" ).is_ok() );
  assert!( validate_tool_choice( "none" ).is_ok() );
  assert!( validate_tool_choice( "required" ).is_ok() );

  // Custom function name (non-empty string also accepted)
  assert!( validate_tool_choice( "my_function" ).is_ok() );
  assert!( validate_tool_choice( "get_weather" ).is_ok() );

  // Empty is invalid
  let result = validate_tool_choice( "" );
  assert!( result.is_err(), "Empty tool_choice must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.to_lowercase().contains( "tool_choice" ) );
  assert!( msg.to_lowercase().contains( "empty" ) );
  }
  else
  {
  panic!( "Expected Validation error for empty tool_choice" );
  }
}

/// Full coverage for `validate_image_size` (previously completely untested).
#[ test ]
fn test_validate_image_size()
{
  use api_huggingface::validation::{ validate_image_size, MAX_IMAGE_SIZE_BYTES };

  // Empty data must fail
  let result = validate_image_size( &[] );
  assert!( result.is_err(), "Empty image data must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "empty" ) );
  }
  else
  {
  panic!( "Expected Validation error for empty image data" );
  }

  // Small valid data
  let small_data = vec![ 0u8; 1024 ];
  assert!( validate_image_size( &small_data ).is_ok(), "1 KB image must be valid" );

  // Exactly at limit: must pass
  let at_limit = vec![ 0u8; MAX_IMAGE_SIZE_BYTES ];
  assert!(
  validate_image_size( &at_limit ).is_ok(),
  "Image exactly at MAX_IMAGE_SIZE_BYTES must be valid"
  );

  // One byte over: must fail
  let over_limit = vec![ 0u8; MAX_IMAGE_SIZE_BYTES + 1 ];
  let result = validate_image_size( &over_limit );
  assert!( result.is_err(), "Image over MAX_IMAGE_SIZE_BYTES must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.to_lowercase().contains( "image" ) );
  assert!( msg.to_lowercase().contains( "large" ) || msg.to_lowercase().contains( "too large" ) );
  }
  else
  {
  panic!( "Expected Validation error for image over size limit" );
  }
}

/// Full coverage for `validate_audio_size` (previously completely untested).
#[ test ]
fn test_validate_audio_size()
{
  use api_huggingface::validation::{ validate_audio_size, MAX_AUDIO_SIZE_BYTES };

  // Empty data must fail
  let result = validate_audio_size( &[] );
  assert!( result.is_err(), "Empty audio data must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "empty" ) );
  }
  else
  {
  panic!( "Expected Validation error for empty audio data" );
  }

  // Small valid data
  let small_data = vec![ 0u8; 1024 ];
  assert!( validate_audio_size( &small_data ).is_ok(), "1 KB audio must be valid" );

  // Exactly at limit: must pass
  let at_limit = vec![ 0u8; MAX_AUDIO_SIZE_BYTES ];
  assert!(
  validate_audio_size( &at_limit ).is_ok(),
  "Audio exactly at MAX_AUDIO_SIZE_BYTES must be valid"
  );

  // One byte over: must fail
  let over_limit = vec![ 0u8; MAX_AUDIO_SIZE_BYTES + 1 ];
  assert!( validate_audio_size( &over_limit ).is_err(), "Audio over MAX_AUDIO_SIZE_BYTES must be rejected" );
}

/// Full coverage for `validate_url` (previously completely untested).
#[ test ]
fn test_validate_url()
{
  use api_huggingface::validation::validate_url;

  // Valid http and https URLs
  assert!( validate_url( "http://example.com" ).is_ok() );
  assert!( validate_url( "https://api.huggingface.co/v1/models" ).is_ok() );
  assert!( validate_url( "https://router.huggingface.co/v1/chat/completions" ).is_ok() );

  // Empty must fail
  let result = validate_url( "" );
  assert!( result.is_err(), "Empty URL must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "empty" ) );
  }
  else
  {
  panic!( "Expected Validation error for empty URL" );
  }

  // Wrong protocol
  let result = validate_url( "ftp://example.com" );
  assert!( result.is_err(), "FTP URL must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "http://" ) || msg.contains( "https://" ) );
  }
  else
  {
  panic!( "Expected Validation error for non-http(s) URL" );
  }

  // No protocol at all
  assert!( validate_url( "example.com" ).is_err(), "URL without protocol must be rejected" );

  // Exactly at 2048 chars: must pass
  let long_but_valid = format!( "https://example.com/{}", "a".repeat( 2048 - "https://example.com/".len() ) );
  assert_eq!( long_but_valid.len(), 2048 );
  assert!( validate_url( &long_but_valid ).is_ok(), "URL at exactly 2048 chars must be valid" );

  // One over 2048 chars: must fail
  let too_long = format!( "https://example.com/{}", "a".repeat( 2048 - "https://example.com/".len() + 1 ) );
  assert!( too_long.len() > 2048 );
  let result = validate_url( &too_long );
  assert!( result.is_err(), "URL exceeding 2048 chars must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "too long" ) || msg.contains( "2048" ) );
  }
  else
  {
  panic!( "Expected Validation error for URL over 2048 chars" );
  }
}

/// Boundary-value tests for `validate_stop_sequences`.
#[ test ]
fn test_validate_stop_sequences_boundary()
{
  use api_huggingface::validation::{ validate_stop_sequences, MAX_STOP_SEQUENCES };

  // Exactly at max count: must pass
  let at_limit : Vec< String > = ( 0..MAX_STOP_SEQUENCES ).map( | i | format!( "stop{i}" ) ).collect();
  assert!(
  validate_stop_sequences( &at_limit ).is_ok(),
  "Exactly MAX_STOP_SEQUENCES sequences must be valid"
  );

  // One over: must fail
  let over_limit : Vec< String > = ( 0..=MAX_STOP_SEQUENCES ).map( | i | format!( "stop{i}" ) ).collect();
  let result = validate_stop_sequences( &over_limit );
  assert!( result.is_err(), "More than MAX_STOP_SEQUENCES must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.contains( "stop" ) || msg.contains( "sequences" ) );
  }
  else
  {
  panic!( "Expected Validation error for too many stop sequences" );
  }

  // Stop sequence exactly at 100 chars: must pass
  let at_char_limit = vec![ "a".repeat( 100 ) ];
  assert!(
  validate_stop_sequences( &at_char_limit ).is_ok(),
  "Stop sequence at exactly 100 chars must be valid"
  );

  // Stop sequence at 101 chars: must fail
  let over_char_limit = vec![ "a".repeat( 101 ) ];
  let result = validate_stop_sequences( &over_char_limit );
  assert!( result.is_err(), "Stop sequence over 100 chars must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!( msg.to_lowercase().contains( "stop" ) || msg.to_lowercase().contains( "long" ) );
  }
  else
  {
  panic!( "Expected Validation error for stop sequence over 100 chars" );
  }
}

/// Full coverage for `InferenceProvider` API (previously untested).
#[ test ]
fn test_inference_provider_api()
{
  use api_huggingface::providers::InferenceProvider;

  // as_str() — all variants return non-empty, expected strings
  assert_eq!( InferenceProvider::OpenAI.as_str(), "openai" );
  assert_eq!( InferenceProvider::Cohere.as_str(), "cohere" );
  assert_eq!( InferenceProvider::Together.as_str(), "together" );
  assert_eq!( InferenceProvider::Groq.as_str(), "groq" );
  assert_eq!( InferenceProvider::HfInference.as_str(), "hf-inference" );

  // available_models() — all providers return non-empty slices
  for provider in [ InferenceProvider::OpenAI, InferenceProvider::Cohere, InferenceProvider::Together, InferenceProvider::Groq, InferenceProvider::HfInference ]
  {
  let models = provider.available_models();
  assert!( !models.is_empty(), "{provider:?} must have at least one available model" );
  for model in models
  {
      assert!( !model.is_empty(), "Model identifier must not be empty" );
      assert!( model.contains( '/' ), "HuggingFace model ID must contain org/name separator" );
  }
  }

  // for_model() with a known model returns Some
  let known = "meta-llama/Llama-2-7b-chat-hf";
  let provider = InferenceProvider::for_model( known );
  assert!( provider.is_some(), "for_model({known}) must return Some provider" );

  // for_model() with an unknown model returns None
  let unknown = "nonexistent/model-that-does-not-exist";
  let result = InferenceProvider::for_model( unknown );
  assert!( result.is_none(), "for_model({unknown}) must return None" );

  // default_pro_model() and fallback_model() return non-empty valid identifiers
  use api_huggingface::environment::HuggingFaceEnvironmentImpl;
  let default_model = api_huggingface::providers::Providers::< HuggingFaceEnvironmentImpl >::default_pro_model();
  assert!( !default_model.is_empty() );
  let fallback = api_huggingface::providers::Providers::< HuggingFaceEnvironmentImpl >::fallback_model();
  assert!( !fallback.is_empty() );
}

/// Reproducing test: `validate_temperature` dead code — NaN/Inf checks after range check
/// are unreachable. NaN input gets "between 0.0 and 2.0" error instead of "valid number".
///
/// Root Cause: `!(0.0..=2.0).contains(&NaN)` evaluates to `true` (NaN is never in any range),
///   so the range-check branch fires first, returning the range error. The subsequent
///   `is_nan() || is_infinite()` block is dead code — never reached for NaN or Infinity.
/// Why Not Caught: Prior tests only assert `is_err()`, not the specific message text.
/// Fix Applied: Move `is_nan() || is_infinite()` check before the range check in all
///   affected validators so the most-specific error fires first.
/// Prevention: When writing float validators, always put the NaN/Inf guard first — float
///   NaN breaks range comparisons silently (NaN `<=`/`>=` always returns false).
/// Pitfall: `contains()` on a float range catches NaN "accidentally" (NaN comparison
///   always false → not contained → error returned), but with the wrong message.
#[ test ]
fn test_validate_temperature_nan_gives_valid_number_error()
{
  use api_huggingface::{ validation::validate_temperature, error::HuggingFaceError };

  let result = validate_temperature( f32::NAN );
  assert!( result.is_err(), "NaN temperature must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!(
      msg.contains( "valid number" ),
      "NaN should produce 'valid number' error, got: {msg}"
  );
  }
  else
  {
  panic!( "Expected Validation error for NaN temperature" );
  }

  // +Infinity also needs the "valid number" message, not "between 0.0 and 2.0"
  let result = validate_temperature( f32::INFINITY );
  assert!( result.is_err(), "INFINITY temperature must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!(
      msg.contains( "valid number" ),
      "INFINITY should produce 'valid number' error, got: {msg}"
  );
  }
  else
  {
  panic!( "Expected Validation error for INFINITY temperature" );
  }
}

/// Reproducing test: `validate_top_p` dead code — same NaN/Inf dead-check pattern.
///
/// Root Cause: Same as VA-01 — range check `!(0.0..=1.0).contains(&NaN)` fires first.
/// Why Not Caught: Prior tests only assert `is_err()`.
/// Fix Applied: NaN/Inf check moved before range check.
/// Prevention/Pitfall: See `test_validate_temperature_nan_gives_valid_number_error`.
#[ test ]
fn test_validate_top_p_nan_gives_valid_number_error()
{
  use api_huggingface::{ validation::validate_top_p, error::HuggingFaceError };

  let result = validate_top_p( f32::NAN );
  assert!( result.is_err(), "NaN top_p must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!(
      msg.contains( "valid number" ),
      "NaN top_p should produce 'valid number' error, got: {msg}"
  );
  }
  else
  {
  panic!( "Expected Validation error for NaN top_p" );
  }
}

/// Reproducing test: `validate_frequency_penalty` dead code — same NaN/Inf dead-check pattern.
///
/// Root Cause: Range check `!(-2.0..=2.0).contains(&NaN)` fires first.
/// Why Not Caught: Prior tests only assert `is_err()`.
/// Fix Applied: NaN/Inf check moved before range check.
/// Prevention/Pitfall: See `test_validate_temperature_nan_gives_valid_number_error`.
#[ test ]
fn test_validate_frequency_penalty_nan_gives_valid_number_error()
{
  use api_huggingface::{ validation::validate_frequency_penalty, error::HuggingFaceError };

  let result = validate_frequency_penalty( f32::NAN );
  assert!( result.is_err(), "NaN frequency_penalty must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!(
      msg.contains( "valid number" ),
      "NaN frequency_penalty should produce 'valid number' error, got: {msg}"
  );
  }
  else
  {
  panic!( "Expected Validation error for NaN frequency_penalty" );
  }
}

/// Reproducing test: `validate_presence_penalty` dead code — same NaN/Inf dead-check pattern.
///
/// Root Cause: Range check `!(-2.0..=2.0).contains(&NaN)` fires first.
/// Why Not Caught: Prior tests only assert `is_err()`.
/// Fix Applied: NaN/Inf check moved before range check.
/// Prevention/Pitfall: See `test_validate_temperature_nan_gives_valid_number_error`.
#[ test ]
fn test_validate_presence_penalty_nan_gives_valid_number_error()
{
  use api_huggingface::{ validation::validate_presence_penalty, error::HuggingFaceError };

  let result = validate_presence_penalty( f32::NAN );
  assert!( result.is_err(), "NaN presence_penalty must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!(
      msg.contains( "valid number" ),
      "NaN presence_penalty should produce 'valid number' error, got: {msg}"
  );
  }
  else
  {
  panic!( "Expected Validation error for NaN presence_penalty" );
  }
}

/// Reproducing test: `validate_repetition_penalty` dead `is_infinite()` check.
///
/// Root Cause: `penalty.is_nan() || penalty.is_infinite()` — the `|| is_infinite()` branch
///   is dead code. +Inf is caught first by `penalty > 10.0` giving "too high" message;
///   -Inf is caught by `penalty <= 0.0` giving "must be positive". Only `is_nan()` is
///   ever reached (because NaN falsifies both `<= 0.0` and `> 10.0`).
///   Consequence: +Inf gives "too high" instead of "must be a valid number".
/// Why Not Caught: Prior tests only assert `is_err()`, not the message text.
/// Fix Applied: NaN/Inf check moved before all other checks.
/// Prevention: Always validate NaN/Inf before applying numeric comparisons.
/// Pitfall: `penalty > 10.0` is true for `+Inf`, silently "catching" it with wrong message.
#[ test ]
fn test_validate_repetition_penalty_infinity_gives_valid_number_error()
{
  use api_huggingface::{ validation::validate_repetition_penalty, error::HuggingFaceError };

  // +Inf must give "valid number" error, not "too high"
  let result = validate_repetition_penalty( f32::INFINITY );
  assert!( result.is_err(), "INFINITY repetition_penalty must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!(
      msg.contains( "valid number" ),
      "INFINITY repetition_penalty should produce 'valid number' error, got: {msg}"
  );
  }
  else
  {
  panic!( "Expected Validation error for INFINITY repetition_penalty" );
  }

  // -Inf must also give "valid number" error, not "must be positive"
  let result = validate_repetition_penalty( f32::NEG_INFINITY );
  assert!( result.is_err(), "NEG_INFINITY repetition_penalty must be rejected" );
  if let Err( HuggingFaceError::Validation( msg ) ) = result
  {
  assert!(
      msg.contains( "valid number" ),
      "NEG_INFINITY repetition_penalty should produce 'valid number' error, got: {msg}"
  );
  }
  else
  {
  panic!( "Expected Validation error for NEG_INFINITY repetition_penalty" );
  }
}
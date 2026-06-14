//! Tests for `OpenAiCompatError` Display formatting and `From` conversions.
//!
//! Validates that every error variant produces a human-readable message containing
//! both a category prefix and the supplied detail string, and that automatic
//! conversions from `serde_json::Error` and `InvalidHeaderValue` map to the
//! expected variant.
//!
//! # Test Matrix
//!
//! | Test | Category | Validates |
//! |------|----------|-----------|
//! | api_error_display_contains_category_and_detail | Display | Api variant formatting |
//! | http_error_display_contains_category_and_detail | Display | Http variant formatting |
//! | network_error_display_contains_category_and_detail | Display | Network variant formatting |
//! | timeout_error_display_contains_category_and_detail | Display | Timeout variant formatting |
//! | invalid_api_key_error_display_contains_category_and_detail | Display | InvalidApiKey variant formatting |
//! | environment_error_display_contains_category_and_detail | Display | Environment variant formatting |
//! | deserialise_error_display_contains_category_and_detail | Display | Deserialise variant formatting |
//! | from_serde_json_error_produces_deserialise_variant | From | serde_json::Error → Deserialise |
//! | from_invalid_header_value_produces_invalid_api_key_variant | From | InvalidHeaderValue → InvalidApiKey |

#![ cfg( feature = "enabled" ) ]

use api_openai_compatible::OpenAiCompatError;

// ------------------------------------------------------------------ //

/// `OpenAiCompatError::Api` must format as `"API error : <detail>"`.
///
/// The display string must contain both the category prefix and the
/// caller-supplied detail so that log aggregators can filter by category
/// while humans can still see the specific failure reason.
#[ test ]
fn api_error_display_contains_category_and_detail()
{
  let err = OpenAiCompatError::Api( "rate limit exceeded".to_string() );
  let msg = err.to_string();

  assert!(
    msg.contains( "API error" ),
    "Api display must contain category prefix; got: {msg}",
  );
  assert!(
    msg.contains( "rate limit exceeded" ),
    "Api display must contain the detail string; got: {msg}",
  );
}

// ------------------------------------------------------------------ //

/// `OpenAiCompatError::Http` must format as `"HTTP error : <detail>"`.
#[ test ]
fn http_error_display_contains_category_and_detail()
{
  let err = OpenAiCompatError::Http( "502 Bad Gateway".to_string() );
  let msg = err.to_string();

  assert!(
    msg.contains( "HTTP error" ),
    "Http display must contain category prefix; got: {msg}",
  );
  assert!(
    msg.contains( "502 Bad Gateway" ),
    "Http display must contain the detail string; got: {msg}",
  );
}

// ------------------------------------------------------------------ //

/// `OpenAiCompatError::Network` must format as `"Network error : <detail>"`.
#[ test ]
fn network_error_display_contains_category_and_detail()
{
  let err = OpenAiCompatError::Network( "connection refused".to_string() );
  let msg = err.to_string();

  assert!(
    msg.contains( "Network error" ),
    "Network display must contain category prefix; got: {msg}",
  );
  assert!(
    msg.contains( "connection refused" ),
    "Network display must contain the detail string; got: {msg}",
  );
}

// ------------------------------------------------------------------ //

/// `OpenAiCompatError::Timeout` must format as `"Timeout : <detail>"`.
#[ test ]
fn timeout_error_display_contains_category_and_detail()
{
  let err = OpenAiCompatError::Timeout( "30s elapsed".to_string() );
  let msg = err.to_string();

  assert!(
    msg.contains( "Timeout" ),
    "Timeout display must contain category prefix; got: {msg}",
  );
  assert!(
    msg.contains( "30s elapsed" ),
    "Timeout display must contain the detail string; got: {msg}",
  );
}

// ------------------------------------------------------------------ //

/// `OpenAiCompatError::InvalidApiKey` must format as `"Invalid API key : <detail>"`.
#[ test ]
fn invalid_api_key_error_display_contains_category_and_detail()
{
  let err = OpenAiCompatError::InvalidApiKey( "non-ASCII char".to_string() );
  let msg = err.to_string();

  assert!(
    msg.contains( "Invalid API key" ),
    "InvalidApiKey display must contain category prefix; got: {msg}",
  );
  assert!(
    msg.contains( "non-ASCII char" ),
    "InvalidApiKey display must contain the detail string; got: {msg}",
  );
}

// ------------------------------------------------------------------ //

/// `OpenAiCompatError::Environment` must format as `"Environment error : <detail>"`.
#[ test ]
fn environment_error_display_contains_category_and_detail()
{
  let err = OpenAiCompatError::Environment( "bad URL".to_string() );
  let msg = err.to_string();

  assert!(
    msg.contains( "Environment error" ),
    "Environment display must contain category prefix; got: {msg}",
  );
  assert!(
    msg.contains( "bad URL" ),
    "Environment display must contain the detail string; got: {msg}",
  );
}

// ------------------------------------------------------------------ //

/// `OpenAiCompatError::Deserialise` must format as `"Deserialisation error : <detail>"`.
#[ test ]
fn deserialise_error_display_contains_category_and_detail()
{
  let err = OpenAiCompatError::Deserialise( "missing field".to_string() );
  let msg = err.to_string();

  assert!(
    msg.contains( "Deserialisation error" ),
    "Deserialise display must contain category prefix; got: {msg}",
  );
  assert!(
    msg.contains( "missing field" ),
    "Deserialise display must contain the detail string; got: {msg}",
  );
}

// ------------------------------------------------------------------ //

/// `From<serde_json::Error>` must produce the `Deserialise` variant.
///
/// `serde_json::from_str::<i32>("not-a-number")` produces a parse error that
/// the `From` impl wraps into `OpenAiCompatError::Deserialise`. The conversion
/// preserves the original error message inside the variant.
#[ test ]
fn from_serde_json_error_produces_deserialise_variant()
{
  let serde_err = serde_json::from_str::< i32 >( "not-a-number" )
    .expect_err( "parsing \"not-a-number\" as i32 must fail" );

  let converted : OpenAiCompatError = serde_err.into();

  match &converted
  {
    OpenAiCompatError::Deserialise( detail ) =>
    {
      assert!(
        !detail.is_empty(),
        "Deserialise detail must contain the serde error message",
      );
    },
    other =>
    {
      panic!( "expected Deserialise variant, got: {other:?}" );
    },
  }
}

// ------------------------------------------------------------------ //

/// `From<InvalidHeaderValue>` must produce the `InvalidApiKey` variant.
///
/// `HeaderValue::from_bytes(&[0x00])` fails because NUL (0x00) is a control
/// character rejected by the HTTP header value parser. The `From` impl wraps
/// this into `InvalidApiKey`.
#[ test ]
fn from_invalid_header_value_produces_invalid_api_key_variant()
{
  let header_err = reqwest::header::HeaderValue::from_bytes( &[ 0x00_u8 ] )
    .expect_err( "NUL byte must be invalid in HTTP header values" );

  let converted : OpenAiCompatError = header_err.into();

  match &converted
  {
    OpenAiCompatError::InvalidApiKey( detail ) =>
    {
      assert!(
        !detail.is_empty(),
        "InvalidApiKey detail must contain the header error message",
      );
    },
    other =>
    {
      panic!( "expected InvalidApiKey variant, got: {other:?}" );
    },
  }
}

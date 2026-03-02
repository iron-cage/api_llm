//! Tests for `OpenAiCompatEnvironment` trait and `OpenAiCompatEnvironmentImpl`.
//!
//! Covers construction defaults, builder overrides, header generation, and
//! validation (empty key rejection).
//! These tests define the authoritative contract for environment types;
//! the implementation in Phase 3 must satisfy them without modifying this file.
//!
//! # Test Matrix
//!
//! | Test | Validates |
//! |------|-----------|
//! | new_has_default_base_url_and_timeout | Default URL + timeout values |
//! | new_fails_with_empty_key | Empty string key returns Err, not panic |
//! | with_base_url_overrides | Builder replaces base URL |
//! | with_timeout_overrides | Builder replaces timeout |
//! | headers_returns_bearer_and_content_type | Authorization + Content-Type headers |

#![ cfg( feature = "enabled" ) ]

use api_openai_compatible::{ OpenAiCompatEnvironment, OpenAiCompatEnvironmentImpl };
use core::time::Duration;

// ------------------------------------------------------------------ //

/// Default `new()` must set base URL to the `OpenAI` endpoint and timeout to 30 s.
///
/// The default values enable zero-configuration usage for the standard `OpenAI` API.
/// Any deviation from these defaults would silently misconfigure callers that
/// rely on defaults without explicit overrides.
#[ test ]
fn new_has_default_base_url_and_timeout()
{
  let env = OpenAiCompatEnvironmentImpl::new( "sk-test-key" )
    .expect( "new() must succeed with a non-empty key" );

  assert_eq!(
    env.base_url(),
    "https://api.openai.com/v1/",
    "default base URL must be the OpenAI v1 endpoint with trailing slash",
  );

  assert_eq!(
    env.timeout().as_secs(),
    30,
    "default timeout must be 30 seconds",
  );
}

// ------------------------------------------------------------------ //

/// `new()` with an empty key string must return `Err`, not panic or silently succeed.
///
/// An empty API key would produce an `Authorization: Bearer ` header with no
/// credential, causing every request to fail with HTTP 401. Catching this at
/// construction time gives callers a clear error instead of a cryptic 401 deep
/// inside a network call.
#[ test ]
fn new_fails_with_empty_key()
{
  let result = OpenAiCompatEnvironmentImpl::new( "" );
  assert!(
    result.is_err(),
    "new() must return Err when given an empty API key, not Ok",
  );
}

// ------------------------------------------------------------------ //

/// `with_base_url()` must replace the base URL stored in the environment.
///
/// KIE.ai places the model slug in the URL path, so callers must be able to
/// supply arbitrary base URLs like `"https://api.kie.ai/{slug}/v1/"`.
#[ test ]
fn with_base_url_overrides()
{
  let custom = "https://api.kie.ai/my-model/v1/";
  let env = OpenAiCompatEnvironmentImpl::new( "sk-test-key" )
    .expect( "new() must succeed" )
    .with_base_url( custom );

  assert_eq!(
    env.base_url(),
    custom,
    "base_url() must reflect the value passed to with_base_url()",
  );
}

// ------------------------------------------------------------------ //

/// `with_timeout()` must replace the timeout stored in the environment.
///
/// Long-running model inference may require timeouts well above the 30 s default.
#[ test ]
fn with_timeout_overrides()
{
  let custom = Duration::from_secs( 120 );
  let env = OpenAiCompatEnvironmentImpl::new( "sk-test-key" )
    .expect( "new() must succeed" )
    .with_timeout( custom );

  assert_eq!(
    env.timeout(),
    custom,
    "timeout() must reflect the value passed to with_timeout()",
  );
}

// ------------------------------------------------------------------ //

/// `headers()` must produce `Authorization: Bearer <key>` and
/// `Content-Type: application/json` for every request.
///
/// The Authorization header must embed the exact key supplied to `new()`.
/// Content-Type must be `application/json` because the request body is always
/// JSON-encoded.
#[ test ]
fn headers_returns_bearer_and_content_type()
{
  let key = "sk-my-secret-key-12345";
  let env = OpenAiCompatEnvironmentImpl::new( key )
    .expect( "new() must succeed" );

  let headers = env.headers().expect( "headers() must succeed with a valid environment" );

  // --- Authorization ---
  let auth = headers
    .get( "Authorization" )
    .expect( "Authorization header must be present" )
    .to_str()
    .expect( "Authorization value must be valid UTF-8" );

  assert!(
    auth.starts_with( "Bearer " ),
    "Authorization header must use the Bearer scheme; got: {auth}",
  );
  assert!(
    auth.contains( key ),
    "Authorization header must contain the API key; got: {auth}",
  );

  // --- Content-Type ---
  let ct = headers
    .get( "Content-Type" )
    .expect( "Content-Type header must be present" )
    .to_str()
    .expect( "Content-Type value must be valid UTF-8" );

  assert_eq!(
    ct,
    "application/json",
    "Content-Type must be application/json",
  );
}

// ------------------------------------------------------------------ //

/// `new()` with a whitespace-only key (e.g. `"   "`) must return `Err`.
///
/// A whitespace-only key passes the `is_empty()` guard but would still produce
/// a useless `Authorization: Bearer    ` header that every API server rejects
/// with HTTP 401. Accepting whitespace-only keys creates a confusing failure
/// mode deep in the network stack instead of a clear construction-time error.
///
/// The implementation must treat whitespace-only as invalid (reject eagerly).
#[ test ]
fn new_fails_with_whitespace_only_key()
{
  let result = OpenAiCompatEnvironmentImpl::new( "   " );
  assert!(
    result.is_err(),
    "new() must return Err for a whitespace-only API key, not Ok",
  );
}

// ------------------------------------------------------------------ //

/// `new()` with a key containing printable special characters must succeed.
///
/// Real API keys from providers sometimes contain hyphens, underscores, and
/// alphanumeric characters. The implementation must not over-restrict key
/// characters beyond what the HTTP `Authorization` header value actually
/// forbids (control characters and a small set of non-ASCII bytes).
#[ test ]
fn new_succeeds_with_key_containing_printable_special_chars()
{
  // ASCII printable characters that are valid in HTTP header values.
  let key = "sk-test_key-ABCD1234.special+chars";
  let result = OpenAiCompatEnvironmentImpl::new( key );
  assert!(
    result.is_ok(),
    "new() must succeed with a key containing printable ASCII special characters; got: {:?}",
    result.err(),
  );

  let env = result.unwrap();
  assert_eq!(
    env.api_key(),
    key,
    "api_key() must return the exact key passed to new()",
  );
}

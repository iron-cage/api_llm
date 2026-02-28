//! Tests for `OpenAiCompatEnvironment` trait and `OpenAiCompatEnvironmentImpl`.
//!
//! Covers construction defaults, builder overrides, and header generation.
//! These tests define the authoritative contract for environment types;
//! the implementation in Phase 3 must satisfy them without modifying this file.

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

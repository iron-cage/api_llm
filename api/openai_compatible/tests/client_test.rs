//! Tests for `Client` construction and HTTP paths.
//!
//! Unit tests validate generic trait acceptance. Integration tests validate
//! the success path (real API key → 200), the non-2xx GET path
//! (fake key → 401 → `Api` error), and the non-2xx POST path (fake key → 401).
//!
//! # Test Matrix
//!
//! | Test | Category | Validates |
//! |------|----------|-----------|
//! | client_build_accepts_custom_environment_implementor | unit | Generic Client<E> trait polymorphism |
//! | client_get_models_succeeds_with_real_key | integration | GET success path returns Ok |
//! | client_get_models_returns_api_error_with_fake_key | integration | GET non-2xx path returns Err |
//! | client_post_chat_returns_api_error_with_fake_key | integration | POST non-2xx path returns Err |

#![ cfg( feature = "enabled" ) ]

// ------------------------------------------------------------------ //
//  Unit tests
// ------------------------------------------------------------------ //

/// `Client::build` must accept any type implementing `OpenAiCompatEnvironment`.
///
/// The generic `Client<E>` is not hardcoded to `OpenAiCompatEnvironmentImpl`.
/// Downstream crates define their own environment structs (e.g. `XaiEnvironment`)
/// and pass them to `Client::build`. This test proves the generic constraint works
/// with an independent custom implementation, ensuring the trait is the only
/// coupling between client and environment.
#[ test ]
fn client_build_accepts_custom_environment_implementor()
{
  use api_openai_compatible::{ Client, OpenAiCompatEnvironment, OpenAiCompatError, Result };
  use core::time::Duration;
  use reqwest::header;

  /// Minimal custom environment for testing the generic `Client<E>`.
  #[ derive( Debug, Clone ) ]
  struct CustomEnv;

  #[ allow( clippy::unnecessary_literal_bound ) ]
  impl OpenAiCompatEnvironment for CustomEnv
  {
    fn api_key( &self ) -> &str { "sk-custom-test" }
    fn base_url( &self ) -> &str { "http://127.0.0.1:1/" }
    fn timeout( &self ) -> Duration { Duration::from_secs( 5 ) }

    fn headers( &self ) -> Result< header::HeaderMap >
    {
      let mut map = header::HeaderMap::new();
      let auth = format!( "Bearer {}", self.api_key() )
        .parse::< header::HeaderValue >()
        .map_err( | e | OpenAiCompatError::InvalidApiKey( e.to_string() ) )?;
      map.insert( header::AUTHORIZATION, auth );
      map.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static( "application/json" ),
      );
      Ok( map )
    }
  }

  let client = Client::build( CustomEnv );

  assert!(
    client.is_ok(),
    "Client::build must accept a custom OpenAiCompatEnvironment implementor; got: {:?}",
    client.err(),
  );
}

// ------------------------------------------------------------------ //
//  Integration tests
// ------------------------------------------------------------------ //

/// `Client::get("models")` with a valid API key must return `Ok`.
///
/// The `OpenAI` `/v1/models` endpoint returns a JSON object with a list of
/// available models. A successful 200 response proves that the GET path
/// constructs the URL correctly, attaches headers, and deserialises the
/// response body.
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn client_get_models_succeeds_with_real_key()
{
  use api_openai_compatible::{ Client, OpenAiCompatEnvironmentImpl };

  let ws = workspace_tools::workspace()
    .expect( "workspace root must be resolvable" );
  let api_key = ws.load_secret_key( "OPENAI_API_KEY", "-secrets.sh" )
    .expect( "OPENAI_API_KEY must be set in secret/-secrets.sh" );

  let env = OpenAiCompatEnvironmentImpl::new( &api_key )
    .expect( "environment construction must succeed" );

  let client = Client::build( env )
    .expect( "Client::build() must succeed" );

  let result : Result< serde_json::Value, _ > = client.get( "models" ).await;

  assert!(
    result.is_ok(),
    "GET models with a valid API key must return Ok; got: {:?}",
    result.err(),
  );

  let body = result.unwrap();
  assert!(
    body.is_object(),
    "response body must be a JSON object; got: {body}",
  );
}

// ------------------------------------------------------------------ //

/// `Client::get("models")` with a fake API key must return `Err`.
///
/// The real `OpenAI` API returns HTTP 401 when the Bearer token is invalid.
/// The client must propagate this as an `Api` error containing the response
/// body text, not swallow it silently or panic.
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn client_get_models_returns_api_error_with_fake_key()
{
  use api_openai_compatible::{ Client, OpenAiCompatEnvironmentImpl };

  let env = OpenAiCompatEnvironmentImpl::new( "sk-fake-integration-test" )
    .expect( "environment construction must succeed with any non-empty key" );

  let client = Client::build( env )
    .expect( "Client::build() must succeed" );

  let result : Result< serde_json::Value, _ > = client.get( "models" ).await;

  assert!(
    result.is_err(),
    "GET models with a fake key must return Err, not Ok",
  );

  let err_msg = result.unwrap_err().to_string();
  assert!(
    err_msg.contains( "API error" ),
    "error must contain 'API error' category prefix; got: {err_msg}",
  );
}

// ------------------------------------------------------------------ //

/// `Client::post("chat/completions", &body)` with a fake key must return `Err`.
///
/// A minimal request body is sent to the real API with an invalid key. The API
/// responds with HTTP 401 before inspecting the body. The client must surface
/// the non-2xx status as an `Api` error containing the response body text.
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn client_post_chat_returns_api_error_with_fake_key()
{
  use api_openai_compatible::{ Client, OpenAiCompatEnvironmentImpl };

  let env = OpenAiCompatEnvironmentImpl::new( "sk-fake-integration-test" )
    .expect( "environment construction must succeed with any non-empty key" );

  let client = Client::build( env )
    .expect( "Client::build() must succeed" );

  let body = serde_json::json!({
    "model": "gpt-4o",
    "messages": [ { "role": "user", "content": "Hi" } ]
  });

  let result : Result< serde_json::Value, _ > = client.post( "chat/completions", &body ).await;

  assert!(
    result.is_err(),
    "POST chat/completions with a fake key must return Err, not Ok",
  );

  let err_msg = result.unwrap_err().to_string();
  assert!(
    err_msg.contains( "API error" ),
    "error must contain 'API error' category prefix; got: {err_msg}",
  );
}

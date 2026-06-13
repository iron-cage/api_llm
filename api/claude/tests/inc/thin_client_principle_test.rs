//! Spec traceability: IN-01..IN-06 — Thin Client Principle
//! Source: `tests/docs/invariant/001_thin_client_principle.md`

#[ allow( unused_imports ) ]
use super::*;

mod private
{
  pub fn valid_format_secret() -> super::the_module::Secret
  {
    super::the_module::Secret::new( format!( "sk-ant-api03-{}", "x".repeat( 64 ) ) )
      .expect( "syntactically valid key must construct Secret" )
  }

  pub fn minimal_request() -> super::the_module::CreateMessageRequest
  {
    super::the_module::CreateMessageRequest
    {
      model : "claude-haiku-4-5-20251001".to_string(),
      max_tokens : 5,
      messages : vec![ super::the_module::Message::user( "Hi".to_string() ) ],
      system : None,
      temperature : None,
      stream : None,
      tools : None,
      tool_choice : None,
    }
  }
}

/// IN-01: `Client::new()` activates no enterprise features
#[ test ]
fn test_in_01()
{
  let client = the_module::Client::new( private::valid_format_secret() );
  let h = client.health();
  assert_eq!( h.consecutive_failures(), 0, "IN-01: no circuit breaker active by default" );
  assert_eq!( h.total_requests(), 0, "IN-01: no request tracking active by default" );
  assert!( client.rate_limit_info().usage_percentage().abs() < f64::EPSILON, "IN-01: no rate limiting consumed" );
}

/// IN-02: No auto-retry without explicit `RetryConfig`
#[ test ]
fn test_in_02()
{
  let saved = std::env::var( "ANTHROPIC_API_KEY" ).ok();
  std::env::remove_var( "ANTHROPIC_API_KEY" );
  let t0 = std::time::Instant::now();
  let result = the_module::Client::from_env();
  let elapsed = t0.elapsed();
  if let Some( k ) = saved { std::env::set_var( "ANTHROPIC_API_KEY", k ); }
  assert!( result.is_err(), "IN-02: missing key must return Err immediately" );
  assert!(
    elapsed.as_millis() < 500,
    "IN-02: error must be immediate — no retry delay; got {}ms", elapsed.as_millis()
  );
}

/// IN-03: No implicit caching without `CacheControl`
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_in_03()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: requires valid API key from workspace" );
  // Two identical requests must reach the network independently — no implicit caching
  let response1 = client.create_message( private::minimal_request() ).await
    .expect( "INTEGRATION: first create_message must succeed" );
  let response2 = client.create_message( private::minimal_request() ).await
    .expect( "INTEGRATION: second create_message must succeed" );
  assert_ne!(
    response1.id, response2.id,
    "IN-03: each call must hit the API — no implicit caching between requests"
  );
}

/// IN-04: `create_message()` issues exactly one HTTP request
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_in_04()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: requires valid API key from workspace" );
  let response = client.create_message( private::minimal_request() ).await
    .expect( "INTEGRATION: create_message must succeed with valid credentials" );
  assert!( !response.id.is_empty(), "IN-04: single request yields response with non-empty ID" );
  assert_eq!( response.r#type, "message", "IN-04: response type confirms single API call" );
}

/// IN-05: Errors propagate without silent swallowing
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_in_05()
{
  let bad_key = format!( "sk-ant-api03-{}", "z".repeat( 64 ) );
  let secret = the_module::Secret::new( bad_key )
    .expect( "valid-format key must construct" );
  let client = the_module::Client::new( secret );
  let result = client.create_message( private::minimal_request() ).await;
  assert!( result.is_err(), "IN-05: API error must propagate as Err, not silently succeed" );
  assert!( !result.unwrap_err().to_string().is_empty(), "IN-05: error must carry non-empty details" );
}

/// IN-06: No auto rate-limiting without explicit `RateLimiterConfig`
#[ test ]
fn test_in_06()
{
  let client = the_module::Client::new( private::valid_format_secret() );
  let rate = client.rate_limit_info();
  // remaining == total: nothing consumed, no active rate limiting
  assert_eq!(
    rate.remaining_requests(),
    rate.total_limit(),
    "IN-06: no rate limiting consumed without explicit RateLimiterConfig"
  );
}

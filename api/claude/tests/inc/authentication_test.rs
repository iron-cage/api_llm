//! Authentication Integration Tests - STRICT FAILURE POLICY
//!
//! MANDATORY INTEGRATION TEST REQUIREMENTS:
//! - These tests use REAL Anthropic API endpoints - NO MOCKING ALLOWED
//! - Tests MUST FAIL IMMEDIATELY if API secrets are not available (no graceful fallbacks)
//! - Tests MUST FAIL IMMEDIATELY on network connectivity issues
//! - Tests MUST FAIL IMMEDIATELY on API authentication failures
//! - Tests MUST FAIL IMMEDIATELY on any API endpoint errors
//! - NO SILENT PASSES allowed when problems occur
//!
//! Run with : cargo test --features authentication,integration
//! Requires : Valid `ANTHROPIC_API_KEY` in environment or ../../secret/-secrets.sh

#[ allow( unused_imports ) ]
use super::*;

/// AP-11: invalid key returns authentication error from real API
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ap_11_invalid_key_returns_auth_error()
{
  let invalid_secret = the_module::Secret::new_unchecked(
    "sk-ant-api03-".to_string() + &"x".repeat( 80 )
  );
  let client = the_module::Client::new( invalid_secret );

  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-haiku-4-5-20251001" )
    .max_tokens( 10 )
    .message( the_module::Message::user( "auth test" ) )
    .build_validated()
    .unwrap();

  let result = client.create_message( request ).await;
  assert!( result.is_err(), "Invalid API key must produce an error" );

  if let Err( the_module::AnthropicError::Api( api_error ) ) = result
  {
    assert_eq!(
      api_error.r#type, "authentication_error",
      "401 from Anthropic must surface as authentication_error type"
    );
  }
}

/// Credential isolation — two clients with different keys hold independent values
#[ test ]
#[ allow( clippy::similar_names ) ]
fn test_workspace_credential_scoping()
{
  let key_a = "sk-ant-api03-".to_string() + &"a".repeat( 80 );
  let key_b = "sk-ant-api03-".to_string() + &"b".repeat( 80 );
  let secret_a = the_module::Secret::new( key_a.clone() ).expect( "valid key" );
  let secret_b = the_module::Secret::new( key_b.clone() ).expect( "valid key" );
  let client_a = the_module::Client::new( secret_a );
  let client_b = the_module::Client::new( secret_b );
  assert_ne!(
    client_a.secret().ANTHROPIC_API_KEY,
    client_b.secret().ANTHROPIC_API_KEY,
    "Clients with different secrets must hold different keys"
  );
}

/// Integration: invalid key must produce a real API error, not a panic
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_authentication_failure_recovery()
{
  let invalid_secret = the_module::Secret::new_unchecked(
    "sk-ant-api03-".to_string() + &"x".repeat( 80 )
  );
  let client = the_module::Client::new( invalid_secret );

  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-haiku-4-5-20251001" )
    .max_tokens( 50 )
    .message( the_module::Message::user( "Auth failure test" ) )
    .build_validated()
    .unwrap();

  let result = client.create_message( request ).await;
  assert!( result.is_err(), "Invalid API key must produce an error" );

  if let Err( the_module::AnthropicError::Api( api_error ) ) = result
  {
    assert!(
      api_error.r#type == "authentication_error" || api_error.r#type == "error",
      "Invalid key must surface as authentication_error, got: {}",
      api_error.r#type
    );
  }
}

/// Key format validation via `Secret::new()` prefix and non-empty checks
#[ test ]
fn test_extended_api_key_format_validation()
{
  let test_cases = vec![
    ( "", false ),                          // Empty key
    ( "bad-key", false ),                   // Wrong prefix
    ( "SK-ANT-test", false ),               // Wrong case prefix
    ( "openai-sk-abcdef", false ),          // Non-Anthropic prefix
    ( "sk-ant-api03-test-value", true ),    // Valid prefix and body
    ( "sk-ant-any-body-value", true ),      // Valid prefix and body
  ];

  for ( api_key, should_be_valid ) in test_cases
  {
    let result = the_module::Secret::new( api_key.to_string() );
    match result
    {
      Ok( _ ) =>
      {
        assert!( should_be_valid, "Expected validation to fail for : {api_key:?}" );
      },
      Err( err ) =>
      {
        assert!(
          !should_be_valid,
          "Expected validation to succeed for {api_key:?} but got: {err}"
        );
      }
    }
  }
}

/// Environment variable loading via `Secret::load_from_env()`
#[ test ]
fn test_environment_variable_precedence()
{
  let saved = std::env::var( "ANTHROPIC_API_KEY" ).ok();
  std::env::remove_var( "ANTHROPIC_API_KEY" );

  // Absent var must return Err
  let absent_result = the_module::Secret::load_from_env( "ANTHROPIC_API_KEY" );
  assert!( absent_result.is_err(), "load_from_env must fail when var is absent" );

  // Set a valid-prefix key — load must succeed and return that exact key
  let test_key = "sk-ant-api03-test-key-value";
  std::env::set_var( "ANTHROPIC_API_KEY", test_key );
  let present_result = the_module::Secret::load_from_env( "ANTHROPIC_API_KEY" );
  assert!( present_result.is_ok(), "load_from_env must succeed when var is set" );
  assert_eq!( present_result.unwrap().ANTHROPIC_API_KEY, test_key );

  // Restore
  std::env::remove_var( "ANTHROPIC_API_KEY" );
  if let Some( key ) = saved
  {
    std::env::set_var( "ANTHROPIC_API_KEY", key );
  }
}

#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_workspace_tools_secret_loading()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: NO GRACEFUL FALLBACKS
  // This test MUST fail if workspace secrets are not available

  let workspace_result = the_module::Secret::from_workspace();

  let secret = workspace_result
    .expect( "INTEGRATION TEST FAILURE: Workspace secret loading MUST work - check ../../secret/-secrets.sh contains ANTHROPIC_API_KEY" );

  // Workspace secret loading working - validate it's a real API key
  let client = the_module::Client::new( secret );
  assert!(
    !client.secret().ANTHROPIC_API_KEY.is_empty(),
    "INTEGRATION TEST FAILURE: Secret loaded but API key is empty"
  );
  assert!(
    client.secret().ANTHROPIC_API_KEY.starts_with( "sk-ant-" ),
    "INTEGRATION TEST FAILURE: API key format invalid - must start with sk-ant-"
  );

  // Test that client creation from workspace also works
  let client_from_workspace = the_module::Client::from_workspace()
    .expect( "INTEGRATION TEST FAILURE: Client::from_workspace() MUST work when Secret::from_workspace() works" );

  assert!(
    !client_from_workspace.secret().ANTHROPIC_API_KEY.is_empty(),
    "INTEGRATION TEST FAILURE: Client workspace secret is empty"
  );
  assert_eq!(
    client.secret().ANTHROPIC_API_KEY,
    client_from_workspace.secret().ANTHROPIC_API_KEY,
    "INTEGRATION TEST FAILURE: Inconsistent secrets between Secret::from_workspace() and Client::from_workspace()"
  );
}

#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_workspace_secret_fallback_to_environment()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: SECRET LOADING MUST WORK
  // Test the fallback mechanism : workspace secrets -> environment variable

  // First try workspace loading
  let workspace_result = the_module::Secret::load_from_workspace( "ANTHROPIC_API_KEY", "-secrets.sh" );

  // Then try environment loading
  let env_result = the_module::Secret::load_from_env( "ANTHROPIC_API_KEY" );

  match ( workspace_result, env_result )
  {
    ( Ok( ws_secret ), Ok( env_secret ) ) =>
    {
      let client_ws = the_module::Client::new( ws_secret );
      let client_env = the_module::Client::new( env_secret );
      assert!(
        !client_ws.secret().ANTHROPIC_API_KEY.is_empty(),
        "INTEGRATION TEST FAILURE: Workspace secret is empty"
      );
      assert!(
        !client_env.secret().ANTHROPIC_API_KEY.is_empty(),
        "INTEGRATION TEST FAILURE: Environment secret is empty"
      );
      assert!(
        client_ws.secret().ANTHROPIC_API_KEY.starts_with( "sk-ant-" ),
        "INTEGRATION TEST FAILURE: Workspace secret format invalid"
      );
      assert!(
        client_env.secret().ANTHROPIC_API_KEY.starts_with( "sk-ant-" ),
        "INTEGRATION TEST FAILURE: Environment secret format invalid"
      );
    },
    ( Ok( ws_secret ), Err( _env_err ) ) =>
    {
      let client = the_module::Client::new( ws_secret );
      assert!(
        !client.secret().ANTHROPIC_API_KEY.is_empty(),
        "INTEGRATION TEST FAILURE: Workspace secret is empty"
      );
      assert!(
        client.secret().ANTHROPIC_API_KEY.starts_with( "sk-ant-" ),
        "INTEGRATION TEST FAILURE: Workspace secret format invalid"
      );
    },
    ( Err( _ws_err ), Ok( env_secret ) ) =>
    {
      let client = the_module::Client::new( env_secret );
      assert!(
        !client.secret().ANTHROPIC_API_KEY.is_empty(),
        "INTEGRATION TEST FAILURE: Environment secret is empty"
      );
      assert!(
        client.secret().ANTHROPIC_API_KEY.starts_with( "sk-ant-" ),
        "INTEGRATION TEST FAILURE: Environment secret format invalid"
      );
    },
    ( Err( ws_err ), Err( env_err ) ) =>
    {
      panic!(
        "INTEGRATION TEST FAILURE: No API secrets available. Workspace error : {ws_err} Environment error : {env_err}. \
         Set ANTHROPIC_API_KEY environment variable or create ../../secret/-secrets.sh"
      );
    }
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_real_api_call_must_work_no_graceful_fallbacks()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: MUST MAKE REAL API CALL
  // This test validates that integration tests actually use real API

  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION TEST FAILURE: Must have valid workspace secret for real API testing" );

  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-haiku-4-5-20251001" )
    .max_tokens( 10 )
    .message( the_module::Message::user( "Hi" ) )
    .build_validated()
    .expect( "INTEGRATION TEST FAILURE: Request construction failed" );

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) )
      if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!(
        "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. \
         Test must fail per Loud Failure Mandate: {}",
        api_err.message
      )
    },
    Err( err ) =>
    {
      panic!(
        "INTEGRATION TEST FAILURE: Real API call MUST work - check network connectivity and API key validity : {err}"
      );
    }
  };

  assert!( !response.id.is_empty(), "INTEGRATION TEST FAILURE: Response ID is empty - not a real API response" );
  assert!( response.r#type == "message", "INTEGRATION TEST FAILURE: Response type incorrect - not a real API response" );
  assert!( response.role == "assistant", "INTEGRATION TEST FAILURE: Response role incorrect - not a real API response" );
  assert!( !response.content.is_empty(), "INTEGRATION TEST FAILURE: Response content is empty - not a real API response" );
}

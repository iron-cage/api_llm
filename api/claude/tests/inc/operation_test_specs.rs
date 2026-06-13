//! Operation Test Specs — OP-01..OP-15
//!
//! Spec source : `tests/docs/operation/001_secret_loading.md`
//! All 15 scenarios in the secret loading operational spec are implemented here
//! as named test functions `test_op_01`..`test_op_15` for CI-auditable traceability.

#[ allow( unused_imports ) ]
use super::*;

/// OP-01: env var success path — `from_env()` returns Ok with key matching env var
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_op_01()
{
  // Load key from workspace, set env var, then verify from_env() reads it correctly
  let ws_client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: workspace must have ANTHROPIC_API_KEY to test from_env() path" );
  let key = ws_client.secret().ANTHROPIC_API_KEY.clone();
  let saved = std::env::var( "ANTHROPIC_API_KEY" ).ok();
  std::env::set_var( "ANTHROPIC_API_KEY", &key );
  let client = the_module::Client::from_env()
    .expect( "INTEGRATION: Client::from_env() must succeed when ANTHROPIC_API_KEY is set" );
  match saved
  {
    Some( k ) => std::env::set_var( "ANTHROPIC_API_KEY", k ),
    None => std::env::remove_var( "ANTHROPIC_API_KEY" ),
  }
  assert_eq!( client.secret().ANTHROPIC_API_KEY, key );
}

/// OP-02: env var absent error — `from_env()` returns Err when `ANTHROPIC_API_KEY` not set
#[ test ]
fn test_op_02()
{
  let saved = std::env::var( "ANTHROPIC_API_KEY" ).ok();
  std::env::remove_var( "ANTHROPIC_API_KEY" );
  let result = the_module::Client::from_env();
  if let Some( key ) = saved
  {
    std::env::set_var( "ANTHROPIC_API_KEY", key );
  }
  assert!( result.is_err(), "from_env() must return Err when ANTHROPIC_API_KEY is absent" );
  assert!( !result.unwrap_err().to_string().is_empty() );
}

/// OP-03: workspace success path — `from_workspace()` returns Ok with valid secrets file
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_op_03()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Client::from_workspace() must succeed with valid secrets file" );
  assert!( client.secret().ANTHROPIC_API_KEY.starts_with( "sk-ant-" ) );
}

/// OP-04: workspace secrets file absent — load from nonexistent secrets filename returns Err
#[ test ]
fn test_op_04()
{
  let result = the_module::Secret::load_from_workspace(
    "ANTHROPIC_API_KEY",
    "-nonexistent-test-secret-file.sh",
  );
  assert!( result.is_err(), "load_from_workspace must return Err when secrets file is absent" );
  assert!( !result.unwrap_err().to_string().is_empty() );
}

/// OP-05: secrets file missing key — file exists in secret/ but has no `ANTHROPIC_API_KEY` entry
#[ test ]
fn test_op_05()
{
  // readme.md is a committed file in secret/ with no ANTHROPIC_API_KEY=value line
  let result = the_module::Secret::load_from_workspace( "ANTHROPIC_API_KEY", "readme.md" );
  assert!( result.is_err(), "load_from_workspace must return Err when key is absent from file" );
  assert!( !result.unwrap_err().to_string().is_empty() );
}

/// OP-06: direct construction valid key — `Secret::new()` + `Client::new()` with valid key
#[ test ]
fn test_op_06()
{
  let valid_key = "sk-ant-api03-valid-key-for-structural-testing-12345".to_string();
  let secret = the_module::Secret::new( valid_key.clone() )
    .expect( "Secret::new() must succeed for valid sk-ant-api03- key with length >= 30" );
  let client = the_module::Client::new( secret );
  assert_eq!( client.secret().ANTHROPIC_API_KEY, valid_key );
}

/// OP-07: direct construction invalid key — `Secret::new()` returns Err for each invalid input
#[ test ]
fn test_op_07()
{
  let invalid_keys = &[ "bad", "", "not-an-anthropic-key", "gpt-4-key", "Bearer sk-ant-" ];
  for &invalid_key in invalid_keys
  {
    let result = the_module::Secret::new( invalid_key.to_string() );
    assert!( result.is_err(), "Secret::new() must return Err for invalid key: {invalid_key:?}" );
    assert!( !result.unwrap_err().to_string().is_empty() );
  }
}

/// OP-08: key format invariant — real workspace key starts with sk-ant- and length > 30
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_op_08()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Client::from_workspace() must succeed" );
  let key = &client.secret().ANTHROPIC_API_KEY;
  assert!( key.starts_with( "sk-ant-" ), "Key must start with sk-ant-, got: {key:?}" );
  assert!( key.len() > 30, "Key length must be > 30, got: {}", key.len() );
}

/// OP-09: env and workspace keys identical — both sources return byte-for-byte same key
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_op_09()
{
  // Load from workspace first, set env var to same key, then verify from_env() matches
  let client_ws = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Client::from_workspace() must succeed with valid secrets file" );
  let saved = std::env::var( "ANTHROPIC_API_KEY" ).ok();
  std::env::set_var( "ANTHROPIC_API_KEY", &client_ws.secret().ANTHROPIC_API_KEY );
  let client_env = the_module::Client::from_env()
    .expect( "INTEGRATION: Client::from_env() must succeed when ANTHROPIC_API_KEY is set" );
  match saved
  {
    Some( k ) => std::env::set_var( "ANTHROPIC_API_KEY", k ),
    None => std::env::remove_var( "ANTHROPIC_API_KEY" ),
  }
  assert_eq!(
    client_env.secret().ANTHROPIC_API_KEY,
    client_ws.secret().ANTHROPIC_API_KEY,
    "Keys from env and workspace must be byte-for-byte identical"
  );
}

/// OP-10: validate secret key present — `validate_anthropic_secret()` returns Ok when key set
#[ test ]
fn test_op_10()
{
  let saved = std::env::var( "ANTHROPIC_API_KEY" ).ok();
  std::env::set_var( "ANTHROPIC_API_KEY", "any-non-empty-value" );
  let result = the_module::environment::validate_anthropic_secret();
  match saved
  {
    Some( key ) => std::env::set_var( "ANTHROPIC_API_KEY", key ),
    None => std::env::remove_var( "ANTHROPIC_API_KEY" ),
  }
  let source = result.expect( "validate_anthropic_secret() must return Ok when key is present" );
  assert!( !source.is_empty(), "returned source identifier must be non-empty" );
}

/// OP-11: validate secret key absent — `validate_anthropic_secret()` returns Err when both sources absent
#[ test ]
fn test_op_11()
{
  let saved_key = std::env::var( "ANTHROPIC_API_KEY" ).ok();
  let saved_ws = std::env::var( "WORKSPACE_PATH" ).ok();
  let saved_cwd = std::env::current_dir().expect( "can get current working directory" );
  std::env::remove_var( "ANTHROPIC_API_KEY" );
  std::env::remove_var( "WORKSPACE_PATH" );
  std::env::set_current_dir( "/tmp" ).expect( "can cd to /tmp for workspace isolation" );
  let result = the_module::environment::validate_anthropic_secret();
  std::env::set_current_dir( &saved_cwd ).expect( "can restore working directory" );
  if let Some( ws ) = saved_ws { std::env::set_var( "WORKSPACE_PATH", ws ); }
  if let Some( key ) = saved_key { std::env::set_var( "ANTHROPIC_API_KEY", key ); }
  assert!( result.is_err(), "validate_anthropic_secret() must return Err when both sources absent" );
  assert!( !result.unwrap_err().to_string().is_empty() );
}

/// OP-12: diagnostic info always callable — `secret_diagnostic_info()` never panics, returns non-empty
#[ test ]
fn test_op_12()
{
  let info = the_module::environment::secret_diagnostic_info();
  assert!( !info.is_empty(), "secret_diagnostic_info() must return non-empty string" );
}

/// OP-13: validate workspace structure valid — `validate_workspace_structure()` returns Ok(path)
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_op_13()
{
  let result = the_module::environment::validate_workspace_structure();
  let path = result.expect( "INTEGRATION: validate_workspace_structure() must succeed in valid workspace" );
  let path_str = path.to_string_lossy();
  assert!( path_str.contains( "-secrets.sh" ), "returned path must reference -secrets.sh, got: {path_str}" );
}

/// OP-14: rollback: unset env var then reload succeeds — `from_workspace()` works after env var removal
#[ cfg( feature = "integration" ) ]
#[ test ]
fn test_op_14()
{
  let saved = std::env::var( "ANTHROPIC_API_KEY" ).ok();
  std::env::set_var( "ANTHROPIC_API_KEY", "incorrect-value-before-rollback" );
  std::env::remove_var( "ANTHROPIC_API_KEY" );
  let result = the_module::Client::from_workspace();
  if let Some( key ) = saved { std::env::set_var( "ANTHROPIC_API_KEY", key ); }
  let client = result.expect( "INTEGRATION: from_workspace() must succeed after env var is unset" );
  assert!( client.secret().ANTHROPIC_API_KEY.starts_with( "sk-ant-" ) );
}

/// OP-15: `from_workspace()` fails when no Cargo workspace reachable
#[ test ]
fn test_op_15()
{
  let saved_ws = std::env::var( "WORKSPACE_PATH" ).ok();
  let saved_cwd = std::env::current_dir().expect( "can get current working directory" );
  std::env::remove_var( "WORKSPACE_PATH" );
  std::env::set_current_dir( "/tmp" ).expect( "can cd to /tmp for workspace isolation" );
  let result = the_module::Client::from_workspace();
  std::env::set_current_dir( &saved_cwd ).expect( "can restore working directory" );
  if let Some( ws ) = saved_ws { std::env::set_var( "WORKSPACE_PATH", ws ); }
  assert!( result.is_err(), "from_workspace() must return Err when no workspace is reachable" );
}

//! Spec traceability: IN-07..IN-12 — Testing Standards
//! Source: `tests/docs/invariant/002_testing_standards.md`

#[ allow( unused_imports ) ]
use super::*;

mod private
{
  use std::path::{ Path, PathBuf };

  pub fn collect_rs_files( dir : &Path ) -> Vec< PathBuf >
  {
    let mut result = vec![];
    let Ok( entries ) = std::fs::read_dir( dir ) else { return result; };
    for entry in entries.flatten()
    {
      let path = entry.path();
      if path.is_dir()
      {
        result.extend( collect_rs_files( &path ) );
      }
      else if path.extension().is_some_and( | ext | ext == "rs" )
      {
        result.push( path );
      }
    }
    result
  }

  pub fn read_tests_files() -> Vec< ( PathBuf, String ) >
  {
    let base = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
    collect_rs_files( &base.join( "tests" ) )
      .into_iter()
      .filter_map( | p | std::fs::read_to_string( &p ).ok().map( | c | ( p, c ) ) )
      .collect()
  }
}

/// IN-07: Integration test functions gated by integration feature
#[ test ]
fn test_in_07()
{
  let files = private::read_tests_files();
  let mut wrong_order = 0usize;
  let mut missing_gate = 0usize;
  for ( path, content ) in &files
  {
    let lines : Vec< &str > = content.lines().collect();
    // Pass 1: detect reversed order (#[tokio::test] before #[cfg(feature="integration")])
    for window in lines.windows( 2 )
    {
      let prev = window[ 0 ].trim();
      let next = window[ 1 ].trim();
      let prev_is_test = prev == "#[ tokio::test ]" || prev == "#[ test ]"
        || prev.contains( "tokio::test" );
      let next_is_gate = next.contains( "cfg( feature = \"integration\"" )
        || next.contains( "cfg(feature = \"integration\"" );
      if prev_is_test && next_is_gate
      {
        eprintln!( "IN-07: wrong gate order in {path:?}" );
        wrong_order += 1;
      }
    }
    // Pass 2: detect missing gate — a #[tokio::test] attribute whose function
    //         calls from_workspace() or from_env() within the next 25 lines is
    //         an integration test and must have cfg(integration) in the 4 lines above.
    //         starts_with("#[") prevents matching tokio::test string literals in Pass 1 code.
    for ( idx, line ) in lines.iter().enumerate()
    {
      if line.trim().starts_with( "#[" ) && line.trim().contains( "tokio::test" )
      {
        // Look ahead up to 25 lines for API credential calls
        // Exclude comment lines to avoid false positives from documentation
        let lookahead_end = ( idx + 25 ).min( lines.len() );
        let calls_api = lines[ idx..lookahead_end ].iter().any( | l |
          ( l.contains( "from_workspace()" ) || l.contains( "from_env()" ) )
          && !l.trim_start().starts_with( "//" )
        );
        if calls_api
        {
          let start = idx.saturating_sub( 4 );
          let has_gate = lines[ start..idx ].iter().any( | l |
            l.contains( "cfg( feature = \"integration\"" )
            || l.contains( "cfg(feature = \"integration\"" )
          );
          if !has_gate
          {
            eprintln!( "IN-07: API-calling test at line {} lacks integration gate in {path:?}", idx + 1 );
            missing_gate += 1;
          }
        }
      }
    }
  }
  assert_eq!( wrong_order, 0,
    "IN-07: found {wrong_order} reversed gate/test pairs — cfg must come before test attribute" );
  assert_eq!( missing_gate, 0,
    "IN-07: found {missing_gate} async test functions without integration feature gate" );
}

/// IN-08: Missing credential causes loud failure
#[ test ]
fn test_in_08()
{
  let saved = std::env::var( "ANTHROPIC_API_KEY" ).ok();
  std::env::remove_var( "ANTHROPIC_API_KEY" );
  let result = the_module::Client::from_env();
  if let Some( k ) = saved { std::env::set_var( "ANTHROPIC_API_KEY", k ); }
  assert!( result.is_err(), "IN-08: from_env() with no key must return Err" );
  let msg = result.unwrap_err().to_string();
  assert!( !msg.is_empty(), "IN-08: error message must be non-empty and actionable" );
}

/// IN-09: No mock HTTP servers in any test file
#[ test ]
fn test_in_09()
{
  // Build patterns at runtime to avoid self-matching when this file is scanned
  let wire_mock = format!( "{}mock", "wire" );
  let http_mock = format!( "{}mock", "http" );
  let files = private::read_tests_files();
  let mut mock_count = 0usize;
  for ( path, content ) in &files
  {
    if content.contains( &wire_mock ) || content.contains( &http_mock )
    {
      eprintln!( "IN-09: mock HTTP server found in {path:?}" );
      mock_count += 1;
    }
  }
  assert_eq!( mock_count, 0,
    "IN-09: found {mock_count} test file(s) importing mock HTTP servers — all tests must use real API" );
}

/// IN-10: No hardcoded API key strings in test files
#[ test ]
fn test_in_10()
{
  // Build pattern at runtime to avoid self-matching when this file is scanned
  let sk_ant_test = format!( "sk-ant{}", "-test-" );
  let files = private::read_tests_files();
  let mut fake_count = 0usize;
  for ( path, content ) in &files
  {
    if content.contains( &sk_ant_test )
    {
      eprintln!( "IN-10: hardcoded test key found in {path:?}" );
      fake_count += 1;
    }
  }
  assert_eq!( fake_count, 0,
    "IN-10: found {fake_count} test file(s) with hardcoded API key test strings" );
}

/// IN-11: No conditional skip logic in integration tests
#[ test ]
fn test_in_11()
{
  // Build patterns at runtime to avoid self-matching when this file is scanned
  let ok_client1 = format!( "if let {}( client )", "Ok" );
  let ok_client2 = format!( "if let {}(client)", "Ok" );
  let files = private::read_tests_files();
  let mut skip_count = 0usize;
  for ( path, content ) in &files
  {
    // These patterns allow test body to be skipped silently when credentials unavailable
    if content.contains( &ok_client1 ) || content.contains( &ok_client2 )
    {
      eprintln!( "IN-11: conditional skip logic found in {path:?}" );
      skip_count += 1;
    }
  }
  assert_eq!( skip_count, 0,
    "IN-11: found {skip_count} test file(s) with conditional skip logic — use .expect() unconditionally" );
}

/// IN-12: No disabled or ignored tests in any test file
#[ test ]
fn test_in_12()
{
  // Build patterns at runtime to avoid self-matching when this file is scanned
  let ignore_spaced = format!( "#[ {} ]", "ignore" );
  let ignore_compact = format!( "#[{}]", "ignore" );
  let commented_test_attr = format!( "// #[ {} ]", "test" );
  let commented_fn = format!( "// {}fn test_", "async " );
  // split "fn " and "test_" to avoid self-match when this file is scanned
  let commented_sync_fn = format!( "// {}test_", "fn " );
  let files = private::read_tests_files();
  let mut ignore_count = 0usize;
  let mut commented_count = 0usize;
  for ( path, content ) in &files
  {
    if content.contains( &ignore_spaced ) || content.contains( &ignore_compact )
    {
      // message uses generic description to prevent self-match in this file
      eprintln!( "IN-12: disabled-test attribute found in {path:?}" );
      ignore_count += 1;
    }
    if content.contains( &commented_test_attr )
      || content.contains( &commented_fn )
      || content.contains( &commented_sync_fn )
    {
      eprintln!( "IN-12: commented-out test found in {path:?}" );
      commented_count += 1;
    }
  }
  assert_eq!( ignore_count, 0,
    "IN-12: found {ignore_count} test file(s) with {ignore_compact} — all tests must be fixed or removed" );
  assert_eq!( commented_count, 0,
    "IN-12: found {commented_count} test file(s) with commented-out test functions — remove or restore" );
}

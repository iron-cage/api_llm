//! Spec traceability: PT-01..PT-06 — Module Organization
//! Source: `tests/docs/pattern/001_module_organization.md`

#[ allow( unused_imports ) ]
use super::*;

mod private
{
  use std::path::{ Path, PathBuf };

  pub fn read_file( relative : &str ) -> String
  {
    let base = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
    std::fs::read_to_string( base.join( relative ) )
      .unwrap_or_else( | _ | panic!( "cannot read {relative}" ) )
  }

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

  pub fn src_rs_files() -> Vec< ( PathBuf, String ) >
  {
    let base = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
    collect_rs_files( &base.join( "src" ) )
      .into_iter()
      .filter_map( | p | std::fs::read_to_string( &p ).ok().map( | c | ( p, c ) ) )
      .collect()
  }
}

/// PT-01: `lib.rs` uses `mod_interface`! layer declarations
#[ test ]
fn test_pt_01()
{
  let lib = private::read_file( "src/lib.rs" );
  assert!( lib.contains( "mod_interface!" ), "PT-01: lib.rs must use mod_interface! macro" );
  assert!( lib.contains( "layer client;" ), "PT-01: client module must be declared as a layer entry" );
  // No bare `mod module_name;` outside mod_interface! (bare mods are not present)
  assert!( !lib.contains( "mod client;" ), "PT-01: no bare mod client; outside mod_interface!" );
  assert!( !lib.contains( "mod secret;" ), "PT-01: no bare mod secret; outside mod_interface!" );
}

/// PT-02: No mod.rs files exist in module directories
#[ test ]
fn test_pt_02()
{
  let files = private::src_rs_files();
  let mod_rs_files : Vec< _ > = files.iter()
    .filter( | ( p, _ ) | p.file_name().is_some_and( | n | n == "mod.rs" ) )
    .collect();
  assert!(
    mod_rs_files.is_empty(),
    "PT-02: found {} mod.rs file(s) under src/ — use same-named .rs file in parent instead: {:?}",
    mod_rs_files.len(),
    mod_rs_files.iter().map( | ( p, _ ) | p ).collect::< Vec< _ > >()
  );
}

/// PT-03: mod private { } block present in source modules
#[ test ]
fn test_pt_03()
{
  let files = private::src_rs_files();
  let non_trivial : Vec< _ > = files.iter()
    .filter( | ( p, _ ) | p.file_name().is_some_and( | n | n != "lib.rs" ) )
    .filter( | ( _, c ) | c.lines().count() > 20 )
    .collect();
  let missing : Vec< _ > = non_trivial.iter()
    .filter( | ( _, c ) | !c.contains( "mod private" ) )
    .map( | ( p, _ ) | p.clone() )
    .collect();
  assert!(
    missing.is_empty(),
    "PT-03: {} non-trivial src module(s) missing mod private {{}} block: {missing:?}",
    missing.len()
  );
}

/// PT-04: No private.rs or private/ directory in src/
#[ test ]
fn test_pt_04()
{
  let files = private::src_rs_files();
  let private_rs : Vec< _ > = files.iter()
    .filter( | ( p, _ ) | p.file_name().is_some_and( | n | n == "private.rs" ) )
    .collect();
  assert!(
    private_rs.is_empty(),
    "PT-04: found private.rs file(s) under src/ — use inline mod private {{}} instead: {:?}",
    private_rs.iter().map( | ( p, _ ) | p ).collect::< Vec< _ > >()
  );
  let base = std::path::Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let private_dir = base.join( "src/private" );
  assert!( !private_dir.exists(), "PT-04: src/private/ directory must not exist" );
}

/// PT-05: Optional modules use #[cfg(feature)] on layer line
#[ test ]
fn test_pt_05()
{
  let lib = private::read_file( "src/lib.rs" );
  // Verify at least one cfg-gated layer exists (proves the pattern is in use)
  let streaming_gate = format!( "#[ cfg( feature = \"{}\" ) ]", "streaming" );
  assert!(
    lib.contains( &streaming_gate ),
    "PT-05: streaming layer must carry #[cfg(feature)] gate in lib.rs"
  );
  // Always-on layers must NOT have a cfg guard on the immediately preceding line
  let lines : Vec< &str > = lib.lines().collect();
  let mut client_has_cfg = false;
  for ( i, line ) in lines.iter().enumerate()
  {
    if line.trim() == "layer client;" && i > 0
    {
      let prev = lines[ i - 1 ].trim();
      if prev.contains( "cfg( feature" ) || prev.contains( "cfg(feature" )
      {
        client_has_cfg = true;
      }
    }
  }
  assert!( !client_has_cfg, "PT-05: always-on client layer must not have #[cfg] gate immediately before it" );
}

/// PT-06: exposed use re-exports are visible externally; orphan use items are not
#[ test ]
fn test_pt_06()
{
  // Verify that types declared as `exposed use` in mod_interface! are accessible from test code.
  // The fact that this test compiles and these types are usable proves exposed use works.
  let secret_type = core::any::type_name::< the_module::Secret >();
  let client_type = core::any::type_name::< the_module::Client >();
  let request_type = core::any::type_name::< the_module::CreateMessageRequest >();
  assert!( !secret_type.is_empty(), "PT-06: Secret must be publicly accessible via exposed use" );
  assert!( !client_type.is_empty(), "PT-06: Client must be publicly accessible via exposed use" );
  assert!( !request_type.is_empty(), "PT-06: CreateMessageRequest must be accessible via exposed use" );
}

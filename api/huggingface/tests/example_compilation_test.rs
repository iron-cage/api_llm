//! Tests for example compilation verification
//!
//! This test suite verifies that all examples compile successfully and are
//! properly configured in Cargo.toml.
//!
//! ## Bug History
//!
//! ### Issue #2: Example Not Registered in Cargo.toml (issue-manual-testing-002)
//!
//! **Root Cause:**
//! The `chat_cached_interactive.rs` example file existed in the `examples/` directory
//! but was not registered as an example target in `Cargo.toml`. This meant the example
//! could not be compiled individually, run with `cargo run --example`, or included in
//! `cargo build --examples`.
//!
//! **Why Not Caught:**
//! - No automated test verifying all example files are registered in Cargo.toml
//! - `cargo build --examples` silently skips unregistered files
//! - No CI check ensuring example files match Cargo.toml entries
//! - Manual testing didnt attempt to run this specific example
//!
//! **Fix Applied:**
//! Added to Cargo.toml after line 162:
//! ```toml
//! [[example]]
//! name = "chat_cached_interactive"
//! path = "examples/chat_cached_interactive.rs"
//! required-features = ["full", "caching"]
//! ```
//!
//! This change:
//! - Makes the example discoverable via `cargo run --example`
//! - Includes the example in `cargo build --examples`
//! - Properly documents required features
//! - Makes the example accessible to users
//!
//! **Prevention:**
//! - Added this test suite to verify all .rs files in examples/ are registered
//! - Added test to verify example names in Cargo.toml match actual files
//! - Added test to verify examples compile successfully
//!
//! **Pitfall:**
//! Cargo silently ignores unregistered example files. When adding new examples,
//! always add a corresponding `[[example]]` entry in Cargo.toml, otherwise the
//! example will exist in the repo but be unusable. Use `cargo run --example < name >`
//! to verify the example is properly registered.

use std::{ process::Command, fs, path::Path };

/// Test that all example files in examples/ directory are registered in Cargo.toml
///
/// This prevents the scenario where example files exist but cant be run.
#[ test ]
fn test_all_examples_registered()
{
  let examples_dir = Path::new( "examples" );
  let cargo_toml = fs::read_to_string( "Cargo.toml" )
  .expect( "Failed to read Cargo.toml" );

  let mut example_files = Vec::new();

  // Collect all .rs files in examples/ directory
  for entry in fs::read_dir( examples_dir ).expect( "Failed to read examples directory" )
  {
  let entry = entry.expect( "Failed to read directory entry" );
  let path = entry.path();

  if path.extension().and_then( | s | s.to_str() ) == Some( "rs" )
  {
      let filename = path.file_name().expect( "[test_all_examples_compile] Path should have filename component - check examples directory structure" ).to_str().expect( "[test_all_examples_compile] Filename should be valid UTF-8 - check filesystem encoding" );
      example_files.push( ( filename.to_string(), path ) );
  }
  }

  // Verify each example file is registered in Cargo.toml
  let mut unregistered = Vec::new();

  for ( filename, _path ) in &example_files
  {
  let expected_path = format!( "examples/{filename}" );

  if !cargo_toml.contains( &expected_path )
  {
      unregistered.push( filename.clone() );
  }
  }

  assert!(
  unregistered.is_empty(),
  "Found {} unregistered examples in Cargo.toml:\n{}\n\n\
     These examples exist but cannot be run. Add [[example]] entries for them.",
  unregistered.len(),
  unregistered.join( "\n" )
  );
}

/// Test that all examples listed in Cargo.toml actually exist
///
/// This prevents broken Cargo.toml entries referencing non-existent files.
#[ test ]
fn test_cargo_toml_examples_exist()
{
  let cargo_toml = fs::read_to_string( "Cargo.toml" )
  .expect( "Failed to read Cargo.toml" );

  let mut missing = Vec::new();

  // Parse [[example]] entries from Cargo.toml
  for line in cargo_toml.lines()
  {
  if line.trim().starts_with( "path = \"examples/" )
  {
      // Extract path from : path = "examples/foo.rs"
      if let Some( start ) = line.find( "\"examples/" )
      {
  if let Some( end ) = line[ start + 1.. ].find( '"' )
  {
          let path_str = &line[ start + 1..start + 1 + end ];
          let path = Path::new( path_str );

          if !path.exists()
          {
      missing.push( path_str.to_string() );
          }
  }
      }
  }
  }

  assert!(
  missing.is_empty(),
  "Found {} Cargo.toml entries for non-existent examples:\n{}",
  missing.len(),
  missing.join( "\n" )
  );
}

/// Test that examples directory contains expected number of examples
///
/// This is a sanity check to catch accidental deletions or missing examples.
#[ test ]
fn test_expected_example_count()
{
  let examples_dir = Path::new( "examples" );
  let count = fs::read_dir( examples_dir )
  .expect( "Failed to read examples directory" )
  .filter_map( core::result::Result::ok )
  .filter( | entry | {
      entry.path().extension().and_then( | s | s.to_str() ) == Some( "rs" )
  } )
  .count();

  // We expect at least 14 examples (as of 2025-11-06)
  // If this number decreases, something was deleted
  assert!(
  count >= 14,
  "Expected at least 14 examples, found {count}. Examples may have been deleted."
  );
}

/// Test that examples compile with all features enabled
///
/// This ensures examples dont have compilation errors.
#[ test ]
fn test_examples_compile_all_features()
{
  let output = Command::new( "cargo" )
  .args( [ "build", "--examples", "--all-features" ] )
  .env( "RUSTFLAGS", "-D warnings" )
  .output()
  .expect( "Failed to run cargo build" );

  if !output.status.success()
  {
  let stderr = String::from_utf8_lossy( &output.stderr );
  panic!(
      "Examples failed to compile with --all-features:\n{stderr}"
  );
  }
}

/// Test that example names follow naming conventions
///
/// Examples should use `snake_case` naming.
#[ test ]
fn test_example_naming_convention()
{
  let cargo_toml = fs::read_to_string( "Cargo.toml" )
  .expect( "Failed to read Cargo.toml" );

  let mut violations = Vec::new();

  // Parse example names from Cargo.toml
  for line in cargo_toml.lines()
  {
  if line.trim().starts_with( "name = \"" )
  {
      if let Some( start ) = line.find( '"' )
      {
  if let Some( end ) = line[ start + 1.. ].find( '"' )
  {
          let name = &line[ start + 1..start + 1 + end ];

          // Check if name is in snake_case (lowercase, underscores only)
          if name.chars().any( | c | c.is_uppercase() || c == '-' )
          {
      violations.push( format!(
              "{name}: Example names should use snake_case, not kebab-case or camelCase"
      ) );
          }
  }
      }
  }
  }

  assert!(
  violations.is_empty(),
  "Found example naming convention violations:\n{}",
  violations.join( "\n" )
  );
}

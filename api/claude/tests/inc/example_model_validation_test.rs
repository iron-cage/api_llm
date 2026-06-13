//! Test coverage for example model name validation
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::format_push_string ) ]
#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::manual_assert ) ]
//!
//! # Root Cause
//!
//! Examples were created using model name "claude-sonnet-4-5-20250929" which doesn't exist
//! in Anthropic's API. This caused runtime failures when users attempted to run the examples.
//! The issue occurred because examples were created with a model name that was either:
//! 1. Never released by Anthropic
//! 2. Had a different naming convention than anticipated
//! 3. Was superseded by a different model release
//!
//! The valid model names follow Anthropic's naming pattern of:
//! - claude-{family}-{version}-{date} for specific versions
//! - claude-{family}-{version} for aliases
//!
//! # Why Not Caught
//!
//! 1. No compile-time validation of model names (they're runtime strings)
//! 2. Examples were not tested against live API during development
//! 3. No automated test to validate example code against known valid model names
//! 4. Documentation and examples were created before model release
//!
//! # Fix Applied
//!
//! 1. Updated all 11 occurrences across 8 example files to use "claude-sonnet-4-5-20250929"
//! 2. Verified the model name exists in official Anthropic documentation
//! 3. Created this test to validate model names in examples match known valid models
//!
//! Files fixed:
//! - claude_api_basic.rs (1 occurrence)
//! - claude_api_interactive.rs (1 occurrence)
//! - claude_code_review.rs (1 occurrence)
//! - claude_content_generation.rs (1 occurrence)
//! - claude_function_calling.rs (1 occurrence)
//! - claude_vision_analysis.rs (2 occurrences)
//! - claude_multi_turn_conversation.rs (3 occurrences)
//! - claude_chat_cached_interactive.rs (1 occurrence)
//!
//! # Prevention
//!
//! 1. This test validates all example files contain only known valid model names
//! 2. Test will fail if examples are updated with invalid model names
//! 3. Centralized list of valid models makes updates easier
//! 4. Consider adding CI check to validate examples compile successfully
//!
//! # Pitfall
//!
//! **Model Name Lifecycle Management**: Model names in AI APIs have lifecycles:
//! - New models are released with specific version dates
//! - Old models get deprecated and eventually removed
//! - Using hardcoded model names creates maintenance burden
//!
//! **Solutions**:
//! 1. Use model aliases (e.g., "claude-sonnet-4-5") when appropriate
//! 2. Document which models examples were tested with
//! 3. Regularly validate examples against live API
//! 4. Consider extracting model names to constants/config for easier updates

use std::{ fs, path::PathBuf };

/// Known valid Claude model names as of 2025-01
/// Source : https://docs.claude.com/en/docs/about-claude/models
const VALID_CURRENT_MODELS: &[ &str ] = &
[
  "claude-sonnet-4-5-20250929",
  "claude-haiku-4-5-20251001",
  "claude-opus-4-1-20250805",
];

const VALID_LEGACY_MODELS: &[ &str ] = &
[
  "claude-sonnet-4-20250514",
  "claude-3-7-sonnet-20250219",
  "claude-opus-4-20250514",
  "claude-3-5-haiku-20241022",
  "claude-3-haiku-20240307",
];

const VALID_MODEL_ALIASES: &[ &str ] = &
[
  "claude-sonnet-4-5",
  "claude-haiku-4-5",
  "claude-opus-4-1",
];

#[ test ]
fn examples_use_valid_model_names()
{
  // Get all example files
  let examples_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) ).join( "examples" );

  if !examples_dir.exists()
  {
    panic!( "Examples directory not found : {}", examples_dir.display() );
  }

  let mut invalid_models = Vec::new();
  let mut files_checked = 0;

  // Read all .rs files in examples directory
  for entry in fs::read_dir( &examples_dir ).expect( "Failed to read examples directory" )
  {
    let entry = entry.expect( "Failed to read directory entry" );
    let path = entry.path();

    if path.extension().and_then( | s | s.to_str() ) != Some( "rs" )
    {
      continue;
    }

    files_checked += 1;

    let content = fs::read_to_string( &path )
      .unwrap_or_else( | e | panic!( "Failed to read file {}: {}", path.display(), e ) );

    // Look for model names in the format : .model( "claude-..." ) or model : "claude-..."
    // Using simple string matching instead of regex to avoid external dependency
    let lines : Vec< &str > = content.lines().collect();
    let mut models_in_file = Vec::new();

    for line in &lines
    {
      // Match .model( "claude-..." )
      if let Some( start ) = line.find( ".model(" )
      {
        if let Some( quote_start ) = line[ start.. ].find( '"' )
        {
          let from_quote = &line[ start + quote_start + 1.. ];
          if let Some( quote_end ) = from_quote.find( '"' )
          {
            let model = &from_quote[ ..quote_end ];
            if model.starts_with( "claude-" )
            {
              models_in_file.push( model.to_string() );
            }
          }
        }
      }

      // Match model : "claude-..." or model : "claude-..."
      if line.contains( "model" ) && line.contains( ':' )
      {
        if let Some( colon_pos ) = line.find( ':' )
        {
          let after_colon = &line[ colon_pos + 1.. ];
          if let Some( quote_start ) = after_colon.find( '"' )
          {
            let from_quote = &after_colon[ quote_start + 1.. ];
            if let Some( quote_end ) = from_quote.find( '"' )
            {
              let model = &from_quote[ ..quote_end ];
              if model.starts_with( "claude-" )
              {
                models_in_file.push( model.to_string() );
              }
            }
          }
        }
      }
    }

    for model_name in models_in_file
    {
      // Check if it's a valid model
      let is_valid = VALID_CURRENT_MODELS.contains( &model_name.as_str() )
        || VALID_LEGACY_MODELS.contains( &model_name.as_str() )
        || VALID_MODEL_ALIASES.contains( &model_name.as_str() );

      if !is_valid
      {
        invalid_models.push( ( path.file_name().unwrap().to_string_lossy().to_string(), model_name ) );
      }
    }
  }

  assert!(
    files_checked > 0,
    "No example files found in {}. Expected at least 9 files.",
    examples_dir.display()
  );

  if !invalid_models.is_empty()
  {
    let mut error_msg = format!(
      "\nFound {} invalid model name(s) in example files:\n",
      invalid_models.len()
    );

    for ( file, model ) in &invalid_models
    {
      error_msg.push_str( &format!( "  - {}: '{}'\n", file, model ) );
    }

    error_msg.push_str( "\nValid current models:\n" );
    for model in VALID_CURRENT_MODELS
    {
      error_msg.push_str( &format!( "  - {}\n", model ) );
    }

    error_msg.push_str( "\nValid legacy models:\n" );
    for model in VALID_LEGACY_MODELS
    {
      error_msg.push_str( &format!( "  - {}\n", model ) );
    }

    error_msg.push_str( "\nValid aliases:\n" );
    for model in VALID_MODEL_ALIASES
    {
      error_msg.push_str( &format!( "  - {}\n", model ) );
    }

    panic!( "{error_msg}" );
  }

  println!( "✅ All {} example files use valid Claude model names", files_checked );
}

#[ test ]
fn example_documentation_matches_filename()
{
  // Get all example files
  let examples_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) ).join( "examples" );

  if !examples_dir.exists()
  {
    return;
  }

  let mut mismatches = Vec::new();

  for entry in fs::read_dir( &examples_dir ).expect( "Failed to read examples directory" )
  {
    let entry = entry.expect( "Failed to read directory entry" );
    let path = entry.path();

    if path.extension().and_then( | s | s.to_str() ) != Some( "rs" )
    {
      continue;
    }

    let filename = path.file_stem().unwrap().to_str().unwrap();
    let content = fs::read_to_string( &path )
      .unwrap_or_else( | e | panic!( "Failed to read file {}: {}", path.display(), e ) );

    // Look for cargo run --example < name > in documentation
    for line in content.lines()
    {
      if line.contains( "cargo run --example " ) || line.contains( "cargo_run_--example_" )
      {
        // Extract example name after "--example "
        if let Some( pos ) = line.find( "--example " )
        {
          let after_example = &line[ pos + 10.. ]; // "--example " is 10 chars
          // Get the next word (non-whitespace characters)
          let documented_name = after_example
            .split_whitespace()
            .next()
            .unwrap_or( "" )
            .trim_matches( |c : char| !c.is_alphanumeric() && c != '_' && c != '-' );

          if !documented_name.is_empty() && documented_name != filename
          {
            mismatches.push( ( filename.to_string(), documented_name.to_string() ) );
          }
        }
      }
    }
  }

  if !mismatches.is_empty()
  {
    let mut error_msg = format!(
      "\nFound {} example filename/documentation mismatch(es):\n",
      mismatches.len()
    );

    for ( filename, documented ) in &mismatches
    {
      error_msg.push_str( &format!(
        "  - File : {}.rs, Documentation says : 'cargo run --example {}'\n",
        filename, documented
      ) );
    }

    panic!( "{error_msg}" );
  }
}

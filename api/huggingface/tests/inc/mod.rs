//! Shared test utilities and implementations

#[ allow( dead_code, missing_docs ) ]
pub mod ai_tutor_shared;

#[ allow( dead_code, missing_docs ) ]
pub mod sentiment_analysis_shared;

#[ allow( dead_code, missing_docs ) ]
pub mod code_assistant_shared;

#[ allow( dead_code, missing_docs ) ]
pub mod qa_system_shared;

#[ allow( dead_code, missing_docs ) ]
pub mod translation_shared;

// ============================================================================
// Canonical integration credential helpers
// ============================================================================

/// Load `HuggingFace` API key for integration tests; panics with a clear message if missing.
#[ allow( dead_code ) ]
pub fn get_api_key_for_integration() -> String
{
  use workspace_tools as workspace;
  let ws = workspace::workspace()
    .expect( "Failed to access workspace — required for integration tests" );
  let secrets = ws.load_secrets_from_file( "-secrets.sh" )
    .expect( "Failed to load secret/-secrets.sh — required for integration tests" );
  secrets.get( "HUGGINGFACE_API_KEY" )
    .expect( "HUGGINGFACE_API_KEY not found in secret/-secrets.sh — get your token from https://huggingface.co/settings/tokens" )
    .clone()
}

/// Load `HuggingFace` API key for tests that can run without credentials.
/// Returns `None` gracefully when the key is absent so tests can skip (not panic).
#[ allow( dead_code ) ]
pub fn get_api_key_for_testing() -> Option< String >
{
  use workspace_tools as workspace;
  let ws = workspace::workspace().ok()?;
  let secrets = ws.load_secrets_from_file( "-secrets.sh" ).ok()?;
  secrets.get( "HUGGINGFACE_API_KEY" ).cloned()
}

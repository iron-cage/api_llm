//! Tests for `SyncClient` construction and blocking semantics.
//!
//! Unit tests verify that `SyncClient::new()` succeeds when a tokio runtime
//! can be created — the only hard requirement before any request is made.
//! Live HTTP tests are gated behind the `integration` feature and require the
//! `KIE_API_KEY` environment variable; they are excluded from CI by default.

#![ cfg( feature = "enabled" ) ]

// ------------------------------------------------------------------ //
//  Unit tests
// ------------------------------------------------------------------ //

/// `SyncClient::new(client)` must succeed when a tokio runtime is available.
///
/// The sync client wraps an async `Client` in a dedicated tokio runtime. If
/// runtime creation fails (e.g. nested runtime conflict), the constructor
/// returns an error instead of panicking, giving callers a chance to recover.
#[ cfg( feature = "sync_api" ) ]
#[ test ]
fn sync_client_new_succeeds()
{
  use api_openai_compatible::{ Client, SyncClient, OpenAiCompatEnvironmentImpl };

  let env = OpenAiCompatEnvironmentImpl::new( "sk-test-key" )
    .expect( "environment construction must succeed with a non-empty key" );

  let client = Client::build( env )
    .expect( "Client::build() must succeed" );

  let _sync_client = SyncClient::new( client )
    .expect( "SyncClient::new() must succeed when a tokio runtime is available" );
}

// ------------------------------------------------------------------ //
//  Integration tests (feature = "integration", requires KIE_API_KEY)
// ------------------------------------------------------------------ //

/// Stub for future live chat round-trip test against the KIE.ai API.
///
/// This test body will be populated in Phase 6 (`iron_creator_kie` integration).
/// It is compiled only when the `integration` feature is enabled, keeping it
/// out of every standard test run.
#[ cfg( feature = "integration" ) ]
#[ test ]
fn live_chat_roundtrip()
{
  // Phase 6: implement live API integration test.
  // Requires: `KIE_API_KEY` environment variable and `integration` feature.
}

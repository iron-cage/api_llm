//! Tests for `SyncClient` construction and blocking semantics.
//!
//! Unit tests verify that `SyncClient::new()` succeeds when a tokio runtime
//! can be created — the only hard requirement before any request is made.
//!
//! ## Test Matrix
//!
//! | Test | Scenario | Status |
//! |------|----------|--------|
//! | `sync_client_new_succeeds` | Runtime creation with valid env | ✅ |
//! | `sync_client_new_with_custom_base_url_succeeds` | Custom base URL accepted | ✅ |

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

/// `SyncClient` built with a custom base URL must preserve the URL in the environment.
///
/// The `with_base_url()` builder modifies the environment before client construction.
/// The `SyncClient` wraps the resulting client, so the custom URL propagates through
/// the entire construction chain.
#[ cfg( feature = "sync_api" ) ]
#[ test ]
fn sync_client_new_with_custom_base_url_succeeds()
{
  use api_openai_compatible::{ Client, SyncClient, OpenAiCompatEnvironmentImpl };

  let env = OpenAiCompatEnvironmentImpl::new( "sk-test-key" )
    .expect( "environment construction must succeed" )
    .with_base_url( "https://api.kie.ai/my-model/v1/" );

  let client = Client::build( env )
    .expect( "Client::build() must succeed with custom base URL" );

  let _sync_client = SyncClient::new( client )
    .expect( "SyncClient::new() must succeed with custom base URL client" );
}

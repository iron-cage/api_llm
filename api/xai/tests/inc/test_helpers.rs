use api_xai::{ Client, Secret, XaiEnvironmentImpl };
use core::time::Duration;

/// Timeout for integration tests (2 minutes).
///
/// Integration tests interact with real external APIs which can be slow due to:
/// - Network latency
/// - API processing time
/// - Rate limiting and queueing
/// - Geographic distance to API servers
///
/// The default 30s timeout is too aggressive for real-world API conditions.
/// 120s provides resilience against transient slowness while still catching actual hangs.
const INTEGRATION_TEST_TIMEOUT_SECS : u64 = 120;

/// Creates a test client with credentials from environment.
///
/// This helper loads the XAI API key from environment variables or workspace secrets
/// and creates a fully configured client for integration testing.
///
/// # Configuration
///
/// - **Timeout**: 120 seconds (vs default 30s) to handle API latency
/// - **Base URL**: Default XAI API endpoint
/// - **Auth**: Bearer token from environment/workspace secrets
///
/// # Panics
///
/// Panics with a descriptive message if credentials cannot be loaded. This is intentional
/// to ensure tests fail loudly when credentials are unavailable (NO SILENT FALLBACKS).
///
/// # Examples
///
/// ```no_run
/// # #[ cfg( feature = "integration" ) ]
/// # {
/// use test_helpers::create_test_client;
///
/// #[ tokio::test ]
/// async fn test_something() {
///   let client = create_test_client();
///   // ... use client for testing
/// }
/// # }
/// ```
pub fn create_test_client() -> Client< XaiEnvironmentImpl >
{
  let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )
    .expect(
      "XAI_API_KEY is required for integration tests. \
       Please set the environment variable or add to workspace secrets. \
       Integration tests MUST fail if credentials are unavailable."
    );

  let env = XaiEnvironmentImpl::new( secret )
    .expect( "Failed to create environment" )
    .with_timeout( Duration::from_secs( INTEGRATION_TEST_TIMEOUT_SECS ) );

  Client::build( env )
    .expect( "Failed to build client" )
}

/// Tries to create a test client, returning None if credentials are unavailable.
///
/// This is useful for conditional test skipping, but should be used sparingly.
/// Prefer `create_test_client()` which fails loudly.
///
/// # Configuration
///
/// Uses the same configuration as `create_test_client()`:
/// - **Timeout**: 120 seconds for API latency resilience
/// - **Base URL**: Default XAI API endpoint
///
/// # Examples
///
/// ```no_run
/// use test_helpers::try_create_test_client;
///
/// #[ tokio::test ]
/// async fn test_optional() {
///   let Some( client ) = try_create_test_client() else {
///     println!( "Skipping test : credentials not available" );
///     return;
///   };
///
///   // ... use client
/// }
/// ```
#[ allow( dead_code ) ]  // Helper for integration tests, may not be used in all test runs
pub fn try_create_test_client() -> Option< Client< XaiEnvironmentImpl > >
{
  let secret = Secret::load_with_fallbacks( "XAI_API_KEY" ).ok()?;
  let env = XaiEnvironmentImpl::new( secret ).ok()?
    .with_timeout( Duration::from_secs( INTEGRATION_TEST_TIMEOUT_SECS ) );
  Client::build( env ).ok()
}

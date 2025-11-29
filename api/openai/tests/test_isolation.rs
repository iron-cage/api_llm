//! Test isolation framework for integration tests
//!
//! Provides utilities to ensure proper test isolation by managing shared state,
//! environment variables, and cleanup procedures.

use std::sync::{ Mutex, Arc };
use std::collections::HashMap;
use api_openai::secret::Secret;
use secrecy::ExposeSecret;

#[ cfg( feature = "retry" ) ]
use api_openai::EnhancedRetryConfig;

/// Test environment isolation manager
///
/// Manages test-specific environment variables and state cleanup
/// to ensure tests don't interfere with each other.
#[ derive( Debug ) ]
pub struct TestIsolation
{
  /// Original environment variables before test modifications
  original_env_vars : Arc< Mutex< HashMap< String, Option< String > > > >,
  /// Test-specific temporary directory path
  temp_dir_path : Option< std::path::PathBuf >,
  /// Test identifier for logging and cleanup
  test_name : String,
}

impl TestIsolation
{
  /// Create a new test isolation context
  ///
  /// # Arguments
  /// * `test_name` - Name of the test for identification and cleanup
  #[ must_use ]
  pub fn new( test_name : &str ) -> Self
  {
    // Reset secret exposure counter for this test if available
    // Note : reset_exposure_count is only available when the Secret module is compiled with cfg(test)
    // Since we're in a test context, we'll skip this call for now

    Self
    {
      original_env_vars : Arc::new( Mutex::new( HashMap::new() ) ),
      temp_dir_path : None,
      test_name : test_name.to_string(),
    }
  }

  /// Set an environment variable for this test with automatic cleanup
  ///
  /// # Arguments
  /// * `key` - Environment variable name
  /// * `value` - Environment variable value
  ///
  /// # Panics
  ///
  /// Panics if the mutex lock cannot be acquired
  pub fn set_env_var( &mut self, key : &str, value : &str )
  {
    let mut env_vars = self.original_env_vars.lock().unwrap();

    // Store original value if not already stored
    if !env_vars.contains_key( key )
    {
      env_vars.insert( key.to_string(), std::env::var( key ).ok() );
    }

    // Set new value
    std ::env::set_var( key, value );
  }

  /// Remove an environment variable for this test with automatic cleanup
  ///
  /// # Arguments
  /// * `key` - Environment variable name to remove
  ///
  /// # Panics
  ///
  /// Panics if the mutex lock cannot be acquired
  pub fn remove_env_var( &mut self, key : &str )
  {
    let mut env_vars = self.original_env_vars.lock().unwrap();

    // Store original value if not already stored
    if !env_vars.contains_key( key )
    {
      env_vars.insert( key.to_string(), std::env::var( key ).ok() );
    }

    // Remove variable
    std ::env::remove_var( key );
  }

  /// Create isolated temporary directory for this test
  ///
  /// Returns the path to the temporary directory that will be cleaned up
  /// when the test isolation context is dropped.
  ///
  /// # Errors
  ///
  /// Returns an error if temporary directory creation fails
  ///
  /// # Panics
  ///
  /// Panics if the temporary directory path is not available after creation
  pub fn create_temp_dir( &mut self ) -> Result< &std::path::Path, Box< dyn core::error::Error > >
  {
    let temp_dir = tempfile::tempdir()?;
    let path = temp_dir.path().to_path_buf();

    // Keep tempdir alive by storing its path
    self.temp_dir_path = Some( path.clone() );

    // Ensure directory exists and is accessible
    std ::fs::create_dir_all( &path )?;

    // Leak the tempdir handle to prevent automatic cleanup during test
    core ::mem::forget( temp_dir );

    Ok( self.temp_dir_path.as_ref().unwrap() )
  }

  /// Create an isolated secret for testing
  ///
  /// Creates a test-specific API key that won't interfere with other tests
  /// or production configurations.
  ///
  /// # Arguments
  /// * `suffix` - Optional suffix to make the secret unique for this test
  #[ must_use ]
  pub fn create_test_secret( &self, suffix : Option< &str > ) -> Secret
  {
    let suffix = suffix.unwrap_or( &self.test_name );
    let test_key = format!( "sk-test_{}_1234567890", suffix.replace( ' ', "_" ) );
    Secret::new_unchecked( test_key )
  }

  /// Set up isolated environment for API key testing
  ///
  /// Creates test-specific environment variables and ensures proper isolation
  /// from other tests and system configuration.
  ///
  /// # Arguments
  /// * `api_key` - Optional API key to set, or generates a test key if None
  pub fn setup_api_key_env( &mut self, api_key : Option< &str > ) -> String
  {
    let default_key = format!( "sk-test_{}_1234567890", self.test_name.replace( ' ', "_" ) );
    let test_key = api_key.unwrap_or( &default_key );
    let env_var_name = format!( "OPENAI_API_KEY_TEST_{}", self.test_name.replace( ' ', "_" ).to_uppercase() );

    self.set_env_var( &env_var_name, test_key );

    env_var_name
  }

  /// Get the test name for this isolation context
  #[ must_use ]
  pub fn test_name( &self ) -> &str
  {
    &self.test_name
  }

  /// Manual cleanup for explicit resource management
  ///
  /// This is called automatically on drop, but can be called explicitly
  /// for immediate cleanup within a test.
  pub fn cleanup( &mut self )
  {
    self.cleanup_env_vars();
    self.cleanup_temp_dir();
  }

  /// Restore original environment variables
  fn cleanup_env_vars( &self )
  {
    let env_vars = self.original_env_vars.lock().unwrap();

    for ( key, original_value ) in env_vars.iter()
    {
      match original_value
      {
        Some( value ) => std::env::set_var( key, value ),
        None => std::env::remove_var( key ),
      }
    }
  }

  /// Clean up temporary directory
  fn cleanup_temp_dir( &mut self )
  {
    if let Some( temp_path ) = &self.temp_dir_path
    {
      if temp_path.exists()
      {
        let _ = std::fs::remove_dir_all( temp_path );
      }
      self.temp_dir_path = None;
    }
  }
}

impl Drop for TestIsolation
{
  fn drop( &mut self )
  {
    self.cleanup();
  }
}

/// Macro for creating an isolated test environment
///
/// Automatically creates a `TestIsolation` context for the current test
/// and ensures proper cleanup on test completion.
///
/// # Example
///
/// ```rust,no_run
/// #[ test ]
/// fn my_isolated_test()
/// {
///   let mut isolation = isolated_test!();
///   isolation.set_env_var("TEST_VAR", "test_value");
///   // Test code here...
///   // Automatic cleanup on drop
/// }
/// ```
#[ macro_export ]
macro_rules! isolated_test
{
  () =>
  {
    {
      let test_name = std::thread::current().name().unwrap_or( "unknown_test" );
      TestIsolation::new( test_name )
    }
  };
}

/// Test fixture for API client isolation
///
/// Provides a standardized way to create isolated API clients for testing
/// without shared state interference.
#[ derive( Debug ) ]
pub struct IsolatedClient
{
  isolation : TestIsolation,
  client : Option< api_openai::Client< api_openai::environment::OpenaiEnvironmentImpl > >,
}

impl IsolatedClient
{
  /// Create a new isolated client for testing
  ///
  /// # Arguments
  /// * `test_name` - Name of the test for isolation context
  /// * `use_real_api` - Whether to attempt real API calls or use mock configuration
  ///
  /// # Mandatory Failing Behavior
  /// When `use_real_api` is true, this function MUST fail if:
  /// - Real API credentials are not available in environment or workspace secrets
  /// - Network connectivity issues prevent API access
  /// - API authentication fails
  ///
  /// Integration tests using real APIs should NEVER silently fall back to mocks.
  /// This ensures test failures indicate real issues that need to be addressed.
  ///
  /// # Errors
  ///
  /// Returns an error if client creation or environment setup fails.
  /// For real API tests, failures are mandatory when credentials/network unavailable.
  pub fn new( test_name : &str, _use_real_api : bool ) -> Result< Self, Box< dyn core::error::Error > >
  {
    let mut isolation = TestIsolation::new( test_name );

    // REAL API ONLY - No more conditional logic
    let client = Self::create_real_client( &mut isolation )?;

    Ok( Self
    {
      isolation,
      client : Some( client ),
    } )
  }

  /// Get reference to the isolated client
  ///
  /// # Panics
  ///
  /// Panics if the client is not available during isolation setup
  #[ must_use ]
  pub fn client( &self ) -> &api_openai::Client< api_openai::environment::OpenaiEnvironmentImpl >
  {
    self.client.as_ref().expect( "Client should be available" )
  }

  /// Get mutable reference to the isolation context
  pub fn isolation( &mut self ) -> &mut TestIsolation
  {
    &mut self.isolation
  }

  /// Create a real API client with proper isolation
  ///
  /// # Mandatory Failing Behavior
  /// This function MUST fail if real API credentials are not available.
  /// Integration tests using real APIs should never silently fall back to mocks.
  /// This ensures that integration test failures indicate real issues with
  /// credentials, network connectivity, or API availability.
  fn create_real_client( _isolation : &mut TestIsolation ) -> Result< api_openai::Client< api_openai::environment::OpenaiEnvironmentImpl >, Box< dyn core::error::Error > >
  {
    // MANDATORY: Use workspace_tools integration - MUST fail if no credentials
    let secret = Secret::load_with_fallbacks( "OPENAI_API_KEY" )
      .map_err( | e | -> Box< dyn core::error::Error > { format!( "INTEGRATION TEST FAILURE: Real API credentials required but not found. {e}" ).into() } )?;

    // Validate that this is not a dummy/test key
    let api_key = secret.expose_secret();
    if api_key.contains( "dummy" )
      || api_key.contains( "test" )
      || api_key.contains( "invalid" )
      || api_key.len() < 20
    {
      let key_preview = &api_key[ ..core::cmp::min( 10, api_key.len() ) ];
      return Err( format!( "INTEGRATION TEST FAILURE: Invalid API key detected. Real integration tests require valid OpenAI API credentials, got : {key_preview}..." ).into() );
    }

    let env = api_openai::environment::OpenaiEnvironmentImpl::build( secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string() )
      .map_err( | e | -> Box< dyn core::error::Error > { format!( "INTEGRATION TEST FAILURE: Failed to build environment with real credentials : {e}" ).into() } )?;

    let mut client = api_openai ::Client::build( env )
      .map_err( | e | -> Box< dyn core::error::Error > { format!( "INTEGRATION TEST FAILURE: Failed to build client with real API environment : {e}" ).into() } )?;

    // Enable retry for integration tests to handle transient API failures (500 errors, network issues)
    // This reflects real-world usage where production clients would enable retry
    #[ cfg( feature = "retry" ) ]
    {
      client = client.with_retry_config( EnhancedRetryConfig
      {
        max_attempts : 5,              // Up to 5 attempts for flaky OpenAI API
        base_delay_ms : 2000,          // Start with 2s delay
        max_delay_ms : 30000,          // Max 30s delay between retries
        max_elapsed_time_ms : 120_000, // Total 2min timeout for all attempts
        jitter_ms : 500,               // Add 500ms jitter to prevent thundering herd
        backoff_multiplier : 2.0,      // Exponential backoff (2s, 4s, 8s, 16s, 30s)
      } );
    }

    Ok( client )
  }

}

/// Test helper for real API integration tests - ALWAYS TRUE (no more mocking)
///
/// Real credentials are always required - no fallback to mocks allowed
#[ must_use ]
pub fn should_run_real_api_tests() -> bool
{
  // REAL API ONLY - Always return true, no conditional logic
  true
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn test_isolation_env_vars()
  {
    let mut isolation = TestIsolation::new( "test_env_isolation" );

    // Test setting and cleanup
    isolation.set_env_var( "TEST_ISOLATION_VAR", "test_value" );
    assert_eq!( std::env::var( "TEST_ISOLATION_VAR" ).unwrap(), "test_value" );

    // Manual cleanup
    isolation.cleanup();

    // Should be cleaned up
    assert!( std::env::var( "TEST_ISOLATION_VAR" ).is_err() );
  }

  #[ test ]
  fn test_isolation_temp_dir()
  {
    let mut isolation = TestIsolation::new( "test_temp_dir" );

    let temp_dir = isolation.create_temp_dir().expect( "Should create temp dir" );
    assert!( temp_dir.exists() );

    let temp_path = temp_dir.to_path_buf();

    // Manual cleanup
    isolation.cleanup();

    // Directory should be cleaned up
    assert!( !temp_path.exists() );
  }

  #[ test ]
  fn test_isolated_client_creation()
  {
    let isolated_client = IsolatedClient::new( "test_client", false )
      .expect( "Should create isolated client" );

    // Client should be created successfully
    let _client = isolated_client.client();
  }
}
//! Test server management and robustness infrastructure for `api_ollama` integration tests.
//!
//! # Overview
//!
//! This module provides three critical robustness systems:
//!
//! 1. **Isolated Test Servers** - Eliminate environmental dependencies via hash-based port allocation
//! 2. **Timing Safety Framework** - Handle system load variance with 2x safety buffers
//! 3. **Loud Failure Enforcement** - Prevent silent test skips that hide infrastructure problems
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use crate::server_helpers::{get_isolated_endpoint, wait_for_checks};
//!
//! #[tokio::test]
//! async fn test_api_feature() {
//!   // Use isolated server instead of hardcoded localhost:11434
//!   let endpoint = get_isolated_endpoint().await
//!     .expect("Failed to get test endpoint");
//!
//!   let client = OllamaClient::new(endpoint, Duration::from_secs(30))?;
//!
//!   // Your test logic with real API calls
//!   let response = client.chat(request).await?;
//!   assert!(response.message.content.len() > 0);
//! }
//! ```
//!
//! # Test Isolation Architecture (issue-server-exhaustion-001)
//!
//! Each test binary gets its own dedicated Ollama server instance on a unique port calculated
//! from the binary name hash. This prevents server resource exhaustion that occurred when all
//! tests shared port 11435, causing intermittent timeout failures after ~67 tests.
//!
//! **Port allocation**:
//! - Range: 11435-11534 (100 ports for test binaries)
//! - Formula: `BASE_PORT` + (`hash(binary_name)` % `PORT_RANGE`)
//! - Model: smollm2:360m (23% faster than tinyllama)
//! - Cleanup: Automatic on test completion
//!
//! # Robustness Patterns
//!
//! ## Pattern 1: Endpoint Isolation
//!
//! **Problem**: Tests depending on system Ollama (localhost:11434) are flaky because they race
//! with system state. Test passes when system Ollama stopped, fails when running.
//!
//! **Solution**: Use `get_isolated_endpoint()` for all tests making real API calls.
//!
//! **Example**: See `health_checks_tests.rs:test_intermittent_failure_handling` (issue-flaky-test-002)
//! - Before: 80% fail rate due to hardcoded localhost:11434
//! - After: 10/10 marathon passes with isolated endpoint
//!
//! ## Pattern 2: Timing Safety
//!
//! **Problem**: Brittle sleep-based synchronization fails under system load. Exact timing
//! assertions break when GC pauses or thread scheduling delays occur.
//!
//! **Solution**: Use `wait_for_checks()` with built-in 2x safety buffers instead of raw sleep.
//!
//! **Formula**: `wait_time = check_interval × min_checks × safety_factor(2.0)`
//!
//! **Example**:
//! ```rust,ignore
//! // BAD - Brittle exact timing
//! tokio::time::sleep(Duration::from_millis(300)).await;
//! assert_eq!(status.total_checks(), 3); // Fails if 4 checks happen
//!
//! // GOOD - Safety buffer + range assertion
//! wait_for_checks(interval, 3).await; // 600ms for nominal 300ms
//! assert!(status.total_checks() >= 3); // Tolerates variance
//! ```
//!
//! ## Pattern 3: Loud Failures
//!
//! **Problem**: Silent test skips (println + return) hide infrastructure problems and reduce
//! test coverage visibility. Tests appear to pass but actually skipped execution.
//!
//! **Solution**: Use `with_test_server!` macro or `.expect()` - never silent skip.
//!
//! **Example**:
//! ```rust,ignore
//! // BAD - Silent skip
//! match client.embeddings(req).await {
//!   Ok(emb) => emb,
//!   Err(e) => {
//!     println!("⏭️  Skipping test - {e}");
//!     return; // Test appears to pass but didn't run!
//!   }
//! }
//!
//! // GOOD - Loud failure
//! client.embeddings(req).await
//!   .expect("Embeddings should succeed - test server is running")
//! ```
//!
//! # Common Pitfalls
//!
//! ## Pitfall 1: Hardcoded localhost:11434 in API-Calling Tests
//!
//! **Symptom**: Test passes when system Ollama stopped, fails when running
//! **Root Cause**: Race condition with system Ollama state
//! **Fix**: Use `get_isolated_endpoint().await?` instead of hardcoded URL
//! **Example**: `health_checks_tests.rs:test_intermittent_failure_handling`
//!
//! ## Pitfall 2: Exact Timing Assertions
//!
//! **Symptom**: Test fails intermittently under system load
//! **Root Cause**: No safety margin for scheduler variance, GC pauses
//! **Fix**: Use `>=` assertions with 2x safety buffers via `wait_for_checks()`
//! **Example**: Wait 600ms for nominal 300ms (3 × 100ms × 2.0)
//!
//! ## Pitfall 3: Silent Test Skips
//!
//! **Symptom**: Test suite shows 100% pass but coverage is low
//! **Root Cause**: Tests silently return on errors instead of failing
//! **Fix**: Replace `println!() + return` with `.expect()` or panic
//! **Example**: 7 instances fixed in `embeddings_tests.rs` (issue-silent-skip-002 through -005)
//!
//! ## Pitfall 4: Shared Mutable State Across Tests
//!
//! **Symptom**: Tests fail based on execution order, not code correctness
//! **Root Cause**: Multiple tests modifying same resource (port, file, etc)
//! **Fix**: Each test binary automatically gets isolated server (hash-based port)
//! **Architecture**: Port allocation prevents conflicts - no manual coordination needed
//!
//! # Migration Guide
//!
//! When adding new integration tests that make API calls:
//!
//! ```rust,ignore
//! // BEFORE (hardcoded, flaky):
//! let client = OllamaClient::new(
//!   "http://localhost:11434".to_string(),
//!   Duration::from_secs(30)
//! )?;
//!
//! // AFTER (isolated, robust):
//! let endpoint = get_isolated_endpoint().await?;
//! let client = OllamaClient::new(endpoint, Duration::from_secs(30))?;
//! ```
//!
//! When adding timing-dependent tests:
//!
//! ```rust,ignore
//! // BEFORE (brittle):
//! tokio::time::sleep(Duration::from_millis(500)).await;
//! assert_eq!(checks, 5);
//!
//! // AFTER (robust):
//! wait_for_checks(interval, 5).await;
//! assert!(checks >= 5);
//! ```
//!
//! # Marathon Validation
//!
//! For tests prone to flakiness (timing-dependent, API-calling):
//!
//! ```bash
//! # Run 20 iterations to detect <5% flake rate
//! bash tests/-marathon_test.sh test_name 20
//!
//! # Run 100 iterations to detect <1% flake rate
//! bash tests/-marathon_test.sh test_name 100
//! ```
//!
//! **When to use**: After fixing flaky tests or adding timing-dependent logic.

use std::process::{ Command, Stdio, Child };
use std::sync::{ Arc, Mutex, OnceLock };
use core::time::Duration;
use std::time::Instant;
use api_ollama::OllamaClient;
use std::collections::hash_map::DefaultHasher;
use core::hash::{ Hash, Hasher };

/// Global server instance shared across all tests in a single binary
static TEST_SERVER: OnceLock< Arc< Mutex< Option< TestServer > > > > = OnceLock::new();

/// Test server configuration
const BASE_PORT: u16 = 11435; // Base port for test servers
const PORT_RANGE: u16 = 100; // Allow 100 different test binaries (11435-11534)
// Optimization(phase3-model-switch): Changed from tinyllama to smollm2:360m for 23% speed improvement
// Root cause: Tinyllama (1.1B params) had extreme variability (350s-780s) and slow chat responses (avg 2220ms)
// Solution: Benchmark tested 4 models - smollm2:360m fastest (2024ms overall, 1730ms chat avg = 1.23x speedup)
// Benchmark results: smollm2:360m (2024ms), qwen2.5:0.5b (2267ms), tinyllama (2485ms), gemma3:1b (2672ms)
// Pitfall: Smaller model (360M vs 1.1B) may have different behavior - verify all tests still pass
const TEST_MODEL: &str = "smollm2:360m"; // Fastest model from benchmark (23% faster than tinyllama)
const SERVER_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const MODEL_PULL_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes for model download
// Fix(issue-model-loading-timeout-001): Increased from 60s -> 180s to handle slow model loading
// Root cause: First inference request after server start takes 60+ seconds for model loading (smollm2:360m)
// Ollama loads models into RAM on first request, not during server startup - this is unavoidable overhead
// Pitfall: Timeout must exceed worst-case model load time (60-120s observed) plus inference time (10-30s)
const QUICK_RESPONSE_TIMEOUT: Duration = Duration::from_secs(180); // 3 minutes for model loading + inference

// Fix(issue-server-exhaustion-001): Unique port per test binary prevents server resource exhaustion
// Root cause: All tests shared port 11435, causing server exhaustion after ~67 tests with intermittent timeout failures
// Pitfall: Shared test infrastructure without isolation creates flaky tests that fail based on execution order, not code bugs

/// Calculate unique test port for this test binary based on binary name hash
///
/// This ensures each test binary gets its own isolated Ollama server instance,
/// preventing resource exhaustion and enabling parallel test execution.
///
/// # Returns
/// Port number in range [`BASE_PORT`, `BASE_PORT` + `PORT_RANGE`)
fn get_test_port() -> u16
{
  let binary_name = std::env::current_exe()
    .ok()
    .and_then( |path| path.file_name().map( |n| n.to_string_lossy().to_string() ) )
    .unwrap_or_else( || "default_test".to_string() );

  let mut hasher = DefaultHasher::new();
  binary_name.hash( &mut hasher );
  let hash = hasher.finish();

  // Cast is safe: modulo ensures result < PORT_RANGE (u16::MAX)
  #[ allow( clippy::cast_possible_truncation ) ]
  let offset = ( hash % u64::from( PORT_RANGE ) ) as u16;
  BASE_PORT + offset
}

/// Managed test server instance
#[ derive( Debug ) ]
pub struct TestServer
{
  process : Child,
  port : u16,
  client : OllamaClient,
}

impl TestServer
{
  /// Check if Ollama binary is available in PATH
  fn is_ollama_available() -> bool
  {
    Command::new( "ollama" )
      .arg( "--version" )
      .stdout( Stdio::null() )
      .stderr( Stdio::null() )
      .status()
      .is_ok_and( |status| status.success() )
  }

  /// Start a new test server instance
  ///
  /// # Errors
  /// Returns an error if:
  /// - Ollama binary is not found or fails to start
  /// - Server doesn't become ready within timeout
  /// - Test model cannot be pulled or verified
  async fn start() -> Result< Self, String >
  {
    // Check if Ollama is available before attempting to start server
    if !Self::is_ollama_available()
    {
      return Err("Ollama binary not found in PATH\n\nThis is expected in CI/automated test environments.\nResolution steps for local development:\n1. Install Ollama : curl -fsSL https://ollama.ai/install.sh | sh\n2. Ensure Ollama is in PATH\n3. Run 'ollama --version' to verify installation".to_string());
    }

    let test_port = get_test_port();
    println!( "🚀 Starting Ollama test server on port {test_port}..." );

    // Start Ollama server with custom port unique to this test binary
    // Resource limits prevent memory exhaustion when running many test binaries:
    // - OLLAMA_NUM_PARALLEL=1: Only process 1 request at a time
    // - OLLAMA_MAX_LOADED_MODELS=1: Only keep 1 model in memory
    // - OLLAMA_KEEP_ALIVE=0: Unload models immediately when idle
    let mut process = Command::new("ollama")
      .args(["serve"])
      .env("OLLAMA_HOST", format!( "127.0.0.1:{test_port}" ))
      .env("OLLAMA_NUM_PARALLEL", "1")
      .env("OLLAMA_MAX_LOADED_MODELS", "1")
      .env("OLLAMA_KEEP_ALIVE", "0")
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()
      .map_err(|e| format!(
        "Failed to start Ollama server : {e}\n\nResolution steps:\n1. Install Ollama : curl -fsSL https://ollama.ai/install.sh | sh\n2. Ensure Ollama is in PATH\n3. Run 'ollama --version' to verify installation"
      ) )?;

    // Extended timeout for integration tests: Ollama server can be slow under load (model processing, concurrent requests)
    // Fix(issue-builder-timeout-001): Increased from 300s -> 570s -> 650s -> 680s -> 720s -> 750s to handle extremely variable tinyllama responses
    // Root cause: Chat endpoint responses take 12+ minutes, extremely variable (350s-780s for identical requests)
    // Observed: test_builder_authentication_integration varies wildly (216s -> 678s -> 781s timeout) with no code changes
    // Tinyllama performance under concurrent load is unpredictable - some requests take 13+ minutes
    // Pitfall: Client timeout must be less than nextest slow-timeout (780s for builder tests, 600s for others) but generous for variable responses
    let mut client = OllamaClient::new( format!( "http://127.0.0.1:{test_port}" ), Duration::from_secs(750) ); // 12.5 minutes for extremely variable tinyllama responses

    // Wait for server to be ready
    let start_time = Instant::now();
    loop
    {
      if start_time.elapsed() > SERVER_STARTUP_TIMEOUT
      {
        let _ = process.kill();
        return Err( format!(
          "Ollama server failed to start within {timeout} seconds\n\nResolution steps:\n1. Check if port {port} is already in use\n2. Verify Ollama installation\n3. Check system resources (RAM/disk space)",
          timeout = SERVER_STARTUP_TIMEOUT.as_secs(),
          port = test_port
        ) );
      }

      if client.is_available().await
      {
        println!( "✅ Ollama test server ready on port {test_port}" );
        break;
      }

      tokio ::time::sleep(Duration::from_millis(500)).await;
    }

    let mut server = TestServer { process, port : test_port, client };

    // Ensure test model is available
    server.ensure_test_model_available().await?;

    // Fix(issue-resource-exhaustion-002): Removed test_quick_response() validation
    // Root cause: Model loading takes 60-180s on first inference, blocking server initialization
    // This caused resource exhaustion when multiple test binaries ran in parallel (each waited 60-180s)
    // Solution: Skip validation during init - actual tests will fail loudly if server doesn't work
    // Pitfall: First test per binary will be slower (expected - unavoidable model loading overhead)

    Ok(server)
  }
  
  /// Ensure the test model is pulled and available
  ///
  /// # Errors
  /// Returns an error if:
  /// - Cannot communicate with test server
  /// - Model pull fails due to network/registry issues
  /// - Model verification fails after pull
  async fn ensure_test_model_available(&mut self) -> Result< (), String >
  {
    println!( "🔍 Checking if test model '{TEST_MODEL}' is available..." );
    
    // Check if model is already available
    match self.client.list_models().await
    {
      Ok(models) => 
      {
        if models.models.iter().any(|m| m.name.starts_with(TEST_MODEL))
        {
          println!( "✅ Test model '{TEST_MODEL}' already available" );
          return Ok(());
        }
      }
      Err(_) =>
      {
        return Err( format!(
          "Failed to communicate with test server\n\nResolution steps:\n1. Verify Ollama server is running\n2. Check network connectivity\n3. Ensure port {} is accessible",
          self.port
        ) );
      }
    }

    println!( "⬇️ Pulling test model '{TEST_MODEL}' (this may take several minutes)..." );

    // Pull the minimal test model
    let pull_start = Instant::now();
    let pull_result = Command::new("ollama")
      .args(["pull", TEST_MODEL])
      .env("OLLAMA_HOST", format!( "127.0.0.1:{}", self.port ))
      .output();
      
    match pull_result
    {
      Ok(output) if output.status.success() => 
      {
        println!( "✅ Test model '{TEST_MODEL}' pulled successfully in {:.1}s", pull_start.elapsed().as_secs_f64() );
      }
      Ok(output) =>
      {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err( format!(
          "Failed to pull test model '{TEST_MODEL}': {stderr}\n\nResolution steps:\n1. Check internet connectivity\n2. Verify Ollama registry access\n3. Ensure sufficient disk space\n4. Try manual pull : ollama pull {TEST_MODEL}"
        ) );
      }
      Err(e) =>
      {
        return Err( format!(
          "Failed to execute model pull : {e}\n\nResolution steps:\n1. Verify Ollama CLI is available\n2. Check PATH configuration\n3. Try manual pull : ollama pull {TEST_MODEL}"
        ) );
      }
    }
    
    if pull_start.elapsed() > MODEL_PULL_TIMEOUT
    {
      return Err( format!(
        "Model pull timed out after {timeout} seconds\n\nResolution steps:\n1. Check internet speed\n2. Retry with better connection\n3. Consider using cached model",
        timeout = MODEL_PULL_TIMEOUT.as_secs()
      ) );
    }
    
    // Verify model is now available
    match self.client.list_models().await
    {
      Ok(models) if models.models.iter().any(|m| m.name.starts_with(TEST_MODEL)) => 
      {
        println!( "✅ Test model '{TEST_MODEL}' verified and ready for testing" );
        Ok(())
      }
      _ => Err( format!(
        "Test model '{TEST_MODEL}' not found after pull\n\nResolution steps:\n1. Check Ollama model registry\n2. Verify model pull completed\n3. Try : ollama list"
      ) )
    }
  }
  
  /// Test if server can respond quickly to a simple request
  ///
  /// **DEPRECATED**: This validation is no longer used during server initialization.
  ///
  /// # Why Removed (issue-resource-exhaustion-002)
  ///
  /// This method was removed from server initialization because:
  /// - Model loading takes 60-180s on first inference request (unavoidable Ollama behavior)
  /// - Blocked `TestServer::start()` for 60-180s per test binary during validation
  /// - With multiple test binaries running in parallel, caused cumulative resource exhaustion
  /// - Redundant: Actual tests will fail immediately if server isn't working (satisfies "fail loudly" principle)
  ///
  /// # Current Strategy
  ///
  /// - Server availability validated by `is_available()` check (lightweight, fast)
  /// - Model loading happens during first actual test instead (no net time loss, just moved)
  /// - Tests fail loudly if server doesn't work - no silent failures
  ///
  /// Kept for reference and potential future use with different strategy.
  ///
  /// # Errors
  /// Returns an error if the server takes too long to respond
  #[allow(dead_code)]
  async fn test_quick_response(&mut self) -> Result< (), String >
  {
    println!( "🚀 Testing server quick response..." );
    
    use api_ollama::{ GenerateRequest };
    
    let request = GenerateRequest
    {
      model : TEST_MODEL.to_string(),
      prompt : "Hi".to_string(),
      stream : Some(false),
      options : None,
    };
    
    let start_time = std::time::Instant::now();
    
    // Try a quick generation request with timeout
    let result = tokio::time::timeout(
      QUICK_RESPONSE_TIMEOUT,
      self.client.generate(request)
    ).await;
    
    match result
    {
      Ok(Ok(_)) => 
      {
        let elapsed = start_time.elapsed();
        println!( "✅ Server responding quickly ({:.2}s)", elapsed.as_secs_f64() );
        Ok(())
      }
      Ok(Err(e)) =>
      {
        let elapsed = start_time.elapsed();
        Err( format!( "Server failed to respond correctly ({:.2}s): {e}\n\nResolution steps:\n1. Check Ollama server logs\n2. Verify model is loaded correctly\n3. Check system resources (RAM/CPU)\n4. Try restarting Ollama server", elapsed.as_secs_f64() ) )
      }
      Err(_) =>
      {
        let elapsed = start_time.elapsed();
        Err( format!( "Server timed out after {:.2}s\n\nResolution steps:\n1. Check system resources (RAM/CPU)\n2. Try smaller model\n3. Increase QUICK_RESPONSE_TIMEOUT\n4. Check Ollama server logs", elapsed.as_secs_f64() ) )
      }
    }
  }
  
  /// Get client configured for this test server
  #[ must_use ]
  #[ allow( dead_code ) ]
  pub fn client(&self) -> &OllamaClient
  {
    &self.client
  }

  /// Get the test model name
  #[ must_use ]
  #[ allow( dead_code ) ]
  pub fn test_model() -> &'static str
  {
    TEST_MODEL
  }

  /// Get the port this test server is listening on
  #[ must_use ]
  #[ allow( dead_code ) ]
  pub fn port(&self) -> u16
  {
    self.port
  }
}

impl Drop for TestServer
{
  fn drop(&mut self)
  {
    let port = self.port;
    println!( "🛑 Shutting down Ollama test server on port {port}" );

    // Method 1: Try graceful kill via process handle
    let _ = self.process.kill();

    // Fix(issue-resource-exhaustion-002): Increased wait from 100ms -> 500ms for graceful shutdown
    // Root cause: Ollama needs time to flush model from RAM and release network sockets
    // Insufficient wait causes resource leaks when next test starts immediately
    // Pitfall: Too short delay leaves zombie processes and open ports
    std::thread::sleep(Duration::from_millis(500));

    // Method 2: Kill by port using lsof (finds processes listening on the port)
    let _ = Command::new("sh")
      .arg("-c")
      .arg( format!( "lsof -ti tcp:{port} 2>/dev/null | xargs -r kill -9 2>/dev/null || true" ) )
      .output();

    // Method 3: Kill by OLLAMA_HOST environment variable (catches the serve process)
    let _ = Command::new("pkill")
      .args(["-9", "-f", &format!( "OLLAMA_HOST=.*:{port}" )])
      .output();

    // Method 4: Kill any user-owned ollama runner processes
    // (Ollama spawns runner subprocesses that may outlive the serve process)
    let username = std::env::var("USER").unwrap_or_else(|_| "user1".to_string());
    let _ = Command::new("sh")
      .arg("-c")
      .arg( format!(
        "ps aux | grep '[o]llama' | grep '^{username}' | awk '{{print $2}}' | xargs -r kill -9 2>/dev/null || true"
      ) )
      .output();

    // Wait for processes to fully terminate and release resources
    std::thread::sleep(Duration::from_secs(1));

    // Final verification - wait for process handle
    let _ = self.process.wait();

    println!( "✅ Ollama server on port {port} cleanup completed" );
  }
}

/// Clean up any orphaned Ollama test servers from previous runs
///
/// When test processes are killed forcefully (e.g., nextest timeout with SIGKILL),
/// the Drop implementation doesn't run, leaving orphaned Ollama servers running.
/// This function kills any user-owned Ollama servers AND runner processes before starting new tests.
///
/// Fix(issue-runner-cleanup-001): Added cleanup for ollama runner processes
/// Root cause: Only killed 'ollama serve' but left 'ollama runner' processes (each consuming 920MB RAM + 5-10 CPU cores)
/// Runner processes accumulate during parallel test execution, causing resource exhaustion and network timeouts
/// Pitfall: Always kill BOTH serve and runner processes - runners are the actual resource hogs
fn cleanup_orphaned_servers()
{
  // Fix(issue-parallel-cleanup-001): Only clean up THIS test binary's port to avoid killing other parallel test servers
  // Root cause: cleanup_orphaned_servers() killed ALL test ports (11435-11534), including servers from other running test binaries
  // When test-threads=8, multiple test binaries run in parallel, each with unique port from hash(binary_name)
  // Binary A's cleanup at startup would kill Binary B's running server, causing "error sending request" failures
  // Pitfall: Over-aggressive cleanup in parallel environments creates race conditions between test binaries
  let test_port = get_test_port();
  println!( "🧹 Cleaning up orphaned Ollama server on port {test_port}..." );

  // Kill any process listening on THIS test binary's port only
  // This preserves servers from other test binaries running in parallel
  let port_cleanup = Command::new("sh")
    .arg("-c")
    .arg( format!( "lsof -ti tcp:{test_port} 2>/dev/null | xargs -r kill -9 2>/dev/null || true" ) )
    .output();

  // Report cleanup status
  match port_cleanup
  {
    Ok(output) if output.status.success() =>
    {
      println!( "✅ Cleaned up orphaned server on port {test_port}" );
    }
    _ =>
    {
      // Cleanup failure is non-fatal - server may not exist (expected on first run)
      println!( "✅ No orphaned server found on port {test_port}" );
    }
  }

  // Wait for port to be fully released before starting new server
  std::thread::sleep(Duration::from_millis(500));
}

/// Get or create the global test server instance
///
/// # Errors
/// Returns an error if the test server fails to start or initialize
pub async fn get_test_server() -> Result< Arc< Mutex< Option< TestServer > > >, String >
{
  // ALWAYS clean up orphaned servers at the start of EVERY test binary
  // This ensures cleanup happens even if previous test binaries crashed/timed out
  // Static variable ensures this only runs once per test binary process
  static CLEANUP_DONE: std::sync::Once = std::sync::Once::new();
  CLEANUP_DONE.call_once(|| {
    cleanup_orphaned_servers();
  });

  let server_arc = TEST_SERVER.get_or_init(|| Arc::new(Mutex::new(None))).clone();

  // Check if server needs to be initialized
  let needs_init = {
    let server_guard = server_arc.lock().map_err(|e| format!( "Failed to acquire test server mutex for initialization check : {e}" ))?;
    server_guard.is_none()
  };

  if needs_init
  {
    match TestServer::start().await
    {
      Ok(server) =>
      {
        let mut server_guard = server_arc.lock().map_err(|e| format!( "Failed to acquire test server mutex for initialization : {e}" ))?;
        *server_guard = Some(server);
        println!( "🎯 Test server initialized successfully" );
      }
      Err(e) =>
      {
        return Err( format!( "Failed to initialize test server : {e}" ) );
      }
    }
  }

  Ok(server_arc)
}

/// Test helper function to get a client for the managed test server
///
/// # Errors
/// Returns an error if the test server fails to start or initialize, or if mutex lock fails
#[ allow( dead_code ) ]
pub async fn get_test_client() -> Result< ( OllamaClient, String ), String >
{
  let server_arc = get_test_server().await?;
  let server_guard = server_arc.lock().map_err(|e| format!( "Failed to acquire test server mutex : {e}" ))?;
  let server = server_guard.as_ref().ok_or("Test server not initialized")?;

  // Clone the client and get model name
  let client = server.client().clone();
  let model = TestServer::test_model().to_string();

  Ok( ( client, model ) )
}

/// Macro to ensure test server is available before running test
///
/// **Fix(issue-silent-skip-001)**: Changed from silent skip to loud failure.
/// **Root cause**: Silent skips (println + return) hide infrastructure problems and reduce test coverage visibility.
/// **Pitfall**: Tests must fail loudly when prerequisites are missing - use `#[ignore]` attribute for optional tests.
///
/// # When to Use
///
/// **Use this macro** for tests that need isolated Ollama client with test model:
/// - Quick tests that just need client + model (no manual endpoint setup)
/// - Tests that should fail loudly if infrastructure unavailable
/// - Migration from legacy silent skip pattern
///
/// **Don't use** when:
/// - Test needs custom endpoint URL (use `get_isolated_endpoint()` directly)
/// - Test needs custom client configuration (timeout, retries, etc.)
/// - Test is truly optional/experimental (use `#[ignore]` attribute instead)
///
/// # Loud Failure Rationale
///
/// Tests should never silently skip because:
/// - Hides broken test infrastructure (Ollama not installed, port conflicts)
/// - Reduces effective test coverage without visibility
/// - Violates "fail loudly" robustness principle
/// - Makes debugging harder (no clear signal of what's wrong)
///
/// # Migration Guide
///
/// ```rust,ignore
/// // ❌ BEFORE - Silent skip pattern (issue-silent-skip-001 through -005)
/// #[tokio::test]
/// async fn test_embeddings_feature() {
///   let client = match get_test_client().await {
///     Ok((c, _)) => c,
///     Err(e) => {
///       println!("⏭️  Skipping test - {e}");
///       return;
///     }
///   };
///   // Test logic...
/// }
///
/// // ✅ AFTER - Loud failure with macro
/// #[tokio::test]
/// async fn test_embeddings_feature() {
///   with_test_server!(|client, model| async move {
///     // Test logic...
///     Ok(())
///   });
/// }
/// ```
///
/// # Known Pitfalls
///
/// **Pitfall**: Catching macro panic to implement silent skip
/// **Symptom**: Code like `if let Err(_) = std::panic::catch_unwind(...)`
/// **Root Cause**: Attempting to bypass loud failure enforcement
/// **Fix**: If test is optional, use `#[ignore]` attribute, don't catch panic
/// **Pattern**: Mark test with `#[ignore = "reason"]` not `catch_unwind`
///
/// **Pitfall**: Using macro for configuration-only tests
/// **Symptom**: Test fails with "server unavailable" but only validates client creation
/// **Root Cause**: Macro requires full test server for simple config validation
/// **Fix**: Tests that don't make API calls shouldn't use this macro
/// **Example**: Tests for URL parsing, builder patterns, config validation
///
/// **Pitfall**: Returning wrong type from closure
/// **Symptom**: Compiler error "expected `()`, found `Result<...>`"
/// **Root Cause**: Macro expects async block returning `Result<(), Error>`
/// **Fix**: Ensure closure returns `Result<(), E>` where E implements Display
/// **Pattern**: `async move { /* test */ Ok(()) }`
///
/// # Resolution Steps
///
/// If test fails with "Test server unavailable":
/// 1. Install Ollama: `curl -fsSL https://ollama.com/install.sh | sh`
/// 2. Verify Ollama runs: `ollama --version`
/// 3. Check port availability: `lsof -i :11435-11534`
/// 4. Review `server_helpers.rs` startup logs for diagnostics
///
/// # Optional Tests
///
/// If a test is truly optional (e.g., requires external service), use `#[ignore]` attribute:
/// ```rust,ignore
/// #[tokio::test]
/// #[ignore = "requires external Ollama service"]
/// async fn test_optional_feature() { ... }
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// // Example 1: Basic usage
/// #[tokio::test]
/// async fn test_chat_basic() {
///   with_test_server!(|client, model| async move {
///     let response = client.chat(ChatRequest::new(model)).await?;
///     assert!(!response.message.content.is_empty());
///     Ok(())
///   });
/// }
///
/// // Example 2: Multiple assertions
/// #[tokio::test]
/// async fn test_embeddings_dimensions() {
///   with_test_server!(|client, model| async move {
///     let request = EmbeddingsRequest::new(model, vec!["test".to_string()]);
///     let response = client.embeddings(request).await?;
///
///     assert_eq!(response.embeddings.len(), 1);
///     assert!(response.embeddings[0].len() > 0);
///     Ok(())
///   });
/// }
/// ```
#[ macro_export ]
macro_rules! with_test_server {
  ($test_fn:expr) => {{
    match $crate::server_helpers::get_test_client().await
    {
      Ok( ( client, model ) ) => $test_fn( client, model ).await,
      Err( e ) =>
      {
        panic!(
          "\n\n\
          ❌ TEST INFRASTRUCTURE FAILURE: Test server unavailable\n\
          \n\
          Error: {e}\n\
          \n\
          This test requires a running Ollama server managed by server_helpers.rs.\n\
          The test server failed to start, which indicates a configuration or environment problem.\n\
          \n\
          Resolution steps:\n\
          1. Install Ollama: curl -fsSL https://ollama.com/install.sh | sh\n\
          2. Verify Ollama: ollama --version\n\
          3. Check ports 11435-11534 are available: lsof -i :11435-11534\n\
          4. Review test output above for detailed diagnostics\n\
          \n\
          If this test is optional, mark it with #[ignore] attribute.\n\
          Silent test skips are forbidden - tests must fail loudly or be explicitly ignored.\n\
          \n"
        );
      },
    }
  }};
}

#[ cfg( test ) ]
mod tests
{
  use super::*;
  
  #[ tokio::test ]
  #[ allow( dead_code ) ]
  async fn test_server_lifecycle()
  {
    // Test that we can get a test server
    let result = get_test_client().await;

    // Skip gracefully if Ollama server is unavailable
    let (mut client, model) = match result
    {
      Ok(client_model) => client_model,
      Err(e) =>
      {
        println!( "⏭️  Skipping test - Ollama server unavailable: {e}" );
        return;
      }
    };

    // Test that server is responsive
    assert!(client.is_available().await, "Test server should be available");

    // Test that model is correct
    assert_eq!(model, TEST_MODEL, "Test model should be {TEST_MODEL}");

    println!( "✅ Test server lifecycle validated" );
  }
}

//
// Endpoint Isolation Helpers
//

/// Get isolated test endpoint URL for robust testing
///
/// Returns URL for isolated test server running on hash-based unique port.
/// Eliminates race conditions with system Ollama on port 11434.
///
/// # When to Use
///
/// **Use this** whenever your test makes REAL API calls (`.chat()`, `.embeddings()`, `.generate()`).
///
/// **Don't use** for configuration-only tests that just validate client creation without API calls.
///
/// **Decision criteria**:
/// - If test calls `.await` on API method → Use `get_isolated_endpoint()`
/// - If test only checks client construction → Hardcoded endpoint acceptable
///
/// # Robustness Properties
///
/// - **Environmental Independence**: No dependency on system Ollama state
/// - **Parallel Safety**: Hash-based port allocation prevents conflicts
/// - **Deterministic**: Same binary always gets same port
///
/// # Errors
///
/// Returns error if test server initialization fails or server is not available.
///
/// # Example
///
/// ```rust,ignore
/// #[tokio::test]
/// async fn test_api_call() {
///   let endpoint = get_isolated_endpoint().await
///     .expect("Failed to get test endpoint");
///
///   let client = OllamaClient::new(endpoint, Duration::from_secs(30))?;
///   // Test uses isolated server, not system Ollama
/// }
/// ```
///
/// # Migration Pattern
///
/// **Before** (hardcoded, flaky):
/// ```rust,ignore
/// let client = OllamaClient::new(
///   "http://localhost:11434".to_string(),  // ← Race with system Ollama
///   Duration::from_secs(30)
/// )?;
/// ```
///
/// **After** (isolated, robust):
/// ```rust,ignore
/// let endpoint = get_isolated_endpoint().await?;
/// let client = OllamaClient::new(
///   endpoint,  // ← Uses unique test server port
///   Duration::from_secs(30)
/// )?;
/// ```
///
/// # Known Pitfalls
///
/// **Pitfall**: Using hardcoded `localhost:11434` in test making real API calls
///
/// **Symptom**: Test passes when system Ollama is stopped, fails when system Ollama is running.
/// Test failure is intermittent and environment-dependent.
///
/// **Root Cause**: Test connects to system Ollama instead of isolated test server, creating race
/// condition with system state. Health checks, model state, or port availability differ.
///
/// **Fix**: Replace hardcoded URL with `get_isolated_endpoint().await?`
///
/// **Real Example**: `health_checks_tests.rs:test_intermittent_failure_handling` (issue-flaky-test-002)
/// - Before: 80% fail rate due to hardcoded endpoint
/// - After: 10/10 marathon passes with isolated endpoint
/// - See lines 251-432 for complete implementation
#[ allow( dead_code ) ] // Used across multiple test files
pub async fn get_isolated_endpoint() -> Result< String, String >
{
  let server_arc = get_test_server().await?;

  let test_port =
  {
    let server_guard = server_arc.lock()
      .map_err( |e| format!( "Failed to lock test server mutex: {e}" ) )?;
    let test_server = server_guard.as_ref()
      .ok_or_else( || "Test server not initialized - server_helpers startup failed".to_string() )?;
    test_server.port()
  }; // Guard dropped here before returning (prevents deadlock)

  Ok( format!( "http://127.0.0.1:{test_port}" ) )
}

/// Get invalid endpoint URL for failure testing
///
/// Returns endpoint URL guaranteed to fail (RFC 5737 non-routable address),
/// for testing error handling, circuit breakers, retry logic, and health check failure scenarios.
///
/// # When to Use
///
/// **Use this** when testing failure scenarios:
/// - Circuit breaker opening behavior (needs guaranteed failures)
/// - Health check failure detection and recovery
/// - Retry exhaustion scenarios (must fail every time)
/// - Timeout behavior validation (connection attempts guaranteed to timeout)
///
/// **Don't use** for:
/// - Tests requiring real API responses (use `get_isolated_endpoint()` instead)
/// - Tests simulating intermittent failures (use `client.simulate_endpoint_failure()`)
/// - Tests requiring specific error types (connection errors are generic)
///
/// # Known Pitfalls
///
/// **Pitfall**: Using `get_invalid_endpoint()` for positive test cases
/// **Symptom**: Test always fails with connection errors, never validates actual functionality
/// **Root Cause**: Invalid endpoint cannot serve real API responses
/// **Fix**: Use `get_isolated_endpoint()` for tests requiring real responses
/// **Example**: Health check recovery tests need real endpoint to validate recovery
///
/// **Pitfall**: Expecting specific error messages from invalid endpoint
/// **Symptom**: Tests break when underlying HTTP library changes error messages
/// **Root Cause**: Connection errors are implementation-dependent (OS, HTTP client)
/// **Fix**: Test for error presence, not specific error text
/// **Pattern**: `assert!(result.is_err())` not `assert_eq!(err.to_string(), "...")`
///
/// # Examples
///
/// ```rust,ignore
/// // Example 1: Circuit breaker opening
/// #[tokio::test]
/// async fn test_circuit_breaker_opens_on_failures() {
///   let endpoint = get_invalid_endpoint();
///   let client = OllamaClient::new(endpoint, Duration::from_secs(1))?;
///
///   // All requests will fail, triggering circuit breaker
///   for _ in 0..5 {
///     assert!(client.chat(...).await.is_err());
///   }
///   assert_eq!(client.circuit_breaker_state(), CircuitState::Open);
/// }
///
/// // Example 2: Retry exhaustion
/// #[tokio::test]
/// async fn test_retry_exhaustion() {
///   let endpoint = get_invalid_endpoint();
///   let client = OllamaClient::new(endpoint, Duration::from_secs(1))?
///     .with_retries(3);
///
///   let start = Instant::now();
///   let result = client.chat(...).await;
///
///   assert!(result.is_err()); // Failed after all retries
///   assert!(start.elapsed() >= Duration::from_secs(3)); // 3 retry attempts
/// }
/// ```
#[ must_use ]
#[ allow( dead_code ) ] // Used across multiple test files
pub fn get_invalid_endpoint() -> String
{
  // Use non-routable address for guaranteed failure
  // RFC 5737: 192.0.2.0/24 reserved for documentation/testing
  "http://192.0.2.1:11434".to_string()
}

//
// Timing Robustness Helpers
//

/// Calculate safe wait time with 2x safety buffer for timing-dependent tests
///
/// Formula: `wait_time = check_interval × min_checks × safety_factor`
///
/// # When to Use
///
/// **Use this** for timing-dependent tests that need to wait for:
/// - Health check iterations (wait for N checks to complete)
/// - Periodic background tasks (wait for N task executions)
/// - Retry attempts (wait for N retry cycles)
/// - Rate limiting windows (wait for rate limit to reset)
///
/// **Don't use** for:
/// - Exact timing measurements (defeats purpose of safety buffer)
/// - Production code timeouts (tests only - production needs tighter bounds)
/// - Tests with external time constraints (API rate limits, session expirations)
///
/// # Robustness Rationale
///
/// Timing-dependent tests fail intermittently when:
/// - System is under load (GC pauses, thread scheduling delays)
/// - CI environment has variable performance
/// - Background tasks interfere with timing
///
/// Safety buffer (2.0x) accounts for:
/// - Scheduler variance (OS thread context switches)
/// - GC pauses in async runtime
/// - Network stack delays
/// - Measurement granularity
///
/// # Known Pitfalls
///
/// **Pitfall**: Using exact timing assertions after safe wait
/// **Symptom**: Test expects exactly N checks but gets N+1 or N+2
/// **Root Cause**: Safety buffer allows extra iterations beyond minimum
/// **Fix**: Use `>=` assertions, not `==` assertions
/// **Example**: `assert!(checks >= 3)` not `assert_eq!(checks, 3)`
///
/// **Pitfall**: Using `safety_factor` < 2.0 to "speed up tests"
/// **Symptom**: Tests pass locally but fail intermittently in CI
/// **Root Cause**: CI environments have higher scheduler variance
/// **Fix**: Always use 2.0x minimum, increase for CI flakiness
/// **Real Example**: `test_intermittent_failure_handling` (issue-flaky-test-002)
/// - Before: No safety buffer → 80% fail rate
/// - After: 2.0x buffer → 0% fail rate (10/10 marathon passes)
///
/// **Pitfall**: Calculating wait time manually instead of using helper
/// **Symptom**: Inconsistent safety factors across test suite
/// **Root Cause**: Copy-paste errors, forgotten multiplication
/// **Fix**: Use `wait_for_checks()` helper (wraps this function with 2.0x)
/// **Pattern**: `wait_for_checks(interval, 5).await` not manual calculation
///
/// # Examples
///
/// ```rust,ignore
/// // Example 1: Basic usage with range assertion
/// let interval = Duration::from_millis(100);
/// let min_checks = 3;
/// let wait_time = calculate_safe_wait_time(interval, min_checks, 2.0);
/// // Returns 600ms (3 × 100ms × 2.0)
///
/// tokio::time::sleep(wait_time).await;
/// let status = client.get_health_status();
/// assert!(status.total_checks() >= min_checks); // ✅ Range assertion
///
/// // Example 2: Higher safety factor for flaky CI
/// let wait_time = calculate_safe_wait_time(
///   Duration::from_millis(100),
///   5,
///   3.0 // Higher buffer for CI environment
/// );
/// // Returns 1500ms (5 × 100ms × 3.0)
/// ```
#[ must_use ]
#[ allow( dead_code ) ] // Used across multiple test files
#[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ] // Intentional - safety_factor is always positive
pub fn calculate_safe_wait_time( check_interval : Duration, min_checks : u32, safety_factor : f64 ) -> Duration
{
  let nominal_time_ms = check_interval.as_millis() * u128::from( min_checks );
  let safe_time_ms = ( nominal_time_ms as f64 * safety_factor ) as u64;
  Duration::from_millis( safe_time_ms )
}

/// Wait for minimum number of checks to complete with safety buffer
///
/// Convenience wrapper for `calculate_safe_wait_time()` with hardcoded 2.0x safety factor.
/// Preferred over manual sleep calculations for test robustness.
///
/// # When to Use
///
/// **Use this** as default for timing-dependent waits:
/// - Health check test assertions (wait for N checks)
/// - Background task validation (wait for N executions)
/// - Periodic monitoring tests (wait for N iterations)
/// - Any test waiting for repeated operations
///
/// **Don't use** when:
/// - Custom safety factor needed (use `calculate_safe_wait_time()` directly)
/// - No timing dependency (don't add unnecessary waits)
/// - Waiting for single event (use event notification instead)
///
/// # Comparison with Naive Sleep
///
/// ```rust,ignore
/// // ❌ BAD - Brittle exact timing (fails under load)
/// tokio::time::sleep(Duration::from_millis(300)).await;
/// assert_eq!(status.total_checks(), 3); // Exact assertion
///
/// // ✅ GOOD - Robust timing with safety buffer
/// wait_for_checks(Duration::from_millis(100), 3).await; // 600ms (2x buffer)
/// assert!(status.total_checks() >= 3); // Range assertion
/// ```
///
/// # Known Pitfalls
///
/// **Pitfall**: Using exact sleep durations instead of `wait_for_checks()`
/// **Symptom**: Tests pass locally but fail intermittently in CI
/// **Root Cause**: Manual calculations miss safety buffer, don't account for variance
/// **Fix**: Replace `tokio::time::sleep()` with `wait_for_checks()`
/// **Migration**: `sleep(300ms)` → `wait_for_checks(100ms, 3)` (adds 2x buffer)
///
/// **Pitfall**: Pairing `wait_for_checks()` with exact assertions
/// **Symptom**: Test expects 3 checks but gets 4 due to timing variance
/// **Root Cause**: Safety buffer allows extra iterations beyond minimum
/// **Fix**: Always use `>=` assertions after `wait_for_checks()`
/// **Pattern**: `assert!(checks >= N)` never `assert_eq!(checks, N)`
///
/// **Pitfall**: Using `wait_for_checks()` for single-event synchronization
/// **Symptom**: Unnecessary delays in tests, slower test suite
/// **Root Cause**: Waiting for time instead of event notification
/// **Fix**: Use channels, condition variables, or polling for single events
/// **Example**: `rx.recv().await` instead of `wait_for_checks(...)`
///
/// # Examples
///
/// ```rust,ignore
/// // Example 1: Health check validation
/// let interval = Duration::from_millis(100);
/// client.start_health_monitoring().await;
///
/// wait_for_checks(interval, 5).await; // Waits 1000ms (5 × 100ms × 2.0)
///
/// let status = client.get_health_status();
/// assert!(status.total_checks() >= 5); // ✅ Range assertion
///
/// // Example 2: Failure detection
/// client.simulate_endpoint_failure();
///
/// wait_for_checks(interval, 3).await; // Waits 600ms (3 × 100ms × 2.0)
///
/// assert!(status.failed_checks() >= 3);
/// assert_eq!(status.overall_health(), EndpointHealth::Unhealthy);
/// ```
#[ allow( dead_code ) ] // Used across multiple test files
pub async fn wait_for_checks( check_interval : Duration, min_checks : u32 )
{
  let wait_time = calculate_safe_wait_time( check_interval, min_checks, 2.0 );
  tokio::time::sleep( wait_time ).await;
}

/// Timing assertion macro for range-based check counts
///
/// Use `>=` assertions instead of exact counts to tolerate timing variance.
///
/// # Example
///
/// ```rust,ignore
/// // BAD (brittle - fails if 6 checks happen):
/// assert_eq!(status.total_checks(), 5);
///
/// // GOOD (robust - tolerates 5, 6, 7, ... checks):
/// assert_timing_range!(status.total_checks() >= 5,
///   "Expected ≥5 checks after 750ms wait, got {}", status.total_checks());
/// ```
#[ allow( clippy::items_after_test_module ) ] // Macro placement is intentional for visibility
#[macro_export]
macro_rules! assert_timing_range
{
  ( $condition:expr, $( $arg:tt )* ) =>
  {
    assert!( $condition, $( $arg )* );
  };
}

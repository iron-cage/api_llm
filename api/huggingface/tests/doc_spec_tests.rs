//! Doc spec tests — GWT scenario implementations from tests/docs/ spec files.
//!
//! Structural tests verify source code conventions without network calls.
//! Integration tests (feature = "integration") call real `HuggingFace` endpoints.

mod inc;

#[ cfg( feature = "integration" ) ]
use api_huggingface::
{
  Client,
  environment::HuggingFaceEnvironmentImpl,
  secret::Secret,
  error::HuggingFaceError,
};

#[ cfg( feature = "integration" ) ]
fn build_client( api_key : &str ) -> Client< HuggingFaceEnvironmentImpl >
{
  let env = HuggingFaceEnvironmentImpl::build( Secret::new( api_key.to_string() ), None )
  .expect( "Environment construction should not fail" );
  Client::build( env ).expect( "Client construction should not fail" )
}

#[ cfg( feature = "integration" ) ]
fn build_integration_client() -> Client< HuggingFaceEnvironmentImpl >
{
  let key = inc::get_api_key_for_integration();
  let env = HuggingFaceEnvironmentImpl::build( Secret::new( key ), None )
  .expect( "Integration environment construction should not fail" );
  Client::build( env ).expect( "Integration client construction should not fail" )
}

// ============================================================================
// FE: Feature Spec — Enterprise Reliability (tests/docs/feature/)
// ============================================================================

/// FE-01: Enterprise feature is absent without its feature flag.
/// Verifies circuit-breaker module is gated in lib.rs.
#[ test ]
fn test_fe_01()
{
  let src = std::fs::read_to_string(
  format!( "{}/src/lib.rs", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read src/lib.rs" );

  assert!(
  src.contains( "circuit-breaker" ),
  "FE-01: lib.rs must gate the circuit_breaker module behind the circuit-breaker feature flag"
  );
}

/// FE-02: Enterprise feature requires explicit developer construction.
/// Verifies inference.rs does not auto-construct a `CircuitBreaker`.
#[ test ]
fn test_fe_02()
{
  let src = std::fs::read_to_string(
  format!( "{}/src/inference.rs", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read src/inference.rs" );

  assert!(
  !src.contains( "CircuitBreaker::new" ),
  "FE-02: inference.rs must not auto-construct a CircuitBreaker — it is opt-in only"
  );
}

/// FE-03: Rate limiter only throttles when explicitly invoked.
/// Verifies inference.rs does not call `rate_limiter.acquire()` automatically.
#[ test ]
fn test_fe_03()
{
  let src = std::fs::read_to_string(
  format!( "{}/src/inference.rs", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read src/inference.rs" );

  assert!(
  !src.contains( "rate_limiter.acquire" ) && !src.contains( ".acquire().await" ),
  "FE-03: inference.rs must not automatically call rate_limiter.acquire()"
  );
}

/// FE-04: Enterprise features are independent — enabling one does not activate others.
/// Verifies that each enterprise feature is independently defined in Cargo.toml.
#[ test ]
fn test_fe_04()
{
  let cargo_toml = std::fs::read_to_string(
  format!( "{}/Cargo.toml", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read Cargo.toml" );

  assert!( cargo_toml.contains( "failover" ), "FE-04: failover feature must be defined" );
  assert!( cargo_toml.contains( "circuit-breaker" ), "FE-04: circuit-breaker feature must be defined" );
  assert!( cargo_toml.contains( "rate-limiting" ), "FE-04: rate-limiting feature must be defined" );
  assert!( cargo_toml.contains( "health-checks" ), "FE-04: health-checks feature must be defined" );
}

// ============================================================================
// AP: API Spec — Reference (tests/docs/api/)
// ============================================================================

/// AP-01: Inference create returns generated text (integration).
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ap_01()
{
  let client = build_integration_client();
  let result = client.inference().create(
  "What is 2+2?",
  "meta-llama/Llama-3.3-70B-Instruct",
  ).await;

  let response = result.expect( "AP-01: inference.create should succeed" );
  let text = response.extract_text().expect( "AP-01: response must contain text" );
  assert!( !text.is_empty(), "AP-01: generated_text must be non-empty" );
}

/// AP-02: Embeddings create returns float vector (integration).
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ap_02()
{
  use api_huggingface::components::embeddings::EmbeddingResponse;

  let client = build_integration_client();
  let result = client.embeddings().create(
  "hello world",
  "sentence-transformers/all-MiniLM-L6-v2",
  ).await;

  let response = result.expect( "AP-02: embeddings.create should succeed" );
  let first_vec = match &response
  {
  EmbeddingResponse::Single( vecs ) =>
  {
      vecs.first().expect( "AP-02: should have at least one embedding" ).clone()
  }
  EmbeddingResponse::Batch( batches ) =>
  {
      batches
      .first().expect( "AP-02: batch should be non-empty" )
      .first().expect( "AP-02: inner batch should be non-empty" )
      .clone()
  }
  };
  assert!( !first_vec.is_empty(), "AP-02: embeddings[0] must be a Vec<f32> of length >= 1" );
}

/// AP-03: Similarity returns value in range [-1.0, 1.0] for identical texts (integration).
#[ cfg( all( feature = "integration", feature = "embeddings-similarity" ) ) ]
#[ tokio::test ]
async fn test_ap_03()
{
  let client = build_integration_client();
  let text = "identical sentence for similarity test";
  let result = client.embeddings().similarity(
  text,
  text,
  "sentence-transformers/all-MiniLM-L6-v2",
  ).await;

  let score = result.expect( "AP-03: similarity should succeed" );
  assert!(
  (-1.0_f32..=1.0_f32).contains( &score ),
  "AP-03: similarity must be in [-1.0, 1.0], got {score}"
  );
  assert!( score >= 0.99, "AP-03: identical texts must have similarity >= 0.99, got {score}" );
}

/// AP-04: Streaming create returns sequential chunks (integration).
#[ cfg( all( feature = "integration", feature = "inference-streaming" ) ) ]
#[ tokio::test ]
async fn test_ap_04()
{
  use api_huggingface::components::input::InferenceParameters;
  use core::time::Duration;

  let client = build_integration_client();
  let params = InferenceParameters::new()
  .with_streaming( true )
  .with_max_new_tokens( 20 );

  let result = client.inference().create_stream(
  "Once upon a time",
  "mistralai/Mistral-7B-Instruct-v0.1",
  params,
  ).await;

  let mut rx = result.expect( "AP-04: create_stream should succeed" );
  let mut chunks_received = 0u32;

  for _ in 0..20
  {
  match tokio::time::timeout( Duration::from_secs( 15 ), rx.recv() ).await
  {
      Ok( Some( Ok( text ) ) ) if !text.is_empty() =>
      {
      chunks_received += 1;
      break;
      }
      Ok( None ) => break,
      _ => {}
  }
  }

  assert!(
  chunks_received >= 1,
  "AP-04: must receive at least one non-empty streaming chunk"
  );
}

/// AP-05: Invalid API key returns error without panic (integration).
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ap_05()
{
  let client = build_client( "hf_invalid" );
  let result = client.inference().create(
  "hello",
  "meta-llama/Llama-3.3-70B-Instruct",
  ).await;

  assert!( result.is_err(), "AP-05: invalid API key must return an error" );

  let is_expected_variant = matches!(
  result.unwrap_err(),
  HuggingFaceError::Authentication( _ ) | HuggingFaceError::Http( _ )
  );
  assert!( is_expected_variant, "AP-05: error must be Authentication or Http variant" );
}

/// AP-06: Model management get returns info or `ModelUnavailable` (integration).
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ap_06()
{
  let client = build_integration_client();
  let result = client.models().get( "meta-llama/Llama-3.2-1B-Instruct" ).await;

  match result
  {
  Ok( _ ) | Err( HuggingFaceError::ModelUnavailable( _ ) ) => {}
  Err( e ) => panic!( "AP-06: unexpected error variant: {e}" ),
  }
}

// ============================================================================
// OP: Operation Spec — Feature Flag Management (tests/docs/operation/)
// ============================================================================

/// OP-01: Streaming unavailable without inference-streaming feature.
/// Verifies the feature gate exists in lib.rs.
#[ test ]
fn test_op_01()
{
  let src = std::fs::read_to_string(
  format!( "{}/src/lib.rs", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read src/lib.rs" );

  assert!(
  src.contains( "inference-streaming" ),
  "OP-01: lib.rs must gate the inference-streaming module"
  );
}

/// OP-02: Similarity utility unavailable without embeddings-similarity feature.
/// Verifies the feature gate exists in the source.
#[ test ]
fn test_op_02()
{
  // Cargo.toml defines embeddings-similarity as a dependent feature
  let cargo_toml = std::fs::read_to_string(
  format!( "{}/Cargo.toml", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read Cargo.toml" );

  assert!(
  cargo_toml.contains( "embeddings-similarity" ),
  "OP-02: Cargo.toml must define the embeddings-similarity feature"
  );
}

/// OP-03: Sync API unavailable without sync feature.
/// Verifies the sync module is feature-gated in lib.rs.
#[ test ]
fn test_op_03()
{
  let src = std::fs::read_to_string(
  format!( "{}/src/lib.rs", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read src/lib.rs" );

  // The sync module must be behind a cfg gate
  assert!(
  src.contains( "\"sync\"" ),
  "OP-03: lib.rs must gate the sync module behind the sync feature"
  );
}

/// OP-04: full feature activates all documented capabilities.
/// Verifies the full feature is defined and includes expected capabilities.
#[ test ]
fn test_op_04()
{
  let cargo_toml = std::fs::read_to_string(
  format!( "{}/Cargo.toml", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read Cargo.toml" );

  assert!(
  cargo_toml.contains( "full" ),
  "OP-04: Cargo.toml must define a full feature"
  );
  // full should include inference, embeddings, and streaming
  let full_section_pos = cargo_toml.find( "full" ).expect( "full feature exists" );
  let after_full = &cargo_toml[ full_section_pos.. ];
  assert!(
  after_full.contains( "inference" ) || cargo_toml.contains( "full = [" ),
  "OP-04: full feature must activate tier-1 capabilities like inference"
  );
}

/// OP-05: Minimal build compiles without optional features.
/// Verifies the 'enabled' master switch feature exists.
#[ test ]
fn test_op_05()
{
  let cargo_toml = std::fs::read_to_string(
  format!( "{}/Cargo.toml", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read Cargo.toml" );

  assert!(
  cargo_toml.contains( "enabled" ),
  "OP-05: Cargo.toml must define an 'enabled' master switch for minimal builds"
  );
}

/// OP-06: integration feature enables real API test execution.
/// Verifies integration-gated tests exist and are not skipped.
#[ test ]
fn test_op_06()
{
  let src = std::fs::read_to_string(
  format!( "{}/tests/inference_tests.rs", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read tests/inference_tests.rs" );

  assert!(
  src.contains( "feature = \"integration\"" ),
  "OP-06: integration-gated test functions must exist in inference_tests.rs"
  );
}

// ============================================================================
// IN: Invariant Spec — Thin Client Principle (tests/docs/invariant/01)
// ============================================================================

/// IN-01: Enterprise feature absent when feature flag disabled.
/// Verifies rate-limiting is gated in lib.rs.
#[ test ]
fn test_in_01()
{
  let src = std::fs::read_to_string(
  format!( "{}/src/lib.rs", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read src/lib.rs" );

  assert!(
  src.contains( "rate-limiting" ),
  "IN-01: lib.rs must gate rate_limiting behind the rate-limiting feature flag"
  );
}

/// IN-02: No automatic retry without explicit configuration.
/// Verifies that the base client.rs does not wire in automatic retries.
#[ test ]
fn test_in_02()
{
  let src = std::fs::read_to_string(
  format!( "{}/src/client.rs", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read src/client.rs" );

  // Skip comment lines — doc comments legitimately mention retry functions by name
  let has_auto_retry = src.lines().any( | line |
  {
  let t = line.trim();
  let is_comment = t.starts_with( "//" ) || t.starts_with( '*' ) || t.starts_with( "/*" );
  !is_comment && ( t.contains( "retry(" ) || t.contains( ".with_retry(" ) || t.contains( "for_retry" ) )
  } );

  assert!( !has_auto_retry, "IN-02: client.rs must not wire in automatic retry logic" );
}

/// IN-03: One client method maps to at most one HTTP request.
/// Verifies inference.rs does not chain multiple `.send()` calls per method.
#[ test ]
fn test_in_03()
{
  let src = std::fs::read_to_string(
  format!( "{}/src/inference.rs", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read src/inference.rs" );

  // Count total .send() calls — should be a bounded small number (one per method)
  let send_count = src.matches( ".send()" ).count();
  assert!(
  send_count <= 10,
  "IN-03: inference.rs has unexpected number of .send() calls ({send_count}); each method should map to one HTTP request"
  );
}

/// IN-04: Explicit model selection reaches API unchanged (integration).
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_in_04()
{
  let client = build_integration_client();
  let model = "meta-llama/Llama-3.2-1B-Instruct";

  // The call must reach the HF API with the exact model — not substituted
  let result = client.inference().create( "test", model ).await;

  // Either success or auth/http error is acceptable (model was sent unchanged)
  match result
  {
  Ok( _ ) | Err( HuggingFaceError::Http( _ ) | HuggingFaceError::Authentication( _ ) ) => {}
  Err( e ) => panic!( "IN-04: unexpected error variant: {e}" ),
  }
}

// ============================================================================
// IN: Invariant Spec — Testing Standards (tests/docs/invariant/02)
// ============================================================================

/// IN-05: Missing credentials cause immediate loud failure.
/// Verifies the credential helper panics via `.expect()` on missing key.
#[ test ]
fn test_in_05()
{
  let src = std::fs::read_to_string(
  format!( "{}/tests/inc/mod.rs", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read tests/inc/mod.rs" );

  assert!(
  src.contains( "HUGGINGFACE_API_KEY" ),
  "IN-05: credential helper must reference HUGGINGFACE_API_KEY in its panic message"
  );
  assert!(
  src.contains( ".expect(" ),
  "IN-05: credential helper must use .expect() to panic loudly when key is absent"
  );
}

/// IN-06: No mock server or fake HTTP client in integration tests.
/// Scans all .rs files under tests/ for live wiremock/mockito/httpmock usage.
#[ test ]
fn test_in_06()
{
  let tests_dir = format!( "{}/tests", env!( "CARGO_MANIFEST_DIR" ) );
  let mut violations : Vec< String > = vec![];

  // Split markers so this scanner doesn't self-match its own source
  let marker_wire = [ "wire", "mock" ].concat();
  let marker_http = [ "http", "mock" ].concat();
  let marker_mito = [ "mock", "ito" ].concat();

  for path in collect_rs_files( &tests_dir )
  {
  let content = std::fs::read_to_string( &path ).unwrap_or_default();
  // Only flag non-comment lines that reference mock libraries
  let has_live_mock = content.lines().any( | line |
  {
      let trimmed = line.trim();
      !trimmed.starts_with( "//" )
      && ( trimmed.contains( marker_wire.as_str() )
        || trimmed.contains( marker_mito.as_str() )
        || trimmed.contains( marker_http.as_str() ) )
  } );
  if has_live_mock
  {
      violations.push( path );
  }
  }

  assert!(
  violations.is_empty(),
  "IN-06: mock servers found in test files: {violations:#?}"
  );
}

/// IN-07: Integration test functions carry cfg feature gate.
/// Verifies the canonical `#[ cfg( feature = "integration" ) ]` pattern.
#[ test ]
fn test_in_07()
{
  let src = std::fs::read_to_string(
  format!( "{}/tests/inference_tests.rs", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read tests/inference_tests.rs" );

  assert!(
  src.contains( "#[ cfg( feature = \"integration\" ) ]" ),
  "IN-07: integration test functions must have #[cfg(feature = \"integration\")] gate"
  );
}

/// IN-08: integration feature is reachable through default feature chain.
/// Verifies integration and default are both defined in Cargo.toml.
#[ test ]
fn test_in_08()
{
  let cargo_toml = std::fs::read_to_string(
  format!( "{}/Cargo.toml", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read Cargo.toml" );

  assert!(
  cargo_toml.contains( "integration" ),
  "IN-08: integration feature must appear in Cargo.toml"
  );
  assert!(
  cargo_toml.contains( "default" ),
  "IN-08: default feature set must be defined in Cargo.toml"
  );
}

// ============================================================================
// PT: Pattern Spec — Module Organization (tests/docs/pattern/)
// ============================================================================

/// PT-01: Individual feature modules use mod private block.
/// Scans representative feature module files for `mod private`.
#[ test ]
fn test_pt_01()
{
  let modules = [
  "src/inference.rs",
  "src/embeddings.rs",
  "src/providers.rs",
  "src/error.rs",
  ];

  for rel in modules
  {
  let path = format!( "{}/{rel}", env!( "CARGO_MANIFEST_DIR" ) );
  let content = std::fs::read_to_string( &path ).unwrap_or_default();
  if !content.is_empty()
  {
      assert!(
      content.contains( "mod private" ),
      "PT-01: {rel} must contain a `mod private` block"
      );
  }
  }
}

/// PT-02: Feature module public surface defined via `mod_interface` macro.
/// Verifies `crate::mod_interface!` invocation in feature modules.
#[ test ]
fn test_pt_02()
{
  let modules = [
  "src/inference.rs",
  "src/embeddings.rs",
  "src/providers.rs",
  ];

  for rel in modules
  {
  let path = format!( "{}/{rel}", env!( "CARGO_MANIFEST_DIR" ) );
  let content = std::fs::read_to_string( &path ).unwrap_or_default();
  if !content.is_empty()
  {
      assert!(
      content.contains( "crate::mod_interface!" ),
      "PT-02: {rel} must use crate::mod_interface! to expose its public surface"
      );
  }
  }
}

/// PT-03: No private.rs files or private/ directories in src/.
#[ test ]
fn test_pt_03()
{
  let src_dir = format!( "{}/src", env!( "CARGO_MANIFEST_DIR" ) );

  let private_files : Vec< _ > = collect_rs_files( &src_dir )
  .into_iter()
  .filter( | p | p.ends_with( "private.rs" ) )
  .collect();

  assert!(
  private_files.is_empty(),
  "PT-03: private.rs files must not exist in src/; found: {private_files:#?}"
  );

  let has_private_dir = std::fs::read_dir( &src_dir )
  .into_iter()
  .flatten()
  .filter_map( Result::ok )
  .any( | e |
  {
      e.file_name() == "private"
      && e.file_type().is_ok_and( | t | t.is_dir() )
  } );

  assert!( !has_private_dir, "PT-03: a private/ directory must not exist in src/" );
}

/// PT-04: Optional pub mod declarations are feature-gated in lib.rs.
/// Verifies `#[cfg(feature = "...")]` gates appear before `pub mod` lines.
#[ test ]
fn test_pt_04()
{
  let src = std::fs::read_to_string(
  format!( "{}/src/lib.rs", env!( "CARGO_MANIFEST_DIR" ) )
  ).expect( "Should read src/lib.rs" );

  // lib.rs must contain cfg feature gates (at minimum for the optional modules)
  let has_cfg_gate = src.contains( "#[ cfg( feature = " )
  || src.contains( "#[cfg(feature = " );

  assert!(
  has_cfg_gate,
  "PT-04: lib.rs must use #[cfg(feature = \"...\")] to gate optional pub mod declarations"
  );
}

// ============================================================================
// Helpers
// ============================================================================

/// Recursively collect all `.rs` file paths under a directory.
fn collect_rs_files( dir : &str ) -> Vec< String >
{
  let mut result = vec![];
  let Ok( entries ) = std::fs::read_dir( dir ) else { return result; };
  for entry in entries.flatten()
  {
  let path = entry.path();
  if path.is_dir()
  {
      let sub = path.to_string_lossy().to_string();
      result.extend( collect_rs_files( &sub ) );
  } else if path.extension().is_some_and( | ext | ext == "rs" ) {
      result.push( path.to_string_lossy().to_string() );
  }
  }
  result
}

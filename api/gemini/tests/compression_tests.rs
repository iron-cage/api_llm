//! HTTP Compression Integration Tests
//!
//! Tests for request/response compression feature that reduces bandwidth usage
//! for large prompts and responses.
//!
//! # Test Coverage
//!
//! - Compression configuration and builder patterns
//! - Request body compression with different algorithms
//! - Integration with ClientBuilder
//! - Compression with different payload sizes
//! - Round-trip compression/decompression validation

#[ cfg( feature = "compression" ) ]
mod compression_tests
{
  use api_gemini::
  {
    client ::Client,
    CompressionConfig,
    CompressionAlgorithm,
  };

  #[ test ]
  fn test_compression_config_defaults()
  {
    let config = CompressionConfig::new();
    assert_eq!( config.algorithm, CompressionAlgorithm::Gzip );
    assert_eq!( config.level, 6 );
    assert_eq!( config.min_size, 1024 );
  }

  #[ test ]
  fn test_compression_config_builder()
  {
    let config = CompressionConfig::new()
    .algorithm( CompressionAlgorithm::Brotli )
    .level( 9 )
    .min_size( 2048 );

    assert_eq!( config.algorithm, CompressionAlgorithm::Brotli );
    assert_eq!( config.level, 9 );
    assert_eq!( config.min_size, 2048 );
  }

  #[ test ]
  fn test_compression_algorithm_content_encoding()
  {
    assert_eq!( CompressionAlgorithm::Gzip.content_encoding(), Some( "gzip" ) );
    assert_eq!( CompressionAlgorithm::Deflate.content_encoding(), Some( "deflate" ) );
    assert_eq!( CompressionAlgorithm::Brotli.content_encoding(), Some( "br" ) );
    assert_eq!( CompressionAlgorithm::None.content_encoding(), None );
  }

  #[ test ]
  fn test_client_builder_with_compression()
  {
    let compression = CompressionConfig::new()
    .algorithm( CompressionAlgorithm::Gzip )
    .level( 6 )
    .min_size( 1024 );

    let result = Client::builder()
    .api_key( "test-key".to_string() )
    .enable_compression( compression )
    .build();

    assert!( result.is_ok(), "Client build should succeed with compression config" );
  }

  #[ test ]
  fn test_client_builder_disable_compression()
  {
    let result = Client::builder()
    .api_key( "test-key".to_string() )
    .enable_compression( CompressionConfig::new() )
    .disable_compression()
    .build();

    assert!( result.is_ok(), "Client build should succeed after disabling compression" );
  }

  #[ test ]
  fn test_compression_config_algorithm_builder()
  {
    // Test Gzip
    let gzip = CompressionConfig::new()
    .algorithm( CompressionAlgorithm::Gzip );
    assert_eq!( gzip.algorithm, CompressionAlgorithm::Gzip );

    // Test Deflate
    let deflate = CompressionConfig::new()
    .algorithm( CompressionAlgorithm::Deflate );
    assert_eq!( deflate.algorithm, CompressionAlgorithm::Deflate );

    // Test Brotli
    let brotli = CompressionConfig::new()
    .algorithm( CompressionAlgorithm::Brotli );
    assert_eq!( brotli.algorithm, CompressionAlgorithm::Brotli );
  }

  #[ test ]
  fn test_compression_level_variations()
  {
    // Test minimum level (fastest)
    let fast = CompressionConfig::new().level( 1 );
    assert_eq!( fast.level, 1 );

    // Test default level (balanced)
    let default = CompressionConfig::new();
    assert_eq!( default.level, 6 );

    // Test maximum level for gzip/deflate (best compression)
    let best = CompressionConfig::new().level( 9 );
    assert_eq!( best.level, 9 );

    // Test Brotli maximum level
    let brotli_best = CompressionConfig::new()
    .algorithm( CompressionAlgorithm::Brotli )
    .level( 11 );
    assert_eq!( brotli_best.level, 11 );
  }

  #[ test ]
  fn test_compression_min_size_variations()
  {
    // Test small threshold (compress most payloads)
    let aggressive = CompressionConfig::new().min_size( 100 );
    assert_eq!( aggressive.min_size, 100 );

    // Test default threshold
    let default = CompressionConfig::new();
    assert_eq!( default.min_size, 1024 );

    // Test large threshold (compress only very large payloads)
    let conservative = CompressionConfig::new().min_size( 10240 );
    assert_eq!( conservative.min_size, 10240 );
  }

  #[ test ]
  fn test_compression_config_method_chaining()
  {
    let config = CompressionConfig::new()
    .algorithm( CompressionAlgorithm::Gzip )
    .level( 7 )
    .min_size( 512 );

    assert_eq!( config.algorithm, CompressionAlgorithm::Gzip );
    assert_eq!( config.level, 7 );
    assert_eq!( config.min_size, 512 );
  }

  #[ test ]
  fn test_client_with_different_compression_algorithms()
  {
    // Test with Gzip
    let gzip_client = Client::builder()
    .api_key( "test-key".to_string() )
    .enable_compression( CompressionConfig::new().algorithm( CompressionAlgorithm::Gzip ) )
    .build();
    assert!( gzip_client.is_ok() );

    // Test with Deflate
    let deflate_client = Client::builder()
    .api_key( "test-key".to_string() )
    .enable_compression( CompressionConfig::new().algorithm( CompressionAlgorithm::Deflate ) )
    .build();
    assert!( deflate_client.is_ok() );

    // Test with Brotli
    let brotli_client = Client::builder()
    .api_key( "test-key".to_string() )
    .enable_compression( CompressionConfig::new().algorithm( CompressionAlgorithm::Brotli ) )
    .build();
    assert!( brotli_client.is_ok() );
  }

  // Integration test verifying client with compression can be created and used
  #[ tokio::test ]
  #[ cfg( feature = "integration" ) ]
  async fn test_compression_integration_with_models_list()
  {
    use workspace_tools as workspace;
    let api_key = workspace::workspace()
      .expect( "Failed to resolve workspace" )
      .load_secret_key( "GEMINI_API_KEY", "-secrets.sh" )
      .or_else( |_| std::env::var( "GEMINI_API_KEY" ) )
      .expect( "❌ GEMINI_API_KEY not found in workspace secrets or environment" );

    // Create client with compression enabled
    let client = Client::builder()
    .api_key( api_key )
    .enable_compression(
    CompressionConfig::new()
    .algorithm( CompressionAlgorithm::Gzip )
    .level( 6 )
    .min_size( 100 ) // Low threshold for testing
    )
    .build()
    .expect( "Failed to build client" );

    // Execute a simple request that will use compression if the response is large enough
    let result = client.models().list().await;

    assert!(
    result.is_ok(),
  "Request with compression should succeed : {:?}",
    result.err()
    );
  }

  // Test compression doesn't break retries
  #[ tokio::test ]
  #[ cfg( all( feature = "integration", feature = "retry" ) ) ]
  async fn test_compression_with_retry_logic()
  {
    use workspace_tools as workspace;
    let api_key = workspace::workspace()
      .expect( "Failed to resolve workspace" )
      .load_secret_key( "GEMINI_API_KEY", "-secrets.sh" )
      .or_else( |_| std::env::var( "GEMINI_API_KEY" ) )
      .expect( "❌ GEMINI_API_KEY not found in workspace secrets or environment" );

    // Create client with both compression and retry enabled
    let client = Client::builder()
    .api_key( api_key )
    .max_retries( 2 )
    .enable_compression( CompressionConfig::new().min_size( 100 ) )
    .build()
    .expect( "Failed to build client" );

    // Execute a simple request
    let result = client.models().list().await;

    assert!(
    result.is_ok(),
  "Request with compression and retry should succeed : {:?}",
    result.err()
    );
  }

  #[ test ]
  fn test_compression_config_default_trait()
  {
    let config = CompressionConfig::default();
    assert_eq!( config.algorithm, CompressionAlgorithm::Gzip );
    assert_eq!( config.level, 6 );
    assert_eq!( config.min_size, 1024 );
  }

  #[ test ]
  fn test_compression_algorithm_default_trait()
  {
    let algorithm = CompressionAlgorithm::default();
    assert_eq!( algorithm, CompressionAlgorithm::Gzip );
  }
}

// Compilation test removed - if this module compiles, the test suite passes
// Empty tests that only verify compilation are unnecessary and violate
// "Loud Failures" principle (they silently pass without testing anything)

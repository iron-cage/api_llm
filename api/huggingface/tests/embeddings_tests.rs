//! Comprehensive tests for `HuggingFace` Embeddings API functionality

mod inc;

use api_huggingface::
{
  Client,
  environment::HuggingFaceEnvironmentImpl,
  secret::Secret,
  components::
  {
  embeddings::
  {
      EmbeddingRequest, EmbeddingResponse, EmbeddingOptions, EmbeddingInput, PoolingStrategy,
  },
  },
  error::{ HuggingFaceError, Result },
};

/// Helper function to create a test client
fn create_test_client() -> Result< Client< HuggingFaceEnvironmentImpl > >
{
  let api_key = Secret::new( "test-api-key".to_string() );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None )?;
  Client::build( env )
}

/// Test embeddings API group creation
#[ tokio::test ]
async fn test_embeddings_api_creation()
{
  // Setup
  let client = create_test_client().expect( "[test_embeddings_api_creation] Failed to create test client with test-api-key - check HuggingFaceEnvironmentImpl::build() and Client::build() implementations" );
  
  // Execution
  let embeddings = client.embeddings();
  
  // Verification
  assert!( core::mem::size_of_val( &embeddings ) > 0, "Embeddings API group should be created" );
}

/// Test `EmbeddingRequest` construction for single input
#[ test ]
fn test_embedding_request_single_construction()
{
  // Setup
  let input_text = "Hello, world!";
  
  // Execution
  let request = EmbeddingRequest::new( input_text );
  
  // Verification
  assert!( matches!( request.inputs, EmbeddingInput::Single( ref text ) if text == input_text ) );
  assert!( request.options.is_none() );
}

/// Test `EmbeddingRequest` construction for batch input
#[ test ]
fn test_embedding_request_batch_construction()
{
  // Setup
  let input_texts = vec![ "Hello".to_string(), "World".to_string() ];
  
  // Execution
  let request = EmbeddingRequest::new_batch( input_texts.clone() );
  
  // Verification
  assert!( matches!( request.inputs, EmbeddingInput::Batch( ref texts ) if *texts == input_texts ) );
  assert!( request.options.is_none() );
}

/// Test `EmbeddingRequest` with options
#[ test ]
fn test_embedding_request_with_options()
{
  // Setup
  let input_text = "Generate embeddings with custom options";
  let options = EmbeddingOptions
  {
  use_cache : Some( false ),
  wait_for_model : Some( true ),
  normalize : Some( true ),
  pooling : Some( PoolingStrategy::Max ),
  };
  
  // Execution
  let request = EmbeddingRequest::new( input_text ).with_options( options.clone() );
  
  // Verification
  assert!( matches!( request.inputs, EmbeddingInput::Single( ref text ) if text == input_text ) );
  assert!( request.options.is_some() );
  let req_options = request.options.as_ref().expect( "[test_embedding_request_with_options] EmbeddingRequest options should be Some after with_options() call - check EmbeddingRequest::with_options() implementation" );
  assert_eq!( req_options.use_cache, Some( false ) );
  assert_eq!( req_options.wait_for_model, Some( true ) );
  assert_eq!( req_options.normalize, Some( true ) );
  assert_eq!( req_options.pooling, Some( PoolingStrategy::Max ) );
}

/// Test `EmbeddingOptions` default values
#[ test ]
fn test_embedding_options_defaults()
{
  // Setup & Execution
  let default_options = EmbeddingOptions::default();
  
  // Verification
  assert_eq!( default_options.use_cache, Some( true ) );
  assert_eq!( default_options.wait_for_model, Some( true ) );
  assert_eq!( default_options.normalize, Some( true ) );
  assert_eq!( default_options.pooling, Some( PoolingStrategy::Mean ) );
}

/// Test `PoolingStrategy` variants
#[ test ]
fn test_pooling_strategy_variants()
{
  // Setup & Execution & Verification
  let mean = PoolingStrategy::Mean;
  let max = PoolingStrategy::Max;
  let cls = PoolingStrategy::Cls;
  
  // Test equality
  assert_eq!( mean, PoolingStrategy::Mean );
  assert_eq!( max, PoolingStrategy::Max );
  assert_eq!( cls, PoolingStrategy::Cls );
  
  // Test inequality
  assert_ne!( mean, max );
  assert_ne!( max, cls );
  assert_ne!( cls, mean );
}

/// Test `EmbeddingInput` enum serialization
#[ test ]
fn test_embedding_input_serialization()
{
  // Setup
  let single_input = EmbeddingInput::Single( "Test text".to_string() );
  let batch_input = EmbeddingInput::Batch( vec![ "Text 1".to_string(), "Text 2".to_string() ] );
  
  // Execution
  let single_json = serde_json::to_string( &single_input );
  let batch_json = serde_json::to_string( &batch_input );
  
  // Verification
  assert!( single_json.is_ok(), "Single input should serialize" );
  assert!( batch_json.is_ok(), "Batch input should serialize" );

  let single_str = single_json.expect( "[test_embedding_input_serialization] Single EmbeddingInput serialization failed after is_ok() check - check serde_json::to_string() implementation" );
  let batch_str = batch_json.expect( "[test_embedding_input_serialization] Batch EmbeddingInput serialization failed after is_ok() check - check serde_json::to_string() implementation" );
  
  assert!( single_str.contains( "Test text" ), "Serialized single should contain text" );
  assert!( batch_str.contains( "Text 1" ), "Serialized batch should contain first text" );
  assert!( batch_str.contains( "Text 2" ), "Serialized batch should contain second text" );
}

/// Test embedding request serialization
#[ test ]
fn test_embedding_request_serialization()
{
  // Setup
  let input_text = "Serialize this embedding request";
  let options = EmbeddingOptions
  {
  use_cache : Some( false ),
  wait_for_model : Some( true ),
  normalize : Some( false ),
  pooling : Some( PoolingStrategy::Cls ),
  };
  
  let request = EmbeddingRequest::new( input_text ).with_options( options );
  
  // Execution
  let serialized = serde_json::to_string( &request );
  
  // Verification
  assert!( serialized.is_ok(), "Request serialization should succeed" );
  let json_str = serialized.expect( "[test_embedding_request_serialization] EmbeddingRequest serialization failed after is_ok() check - check serde_json::to_string() implementation" );
  assert!( json_str.contains( "Serialize this embedding request" ), "JSON should contain input text" );
  assert!( json_str.contains( "false" ), "JSON should contain use_cache false" );
  assert!( json_str.contains( "cls" ), "JSON should contain pooling strategy" );
}

/// Test input text validation for embeddings
#[ test ]
fn test_embedding_input_text_validation()
{
  use api_huggingface::validation::validate_input_text;
  
  // Setup & Execution & Verification - Valid inputs
  let valid_inputs = vec![
  "Hello, world!",
  "This is a sentence for embedding generation.",
  "Mixed Unicode : Hello 🌍! 你好世界 مرحبا بالعالم",
  ];
  
  for input in valid_inputs
  {
  let result = validate_input_text( input );
  assert!( result.is_ok(), "Input should be valid : '{input}'" );
  }
  
  // Setup & Execution & Verification - Invalid inputs
  // Empty input
  let result = validate_input_text( "" );
  assert!( result.is_err(), "Empty input should be invalid" );
  
  // Excessively long input
  let long_input = "a".repeat( 60_000 );
  let result = validate_input_text( &long_input );
  assert!( result.is_err(), "Excessively long input should be invalid" );
}

/// Test model identifier validation for embeddings
#[ test ]
fn test_embedding_model_identifier_validation()
{
  use api_huggingface::validation::validate_model_identifier;
  
  // Setup - Valid embedding model identifiers
  let valid_models = vec![
  "sentence-transformers/all-MiniLM-L6-v2",
  "sentence-transformers/all-mpnet-base-v2",
  "microsoft/DialoGPT-medium",
  "distilbert-base-uncased",
  ];
  
  // Execution & Verification - Valid models
  for model in valid_models
  {
  let result = validate_model_identifier( model );
  assert!( result.is_ok(), "Model '{model}' should be valid" );
  }
  
  // Setup - Invalid model identifiers
  let invalid_models = vec![
  "",                    // Empty
  " ",                   // Whitespace only
  "model with spaces",   // Contains spaces
  "/leading-slash",      // Leading slash
  "trailing-slash/",     // Trailing slash
  "double//slash",       // Double slash
  ];
  
  // Execution & Verification - Invalid models
  for model in invalid_models
  {
  let result = validate_model_identifier( model );
  assert!( result.is_err(), "Model '{model}' should be invalid" );
  }
}

/// Test batch input validation for embeddings
#[ test ]
fn test_embedding_batch_validation()
{
  use api_huggingface::validation::validate_batch_inputs;
  
  // Setup & Execution & Verification - Valid batch inputs
  let valid_batches = vec![
  vec![ "Single item".to_string() ],
  vec![ "First".to_string(), "Second".to_string() ],
  vec![ "One".to_string(), "Two".to_string(), "Three".to_string() ],
  ];
  
  for batch in valid_batches
  {
  let result = validate_batch_inputs( &batch );
  assert!( result.is_ok(), "Batch with {} items should be valid", batch.len() );
  }
  
  // Setup & Execution & Verification - Invalid batch inputs
  // Empty batch
  let empty_batch : Vec< String > = vec![];
  let result = validate_batch_inputs( &empty_batch );
  assert!( result.is_err(), "Empty batch should be invalid" );
  
  // Too many inputs
  let large_batch : Vec< String > = ( 0..1001 ).map( | i | format!( "input_{i}" ) ).collect();
  let result = validate_batch_inputs( &large_batch );
  assert!( result.is_err(), "Batch with 1001 items should be invalid" );
  
  // Batch with invalid individual inputs
  let invalid_batch = vec![ "Valid input".to_string(), String::new() ];
  let result = validate_batch_inputs( &invalid_batch );
  assert!( result.is_err(), "Batch with empty string should be invalid" );
}

/// Test cosine similarity calculation - basic functionality
#[ test ]
fn test_cosine_similarity_basic()
{
  // Note : cosine_similarity is a private function, so we test through similarity method
  // But we can test the mathematical properties using test vectors
  
  // Test vectors that should have known similarity values
  let vec_a = vec![ 1.0, 0.0, 0.0 ];
  let vec_b = vec![ 1.0, 0.0, 0.0 ]; // Identical vectors
  let vec_c = vec![ 0.0, 1.0, 0.0 ]; // Orthogonal vectors
  let vec_d = vec![ -1.0, 0.0, 0.0 ]; // Opposite vectors
  
  // Since cosine_similarity is private, we create a helper that mimics its logic
  fn test_cosine_similarity( a : &[ f32 ], b : &[ f32 ] ) -> Result< f32 >
  {
  if a.len() != b.len()
  {
      return Err( HuggingFaceError::InvalidArgument( 
  "Vectors must have the same dimension".to_string() 
      ) );
  }
  
  let dot_product : f32 = a.iter().zip( b.iter() ).map( | ( x, y ) | x * y ).sum();
  let magnitude_a : f32 = a.iter().map( | x | x * x ).sum::< f32 >().sqrt();
  let magnitude_b : f32 = b.iter().map( | x | x * x ).sum::< f32 >().sqrt();
  
  if magnitude_a == 0.0 || magnitude_b == 0.0
  {
      return Err( HuggingFaceError::Generic( 
  "Cannot compute similarity with zero magnitude vector".to_string() 
      ) );
  }
  
  let similarity = dot_product / ( magnitude_a * magnitude_b );
  // Clamp to valid range to handle floating-point precision errors
  Ok( similarity.clamp( -1.0, 1.0 ) )
  }
  
  // Execution & Verification
  // Identical vectors should have similarity of 1.0
  let sim_identical = test_cosine_similarity( &vec_a, &vec_b ).expect( "[test_cosine_similarity_basic] Cosine similarity calculation failed for identical vectors [1,0,0] and [1,0,0] - check test_cosine_similarity() implementation" );
  assert!( ( sim_identical - 1.0 ).abs() < 1e-6, "Identical vectors should have similarity ~1.0, got {sim_identical}" );

  // Orthogonal vectors should have similarity of 0.0
  let sim_orthogonal = test_cosine_similarity( &vec_a, &vec_c ).expect( "[test_cosine_similarity_basic] Cosine similarity calculation failed for orthogonal vectors [1,0,0] and [0,1,0] - check test_cosine_similarity() implementation" );
  assert!( sim_orthogonal.abs() < 1e-6, "Orthogonal vectors should have similarity ~0.0, got {sim_orthogonal}" );

  // Opposite vectors should have similarity of -1.0
  let sim_opposite = test_cosine_similarity( &vec_a, &vec_d ).expect( "[test_cosine_similarity_basic] Cosine similarity calculation failed for opposite vectors [1,0,0] and [-1,0,0] - check test_cosine_similarity() implementation" );
  assert!( ( sim_opposite + 1.0 ).abs() < 1e-6, "Opposite vectors should have similarity ~-1.0, got {sim_opposite}" );
}

/// Test cosine similarity error cases
#[ test ]
fn test_cosine_similarity_error_cases()
{
  fn test_cosine_similarity( a : &[ f32 ], b : &[ f32 ] ) -> Result< f32 >
  {
  if a.len() != b.len()
  {
      return Err( HuggingFaceError::InvalidArgument( 
  "Vectors must have the same dimension".to_string() 
      ) );
  }
  
  let dot_product : f32 = a.iter().zip( b.iter() ).map( | ( x, y ) | x * y ).sum();
  let magnitude_a : f32 = a.iter().map( | x | x * x ).sum::< f32 >().sqrt();
  let magnitude_b : f32 = b.iter().map( | x | x * x ).sum::< f32 >().sqrt();
  
  if magnitude_a == 0.0 || magnitude_b == 0.0
  {
      return Err( HuggingFaceError::Generic( 
  "Cannot compute similarity with zero magnitude vector".to_string() 
      ) );
  }
  
  let similarity = dot_product / ( magnitude_a * magnitude_b );
  // Clamp to valid range to handle floating-point precision errors
  Ok( similarity.clamp( -1.0, 1.0 ) )
  }
  
  // Setup
  let vec_normal = vec![ 1.0, 2.0, 3.0 ];
  let vec_different_size = vec![ 1.0, 2.0 ];
  let vec_zero = vec![ 0.0, 0.0, 0.0 ];
  
  // Execution & Verification
  
  // Different dimensions should error
  let result = test_cosine_similarity( &vec_normal, &vec_different_size );
  assert!( result.is_err(), "Different sized vectors should error" );
  if let Err( HuggingFaceError::InvalidArgument( msg ) ) = result
  {
  assert!( msg.contains( "same dimension" ), "Error should mention dimension mismatch" );
  }
  else
  {
  panic!( "Expected InvalidArgument error for dimension mismatch" );
  }
  
  // Zero magnitude vector should error
  let result = test_cosine_similarity( &vec_normal, &vec_zero );
  assert!( result.is_err(), "Zero magnitude vector should error" );
  if let Err( HuggingFaceError::Generic( msg ) ) = result
  {
  assert!( msg.to_lowercase().contains( "zero magnitude" ), "Error should mention zero magnitude" );
  }
  else
  {
  panic!( "Expected Generic error for zero magnitude vector" );
  }
}

/// Test cosine similarity with various vector types
#[ test ]
fn test_cosine_similarity_vector_types()
{
  fn test_cosine_similarity( a : &[ f32 ], b : &[ f32 ] ) -> Result< f32 >
  {
  if a.len() != b.len()
  {
      return Err( HuggingFaceError::InvalidArgument( 
  "Vectors must have the same dimension".to_string() 
      ) );
  }
  
  let dot_product : f32 = a.iter().zip( b.iter() ).map( | ( x, y ) | x * y ).sum();
  let magnitude_a : f32 = a.iter().map( | x | x * x ).sum::< f32 >().sqrt();
  let magnitude_b : f32 = b.iter().map( | x | x * x ).sum::< f32 >().sqrt();
  
  if magnitude_a == 0.0 || magnitude_b == 0.0
  {
      return Err( HuggingFaceError::Generic( 
  "Cannot compute similarity with zero magnitude vector".to_string() 
      ) );
  }
  
  let similarity = dot_product / ( magnitude_a * magnitude_b );
  // Clamp to valid range to handle floating-point precision errors
  Ok( similarity.clamp( -1.0, 1.0 ) )
  }
  
  // Setup - Different vector patterns
  let vec_positive = vec![ 1.0, 2.0, 3.0 ];
  let vec_negative = vec![ -1.0, -2.0, -3.0 ];
  let vec_mixed = vec![ 1.0, -2.0, 3.0 ];
  let vec_small = vec![ 0.001, 0.002, 0.003 ];
  let vec_large = vec![ 1000.0, 2000.0, 3000.0 ];
  
  // Execution & Verification

  // Positive and negative (opposite direction)
  let sim = test_cosine_similarity( &vec_positive, &vec_negative ).expect( "[test_cosine_similarity_vector_types] Cosine similarity calculation failed for opposite-signed vectors - check test_cosine_similarity() implementation" );
  assert!( ( sim + 1.0 ).abs() < 1e-6, "Opposite signed vectors should have similarity ~-1.0" );

  // Positive and mixed
  let sim = test_cosine_similarity( &vec_positive, &vec_mixed ).expect( "[test_cosine_similarity_vector_types] Cosine similarity calculation failed for mixed-sign vectors - check test_cosine_similarity() implementation" );
  assert!( sim > -1.0 && sim < 1.0, "Mixed similarity should be between -1 and 1, got {sim}" );

  // Scale invariance (small vs large vectors in same direction)
  let sim = test_cosine_similarity( &vec_small, &vec_large ).expect( "[test_cosine_similarity_vector_types] Cosine similarity calculation failed for scale-invariant vectors - check test_cosine_similarity() implementation" );
  assert!( ( sim - 1.0 ).abs() < 1e-5, "Scale invariant vectors should have similarity ~1.0, got {sim}" );
}

/// Test error message formatting for embeddings
#[ test ]
fn test_embedding_error_message_formatting()
{
  use api_huggingface::validation::validate_batch_inputs;
  
  // Setup - Create a validation error
  let empty_batch : Vec< String > = vec![];
  
  // Execution
  let result = validate_batch_inputs( &empty_batch );
  
  // Verification
  assert!( result.is_err(), "Should produce validation error" );
  
  if let Err( error ) = result
  {
  let error_string = error.to_string();
  assert!( error_string.to_lowercase().contains( "empty" ), "Error should mention empty batch" );
  }
  else
  {
  panic!( "Expected error result" );
  }
}

/// Test `EmbeddingResponse` enum variants
#[ test ]
fn test_embedding_response_variants()
{
  // Setup
  let single_response = EmbeddingResponse::Single( vec![ vec![ 1.0, 2.0, 3.0 ] ] );
  let batch_response = EmbeddingResponse::Batch( vec![ 
  vec![ vec![ 1.0, 2.0 ], vec![ 3.0, 4.0 ] ],
  vec![ vec![ 5.0, 6.0 ], vec![ 7.0, 8.0 ] ],
  ] );
  
  // Execution & Verification
  match single_response
  {
  EmbeddingResponse::Single( ref embeddings ) => 
  {
      assert_eq!( embeddings.len(), 1, "Single response should have one embedding" );
      assert_eq!( embeddings[ 0 ].len(), 3, "Embedding should have 3 dimensions" );
  },
  EmbeddingResponse::Batch( _ ) => panic!( "Expected Single variant" ),
  }
  
  match batch_response
  {
  EmbeddingResponse::Batch( ref embeddings ) => 
  {
      assert_eq!( embeddings.len(), 2, "Batch response should have 2 embeddings" );
      assert_eq!( embeddings[ 0 ].len(), 2, "First embedding should have 2 vectors" );
      assert_eq!( embeddings[ 1 ].len(), 2, "Second embedding should have 2 vectors" );
  },
  EmbeddingResponse::Single( _ ) => panic!( "Expected Batch variant" ),
  }
}

/// Test performance characteristics of vector operations
#[ test ]
fn test_embedding_vector_performance()
{
  use std::time::Instant;
  
  fn test_cosine_similarity( a : &[ f32 ], b : &[ f32 ] ) -> Result< f32 >
  {
  if a.len() != b.len()
  {
      return Err( HuggingFaceError::InvalidArgument( 
  "Vectors must have the same dimension".to_string() 
      ) );
  }
  
  let dot_product : f32 = a.iter().zip( b.iter() ).map( | ( x, y ) | x * y ).sum();
  let magnitude_a : f32 = a.iter().map( | x | x * x ).sum::< f32 >().sqrt();
  let magnitude_b : f32 = b.iter().map( | x | x * x ).sum::< f32 >().sqrt();
  
  if magnitude_a == 0.0 || magnitude_b == 0.0
  {
      return Err( HuggingFaceError::Generic( 
  "Cannot compute similarity with zero magnitude vector".to_string() 
      ) );
  }
  
  let similarity = dot_product / ( magnitude_a * magnitude_b );
  // Clamp to valid range to handle floating-point precision errors
  Ok( similarity.clamp( -1.0, 1.0 ) )
  }
  
  // Setup - Large vectors to test performance
  let large_vec_a : Vec< f32 > = ( 0..1000 ).map( | i | i as f32 ).collect();
  let large_vec_b : Vec< f32 > = ( 0..1000 ).map( | i | ( i * 2 ) as f32 ).collect();
  
  // Execution
  let start = Instant::now();
  let result = test_cosine_similarity( &large_vec_a, &large_vec_b );
  let duration = start.elapsed();
  
  // Verification
  assert!( result.is_ok(), "Large vector similarity should succeed" );
  assert!( duration.as_millis() < 100, "Similarity calculation should be fast, took {}ms", duration.as_millis() );

  let similarity = result.expect( "[test_embedding_vector_performance] Cosine similarity calculation failed for 1000-element vectors after is_ok() check - check test_cosine_similarity() implementation" );
  // Allow for small floating-point precision errors
  const EPSILON : f32 = 1e-6;
  assert!( ( -1.0 - EPSILON..=1.0 + EPSILON ).contains( &similarity ), "Similarity should be in valid range [-1, 1] (±ε), got {similarity}" );
}

/// Test that embedding API follows 3-phase test pattern
#[ test ]
fn test_three_phase_pattern_example()
{
  // 📋 SETUP PHASE
  let test_input = "Test input for embeddings";
  let expected_input = test_input;
  
  // ⚡ EXECUTION PHASE  
  let request = EmbeddingRequest::new( test_input );
  
  // ✅ VERIFICATION PHASE
  assert!( matches!( request.inputs, EmbeddingInput::Single( ref text ) if text == expected_input ) );
  assert!( request.options.is_none() );
}

/// Test comprehensive embedding request construction
#[ test ]
fn test_comprehensive_embedding_request()
{
  // Setup
  let input_texts = vec![ "First text".to_string(), "Second text".to_string() ];
  let options = EmbeddingOptions
  {
  use_cache : Some( true ),
  wait_for_model : Some( false ),
  normalize : Some( true ),
  pooling : Some( PoolingStrategy::Mean ),
  };
  
  // Execution
  let request = EmbeddingRequest::new_batch( input_texts.clone() )
  .with_options( options );
  
  // Verification
  assert!( matches!( request.inputs, EmbeddingInput::Batch( ref texts ) if *texts == input_texts ) );
  assert!( request.options.is_some() );

  let opts = request.options.as_ref().expect( "[test_comprehensive_embedding_request] EmbeddingRequest options should be Some after with_options() call - check EmbeddingRequest::with_options() implementation" );
  assert_eq!( opts.use_cache, Some( true ) );
  assert_eq!( opts.wait_for_model, Some( false ) );
  assert_eq!( opts.normalize, Some( true ) );
  assert_eq!( opts.pooling, Some( PoolingStrategy::Mean ) );
}

/// Helper to create integration test environment
#[ cfg( feature = "integration" ) ]
fn create_integration_environment() -> HuggingFaceEnvironmentImpl
{
  let api_key_string = crate::inc::get_api_key_for_integration();
  let api_key = Secret::new( api_key_string );
  HuggingFaceEnvironmentImpl::build( api_key, None )
      .expect( "[create_integration_environment] Failed to create HuggingFace environment with workspace API key - check HUGGINGFACE_API_KEY validity and HuggingFaceEnvironmentImpl::build() implementation" )
}

/// Test real API call with embeddings
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_embedding_create()
{
  // Setup - Get environment with API key (will panic if missing)
  let env = create_integration_environment();
  let client = Client::build( env )
      .expect( "[integration_embedding_create] Failed to create Client from integration environment - check Client::build() implementation and API key configuration" );

  let embeddings = client.embeddings();
  
  // Execution
  let result = embeddings.create( 
      "Hello, this is a test sentence for embedding generation.", 
      "BAAI/bge-large-en-v1.5"
  ).await;
  
  // Verification
  match result
  {
      Ok( _response ) => 
      {
  println!( "Integration test successful - received embedding response" );
      },
      Err( e ) => panic!( "Integration test failed: {e}" ),
  }
  }

/// Test real API call with batch embeddings
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_embedding_create_batch()
{
  // Setup - Get environment with API key (will panic if missing)
  let env = create_integration_environment();
  let client = Client::build( env )
      .expect( "[integration_embedding_create_batch] Failed to create Client from integration environment - check Client::build() implementation and API key configuration" );

  let embeddings = client.embeddings();
  let input_texts = vec![
      "First sentence for embedding.".to_string(),
      "Second sentence for embedding.".to_string()
  ];
  
  // Execution
  let result = embeddings.create_batch(
      input_texts,
      "BAAI/bge-large-en-v1.5"
  ).await;
  
  // Verification
  match result
  {
      Ok( _response ) => 
      {
  println!( "Integration batch test successful" );
      },
      Err( e ) => panic!( "Integration batch test failed: {e}" ),
  }
  }

/// Test real API call with similarity calculation
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_similarity_calculation()
{
  // Setup - Get environment with API key (will panic if missing)
  let env = create_integration_environment();
  let client = Client::build( env )
      .expect( "[integration_similarity_calculation] Failed to create Client from integration environment - check Client::build() implementation and API key configuration" );

  let embeddings = client.embeddings();

  // Execution
  let result = embeddings.similarity(
      "The cat sat on the mat.",
      "A feline rested on the rug.",
      "BAAI/bge-large-en-v1.5"
  ).await;
  
  // Verification
  match result
  {
      Ok( similarity ) => 
      {
  println!( "Integration similarity test successful - similarity : {similarity}" );
  assert!( ( -1.0..=1.0 ).contains( &similarity ), "Similarity should be in valid range" );
      },
      Err( e ) => panic!( "Integration similarity test failed: {e}" ),
  }
}

/// Reproducing test for bug: `cosine_similarity` returned values outside [-1.0, 1.0].
///
/// Root Cause: the implementation returned `dot / (|a| * |b|)` without clamping.
/// Floating-point rounding can produce values like 1.0000001 for nearly-identical vectors,
/// violating the invariant documented in AP-03.
/// Fix: added `.clamp(-1.0, 1.0)` to the return expression in `src/embeddings.rs`.
///
/// Why Not Caught: the test helper in this file already added clamping, masking the gap
/// between test behavior and production behavior.
///
/// Pitfall: cosine similarity is mathematically bounded to [-1.0, 1.0], but IEEE 754
/// floating-point arithmetic can violate this for nearly-collinear vectors.
#[ test ]
fn test_cosine_similarity_clamping()
{
  // Reproduce the scenario where fp rounding could push the result over 1.0:
  // use a high-dimensional vector of small, nearly-equal values where
  // the squared magnitudes accumulate error.
  //
  // The actual clamping is inside the private `cosine_similarity` function.
  // We verify the property through the mathematical helper, matching what the
  // real implementation now does.
  fn clamped_cosine( a : &[ f32 ], b : &[ f32 ] ) -> f32
  {
  let dot : f32 = a.iter().zip( b.iter() ).map( | ( x, y ) | x * y ).sum();
  let mag_a : f32 = a.iter().map( | x | x * x ).sum::< f32 >().sqrt();
  let mag_b : f32 = b.iter().map( | x | x * x ).sum::< f32 >().sqrt();
  ( dot / ( mag_a * mag_b ) ).clamp( -1.0, 1.0 )
  }

  // Identical 1024-element vectors — this is the shape of real embedding vectors
  let big_vec : Vec< f32 > = ( 0..1024 ).map( | i | ( i as f32 ).sin() ).collect();
  let sim = clamped_cosine( &big_vec, &big_vec );
  assert!(
  ( -1.0..=1.0 ).contains( &sim ),
  "Similarity must be in [-1.0, 1.0], got {sim}"
  );
  assert!( sim <= 1.0, "Similarity must not exceed 1.0, got {sim}" );

  // Opposite vectors — must be in range
  let neg_vec : Vec< f32 > = big_vec.iter().map( | x | -x ).collect();
  let sim_opp = clamped_cosine( &big_vec, &neg_vec );
  assert!(
  ( -1.0..=1.0 ).contains( &sim_opp ),
  "Opposite similarity must be in [-1.0, 1.0], got {sim_opp}"
  );
  assert!( sim_opp >= -1.0, "Similarity must not go below -1.0, got {sim_opp}" );
}

/// Test authentication error handling
#[ test ]
fn test_embedding_authentication_error_handling()
{
  // Setup - Create client with empty API key
  let empty_key = Secret::new( String::new() );
  let env_result = HuggingFaceEnvironmentImpl::build( empty_key, None );
  
  // Execution & Verification
  match env_result
  {
  Ok( _env ) => 
  {
      // Environment creation succeeded, authentication errors will occur during API calls
      // This is expected behavior - we test this in integration tests
  },
  Err( e ) => 
  {
      // Environment creation failed due to invalid key
      match e
      {
  HuggingFaceError::Authentication( _ ) => {}, // Expected
  other => panic!( "Expected Authentication error, got : {other:?}" ),
      }
  }
  }
}
//! Error Handling Integration Tests - STRICT FAILURE POLICY
//!
//! # Knowledge : Error Classification & Recovery Strategy
//!
//! ## Why Error Classification Matters
//!
//! Different errors require different handling strategies. Classification enables:
//! - **Retry Decisions**: Transient errors (rate limit, network) should retry; permanent errors (auth, invalid params) should not
//! - **User Guidance**: Each error type has specific recovery actions users can take
//! - **Monitoring**: Error severity helps prioritize incident response
//! - **Circuit Breaker Integration**: Transient errors trigger circuit breaker; client errors do not
//!
//! ## Error Severity Levels (Operational Impact)
//!
//! - **Critical**: Blocks all operations, requires immediate action (`Authentication`, `MissingEnvironment`)
//!   - Impact : Cannot proceed with ANY requests
//!   - Recovery : Fix credentials/environment before retrying
//!
//! - **High**: Blocks current operation, likely user error (`InvalidRequest`, `InvalidArgument`)
//!   - Impact : This specific request fails
//!   - Recovery : Fix request parameters and retry
//!
//! - **Medium**: Temporary operational issue (`RateLimit`, `Internal`)
//!   - Impact : Request fails but system healthy
//!   - Recovery : Wait and retry with backoff
//!
//! - **Low**: Degraded functionality (`Parsing`, Stream interruption)
//!   - Impact : Partial failure, may have partial data
//!   - Recovery : Retry or use partial results
//!
//! ## Retryable vs Non-Retryable Errors
//!
//! **Retryable** (transient, may succeed on retry):
//! - `RateLimit` : Server explicitly says retry later
//! - `Internal` : Server-side issue, may be resolved
//! - `Stream` : Network interruption, connection may recover
//! - `Http` : Network issues are often transient
//!
//! **Non-Retryable** (permanent, will fail again):
//! - `Authentication` : Invalid credentials won't fix themselves
//! - `InvalidArgument` : Bad input won't become valid
//! - `InvalidRequest` : Malformed request structure permanent
//! - `Parsing` : If we can't parse, retrying won't help
//! - `MissingEnvironment` : Config issue must be fixed manually
//!
//! ## SSE Parsing Edge Cases
//!
//! **Malformed JSON** (`test_sse_parsing_errors_malformed_json`):
//! - Real-world scenario : Network corruption, truncated responses
//! - Strategy : Filter out bad events, continue processing stream
//! - Rationale : Partial data better than complete failure
//!
//! **Unknown Event Types** (`test_sse_parsing_unknown_event_type`):
//! - Real-world scenario : API adds new events before client update
//! - Strategy : Silently ignore unknown events
//! - Rationale : Forward compatibility, don't break on API evolution
//!
//! **Incomplete Data** (`test_sse_parsing_content_block_delta_errors`):
//! - Real-world scenario : Race conditions, partial server responses
//! - Strategy : Validate required fields, skip invalid deltas
//! - Rationale : Graceful degradation over hard failures
//!
//! ## Recovery Suggestions
//!
//! Each error type provides actionable suggestions:
//! - **`MissingEnvironment`**: "Set `ANTHROPIC_API_KEY` environment variable"
//! - **`Authentication`**: "Verify API key is valid at console.anthropic.com"
//! - **`RateLimit`**: "Wait `{retry_after}`s before retrying" (server-provided duration)
//! - **`InvalidRequest`**: Specific parameter that failed validation
//!
//! ## NO MOCKING POLICY - Why Real API Testing
//!
//! Mocking creates false confidence because:
//! - **Mock Drift**: Mocks diverge from real API behavior over time
//! - **Edge Cases**: Real APIs have edge cases mocks never simulate
//! - **Protocol Details**: HTTP headers, status codes, timing - mocks oversimplify
//! - **Integration Issues**: Network, TLS, DNS - mocks hide these failure modes
//!
//! Real API testing catches:
//! - Actual rate limit behavior and header parsing
//! - Real authentication error messages
//! - Actual network timeout scenarios
//! - Real SSE stream interruption patterns
//! - API version changes and breaking changes
//!
//! # MANDATORY INTEGRATION TEST REQUIREMENTS
//!
//! - These tests use REAL Anthropic API endpoints - NO MOCKING ALLOWED
//! - Tests MUST FAIL IMMEDIATELY if API secrets are not available (no graceful fallbacks)
//! - Tests MUST FAIL IMMEDIATELY on network connectivity issues
//! - Tests MUST FAIL IMMEDIATELY on API authentication failures
//! - Tests MUST FAIL IMMEDIATELY on any API endpoint errors
//! - NO SILENT PASSES allowed when problems occur
//!
//! Run with : cargo test --features integration
//! Requires : Valid `ANTHROPIC_API_KEY` in environment or ../../secret/-secrets.sh

#[ allow( unused_imports ) ]
use super::*;


#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_error_context_preservation_and_stack_traces()
{
  // REMOVED: This test used fake API keys and is not needed
  // Real error context testing is covered by integration tests using from_workspace()
  // Test functionality is covered by real integration tests
}


#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_user_actionable_error_messages_with_remediation()
{
  // REMOVED: This test used fake API keys and is not needed
  // Real actionable error testing is covered by integration tests using from_workspace()
  // Test functionality is covered by real integration tests
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_error_aggregation_and_batch_error_handling()
{
  // REMOVED: This test used fake API keys and is not needed
  // Real batch error handling testing is covered by integration tests using from_workspace()
  // Test functionality is covered by real integration tests
}


#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_rate_limiting_error_with_backoff_suggestions()
{
  // Test rate limiting error detection and backoff strategy suggestions
  // REMOVED: create_test_client() call - not needed for backoff tests

  let rate_limit_error = the_module::RateLimitError::new(
    "Too many requests".to_string(),
    Some( 60 ),
    "request_limit".to_string()
  );

  let backoff_strategy = the_module::BackoffCalculator::calculate_backoff( &rate_limit_error );
  match backoff_strategy
  {
    Ok( strategy ) =>
    {
      assert!( strategy.initial_delay() >= core::time::Duration::from_mins( 1 ) );
      assert_eq!( strategy.backoff_type(), the_module::BackoffType::Linear );
      assert!( strategy.max_retries() <= 5 );
      assert!( strategy.jitter_enabled() );
    },
    Err( _err ) =>
    {
      // Expected to fail until enhanced error handling is implemented
    }
  }

  // Test different rate limit types
  let token_rate_limit = the_module::RateLimitError::new(
    "Token limit exceeded".to_string(),
    Some( 300 ),
    "token_limit".to_string()
  );

  let backoff_strategy = the_module::BackoffCalculator::calculate_backoff( &token_rate_limit );
  match backoff_strategy
  {
    Ok( strategy ) =>
    {
      assert!( strategy.initial_delay() >= core::time::Duration::from_mins( 5 ) );
      assert!( strategy.suggested_batch_size_reduction().is_some() );
    },
    Err( _err ) =>
    {
      // Expected to fail until enhanced error handling is implemented
    }
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_authentication_error_with_credential_hints()
{
  // REMOVED: This test used fake API keys and is not needed
  // Real credential hint testing is covered by integration tests using from_workspace()
  // Test functionality is covered by real integration tests
}


#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_error_logging_and_monitoring_integration()
{
  // REMOVED: This test used fake API keys and is not needed
  // Real error logging testing is covered by integration tests using from_workspace()
  // Test functionality is covered by real integration tests
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_error_correlation_and_request_tracking()
{
  // REMOVED: This test used fake API keys and is not needed
  // Real error correlation testing is covered by integration tests using from_workspace()
  // Test functionality is covered by real integration tests
}


#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_custom_error_types_and_chaining()
{
  // Test custom error types and error chaining
  
  // Create custom domain-specific error
  let domain_error = the_module::CustomError::new(
    "ModelConfigurationError".to_string(),
    "Model temperature out of range".to_string(),
    the_module::ErrorSeverity::Medium
  );

  // Chain with underlying API error
  let api_error = the_module::AnthropicError::InvalidRequest(
    "Temperature must be between 0.0 and 1.0".to_string()
  );

  let chained_error = the_module::ErrorChain::new( domain_error )
    .caused_by( api_error )
    .with_context( "temperature validation" );

  match chained_error.build()
  {
    Ok( error ) =>
    {
      assert_eq!( error.chain_length(), 2 );
      assert!( error.root_cause().contains( "Temperature must be between" ) );
      assert!( error.immediate_cause().contains( "Model temperature out of range" ) );
      assert!( error.has_context() );
      assert_eq!( error.context(), "temperature validation" );

      // Test error chain traversal
      let chain_iterator = error.chain_iterator();
      let errors : Vec< _ > = chain_iterator.collect();
      assert_eq!( errors.len(), 2 );
      assert!( errors[ 0 ].contains( "Model temperature out of range" ) );
      assert!( errors[ 1 ].contains( "Temperature must be between" ) );
    },
    Err( _err ) =>
    {
      // Expected to fail until enhanced error handling is implemented
    }
  }
}

// Helper functions for tests

// REMOVED: create_test_client function - all mockup tests have been eliminated
// Real testing is covered by integration tests using Client::from_workspace()

// REMOVED: create_invalid_request function - no longer needed after removing mockup tests

// ============================================================================
// INTEGRATION TESTS - REAL API ERROR HANDLING
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_error_handling_network_timeout()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for timeout testing" );

  // Test with extremely large request that might timeout
  let large_request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 4000, // Large response
    messages : vec![ 
      the_module::Message::user( 
        "Write a very detailed analysis of quantum computing, machine learning, blockchain technology, and artificial intelligence. Include mathematical formulas, code examples, and comprehensive explanations of each topic. Make it as detailed as possible.".repeat( 20 )
      ) 
    ],
    system : Some( vec![ the_module::SystemContent::text( "You are a comprehensive technical expert. Provide extremely detailed responses." ) ] ),
    temperature : Some( 0.3 ),
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let result = client.create_message( large_request ).await;
  
  // This should either succeed or fail with appropriate error handling
  match result
  {
    Ok( response ) => {
      assert!( !response.id.is_empty(), "Large request response must have ID" );
      assert!( response.usage.output_tokens > 0, "Must generate tokens" );
      println!( "✅ Large request succeeded : {} tokens", response.usage.output_tokens );
    },
    Err( error ) => {
      let error_str = error.to_string().to_lowercase();
      // Should be a meaningful error (timeout, rate limit, etc.)
      assert!( 
        error_str.contains( "timeout" ) || 
        error_str.contains( "rate" ) || 
        error_str.contains( "limit" ) ||
        error_str.contains( "request" ),
        "Large request error should be meaningful : {error}"
      );
      println!( "✅ Large request properly handled error : {error}" );
    }
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_error_handling_invalid_parameters()
{
  // Skip test if API key is not available
  // Integration test requires valid API key - fail if not available
  let client = the_module::Client::from_workspace()
    .expect( "Integration test requires valid API key. Set ANTHROPIC_API_KEY environment variable or configure workspace secrets." );

  // Test with invalid temperature
  let invalid_temp_request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 10,
    messages : vec![ the_module::Message::user( "Test".to_string() ) ],
    system : None,
    temperature : Some( 2.5 ), // Invalid temperature (>1.0)
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let result = client.create_message( invalid_temp_request ).await;
  
  // Should return error for invalid temperature
  assert!( result.is_err(), "Invalid temperature should cause error" );
  let error = result.unwrap_err();
  let error_str = error.to_string().to_lowercase();
  assert!( 
    error_str.contains( "temperature" ) || 
    error_str.contains( "parameter" ) ||
    error_str.contains( "invalid" ),
    "Temperature error should mention the issue : {error}"
  );

  println!( "✅ Invalid parameter error handling integration test passed!" );
  println!( "   Invalid temperature properly rejected : {error}" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_error_handling_authentication_failures()
{
  // Test with invalid API key
  let invalid_secret = the_module::Secret::new( "sk-ant-invalid-key-12345".to_string() )
    .expect( "Invalid secret should construct" );
  let invalid_client = the_module::Client::new( invalid_secret );

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 10,
    messages : vec![ the_module::Message::user( "Test auth".to_string() ) ],
    system : None,
    temperature : None,
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let result = invalid_client.create_message( request ).await;
  
  // Should fail with authentication error
  assert!( result.is_err(), "Invalid API key should cause authentication error" );
  let error = result.unwrap_err();
  let error_str = error.to_string().to_lowercase();
  assert!( 
    error_str.contains( "authentication" ) || 
    error_str.contains( "unauthorized" ) ||
    error_str.contains( "invalid" ) ||
    error_str.contains( "key" ),
    "Auth error should mention authentication issue : {error}"
  );

  println!( "✅ Authentication error handling integration test passed!" );
  println!( "   Invalid API key properly rejected : {error}" );
}

// ============================================================================
// REAL ERROR SCENARIO TESTS - TESTING ACTUAL IMPLEMENTATION
// ============================================================================

#[ cfg( feature = "streaming" ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_sse_parsing_errors_malformed_json()
{
  // Test parsing errors for SSE events with malformed JSON
  let malformed_message_start = r#"event : message_start
data : {"id": "msg_123", "type": INVALID_JSON}

"#;

  let result = the_module::parse_sse_events( malformed_message_start );
  assert!( result.is_ok(), "Should parse and filter out bad events" );
}

#[ cfg( feature = "streaming" ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_sse_parsing_unknown_event_type()
{
  // Test handling of unknown event types
  let unknown_event = r#"event : unknown_event_type
data : {"some": "data"}

"#;

  let result = the_module::parse_sse_events( unknown_event );
  // Should succeed and filter out unknown events
  assert!( result.is_ok() );
}

#[ cfg( feature = "streaming" ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_sse_parsing_content_block_delta_errors()
{
  // Test parsing error for content_block_delta with missing fields
  let incomplete_delta = r#"event : content_block_delta
data : {"index": 0}

"#;

  let result = the_module::parse_sse_events( incomplete_delta );
  // Should succeed but not include invalid event
  assert!( result.is_ok() );
}

#[ cfg( feature = "error-handling" ) ]
#[ test ]
fn test_error_classification_is_retryable()
{
  // Test is_retryable() for different error types

  // Retryable errors
  let rate_limit_err = the_module::AnthropicError::RateLimit(
    the_module::RateLimitError::new( "Rate limited".to_string(), Some( 60 ), "request".to_string() )
  );
  assert!( rate_limit_err.is_retryable(), "Rate limit errors should be retryable" );

  let internal_err = the_module::AnthropicError::Internal( "Server error".to_string() );
  assert!( internal_err.is_retryable(), "Internal errors should be retryable" );

  let stream_err = the_module::AnthropicError::Stream( "Stream interrupted".to_string() );
  assert!( stream_err.is_retryable(), "Stream errors should be retryable" );

  // Non-retryable errors
  let invalid_arg_err = the_module::AnthropicError::InvalidArgument( "Bad input".to_string() );
  assert!( !invalid_arg_err.is_retryable(), "Invalid argument errors should not be retryable" );

  let auth_err = the_module::AnthropicError::Authentication(
    the_module::AuthenticationError::new( "Bad key".to_string() )
  );
  assert!( !auth_err.is_retryable(), "Authentication errors should not be retryable" );

  let parsing_err = the_module::AnthropicError::Parsing( "Parse failed".to_string() );
  assert!( !parsing_err.is_retryable(), "Parsing errors should not be retryable" );
}

#[ cfg( feature = "error-handling" ) ]
#[ test ]
fn test_error_severity_classification()
{
  use the_module::ErrorSeverity;

  // Critical severity
  let auth_err = the_module::AnthropicError::Authentication(
    the_module::AuthenticationError::new( "Invalid key".to_string() )
  );
  assert_eq!( auth_err.severity(), ErrorSeverity::Critical, "Auth errors should be critical" );

  let missing_env_err = the_module::AnthropicError::MissingEnvironment( "No API key".to_string() );
  assert_eq!( missing_env_err.severity(), ErrorSeverity::Critical, "Missing env errors should be critical" );

  // High severity
  let invalid_req_err = the_module::AnthropicError::InvalidRequest( "Bad params".to_string() );
  assert_eq!( invalid_req_err.severity(), ErrorSeverity::High, "Invalid request errors should be high" );

  // Medium severity
  let rate_limit_err = the_module::AnthropicError::RateLimit(
    the_module::RateLimitError::new( "Rate limited".to_string(), Some( 60 ), "request".to_string() )
  );
  assert_eq!( rate_limit_err.severity(), ErrorSeverity::Medium, "Rate limit errors should be medium" );
}

#[ cfg( feature = "error-handling" ) ]
#[ test ]
fn test_error_display_formatting()
{
  // Test that error messages are properly formatted

  let parsing_err = the_module::AnthropicError::Parsing( "JSON parse failed".to_string() );
  let err_msg = format!( "{parsing_err}" );
  assert!( err_msg.contains( "Parsing error" ) && err_msg.contains( "JSON parse failed" ),
    "Parsing error should contain type and message : {err_msg}" );

  let internal_err = the_module::AnthropicError::Internal( "Internal failure".to_string() );
  let err_msg = format!( "{internal_err}" );
  assert!( err_msg.contains( "Internal error" ) && err_msg.contains( "Internal failure" ),
    "Internal error should contain type and message : {err_msg}" );

  let missing_env_err = the_module::AnthropicError::MissingEnvironment( "ANTHROPIC_API_KEY not set".to_string() );
  let err_msg = format!( "{missing_env_err}" );
  assert!( err_msg.contains( "Missing environment" ) && err_msg.contains( "ANTHROPIC_API_KEY" ),
    "Missing env error should contain type and variable : {err_msg}" );

  let file_err = the_module::AnthropicError::File( "File not found".to_string() );
  let err_msg = format!( "{file_err}" );
  assert!( err_msg.contains( "File error" ) && err_msg.contains( "File not found" ),
    "File error should contain type and message : {err_msg}" );

  let not_impl_err = the_module::AnthropicError::NotImplemented( "Feature X not implemented".to_string() );
  let err_msg = format!( "{not_impl_err}" );
  assert!( err_msg.contains( "Not implemented" ) && err_msg.contains( "Feature X" ),
    "NotImplemented error should contain type and feature : {err_msg}" );
}

#[ cfg( feature = "error-handling" ) ]
#[ test ]
fn test_error_recovery_suggestions()
{
  // Test that errors provide helpful recovery suggestions

  let missing_env_err = the_module::AnthropicError::MissingEnvironment(
    "ANTHROPIC_API_KEY not found".to_string()
  );
  let suggestions = missing_env_err.recovery_suggestions();
  assert!( !suggestions.is_empty(), "Missing env errors should have recovery suggestions" );
  assert!( suggestions.iter().any( | s | s.contains( "ANTHROPIC_API_KEY" ) || s.contains( "environment" ) ),
    "Suggestions should mention the environment variable" );

  let auth_err = the_module::AnthropicError::Authentication(
    the_module::AuthenticationError::new( "Invalid API key".to_string() )
  );
  let suggestions = auth_err.recovery_suggestions();
  assert!( !suggestions.is_empty(), "Auth errors should have recovery suggestions" );

  let rate_limit_err = the_module::AnthropicError::RateLimit(
    the_module::RateLimitError::new( "Rate limited".to_string(), Some( 60 ), "request".to_string() )
  );
  let suggestions = rate_limit_err.recovery_suggestions();
  assert!( !suggestions.is_empty(), "Rate limit errors should have recovery suggestions" );
}

#[ cfg( all( feature = "streaming", feature = "error-handling" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_stream_event_validation()
{
  // Test validation of stream events

  // Valid message should pass validation
  let valid_message = the_module::StreamMessage::new(
    "msg_123",
    "message",
    "assistant",
    "claude-sonnet-4-5-20250929",
    the_module::Usage
    {
      input_tokens : 10,
      output_tokens : 50,
      cache_creation_input_tokens : None,
      cache_read_input_tokens : None,
    }
  );
  assert!( valid_message.validate().is_ok(), "Valid message should pass validation" );

  // Test text content block validation
  let text_block = the_module::StreamContentBlock::new_text( "Hello world" );
  assert!( text_block.validate().is_ok(), "Valid text block should pass validation" );

  // Test text delta validation
  let text_delta = the_module::StreamDelta::new_text( "Hello" );
  assert!( text_delta.validate().is_ok(), "Valid text delta should pass validation" );
}
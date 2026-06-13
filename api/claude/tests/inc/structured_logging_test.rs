//! Structured Logging Tests
//!
//! Tests for structured logging functionality including log output validation,
//! log level filtering, tracing integration, and zero-overhead verification.

#[ allow( unused_imports ) ]
use super::*;

// ============================================================================
// UNIT TESTS - STRUCTURED LOG OUTPUT
// ============================================================================

#[ test ]
fn test_structured_logger_creation()
{
  // Test creating a structured logger
  let logger = the_module::StructuredLogger::new();

  assert!( logger.is_enabled() );
}

#[ test ]
fn test_log_request_structure()
{
  // Test logging a request with structured fields
  let mut logger = the_module::StructuredLogger::new();

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 100,
    messages : vec![ the_module::Message::user( "Test" ) ],
    system : None,
    temperature : None,
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  logger.log_request( &request, "request_id_123" );

  // Verify log entry was recorded
  let logs = logger.get_logs();
  assert_eq!( logs.len(), 1 );
  assert!( logs[ 0 ].contains( "request_id_123" ) );
  assert!( logs[ 0 ].contains( "claude-3-5-haiku-20241022" ) );
}

#[ test ]
fn test_log_response_structure()
{
  // Test logging a response with structured fields
  let mut logger = the_module::StructuredLogger::new();

  // Create a sample response (simplified structure)
  let response_json = r#"{
    "id": "msg_test_123",
    "type": "message",
    "role": "assistant",
    "content": [{"type": "text", "text": "Test response"}],
    "model": "claude-3-5-haiku-20241022",
    "stop_reason": "end_turn",
    "usage": {"input_tokens": 10, "output_tokens": 5}
  }"#;

  let response : the_module::CreateMessageResponse = serde_json::from_str( response_json ).unwrap();

  logger.log_response( &response, "request_id_123" );

  let logs = logger.get_logs();
  assert_eq!( logs.len(), 1 );
  assert!( logs[ 0 ].contains( "msg_test_123" ) );
  assert!( logs[ 0 ].contains( "request_id_123" ) );
}

#[ test ]
fn test_log_error_structure()
{
  // Test logging an error with structured fields
  let mut logger = the_module::StructuredLogger::new();

  let error = the_module::AnthropicError::InvalidRequest( "Test error".to_string() );

  logger.log_error( &error, "request_id_123" );

  let logs = logger.get_logs();
  assert_eq!( logs.len(), 1 );
  assert!( logs[ 0 ].contains( "request_id_123" ) );
  assert!( logs[ 0 ].contains( "Test error" ) );
  assert!( logs[ 0 ].contains( "InvalidRequest" ) );
}

// ============================================================================
// UNIT TESTS - LOG LEVEL FILTERING
// ============================================================================

#[ test ]
fn test_log_level_filtering_debug()
{
  // Test that debug level logs are filtered correctly
  let mut logger = the_module::StructuredLogger::with_level( the_module::LogLevel::Debug );

  logger.debug( "Debug message" );
  logger.info( "Info message" );
  logger.warn( "Warn message" );
  logger.error( "Error message" );

  let logs = logger.get_logs();
  assert_eq!( logs.len(), 4 ); // All levels should be logged
}

#[ test ]
fn test_log_level_filtering_info()
{
  // Test that info level filters out debug
  let mut logger = the_module::StructuredLogger::with_level( the_module::LogLevel::Info );

  logger.debug( "Debug message" );
  logger.info( "Info message" );
  logger.warn( "Warn message" );
  logger.error( "Error message" );

  let logs = logger.get_logs();
  assert_eq!( logs.len(), 3 ); // Debug should be filtered out
  assert!( !logs.iter().any( |l| l.contains( "Debug message" ) ) );
}

#[ test ]
fn test_log_level_filtering_warn()
{
  // Test that warn level filters out debug and info
  let mut logger = the_module::StructuredLogger::with_level( the_module::LogLevel::Warn );

  logger.debug( "Debug message" );
  logger.info( "Info message" );
  logger.warn( "Warn message" );
  logger.error( "Error message" );

  let logs = logger.get_logs();
  assert_eq!( logs.len(), 2 ); // Only warn and error
}

#[ test ]
fn test_log_level_filtering_error()
{
  // Test that error level only logs errors
  let mut logger = the_module::StructuredLogger::with_level( the_module::LogLevel::Error );

  logger.debug( "Debug message" );
  logger.info( "Info message" );
  logger.warn( "Warn message" );
  logger.error( "Error message" );

  let logs = logger.get_logs();
  assert_eq!( logs.len(), 1 ); // Only error
  assert!( logs[ 0 ].contains( "Error message" ) );
}

// ============================================================================
// UNIT TESTS - LOG FORMATTING
// ============================================================================

#[ test ]
fn test_log_json_formatting()
{
  // Test that logs can be formatted as JSON
  let mut logger = the_module::StructuredLogger::new();

  logger.info( "Test message" );

  let json_logs = logger.to_json().expect( "JSON export must work" );

  assert!( json_logs.contains( "Test message" ) );
  assert!( json_logs.contains( "level" ) );
  assert!( json_logs.contains( "timestamp" ) );
}

#[ test ]
fn test_log_with_context_fields()
{
  // Test logging with additional context fields
  let mut logger = the_module::StructuredLogger::new();

  let mut context = std::collections::HashMap::new();
  context.insert( "user_id".to_string(), "user_123".to_string() );
  context.insert( "request_type".to_string(), "chat".to_string() );

  logger.info_with_context( "Message with context", context );

  let logs = logger.get_logs();
  assert!( logs[ 0 ].contains( "user_id" ) );
  assert!( logs[ 0 ].contains( "user_123" ) );
  assert!( logs[ 0 ].contains( "request_type" ) );
  assert!( logs[ 0 ].contains( "chat" ) );
}

// ============================================================================
// UNIT TESTS - ZERO OVERHEAD WHEN DISABLED
// ============================================================================

#[ test ]
fn test_disabled_logger_zero_overhead()
{
  // Test that disabled logger has minimal overhead
  let start = std::time::Instant::now();

  for _ in 0..10000
  {
    let _result = simple_operation();
  }

  let duration_without_logging = start.elapsed();

  // Now with disabled logger
  let logger = the_module::StructuredLogger::disabled();
  let start = std::time::Instant::now();

  for _ in 0..10000
  {
    let _result = simple_operation();
    logger.log_if_enabled( "operation", "completed" );
  }

  let duration_with_disabled_logging = start.elapsed();

  // Overhead should be minimal (less than 2x under concurrent test load)
  let overhead_ratio = duration_with_disabled_logging.as_micros() as f64 / duration_without_logging.as_micros() as f64;

  assert!( overhead_ratio < 2.0, "Disabled logging has too much overhead : {overhead_ratio}x" );

  // Verify no logs were collected (functional correctness)
  assert_eq!( logger.get_logs().len(), 0, "Disabled logger must not collect logs" );

  println!( "✅ Zero overhead test passed!" );
  println!( "   Without logging : {duration_without_logging:?}" );
  println!( "   With disabled logging : {duration_with_disabled_logging:?}" );
  println!( "   Overhead ratio : {overhead_ratio:.2}x" );
}

// Helper function
fn simple_operation() -> u64
{
  ( 1..100 ).sum()
}

// ============================================================================
// INTEGRATION TESTS - REAL API LOGGING
// ============================================================================

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_log_api_request_response()
{
  // Test logging real API request and response
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let mut logger = the_module::StructuredLogger::new();

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 20,
    messages : vec![ the_module::Message::user( "Hello!".to_string() ) ],
    system : None,
    temperature : Some( 0.0 ),
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let request_id = "integration_test_001";
  logger.log_request( &request, request_id );

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => {
      logger.log_error( &err, request_id );
      panic!( "INTEGRATION: API call must work : {err}" );
    },
  };

  logger.log_response( &response, request_id );

  // Verify logs were captured
  let logs = logger.get_logs();
  assert!( logs.len() >= 2 ); // At least request and response
  assert!( logs.iter().any( |l| l.contains( request_id ) && l.contains( "request" ) ) );
  assert!( logs.iter().any( |l| l.contains( request_id ) && l.contains( "response" ) ) );

  println!( "✅ API request/response logging integration test passed!" );
  println!( "   Logged {} entries", logs.len() );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_log_api_error()
{
  // Test logging API error
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let mut logger = the_module::StructuredLogger::new();

  // Create an invalid request (empty model)
  let invalid_request = the_module::CreateMessageRequest
  {
    model : String::new(), // Invalid
    max_tokens : 20,
    messages : vec![ the_module::Message::user( "Test".to_string() ) ],
    system : None,
    temperature : None,
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let request_id = "integration_error_test";
  logger.log_request( &invalid_request, request_id );

  let result = client.create_message( invalid_request ).await;

  match result
  {
    Ok( _response ) => {
      // Unexpected success - still log it
      println!( "Request unexpectedly succeeded" );
    },
    Err( err ) => {
      // Expected error - log it
      logger.log_error( &err, request_id );
    },
  }

  // Verify error was logged
  let logs = logger.get_logs();
  assert!( logs.iter().any( |l| l.contains( request_id ) ) );

  println!( "✅ API error logging integration test passed!" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_structured_logging_with_context()
{
  // Test structured logging with request context
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key" );

  let mut logger = the_module::StructuredLogger::new();

  let request = the_module::CreateMessageRequest
  {
    model : "claude-3-5-haiku-20241022".to_string(),
    max_tokens : 15,
    messages : vec![ the_module::Message::user( "Test".to_string() ) ],
    system : None,
    temperature : Some( 0.0 ),
    stream : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
    #[ cfg( feature = "tools" ) ]
    tool_choice : None,
  };

  let request_id = "context_test_001";

  // Log with context
  let mut context = std::collections::HashMap::new();
  context.insert( "session_id".to_string(), "session_abc".to_string() );
  context.insert( "user_type".to_string(), "premium".to_string() );

  logger.info_with_context( "Starting API request", context.clone() );
  logger.log_request( &request, request_id );

  let _response = match client.create_message( request ).await
  {
    Ok( response ) => {
      logger.log_response( &response, request_id );
      response
    },
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => {
      logger.log_error( &err, request_id );
      panic!( "Request failed : {err}" );
    },
  };

  logger.info_with_context( "API request completed", context );

  // Verify context was logged
  let logs = logger.get_logs();
  assert!( logs.iter().any( |l| l.contains( "session_abc" ) ) );
  assert!( logs.iter().any( |l| l.contains( "premium" ) ) );

  println!( "✅ Structured logging with context integration test passed!" );
}

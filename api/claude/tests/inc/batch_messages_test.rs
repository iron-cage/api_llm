//! Batch Messages API Tests
//!
//! Tests for the Anthropic Batch Messages API functionality.
//!
//! ## Test Coverage
//!
//! - Batch request creation and validation
//! - Batch status retrieval
//! - Batch listing with pagination
//! - Batch cancellation
//! - Error handling for batch operations

#[ cfg( all( feature = "batch-processing", feature = "error-handling" ) ) ]
mod batch_api_tests
{
  use crate::inc::the_module;

  #[ test ]
  fn test_batch_request_item_validation()
  {
    let message_request = the_module::CreateMessageRequest::builder()
      .model( the_module::RECOMMENDED_MODEL.to_string() )
      .max_tokens( 100 )
      .message( the_module::Message::user( "Hello".to_string() ) )
      .build();

    // Valid batch request item
    let valid_item = the_module::BatchRequestItem::new(
      "req-001".to_string(),
      message_request.clone()
    );
    assert!( valid_item.validate().is_ok() );

    // Empty custom_id should fail
    let empty_id_item = the_module::BatchRequestItem::new(
      String::new(),
      message_request.clone()
    );
    assert!( empty_id_item.validate().is_err() );

    // custom_id exceeding 256 characters should fail
    let long_id = "a".repeat( 257 );
    let long_id_item = the_module::BatchRequestItem::new(
      long_id,
      message_request
    );
    assert!( long_id_item.validate().is_err() );
  }

  #[ test ]
  fn test_create_batch_request_validation()
  {
    let message_request = the_module::CreateMessageRequest::builder()
      .model( the_module::RECOMMENDED_MODEL.to_string() )
      .max_tokens( 100 )
      .message( the_module::Message::user( "Hello".to_string() ) )
      .build();

    // Valid batch with one request
    let valid_batch = the_module::CreateBatchRequest::new( vec![
      the_module::BatchRequestItem::new( "req-001".to_string(), message_request.clone() )
    ] );
    assert!( valid_batch.validate().is_ok() );

    // Empty batch should fail
    let empty_batch = the_module::CreateBatchRequest::new( vec![] );
    assert!( empty_batch.validate().is_err() );

    // Batch with multiple valid requests
    let multi_batch = the_module::CreateBatchRequest::new( vec![
      the_module::BatchRequestItem::new( "req-001".to_string(), message_request.clone() ),
      the_module::BatchRequestItem::new( "req-002".to_string(), message_request.clone() ),
      the_module::BatchRequestItem::new( "req-003".to_string(), message_request ),
    ] );
    assert!( multi_batch.validate().is_ok() );
  }

  #[ test ]
  fn test_batch_response_helpers()
  {
    // Create a completed batch response
    let completed_batch = the_module::BatchResponse
    {
      id : "batch_123".to_string(),
      r#type : "message_batch".to_string(),
      processing_status : the_module::BatchProcessingStatus::Ended,
      request_counts : the_module::RequestCounts
      {
        processing : 0,
        succeeded : 5,
        errored : 2,
        canceled : 0,
        expired : 0,
      },
      ended_at : Some( "2024-01-01T00:00:00Z".to_string() ),
      created_at : "2024-01-01T00:00:00Z".to_string(),
      expires_at : "2024-01-02T00:00:00Z".to_string(),
      results_url : Some( "https://api.anthropic.com/v1/messages/batches/batch_123/results".to_string() ),
    };

    assert!( completed_batch.is_completed() );
    assert!( completed_batch.has_results() );
    assert_eq!( completed_batch.total_requests(), 7 );

    // Create an in-progress batch response
    let in_progress_batch = the_module::BatchResponse
    {
      id : "batch_456".to_string(),
      r#type : "message_batch".to_string(),
      processing_status : the_module::BatchProcessingStatus::InProgress,
      request_counts : the_module::RequestCounts
      {
        processing : 10,
        succeeded : 5,
        errored : 0,
        canceled : 0,
        expired : 0,
      },
      ended_at : None,
      created_at : "2024-01-01T00:00:00Z".to_string(),
      expires_at : "2024-01-02T00:00:00Z".to_string(),
      results_url : None,
    };

    assert!( !in_progress_batch.is_completed() );
    assert!( !in_progress_batch.has_results() );
    assert_eq!( in_progress_batch.total_requests(), 15 );
  }

  #[ test ]
  fn test_batch_status_serialization()
  {
    use serde_json;

    // Test BatchProcessingStatus serialization
    let in_progress = the_module::BatchProcessingStatus::InProgress;
    let json = serde_json::to_string( &in_progress ).unwrap();
    assert_eq!( json, "\"in_progress\"" );

    let ended = the_module::BatchProcessingStatus::Ended;
    let json = serde_json::to_string( &ended ).unwrap();
    assert_eq!( json, "\"ended\"" );

    let canceling = the_module::BatchProcessingStatus::Canceling;
    let json = serde_json::to_string( &canceling ).unwrap();
    assert_eq!( json, "\"canceling\"" );

    let expired = the_module::BatchProcessingStatus::Expired;
    let json = serde_json::to_string( &expired ).unwrap();
    assert_eq!( json, "\"expired\"" );
  }

  #[ test ]
  fn test_batch_request_serialization()
  {
    use serde_json;

    let message_request = the_module::CreateMessageRequest::builder()
      .model( the_module::RECOMMENDED_MODEL.to_string() )
      .max_tokens( 100 )
      .message( the_module::Message::user( "Test message".to_string() ) )
      .build();

    let batch_request = the_module::CreateBatchRequest::new( vec![
      the_module::BatchRequestItem::new( "req-001".to_string(), message_request )
    ] );

    let json = serde_json::to_string( &batch_request ).unwrap();
    assert!( json.contains( "\"custom_id\":\"req-001\"" ) );
    assert!( json.contains( "\"model\"" ) );
    assert!( json.contains( "\"max_tokens\"" ) );
  }

  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
async fn integration_batch_create_and_retrieve()
  {
    // This test requires real API credentials
    let client = the_module::Client::from_workspace()
      .expect( "Failed to create client from workspace secrets" );

    // Create a small batch with 2 requests
    let message_request1 = the_module::CreateMessageRequest::builder()
      .model( the_module::RECOMMENDED_MODEL.to_string() )
      .max_tokens( 50 )
      .message( the_module::Message::user( "What is 2+2?".to_string() ) )
      .build();

    let message_request2 = the_module::CreateMessageRequest::builder()
      .model( the_module::RECOMMENDED_MODEL.to_string() )
      .max_tokens( 50 )
      .message( the_module::Message::user( "What is the capital of France?".to_string() ) )
      .build();

    let batch_request = the_module::CreateBatchRequest::new( vec![
      the_module::BatchRequestItem::new( "math-001".to_string(), message_request1 ),
      the_module::BatchRequestItem::new( "geography-001".to_string(), message_request2 ),
    ] );

    // Create batch
    let batch_response = client.create_messages_batch( batch_request ).await
      .expect( "Failed to create batch" );

    assert!( !batch_response.id.is_empty() );
    assert_eq!( batch_response.r#type, "message_batch" );

    // Retrieve batch status
    let retrieved = client.retrieve_batch( &batch_response.id ).await
      .expect( "Failed to retrieve batch" );

    assert_eq!( retrieved.id, batch_response.id );
  }

  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
async fn integration_batch_list()
  {
    let client = the_module::Client::from_workspace()
      .expect( "Failed to create client from workspace secrets" );

    // List batches without pagination
    let list_response = client.list_batches( None, None, Some( 10 ) ).await
      .expect( "Failed to list batches" );

    // Should get a valid response even if no batches exist
    assert!( list_response.data.len() <= 10 );
  }

  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
async fn integration_batch_validation_errors()
  {
    let client = the_module::Client::from_workspace()
      .expect( "Failed to create client from workspace secrets" );

    // Test empty custom_id validation
    let message_request = the_module::CreateMessageRequest::builder()
      .model( the_module::RECOMMENDED_MODEL.to_string() )
      .max_tokens( 50 )
      .message( the_module::Message::user( "Test".to_string() ) )
      .build();

    let invalid_batch = the_module::CreateBatchRequest::new( vec![
      the_module::BatchRequestItem::new( String::new(), message_request )
    ] );

    // Should fail validation
    let result = client.create_messages_batch( invalid_batch ).await;
    assert!( result.is_err() );
  }

  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
async fn integration_batch_retrieve_invalid_id()
  {
    let client = the_module::Client::from_workspace()
      .expect( "Failed to create client from workspace secrets" );

    // Empty batch_id should fail validation
    let result = client.retrieve_batch( "" ).await;
    assert!( result.is_err() );

    // Non-existent batch_id should return API error
    let result = client.retrieve_batch( "batch_nonexistent" ).await;
    assert!( result.is_err() );
  }
}

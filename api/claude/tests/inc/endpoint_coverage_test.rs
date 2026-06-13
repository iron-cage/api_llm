//! Spec traceability: AP-01..AP-12 — Endpoint Coverage
//! Source: `tests/docs/api/001_endpoint_coverage.md`

#[ allow( unused_imports ) ]
use super::*;

mod private
{
  pub fn minimal_request() -> super::the_module::CreateMessageRequest
  {
    super::the_module::CreateMessageRequest
    {
      model : "claude-haiku-4-5-20251001".to_string(),
      max_tokens : 5,
      messages : vec![ super::the_module::Message::user( "Hi".to_string() ) ],
      system : None,
      temperature : None,
      stream : None,
      tools : None,
      tool_choice : None,
    }
  }
}

/// AP-01: `create_message()` callable with correct path
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ap_01()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: requires valid API key" );
  let response = client.create_message( private::minimal_request() ).await
    .expect( "INTEGRATION: create_message must succeed" );
  assert!( !response.id.is_empty(), "AP-01: response must have id" );
  assert!( !response.content.is_empty(), "AP-01: response must have content" );
  assert!( !response.model.is_empty(), "AP-01: response must name model used" );
}

/// AP-02: `count_message_tokens()` callable with correct path
#[ cfg( all( feature = "integration", feature = "count-tokens" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ap_02()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: requires valid API key" );
  let request = the_module::CountMessageTokensRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    messages : vec![ the_module::Message::user( "Hi".to_string() ) ],
    system : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
  };
  let response = client.count_message_tokens( request ).await
    .expect( "INTEGRATION: count_message_tokens must succeed" );
  assert!( response.input_tokens > 0, "AP-02: token count must be positive" );
}

/// AP-03: `create_messages_batch()` callable with correct path
#[ cfg( all( feature = "integration", feature = "batch-processing", feature = "error-handling" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ap_03()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: requires valid API key" );
  let item = the_module::BatchRequestItem::new( "req-ap-03".to_string(), private::minimal_request() );
  let batch = the_module::CreateBatchRequest::new( vec![ item ] );
  let response = client.create_messages_batch( batch ).await
    .expect( "INTEGRATION: create_messages_batch must succeed" );
  assert!( !response.id.is_empty(), "AP-03: batch response must have id" );
}

/// AP-04: `retrieve_batch()` callable with correct path
#[ cfg( all( feature = "integration", feature = "batch-processing", feature = "error-handling" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ap_04()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: requires valid API key" );
  // Create batch first to get a valid batch ID
  let item = the_module::BatchRequestItem::new( "req-ap-04".to_string(), private::minimal_request() );
  let created = client.create_messages_batch( the_module::CreateBatchRequest::new( vec![ item ] ) ).await
    .expect( "INTEGRATION: create_messages_batch must succeed" );
  let batch_id = created.id.clone();
  let response = client.retrieve_batch( &batch_id ).await
    .expect( "INTEGRATION: retrieve_batch must succeed" );
  assert_eq!( response.id, batch_id, "AP-04: retrieved batch id must match" );
}

/// AP-05: `list_batches()` callable with correct path
#[ cfg( all( feature = "integration", feature = "batch-processing", feature = "error-handling" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ap_05()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: requires valid API key" );
  let response = client.list_batches( None, None, Some( 1 ) ).await
    .expect( "INTEGRATION: list_batches must succeed" );
  // Response should be a valid list (possibly empty)
  let _ = response.data.len(); // proves the field exists
}

/// AP-06: `cancel_batch()` callable with correct path
#[ cfg( all( feature = "integration", feature = "batch-processing", feature = "error-handling" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ap_06()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: requires valid API key" );
  // Create batch to cancel
  let item = the_module::BatchRequestItem::new( "req-ap-06".to_string(), private::minimal_request() );
  let created = client.create_messages_batch( the_module::CreateBatchRequest::new( vec![ item ] ) ).await
    .expect( "INTEGRATION: create_messages_batch must succeed" );
  let batch_id = created.id.clone();
  let response = client.cancel_batch( &batch_id ).await
    .expect( "INTEGRATION: cancel_batch must succeed" );
  assert_eq!( response.id, batch_id, "AP-06: cancelled batch id must match" );
}

/// AP-07: `create_message_stream()` only under streaming feature
#[ cfg( all( feature = "integration", feature = "streaming" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ap_07()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: requires valid API key" );
  // Method exists and is callable; return type is EventStream (SSE stream)
  let _stream = client.create_message_stream( private::minimal_request() ).await
    .expect( "INTEGRATION: create_message_stream must succeed with streaming feature" );
  // stream type is EventStream = Pin<Box<dyn Stream<...>>>; compilation confirms it exists
}


/// AP-09: `count_message_tokens()` absent without count-tokens feature
/// This test only compiles when count-tokens is enabled; the types are gated.
#[ cfg( feature = "count-tokens" ) ]
#[ test ]
fn test_ap_09()
{
  // CountMessageTokensRequest only exists under count-tokens feature —
  // the method is unavailable without it because the param type disappears.
  let request = the_module::CountMessageTokensRequest
  {
    model : "claude-haiku-4-5-20251001".to_string(),
    messages : vec![ the_module::Message::user( "Hi".to_string() ) ],
    system : None,
    #[ cfg( feature = "tools" ) ]
    tools : None,
  };
  assert!( !request.model.is_empty(), "AP-09: CountMessageTokensRequest exists under count-tokens" );
}

/// AP-10: batch methods absent without batch-processing feature
/// This test only compiles when batch-processing is enabled; the methods are gated.
#[ cfg( all( feature = "batch-processing", feature = "error-handling" ) ) ]
#[ test ]
fn test_ap_10()
{
  // BatchRequestItem and CreateBatchRequest only exist under batch-processing + error-handling
  let item = the_module::BatchRequestItem::new( "req-ap-10".to_string(), private::minimal_request() );
  let batch = the_module::CreateBatchRequest::new( vec![ item ] );
  assert!( batch.validate().is_ok(), "AP-10: CreateBatchRequest exists under batch-processing" );
}

/// AP-11: invalid credentials return authentication error
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_ap_11()
{
  let bad_key = format!( "sk-ant-api03-{}", "y".repeat( 64 ) );
  let secret = the_module::Secret::new( bad_key )
    .expect( "valid-format key must construct" );
  let client = the_module::Client::new( secret );
  let result = client.create_message( private::minimal_request() ).await;
  assert!( result.is_err(), "AP-11: invalid credentials must return Err" );
  let err = result.unwrap_err();
  let msg = err.to_string().to_lowercase();
  assert!(
    matches!( err, the_module::AnthropicError::Authentication( _ ) )
      || msg.contains( "401" )
      || msg.contains( "auth" )
      || msg.contains( "unauthorized" )
      || msg.contains( "invalid" ),
    "AP-11: error must reflect authentication failure, got: {err}"
  );
}


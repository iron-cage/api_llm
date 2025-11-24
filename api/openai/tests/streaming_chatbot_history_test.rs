//! Integration test for streaming chatbot conversation history preservation using `previous_response_id`
//!
//! This test demonstrates that conversation history should be preserved
//! across multiple interactions using the `previous_response_id` mechanism.

use api_openai::ClientApiAccessors;
#[ cfg( feature = "integration" ) ]
use api_openai::
{
  Client,
  components ::
  {
    responses ::{ CreateResponseRequest, ResponseInput, ResponseStreamEvent },
  },
};

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_conversation_history_preservation() -> Result< (), Box< dyn std::error::Error > >
{
  // Load API key from workspace secrets
  let secret = api_openai::secret::Secret::load_with_fallbacks("OPENAI_API_KEY")
    .expect("Failed to load OPENAI_API_KEY for integration test");

  let env = api_openai::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
    .expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  // System instructions for the math assistant
  let system_instructions = "You are a helpful math assistant. Remember values that users give you and use them in calculations.".to_string();

  // Initialize conversation state
  let mut previous_response_id : Option< String > = None;

  // First interaction : set x=13
  let request1 = CreateResponseRequest::former()
    .model("gpt-5-mini".to_string())
    .input(ResponseInput::String("x=13".to_string()))
    .instructions(system_instructions.clone())
    .stream(true);

  // No previous_response_id for first request
  let request1 = request1.form();

  println!("Sending first request : {request1:?}");
  let mut receiver1 = client.responses().create_stream(request1).await?;
  let mut response1 = String::new();
  println!("Started receiving first response...");

  // Collect first response and get response ID
  while let Some(event_result) = receiver1.recv().await
  {
    match event_result?
    {
      ResponseStreamEvent::ResponseTextDelta(e) =>
      {
        response1.push_str(&e.delta);
      },
      ResponseStreamEvent::ResponseCompleted(e) =>
      {
        // Store the response ID for conversation continuity
        previous_response_id = Some(e.response.id);
        break;
      },
      _ => {} // Handle other events
    }
  }

  // Second interaction : ask for x*3 using previous_response_id for context
  let mut request2 = CreateResponseRequest::former()
    .model("gpt-5-mini".to_string())
    .input(ResponseInput::String("Please calculate x times 3".to_string()))
    .instructions(system_instructions.clone())
    .stream(true);

  // Add conversation context using previous_response_id
  if let Some(ref prev_id) = previous_response_id
  {
    request2 = request2.previous_response_id(prev_id.clone());
  }

  let request2 = request2.form();

  println!("Sending second request : {request2:?}");
  let mut receiver2 = client.responses().create_stream(request2).await?;
  println!("Started receiving second response...");
  let mut response2 = String::new();

  // Collect second response
  let mut events_received = 0;
  while let Some(event_result) = receiver2.recv().await
  {
    events_received += 1;
    println!("Received event {events_received}: {event_result:?}");
    match event_result?
    {
      ResponseStreamEvent::ResponseTextDelta(e) =>
      {
        println!("Text delta : '{}'", e.delta);
        response2.push_str(&e.delta);
      },
      ResponseStreamEvent::ResponseCompleted(_) =>
      {
        println!("Response completed");
        break;
      },
      other =>
      {
        println!("Other event : {other:?}");
      }
    }
  }
  println!("Total events received for second response : {events_received}");

  println!("First response : {response1}");
  println!("Second response : {response2}");

  // This test should pass if conversation history is properly preserved via previous_response_id
  // The assistant should remember that x=13 from the first interaction and calculate x*3=39
  assert!(
    response2.contains("39") || response2.contains("thirty-nine"),
    "Assistant should remember x=13 from previous interaction and calculate x*3=39. Got response : {response2}. Previous response ID was : {previous_response_id:?}"
  );

  Ok(())
}
//! Example of canceling a response using the OpenAI API.
#![ allow( clippy::doc_markdown ) ]
//!
//! This example demonstrates how to:
//! 1. Create a streaming response
//! 2. Cancel it while in progress
//!
//! Run with:
//! `cargo run --example responses_cancel`
//!
//! Make sure you have set the `OPENAI_API_KEY` environment variable
//! or have a `secret/-secret.sh` file with the key.

use api_openai::
{
  ClientApiAccessors,
  Client,
  components ::
  {
    responses ::{ CreateResponseRequest, ResponseInput, ResponseStreamEvent },
    common ::ModelIdsResponses,
  },
};

#[ tokio::main( flavor = "current_thread" ) ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  println!( "Initializing client..." );
  let secret = api_openai::secret::Secret::load_with_fallbacks( "OPENAI_API_KEY" )
    .expect( "Failed to load OPENAI_API_KEY. Please set environment variable or add to workspace secrets file." );
  let env = api_openai::environment::OpenaiEnvironmentImpl::build(
    secret,
    None,
    None,
    api_openai::environment::OpenAIRecommended::base_url().to_string(),
    api_openai::environment::OpenAIRecommended::realtime_base_url().to_string(),
  ).expect( "Failed to create environment" );
  let client = Client::build( env ).expect( "Failed to create client" );

  // 1. Create a long-running streaming response
  println!("Creating a streaming response that we can cancel...");
  let create_request = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-5-nano".to_string()))
    .input(ResponseInput::String("Write a very detailed, long story about a magical kingdom. Include many characters and plot twists.".to_string()))
    .max_output_tokens(2000)
    .stream(true)
    .form();

  let mut receiver = client.responses().create_stream(create_request).await?;

  // 2. Wait for the response to be created and get its ID
  let mut response_id = None;
if let Some(event_result) = receiver.recv().await
{
    match event_result?
    {
      ResponseStreamEvent::ResponseCreated(event) =>
      {
        response_id = Some(event.response.id.clone());
        println!("Response created with ID: {}", event.response.id);
      },
      _ => println!("Received unexpected event type"),
    }
  }

if let Some(id) = response_id
{
    // 3. Let the response run for a moment
    println!("Allowing response to generate some content...");
    let mut event_count = 0;
    while let Some(event_result) = receiver.recv().await
    {
      match event_result?
      {
        ResponseStreamEvent::ResponseTextDelta(delta) =>
        {
          print!("{}", delta.delta);
          event_count += 1;
          // Cancel after receiving a few text deltas
if event_count >= 5
{
            break;
          }
        },
        ResponseStreamEvent::ResponseCompleted(_) =>
        {
          println!("\nResponse completed before cancellation");
          return Ok(());
        },
        _ => {},
      }
    }

    // 4. Cancel the response
    println!("\n\nCanceling the response...");
    let cancelled_response = client.responses().cancel(&id).await?;

    println!("Response cancelled successfully!");
    println!("Final status : {}", cancelled_response.status);

if let Some(incomplete_details) = cancelled_response.incomplete_details
{
      println!("Cancellation reason : {}", incomplete_details.reason);
    }

    // 5. Verify cancellation by trying to retrieve the response
    let retrieved_response = client.responses().retrieve(&id).await?;
    println!("Retrieved status after cancellation : {}", retrieved_response.status);
  }

  Ok(())
}
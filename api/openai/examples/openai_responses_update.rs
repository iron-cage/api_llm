//! Example of updating a response using the OpenAI API.
#![ allow( clippy::doc_markdown ) ]
//!
//! This example demonstrates how to:
//! 1. Create a response
//! 2. Update its metadata
//!
//! Run with:
//! `cargo run --example responses_update`
//!
//! Make sure you have set the `OPENAI_API_KEY` environment variable
//! or have a `secret/-secret.sh` file with the key.

use api_openai::
{
  ClientApiAccessors,
  Client,
  components ::
  {
    responses ::{ CreateResponseRequest, ResponseInput, ResponseObject },
    common ::ModelIdsResponses,
  },
};
use chrono::Utc;
use serde_json::json;

#[ allow( deprecated ) ]
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

  // 1. First create a response
  println!("Creating a response...");
  let create_request = CreateResponseRequest::former()
    .model(ModelIdsResponses::from("gpt-5-nano".to_string()))
    .input(ResponseInput::String("Tell me a short joke.".to_string()))
    .max_output_tokens(100)
    .form();

  let response : ResponseObject = client.responses().create(create_request).await?;
  println!("Created response with ID: {}", response.id);

  // 2. Update the response with new metadata
  println!("Updating response metadata...");
  let update_data = json!({
    "metadata": {
      "category": "joke",
      "updated_at": Utc::now().to_rfc3339(),
      "custom_field": "example_value"
    }
  });

  let updated_response : ResponseObject = client.responses().update(&response.id, update_data).await?;

  println!("Response updated successfully!");
  println!("Original metadata : {:?}", response.metadata);
  println!("Updated metadata : {:?}", updated_response.metadata);

  // Verify the update
if let Some(metadata) = updated_response.metadata
{
    println!("Update verification:");
    for (key, value) in &metadata.0
    {
      println!( "  {key}: {value}" );
    }
  }

  Ok(())
}
//! Example of retrieving a response using the OpenAI Responses API.
//!
//! Demonstrates creating a response, retrieving it by ID, then deleting it.
//!
//! Run with:
//! `cargo run --example openai_responses_get`

use api_openai::
{
  ClientApiAccessors,
  Client,
  components ::
  {
    responses ::{ CreateResponseRequest, ResponseInput },
    common ::ModelIdsResponses,
  },
};

#[ tokio::main( flavor = "current_thread" ) ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
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

  // Create a response to get an ID
  let request = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "gpt-5.1-chat-latest".to_string() ) )
    .input( ResponseInput::String( "Hello for get example!".to_string() ) )
    .form();
  let created = client.responses().create( request ).await?;
  println!( "Created response ID : {}", created.id );

  // Retrieve it by ID
  let retrieved = client.responses().retrieve( &created.id ).await?;
  println!( "Retrieved response ID : {}", retrieved.id );
  println!( "Status : {}", retrieved.status );

  // Clean up
  client.responses().delete( &created.id ).await?;
  println!( "Deleted response ID : {}", created.id );

  Ok( () )
}

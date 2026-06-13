//! Example of listing input items for a response using the OpenAI Responses API.
//!
//! Demonstrates creating a response and listing its input items.
//!
//! Run with:
//! `cargo run --example openai_responses_list_input_items`

use api_openai::
{
  ClientApiAccessors,
  Client,
  components ::
  {
    responses ::{ CreateResponseRequest, ResponseInput },
    common ::{ ModelIdsResponses, ListQuery },
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

  // Create a response to list its input items
  let request = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "gpt-5.1-chat-latest".to_string() ) )
    .input( ResponseInput::String( "Hello for list input items example!".to_string() ) )
    .form();
  let created = client.responses().create( request ).await?;
  let response_id = created.id;
  println!( "Created response ID : {response_id}" );

  // List input items for the response
  let query = ListQuery { limit : Some( 10 ) };
  let input_items = client.responses().list_input_items( &response_id, Some( query ) ).await?;
  println!( "Listed {} input item(s) for response ID : {response_id}", input_items.data.len() );

  // Clean up
  client.responses().delete( &response_id ).await?;
  println!( "Deleted response ID : {response_id}" );

  Ok( () )
}

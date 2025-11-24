use api_openai::ClientApiAccessors;
/*
use api_openai::
{
  client ::Client,
  error ::OpenAIError,
  api ::responses::
  {
    CreateResponseRequest,
    ResponseInput,
  },
  components ::
  {
    common ::{ ModelIdsResponses, ListQuery },
  },
};

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  let client = Client::new();

  // First, create a response to get an ID
  let create_request = CreateResponseRequest
  {
    model : ModelIdsResponses::from( "gpt-5.1-chat-latest".to_string() ),
    input : ResponseInput::String( "Hello for list input items example!".to_string() ),
    previous_response_id : None,
    reasoning : None,
    max_output_tokens : None,
    instructions : None,
    text : None,
    tools : None,
    tool_choice : None,
    truncation : None,
    metadata : None,
    temperature : None,
    top_p : None,
    user : None,
    include : None,
    parallel_tool_calls : None,
    store : None,
    stream : None,
  };
  let created_response = client.responses().create( create_request ).await?;
  let response_id = created_response.id;
  println!( "Created response ID: {}", response_id );

  // Now, list input items for the response
  let query = ListQuery::former().limit( 10 ).form();
  let input_items = client.responses().list_input_items( &response_id, Some( query ) ).await?;
  println!( "Listed {} input items for response ID: {}", input_items.data.len(), response_id );

  // Clean up : delete the created response
  client.responses().delete( &response_id ).await?;

  Ok( () )
}
*/
